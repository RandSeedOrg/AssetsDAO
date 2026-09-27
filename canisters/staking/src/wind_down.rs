use std::{borrow::Cow, cell::RefCell};

use candid::{CandidType, Decode, Encode, Principal};
use ic_ledger_types::BlockIndex;
use ic_stable_structures::{memory_manager::MemoryId, storable::Bound, StableBTreeMap, Storable};
use serde::{Deserialize, Serialize};
use system_configs_macro::has_permission_result;
use types::{
  stable_structures::Memory,
  staking::{StakingAccountId, StakingPoolId},
  TimestampNanos, E8S,
};

#[cfg(feature = "recovery-tests")]
mod fixture;
mod recovery;
use crate::{
  account::{
    crud_utils::{query_recoverable_account_ids, query_staking_accounts_by_pool},
    stable_structures::{StakingAccount, StakingAccountRecoverableError, StakingAccountStatus},
  },
  event_log::stake_and_unstake_events::{save_dissolve_event, save_unstake_event},
  guard_keys::get_staking_pool_wind_down_guard_key,
  on_chain::{
    address::{generate_staking_account_account_identifier, generate_staking_pool_account_identifier, generate_staking_pool_chain_address},
    query::balance_of,
    transfer::transfer_from_staking_pool_to_pay_center_gross,
  },
  parallel_guard::EntryGuard,
  pool::{crud_utils::query_staking_pool_by_id, stable_structures::StakingPool, STAKING_POOL_MAP},
  pool_transaction_record::utils::{reconcile_release_repairs, ReleaseRepair},
  system_configs::get_exteral_canister_id,
  MEMORY_MANAGER,
};
use recovery::{Budget, RecoveryProgress, Stage, StepError, StepResult};

const ICP_FEE: E8S = 10_000;
const MAX_BATCH_SIZE: u32 = 20;

fn validate_batch_size(batch_size: u32) -> Result<(), String> {
  if batch_size == 0 || batch_size > MAX_BATCH_SIZE {
    return Err(format!("batch_size must be between 1 and {}", MAX_BATCH_SIZE));
  }
  Ok(())
}

fn estimate_residual(pool_balance: E8S, active_principal: E8S, active_count: u64) -> E8S {
  let account_cost = active_principal.saturating_add(active_count.saturating_mul(20_000));
  pool_balance.saturating_sub(account_cost)
}

fn sweepable_residual(balance: E8S) -> Option<E8S> {
  balance.checked_sub(ICP_FEE).filter(|amount| *amount > 0)
}

#[derive(Debug, Clone, Serialize, Deserialize, CandidType, PartialEq, Eq)]
pub enum WindDownPhase {
  Prepared,
  Running,
  Paused,
  ResidualPending,
  Finalized,
}

impl WindDownPhase {
  fn as_text(&self) -> String {
    match self {
      Self::Prepared => "Prepared",
      Self::Running => "Running",
      Self::Paused => "Paused",
      Self::ResidualPending => "ResidualPending",
      Self::Finalized => "Finalized",
    }
    .to_string()
  }
}

#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct WindDownExpected {
  pub account_count: u64,
  pub active_principal: E8S,
  pub released_count: u64,
  pub released_amount: E8S,
  pub pool_balance: E8S,
  pub nns_neuron_occupies_funds: E8S,
  pub pay_center_address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct WindDownPreview {
  pub pool_id: StakingPoolId,
  pub account_count: u64,
  pub active_count: u64,
  pub active_principal: E8S,
  pub released_count: u64,
  pub released_amount: E8S,
  pub dissolved_count: u64,
  pub created_count: u64,
  pub recoverable_error_count: u64,
  pub pool_balance: E8S,
  pub nns_neuron_occupies_funds: E8S,
  pub estimated_residual: E8S,
  pub pay_center_address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct WindDownStatus {
  pub pool_id: StakingPoolId,
  pub phase: String,
  pub cursor: StakingAccountId,
  pub expected_account_count: u64,
  pub completed_account_count: u64,
  pub expected_amount: E8S,
  pub completed_amount: E8S,
  pub residual_amount: E8S,
  pub residual_onchain_tx_id: BlockIndex,
  pub residual_pay_center_tx_id: u64,
  pub fee_dust: E8S,
  pub nns_recovered: bool,
  pub pay_center_address: String,
  pub updated_at: TimestampNanos,
  pub recovery: Option<RecoveryProgress>,
}

#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct WindDownPlan {
  pub status: WindDownStatus,
  pub preview: WindDownPreview,
}

#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct WindDownBatchResult {
  pub status: WindDownStatus,
  pub processed_count: u32,
  pub remaining_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct WindDownFinalReport {
  pub status: WindDownStatus,
  pub all_accounts_dissolved: bool,
  pub pool_balance_after_sweep: E8S,
}

#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
struct WindDownJob {
  pool_id: StakingPoolId,
  phase: WindDownPhase,
  cursor: StakingAccountId,
  expected_account_count: u64,
  completed_account_count: u64,
  expected_amount: E8S,
  completed_amount: E8S,
  residual_amount: E8S,
  residual_onchain_tx_id: BlockIndex,
  residual_pay_center_tx_id: u64,
  fee_dust: E8S,
  nns_recovered: bool,
  pay_center_address: String,
  updated_at: TimestampNanos,
}

#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
struct WindDownAccountReceipt {
  pool_id: StakingPoolId,
  account_id: StakingAccountId,
  release_onchain_tx_id: BlockIndex,
  dissolve_onchain_tx_id: BlockIndex,
  dissolve_pay_center_tx_id: u64,
  updated_at: TimestampNanos,
}

impl Storable for WindDownJob {
  fn to_bytes(&self) -> Cow<'_, [u8]> {
    Cow::Owned(Encode!(self).unwrap())
  }
  fn from_bytes(bytes: Cow<[u8]>) -> Self {
    Decode!(bytes.as_ref(), Self).unwrap()
  }
  const BOUND: Bound = Bound::Unbounded;
}

impl Storable for WindDownAccountReceipt {
  fn to_bytes(&self) -> Cow<'_, [u8]> {
    Cow::Owned(Encode!(self).unwrap())
  }
  fn from_bytes(bytes: Cow<[u8]>) -> Self {
    Decode!(bytes.as_ref(), Self).unwrap()
  }
  const BOUND: Bound = Bound::Unbounded;
}

thread_local! {
  static WIND_DOWN_JOB_MAP: RefCell<StableBTreeMap<StakingPoolId, WindDownJob, Memory>> = RefCell::new(
    StableBTreeMap::init(MEMORY_MANAGER.with(|manager| manager.borrow().get(MemoryId::new(crate::memory_ids::STAKING_WIND_DOWN_JOB))))
  );
  static WIND_DOWN_ACCOUNT_RECEIPT_MAP: RefCell<StableBTreeMap<StakingAccountId, WindDownAccountReceipt, Memory>> = RefCell::new(
    StableBTreeMap::init(MEMORY_MANAGER.with(|manager| manager.borrow().get(MemoryId::new(crate::memory_ids::STAKING_WIND_DOWN_ACCOUNT_RECEIPT))))
  );

}

pub fn is_pool_locked(pool_id: StakingPoolId) -> bool {
  WIND_DOWN_JOB_MAP.with(|map| {
    map
      .borrow()
      .get(&pool_id)
      .map(|job| {
        matches!(
          job.phase,
          WindDownPhase::Prepared | WindDownPhase::Running | WindDownPhase::Paused | WindDownPhase::ResidualPending
        )
      })
      .unwrap_or(false)
  })
}

fn status_from_job(job: &WindDownJob) -> WindDownStatus {
  WindDownStatus {
    pool_id: job.pool_id,
    phase: job.phase.as_text(),
    cursor: job.cursor,
    expected_account_count: job.expected_account_count,
    completed_account_count: job.completed_account_count,
    expected_amount: job.expected_amount,
    completed_amount: job.completed_amount,
    residual_amount: job.residual_amount,
    residual_onchain_tx_id: job.residual_onchain_tx_id,
    residual_pay_center_tx_id: job.residual_pay_center_tx_id,
    fee_dust: job.fee_dust,
    nns_recovered: job.nns_recovered,
    pay_center_address: job.pay_center_address.clone(),
    updated_at: job.updated_at,
    recovery: recovery::progress(job.pool_id),
  }
}

fn save_job(job: &WindDownJob) {
  WIND_DOWN_JOB_MAP.with(|map| map.borrow_mut().insert(job.pool_id, job.clone()));
}

fn get_job(pool_id: StakingPoolId) -> Result<WindDownJob, String> {
  WIND_DOWN_JOB_MAP.with(|map| map.borrow().get(&pool_id).ok_or_else(|| format!("No wind-down job for pool {}", pool_id)))
}

fn save_receipt(receipt: &WindDownAccountReceipt) {
  WIND_DOWN_ACCOUNT_RECEIPT_MAP.with(|map| map.borrow_mut().insert(receipt.account_id, receipt.clone()));
}

fn get_receipt(account_id: StakingAccountId) -> Option<WindDownAccountReceipt> {
  WIND_DOWN_ACCOUNT_RECEIPT_MAP.with(|map| map.borrow().get(&account_id))
}

#[cfg(test)]
fn reconciliation_amount(on_chain_balance: E8S, virtual_balance: E8S) -> Result<Option<E8S>, String> {
  if on_chain_balance < virtual_balance {
    return Err(format!(
      "On-chain pool balance {} is below virtual transaction balance {}",
      on_chain_balance, virtual_balance
    ));
  }

  let difference = on_chain_balance - virtual_balance;
  Ok((difference > 0).then_some(difference))
}

fn validate_dissolved_receipt(account: &StakingAccount) -> Result<(), String> {
  if account.get_released_amount() == 0 {
    return Ok(());
  }
  if let Some(receipt) = get_receipt(account.get_id()) {
    if receipt.pool_id != account.get_pool_id() || receipt.dissolve_onchain_tx_id == 0 || receipt.dissolve_pay_center_tx_id == 0 {
      return Err(format!("Invalid wind-down receipt for dissolved account {}", account.get_id()));
    }
    return Ok(());
  }

  let dissolve_onchain_tx_id = account.get_dissolve_onchain_tx_id();
  let dissolve_pay_center_tx_id = account.get_dissolve_pay_center_tx_id();
  if dissolve_onchain_tx_id == 0 || dissolve_pay_center_tx_id == 0 {
    return Err(format!("Missing dissolve receipt for account {}", account.get_id()));
  }
  save_receipt(&WindDownAccountReceipt {
    pool_id: account.get_pool_id(),
    account_id: account.get_id(),
    release_onchain_tx_id: account.get_release_onchain_tx_id(),
    dissolve_onchain_tx_id,
    dissolve_pay_center_tx_id,
    updated_at: ic_cdk::api::time(),
  });
  Ok(())
}

async fn build_preview(pool_id: StakingPoolId) -> Result<WindDownPreview, String> {
  let pool = query_staking_pool_by_id(pool_id)?;
  let accounts = query_staking_accounts_by_pool(pool_id);
  let mut active_count: u64 = 0;
  let mut active_principal: E8S = 0;
  let mut released_count: u64 = 0;
  let mut released_amount: E8S = 0;
  let mut dissolved_count: u64 = 0;
  let mut created_count: u64 = 0;
  let mut recoverable_error_count: u64 = 0;

  for account in &accounts {
    if account.recoverable_error.is_some() {
      recoverable_error_count += 1;
    }
    match account.get_status() {
      StakingAccountStatus::InStake => {
        active_count += 1;
        active_principal = active_principal.saturating_add(account.get_staked_amount());
      }
      StakingAccountStatus::Released => {
        released_count += 1;
        released_amount = released_amount.saturating_add(account.get_released_amount());
      }
      StakingAccountStatus::Dissolved => dissolved_count += 1,
      StakingAccountStatus::Created => created_count += 1,
    }
  }

  let pool_balance = balance_of(&generate_staking_pool_account_identifier(pool_id)).await?;
  let pay_center_id = get_exteral_canister_id(types::sys::ExteralCanisterLabels::PayCenter);
  let pay_center = common_canisters::pay_center::Service(pay_center_id);
  let (pay_center_address,) = pay_center
    .get_address()
    .await
    .map_err(|e| format!("Failed to obtain payment center address: {:?}", e))?;
  Ok(WindDownPreview {
    pool_id,
    account_count: active_count + released_count,
    active_count,
    active_principal,
    released_count,
    released_amount,
    dissolved_count,
    created_count,
    recoverable_error_count,
    pool_balance,
    nns_neuron_occupies_funds: pool.get_nns_neuron_occupies_funds(),
    estimated_residual: estimate_residual(pool_balance, active_principal, active_count),
    pay_center_address,
  })
}

fn compare_expected(preview: &WindDownPreview, expected: &WindDownExpected) -> Result<(), String> {
  let matches = preview.account_count == expected.account_count
    && preview.active_principal == expected.active_principal
    && preview.released_count == expected.released_count
    && preview.released_amount == expected.released_amount
    && preview.pool_balance == expected.pool_balance
    && preview.nns_neuron_occupies_funds == expected.nns_neuron_occupies_funds
    && preview.pay_center_address == expected.pay_center_address;
  if matches {
    Ok(())
  } else {
    Err("Wind-down snapshot no longer matches the expected values".to_string())
  }
}

#[ic_cdk::update]
#[has_permission_result("staking::pool::wind_down")]
pub async fn get_staking_wind_down_preview(pool_id: StakingPoolId) -> Result<WindDownPreview, String> {
  build_preview(pool_id).await
}

#[ic_cdk::query]
pub fn get_staking_wind_down(pool_id: StakingPoolId) -> Option<WindDownStatus> {
  WIND_DOWN_JOB_MAP.with(|map| map.borrow().get(&pool_id).map(|job| status_from_job(&job)))
}

#[ic_cdk::update]
#[has_permission_result("staking::pool::wind_down")]
pub async fn prepare_staking_wind_down(pool_id: StakingPoolId, expected: WindDownExpected) -> Result<WindDownPlan, String> {
  let _pool_guard =
    EntryGuard::new(get_staking_pool_wind_down_guard_key(pool_id)).map_err(|_| "The staking pool has an operation in progress".to_string())?;
  if let Ok(existing) = get_job(pool_id) {
    if existing.phase != WindDownPhase::Finalized {
      return Err(format!(
        "A wind-down job already exists for this pool (phase={}, cursor={}). Resume it with execute_staking_wind_down_batch.",
        existing.phase.as_text(),
        existing.cursor
      ));
    }
    return Err("This pool has already been finalized".to_string());
  }

  let mut preview = build_preview(pool_id).await?;
  if preview.nns_neuron_occupies_funds > 0 {
    crate::nns::disburse_for_wind_down(pool_id)
      .await
      .map_err(|e| format!("NNS neuron is not ready for disbursement: {}", e))?;
    preview = build_preview(pool_id).await?;
  }
  if preview.created_count > 0 {
    return Err("Created staking accounts must be resolved before wind-down".to_string());
  }
  if preview.recoverable_error_count > 0 || !query_recoverable_account_ids(pool_id).is_empty() {
    return Err("Recoverable staking account errors must be resolved before wind-down".to_string());
  }
  if preview.nns_neuron_occupies_funds != 0 {
    return Err("NNS neuron funds have not been recovered".to_string());
  }
  compare_expected(&preview, &expected)?;
  // Defer reconciliation until all historical release debits have been audited.

  let mut pool = query_staking_pool_by_id(pool_id)?;
  pool.close_for_wind_down()?;
  STAKING_POOL_MAP.with(|map| map.borrow_mut().insert(pool_id, pool));

  let job = WindDownJob {
    pool_id,
    phase: WindDownPhase::Prepared,
    cursor: 0,
    expected_account_count: preview.account_count,
    completed_account_count: 0,
    expected_amount: preview.active_principal.saturating_add(preview.released_amount),
    completed_amount: 0,
    residual_amount: 0,
    residual_onchain_tx_id: 0,
    residual_pay_center_tx_id: 0,
    fee_dust: 0,
    nns_recovered: true,
    pay_center_address: preview.pay_center_address.clone(),
    updated_at: ic_cdk::api::time(),
  };
  save_job(&job);
  Ok(WindDownPlan {
    status: status_from_job(&job),
    preview,
  })
}

fn nonzero(block: u64) -> Option<u64> {
  (block != 0).then_some(block)
}

fn transfer_spec(account: &StakingAccount, stage: Stage) -> Result<recovery::Spec, String> {
  use crate::on_chain::address::{generate_staking_account_subaccount, generate_staking_pool_subaccount};
  use crate::on_chain::transfer::{TRANSFER_SCENE_PAY_CENTER, TRANSFER_SCENE_UNSTAKE};
  let principal = if account.get_status() == StakingAccountStatus::InStake {
    account.get_staked_amount()
  } else {
    account.get_released_amount()
  };
  let account_address = generate_staking_account_account_identifier(account.get_id()).to_hex();
  let (from_subaccount, from, to, amount, memo, lower_bound) = match stage {
    Stage::Release => (
      generate_staking_pool_subaccount(account.get_pool_id()).0,
      generate_staking_pool_account_identifier(account.get_pool_id()).to_hex(),
      account_address,
      principal.checked_add(ICP_FEE).ok_or("Release amount overflow")?,
      TRANSFER_SCENE_UNSTAKE,
      nonzero(account.get_stake_account_to_pool_onchain_tx_id()),
    ),
    Stage::Dissolve => (
      generate_staking_account_subaccount(account.get_id()).0,
      account_address,
      get_job(account.get_pool_id())?.pay_center_address,
      principal,
      TRANSFER_SCENE_PAY_CENTER,
      nonzero(account.get_release_onchain_tx_id()),
    ),
  };
  Ok(recovery::Spec {
    pool_id: account.get_pool_id(),
    account_id: account.get_id(),
    stage,
    from_subaccount,
    from,
    to,
    amount,
    fee: ICP_FEE,
    memo,
    lower_bound,
  })
}

fn receipt_for(account: &StakingAccount) -> Result<WindDownAccountReceipt, String> {
  let receipt = get_receipt(account.get_id()).unwrap_or(WindDownAccountReceipt {
    pool_id: account.get_pool_id(),
    account_id: account.get_id(),
    release_onchain_tx_id: account.get_release_onchain_tx_id(),
    dissolve_onchain_tx_id: account.get_dissolve_onchain_tx_id(),
    dissolve_pay_center_tx_id: account.get_dissolve_pay_center_tx_id(),
    updated_at: ic_cdk::api::time(),
  });
  if receipt.pool_id != account.get_pool_id() {
    return Err("Receipt belongs to a different pool".into());
  }
  Ok(receipt)
}
fn repair_for(account: &StakingAccount, v: recovery::Verified) -> ReleaseRepair {
  ReleaseRepair {
    account_id: account.get_id(),
    block: v.block,
    principal: if account.get_status() == StakingAccountStatus::InStake {
      account.get_staked_amount()
    } else {
      account.get_released_amount()
    },
    transfer_amount: v.amount,
    ledger_fee: v.fee,
    timestamp: v.timestamp,
  }
}

/// Before any new transfer, account for all historical confirmed pool debits.
async fn audit_pool(pool: u64, budget: &mut Budget) -> StepResult<()> {
  let mut audit = recovery::audit(pool);
  if audit.version > 1 {
    return Err(StepError::Failed("Unsupported wind-down audit version".into()));
  }
  audit.version = 1;
  if audit.complete {
    return Ok(());
  }
  let accounts: Vec<_> = query_staking_accounts_by_pool(pool)
    .into_iter()
    .filter(|a| a.get_id() > audit.cursor)
    .take(20)
    .collect();
  for account in accounts {
    if account.get_status() != StakingAccountStatus::Created
      && (account.get_status() == StakingAccountStatus::InStake || account.get_released_amount() > 0)
    {
      let mut receipt = receipt_for(&account)?;
      let verified = recovery::resolve(transfer_spec(&account, Stage::Release)?, nonzero(receipt.release_onchain_tx_id), false, budget).await?;
      budget.check()?;
      if let Some(v) = verified {
        receipt.release_onchain_tx_id = v.block;
        if account.get_status() != StakingAccountStatus::Dissolved {
          save_receipt(&receipt);
        }
        audit.repairs.push(repair_for(&account, v));
      } else if account.get_status() != StakingAccountStatus::InStake {
        return Err(StepError::Failed(format!("Released account {} has no verifiable release", account.get_id())));
      }
    }
    audit.cursor = account.get_id();
    audit.progress = recovery::progress(pool);
    recovery::save_audit(pool, audit.clone());
  }
  if query_staking_accounts_by_pool(pool).iter().any(|a| a.get_id() > audit.cursor) {
    return Err(StepError::Pending("Historical release audit continues on the next batch".into()));
  }
  budget.take(1)?;
  let balance = balance_of(&generate_staking_pool_account_identifier(pool))
    .await
    .map_err(StepError::Pending)?;
  budget.check()?;
  reconcile_release_repairs(pool, &audit.repairs, balance, ic_cdk::api::time())?;
  audit.complete = true;
  audit.repairs.clear();
  audit.progress = None;
  recovery::save_audit(pool, audit);
  Ok(())
}

async fn release_account(account_id: u64, budget: &mut Budget) -> StepResult<StakingAccount> {
  let account = StakingAccount::query_by_id(account_id)?;
  if account.get_status() == StakingAccountStatus::Dissolved {
    return Ok(account);
  }
  if account.get_status() == StakingAccountStatus::Created {
    return Err(StepError::Failed("Created account cannot be released".into()));
  }
  if account.get_status() == StakingAccountStatus::Released && account.get_released_amount() == 0 {
    return Ok(account);
  }
  let mut receipt = receipt_for(&account)?;
  let verified = recovery::resolve(transfer_spec(&account, Stage::Release)?, nonzero(receipt.release_onchain_tx_id), true, budget)
    .await?
    .ok_or("Release was not resolved".to_string())?;
  receipt.release_onchain_tx_id = verified.block;
  receipt.updated_at = ic_cdk::api::time();
  save_receipt(&receipt);
  // This read also commits the verified receipt before accounting can fail.
  budget.take(1)?;
  let balance = balance_of(&generate_staking_pool_account_identifier(account.get_pool_id()))
    .await
    .map_err(StepError::Pending)?;
  budget.check()?;
  let account = StakingAccount::query_by_id(account_id)?;
  let current_users = crate::account::crud_utils::query_user_in_stake_accounts(account.get_owner(), account.get_pool_id());
  if account.get_status() == StakingAccountStatus::InStake {
    let pool = query_staking_pool_by_id(account.get_pool_id())?;
    pool
      .get_staked_amount()
      .checked_sub(account.get_staked_amount())
      .ok_or("Pool principal underflow".to_string())?;
    if current_users.len() == 1 {
      pool
        .get_staked_user_count()
        .checked_sub(1)
        .ok_or("Pool user count underflow".to_string())?;
    }
  }
  reconcile_release_repairs(account.get_pool_id(), &[repair_for(&account, verified.clone())], balance, ic_cdk::api::time())?;
  if account.get_status() == StakingAccountStatus::Released {
    return Ok(account);
  }
  // All fallible financial checks above precede this non-awaiting local commit.
  // An unexpected failure must roll back this message, not return partial success.
  let pool = StakingPool::unstake_account(&account, &current_users).unwrap_or_else(|e| ic_cdk::trap(&e));
  let updated = account
    .change_to_un_stake(verified.block, account.get_staked_amount(), 0, verified.timestamp, 0, 0)
    .unwrap_or_else(|e| ic_cdk::trap(&e));
  save_unstake_event(&pool, &updated);
  Ok(updated)
}

async fn dissolve_account(account: StakingAccount, budget: &mut Budget) -> StepResult<StakingAccount> {
  budget.check()?;
  if account.get_status() == StakingAccountStatus::Dissolved {
    return Ok(account);
  }
  if account.get_status() != StakingAccountStatus::Released {
    return Err(StepError::Failed("Account is not Released".into()));
  }
  let mut receipt = receipt_for(&account)?;
  if account.get_released_amount() == 0 {
    save_receipt(&receipt);
    let updated = account.change_to_dissolved(0, 0)?;
    save_dissolve_event(&updated);
    return Ok(updated);
  }
  if receipt.dissolve_onchain_tx_id == 0 {
    if let Some(StakingAccountRecoverableError::DissolvePayCenterFailed(block)) = account.recoverable_error {
      receipt.dissolve_onchain_tx_id = block;
    }
  }
  let verified = recovery::resolve(transfer_spec(&account, Stage::Dissolve)?, nonzero(receipt.dissolve_onchain_tx_id), true, budget)
    .await?
    .ok_or("Dissolve was not resolved".to_string())?;
  receipt.dissolve_onchain_tx_id = verified.block;
  receipt.updated_at = ic_cdk::api::time();
  save_receipt(&receipt);
  budget.take(1)?;
  let owner = Principal::from_text(account.get_owner()).map_err(|_| "Invalid owner principal".to_string())?;
  let pay_center = common_canisters::pay_center::Service(get_exteral_canister_id(types::sys::ExteralCanisterLabels::PayCenter));
  let result = pay_center
    .dissolve(
      owner,
      account.get_released_amount(),
      verified.block,
      account.get_onchain_address(),
      account.get_id(),
    )
    .await
    .map_err(|e| StepError::Pending(format!("Pay center confirmation pending: {e:?}")))?;
  let tx = match result.0 {
    common_canisters::pay_center::Result2::Ok(tx) => tx,
    common_canisters::pay_center::Result2::Err(error) => {
      if error.starts_with("Dissolve historical receipt check is incomplete;") {
        return Err(StepError::Pending(error));
      }
      return Err(StepError::Failed(format!("Pay center rejected dissolve confirmation: {error}")));
    }
  };
  receipt.dissolve_pay_center_tx_id = tx;
  save_receipt(&receipt);
  budget.check()?;
  let updated = StakingAccount::query_by_id(account.get_id())?.change_to_dissolved(verified.block, tx)?;
  save_dissolve_event(&updated);
  Ok(updated)
}

#[ic_cdk::update]
#[has_permission_result("staking::pool::wind_down")]
pub async fn execute_staking_wind_down_batch(pool_id: StakingPoolId, batch_size: u32) -> Result<WindDownBatchResult, String> {
  validate_batch_size(batch_size)?;
  let _pool_guard = EntryGuard::new(get_staking_pool_wind_down_guard_key(pool_id))
    .map_err(|_| "The staking pool wind-down is already processing a batch".to_string())?;
  let mut job = get_job(pool_id)?;
  if job.phase == WindDownPhase::Finalized {
    return Err("Wind-down is already finalized".into());
  }
  if job.phase == WindDownPhase::ResidualPending {
    return Err("All accounts are processed; call finalize_staking_wind_down".into());
  }
  job.phase = WindDownPhase::Running;
  job.updated_at = ic_cdk::api::time();
  save_job(&job);
  let mut budget = Budget::new(pool_id);
  let mut processed_count = 0;
  let work: StepResult<()> = async {
    audit_pool(pool_id, &mut budget).await?;
    // Include dissolved accounts after the cursor: a prior callback may have committed
    // the account before the job count was saved.
    let accounts: Vec<_> = query_staking_accounts_by_pool(pool_id)
      .into_iter()
      .filter(|a| a.get_id() > job.cursor && (a.get_status() != StakingAccountStatus::Dissolved || get_receipt(a.get_id()).is_some()))
      .take(batch_size as usize)
      .collect();
    for account in accounts {
      budget.check()?;
      let updated = if account.get_status() == StakingAccountStatus::Dissolved {
        // Only wind-down receipts may advance a legacy job's completion count.
        if get_receipt(account.get_id()).is_none() {
          continue;
        }
        validate_dissolved_receipt(&account)?;
        account
      } else {
        let released = release_account(account.get_id(), &mut budget).await?;
        dissolve_account(released, &mut budget).await?
      };
      budget.check()?;
      job = get_job(pool_id)?;
      job.completed_account_count = job.completed_account_count.checked_add(1).ok_or("Completed count overflow".to_string())?;
      job.completed_amount = job
        .completed_amount
        .checked_add(updated.get_released_amount())
        .ok_or("Completed amount overflow".to_string())?;
      job.cursor = updated.get_id();
      job.updated_at = ic_cdk::api::time();
      save_job(&job);
      recovery::clear_progress(pool_id);
      processed_count += 1;
    }
    Ok(())
  }
  .await;
  job = get_job(pool_id)?;
  match work {
    Ok(()) => {}
    Err(StepError::Pending(message)) => recovery::report(pool_id, message, false),
    Err(StepError::Failed(message)) => {
      recovery::report(pool_id, message.clone(), true);
      job.phase = WindDownPhase::Paused;
      job.updated_at = ic_cdk::api::time();
      save_job(&job);
      return Err(message);
    }
  }
  let remaining_count = query_staking_accounts_by_pool(pool_id)
    .iter()
    .filter(|a| a.get_status() != StakingAccountStatus::Dissolved)
    .count() as u64;
  if remaining_count == 0 && job.phase != WindDownPhase::Paused && job.completed_account_count == job.expected_account_count {
    job.phase = WindDownPhase::ResidualPending;
    job.updated_at = ic_cdk::api::time();
    save_job(&job);
  }
  Ok(WindDownBatchResult {
    status: status_from_job(&job),
    processed_count,
    remaining_count,
  })
}

#[ic_cdk::update]
#[has_permission_result("staking::pool::wind_down")]
pub fn pause_staking_wind_down(pool_id: StakingPoolId) -> Result<WindDownStatus, String> {
  let mut job = get_job(pool_id)?;
  if job.phase == WindDownPhase::Finalized {
    return Err("Wind-down is already finalized".to_string());
  }
  job.phase = WindDownPhase::Paused;
  job.updated_at = ic_cdk::api::time();
  save_job(&job);
  Ok(status_from_job(&job))
}

async fn sweep_residual(job: &mut WindDownJob) -> Result<(), String> {
  if job.residual_pay_center_tx_id != 0 {
    return Ok(());
  }
  let (tx_id, amount) = if job.residual_onchain_tx_id != 0 {
    // The transfer can succeed even when the pay-center callback is rejected
    // or its response is lost. Retry the persisted receipt before consulting
    // the now-depleted pool balance so the job cannot finalize uncredited.
    (job.residual_onchain_tx_id, job.residual_amount)
  } else {
    let balance = balance_of(&generate_staking_pool_account_identifier(job.pool_id)).await?;
    let Some(amount_after_fee) = sweepable_residual(balance) else {
      job.fee_dust = balance;
      return Ok(());
    };
    let gross_amount = amount_after_fee.saturating_add(ICP_FEE);
    let (tx_id, amount) = transfer_from_staking_pool_to_pay_center_gross(job.pool_id, gross_amount).await?;
    job.residual_onchain_tx_id = tx_id;
    job.residual_amount = amount;
    job.updated_at = ic_cdk::api::time();
    save_job(job);
    (tx_id, amount)
  };

  let pay_center_id = get_exteral_canister_id(types::sys::ExteralCanisterLabels::PayCenter);
  let pay_center = common_canisters::pay_center::Service(pay_center_id);
  let source_address = generate_staking_pool_chain_address(job.pool_id);
  match pay_center
    .receive_staking_wind_down_residual(job.pool_id, amount, tx_id, source_address)
    .await
  {
    Ok((common_canisters::pay_center::Result2::Ok(pay_center_tx_id),)) => job.residual_pay_center_tx_id = pay_center_tx_id,
    Ok((common_canisters::pay_center::Result2::Err(error),)) => return Err(error),
    Err(error) => return Err(format!("Pay center residual call failed: {:?}", error)),
  }
  job.updated_at = ic_cdk::api::time();
  save_job(job);
  Ok(())
}

#[ic_cdk::update]
#[has_permission_result("staking::pool::wind_down")]
pub async fn finalize_staking_wind_down(pool_id: StakingPoolId) -> Result<WindDownFinalReport, String> {
  let _pool_guard =
    EntryGuard::new(get_staking_pool_wind_down_guard_key(pool_id)).map_err(|_| "The staking pool wind-down is already processing".to_string())?;
  let mut job = get_job(pool_id)?;
  let accounts = query_staking_accounts_by_pool(pool_id);
  for account in &accounts {
    if account.get_status() == StakingAccountStatus::Dissolved {
      validate_dissolved_receipt(account)?;
    }
  }
  let remaining = accounts
    .iter()
    .filter(|account| account.get_status() != StakingAccountStatus::Dissolved)
    .count();
  if remaining != 0 {
    return Err(format!("{} staking accounts are not Dissolved", remaining));
  }
  if job.phase == WindDownPhase::Finalized {
    let balance = balance_of(&generate_staking_pool_account_identifier(pool_id)).await?;
    return Ok(WindDownFinalReport {
      status: status_from_job(&job),
      all_accounts_dissolved: true,
      pool_balance_after_sweep: balance,
    });
  }
  job.phase = WindDownPhase::ResidualPending;
  sweep_residual(&mut job).await?;
  job.phase = WindDownPhase::Finalized;
  job.updated_at = ic_cdk::api::time();
  save_job(&job);
  let balance = balance_of(&generate_staking_pool_account_identifier(pool_id)).await?;
  Ok(WindDownFinalReport {
    status: status_from_job(&job),
    all_accounts_dissolved: true,
    pool_balance_after_sweep: balance,
  })
}

#[cfg(test)]
mod tests {
  #[test]
  fn batch_and_balance_boundaries() {
    assert!(super::validate_batch_size(0).is_err());
    assert!(super::validate_batch_size(20).is_ok());
    assert!(super::validate_batch_size(21).is_err());
    assert_eq!(super::estimate_residual(250_000, 100_000, 1), 130_000);
    assert_eq!(super::sweepable_residual(10_000), None);
    assert_eq!(super::reconciliation_amount(150, 100).unwrap(), Some(50));
    assert_eq!(super::reconciliation_amount(100, 100).unwrap(), None);
    assert!(super::reconciliation_amount(99, 100).is_err());
  }
}
