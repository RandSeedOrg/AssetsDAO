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

use crate::{
  account::{
    crud_utils::{query_recoverable_account_ids, query_staking_accounts_by_pool},
    stable_structures::{StakingAccount, StakingAccountRecoverableError, StakingAccountStatus},
  },
  event_log::{
    stake_and_unstake_events::{save_dissolve_event, save_unstake_event},
    transfer_events::{
      save_dissolve_pay_center_receive_fail_event, save_dissolve_pay_center_receive_ok_event, save_dissolve_pay_center_receive_start_event,
      save_dissolve_pay_center_transfer_fail_event, save_dissolve_pay_center_transfer_ok_event, save_dissolve_pay_center_transfer_start_event,
      save_unstake_transfer_fail_event, save_unstake_transfer_ok_event, save_unstake_transfer_start_event,
    },
  },
  guard_keys::{get_staking_pool_wind_down_guard_key, get_unstake_guard_key},
  on_chain::{
    address::{generate_staking_pool_account_identifier, generate_staking_pool_chain_address},
    query::balance_of,
    transfer::{
      transfer_from_staking_account_to_pay_center, transfer_from_staking_pool_to_pay_center_gross, transfer_from_staking_pool_to_staking_account,
    },
  },
  parallel_guard::EntryGuard,
  pool::{crud_utils::query_staking_pool_by_id, stable_structures::StakingPool, STAKING_POOL_MAP},
  system_configs::get_exteral_canister_id,
  MEMORY_MANAGER,
};

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

fn validate_dissolved_receipt(account: &StakingAccount) -> Result<(), String> {
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
      return Err("A wind-down job already exists for this pool".to_string());
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

fn find_next_accounts(pool_id: StakingPoolId, cursor: StakingAccountId, limit: usize) -> Vec<StakingAccount> {
  query_staking_accounts_by_pool(pool_id)
    .into_iter()
    .filter(|account| account.get_id() > cursor && account.get_status() != StakingAccountStatus::Dissolved)
    .take(limit)
    .collect()
}

async fn release_account(account_id: StakingAccountId) -> Result<StakingAccount, String> {
  let _guard = EntryGuard::new(get_unstake_guard_key(account_id)).map_err(|_| format!("Account {} is already being processed", account_id))?;
  let account = StakingAccount::query_by_id(account_id)?;
  if account.get_status() == StakingAccountStatus::Dissolved {
    return Ok(account);
  }
  if account.get_status() == StakingAccountStatus::Created {
    return Err("Created account cannot be released".to_string());
  }
  if account.get_status() == StakingAccountStatus::Released {
    if get_receipt(account_id).is_none() {
      save_receipt(&WindDownAccountReceipt {
        pool_id: account.get_pool_id(),
        account_id,
        release_onchain_tx_id: account.get_release_onchain_tx_id(),
        dissolve_onchain_tx_id: 0,
        dissolve_pay_center_tx_id: 0,
        updated_at: ic_cdk::api::time(),
      });
    }
    return Ok(account);
  }

  let receipt = get_receipt(account_id);
  let release_tx_id = if let Some(receipt) = &receipt {
    if receipt.pool_id != account.get_pool_id() {
      return Err("Wind-down receipt belongs to another pool".to_string());
    }
    receipt.release_onchain_tx_id
  } else {
    save_unstake_transfer_start_event(account_id, account.get_pool_id());
    match transfer_from_staking_pool_to_staking_account(account.get_pool_id(), account_id, account.get_staked_amount()).await {
      Ok(tx_id) => {
        save_unstake_transfer_ok_event(account_id, account.get_pool_id(), tx_id);
        let receipt = WindDownAccountReceipt {
          pool_id: account.get_pool_id(),
          account_id,
          release_onchain_tx_id: tx_id,
          dissolve_onchain_tx_id: 0,
          dissolve_pay_center_tx_id: 0,
          updated_at: ic_cdk::api::time(),
        };
        save_receipt(&receipt);
        tx_id
      }
      Err(error) => {
        save_unstake_transfer_fail_event(account_id, account.get_pool_id(), error.clone());
        return Err(error);
      }
    }
  };

  let current_user_accounts = crate::account::crud_utils::query_user_in_stake_accounts(account.get_owner(), account.get_pool_id());
  let pool = StakingPool::unstake_account(&account, &current_user_accounts)?;
  let updated = account.change_to_un_stake(release_tx_id, account.get_staked_amount(), 0, ic_cdk::api::time(), 0, 0)?;
  save_unstake_event(&pool, &updated);
  Ok(updated)
}

async fn dissolve_account(account: StakingAccount) -> Result<StakingAccount, String> {
  let _guard =
    EntryGuard::new(get_unstake_guard_key(account.get_id())).map_err(|_| format!("Account {} is already being processed", account.get_id()))?;
  let account = StakingAccount::query_by_id(account.get_id())?;
  if account.get_status() == StakingAccountStatus::Dissolved {
    return Ok(account);
  }
  if account.get_status() != StakingAccountStatus::Released {
    return Err("Account is not Released".to_string());
  }
  let owner = Principal::from_text(account.get_owner()).map_err(|_| format!("Invalid account owner principal: {}", account.get_owner()))?;
  let pay_center_id = get_exteral_canister_id(types::sys::ExteralCanisterLabels::PayCenter);
  let pay_center = common_canisters::pay_center::Service(pay_center_id);
  let receipt = get_receipt(account.get_id()).ok_or_else(|| "Missing wind-down account receipt".to_string())?;
  let dissolve_tx_id = if receipt.dissolve_onchain_tx_id != 0 {
    receipt.dissolve_onchain_tx_id
  } else {
    save_dissolve_pay_center_transfer_start_event(account.get_id(), pay_center_id.to_string());
    let tx_id = match transfer_from_staking_account_to_pay_center(account.get_id(), account.get_released_amount()).await {
      Ok(tx_id) => tx_id,
      Err(error) => {
        save_dissolve_pay_center_transfer_fail_event(account.get_id(), pay_center_id.to_string(), error.clone());
        return Err(error);
      }
    };
    save_dissolve_pay_center_transfer_ok_event(account.get_id(), pay_center_id.to_string(), tx_id);
    let mut next_receipt = receipt.clone();
    next_receipt.dissolve_onchain_tx_id = tx_id;
    next_receipt.updated_at = ic_cdk::api::time();
    save_receipt(&next_receipt);
    tx_id
  };

  save_dissolve_pay_center_receive_start_event(account.get_id(), pay_center_id.to_string(), dissolve_tx_id);
  let pay_center_tx_id = match pay_center
    .dissolve(
      owner,
      account.get_released_amount(),
      dissolve_tx_id,
      account.get_onchain_address(),
      account.get_id(),
    )
    .await
  {
    Ok((common_canisters::pay_center::Result2::Ok(tx_id),)) => {
      save_dissolve_pay_center_receive_ok_event(account.get_id(), pay_center_id.to_string(), dissolve_tx_id, tx_id);
      tx_id
    }
    Ok((common_canisters::pay_center::Result2::Err(error),)) => {
      save_dissolve_pay_center_receive_fail_event(account.get_id(), pay_center_id.to_string(), error.clone());
      account.stable_to_recoverable_error(StakingAccountRecoverableError::DissolvePayCenterFailed(dissolve_tx_id));
      return Err(error);
    }
    Err(error) => {
      let message = format!("Pay center dissolve call failed: {:?}", error);
      save_dissolve_pay_center_receive_fail_event(account.get_id(), pay_center_id.to_string(), message.clone());
      account.stable_to_recoverable_error(StakingAccountRecoverableError::DissolvePayCenterFailed(dissolve_tx_id));
      return Err(message);
    }
  };

  let updated = account.change_to_dissolved(dissolve_tx_id, pay_center_tx_id)?;
  let mut next_receipt = receipt;
  next_receipt.dissolve_onchain_tx_id = dissolve_tx_id;
  next_receipt.dissolve_pay_center_tx_id = pay_center_tx_id;
  next_receipt.updated_at = ic_cdk::api::time();
  save_receipt(&next_receipt);
  save_dissolve_event(&updated);
  Ok(updated)
}

async fn process_account(account: StakingAccount) -> Result<StakingAccount, String> {
  let released = release_account(account.get_id()).await?;
  dissolve_account(released).await
}

#[ic_cdk::update]
#[has_permission_result("staking::pool::wind_down")]
pub async fn execute_staking_wind_down_batch(pool_id: StakingPoolId, batch_size: u32) -> Result<WindDownBatchResult, String> {
  validate_batch_size(batch_size)?;
  let _pool_guard = EntryGuard::new(get_staking_pool_wind_down_guard_key(pool_id))
    .map_err(|_| "The staking pool wind-down is already processing a batch".to_string())?;
  let mut job = get_job(pool_id)?;
  if job.phase == WindDownPhase::Finalized {
    return Err("Wind-down is already finalized".to_string());
  }
  if job.phase == WindDownPhase::ResidualPending {
    return Err("All accounts are processed; call finalize_staking_wind_down".to_string());
  }
  job.phase = WindDownPhase::Running;
  job.updated_at = ic_cdk::api::time();
  save_job(&job);

  let accounts = find_next_accounts(pool_id, job.cursor, batch_size as usize);
  let mut processed_count = 0;
  for account in accounts {
    let account_id = account.get_id();
    let updated = process_account(account).await?;
    job.cursor = account_id;
    job.completed_account_count = job.completed_account_count.saturating_add(1);
    job.completed_amount = job.completed_amount.saturating_add(updated.get_released_amount());
    job.updated_at = ic_cdk::api::time();
    save_job(&job);
    processed_count += 1;
  }

  let remaining_count = query_staking_accounts_by_pool(pool_id)
    .into_iter()
    .filter(|account| account.get_status() != StakingAccountStatus::Dissolved)
    .count() as u64;
  if remaining_count == 0 {
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
  let balance = balance_of(&generate_staking_pool_account_identifier(job.pool_id)).await?;
  let Some(amount_after_fee) = sweepable_residual(balance) else {
    job.fee_dust = balance;
    return Ok(());
  };

  let (tx_id, amount) = if job.residual_onchain_tx_id != 0 {
    (job.residual_onchain_tx_id, job.residual_amount)
  } else {
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
  use super::{estimate_residual, validate_batch_size};

  #[test]
  fn batch_size_is_limited_to_one_through_twenty() {
    assert!(validate_batch_size(0).is_err());
    assert!(validate_batch_size(1).is_ok());
    assert!(validate_batch_size(20).is_ok());
    assert!(validate_batch_size(21).is_err());
  }

  #[test]
  fn residual_estimate_accounts_for_principal_and_two_fees_per_release() {
    assert_eq!(estimate_residual(250_000, 100_000, 1), 130_000);
    assert_eq!(estimate_residual(100_000, 100_000, 1), 0);
  }

  #[test]
  fn residual_at_or_below_the_ledger_fee_is_fee_dust() {
    assert_eq!(super::sweepable_residual(10_000), None);
    assert_eq!(super::sweepable_residual(10_001), Some(1));
  }
}
