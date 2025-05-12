use std::{borrow::Cow, time::Duration};

use candid::{CandidType, Decode, Encode};
use ic_ledger_types::BlockIndex;
use ic_stable_structures::{storable::Bound, Storable};
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};
use types::{date::YearMonthDay, entities::{add_indexed_id, remove_indexed_id}, stable_structures::{new_entity_id, MetaData}, staking::StakingAccountId, EntityId, TimestampNanos, UserId, E8S};

use crate::{on_chain::address::generate_staking_account_chain_address, pool::stable_structures::{RewardConfig, StakingPool}, reward::stable_structures::StakingReward};

use super::{client_transport_structures::StakeDto, STAKING_ACCOUNT_ID, STAKING_ACCOUNT_MAP, STAKING_RECOVERABLE_ERROR_ACCOUNT_INDEX_MAP, STAKING_UNSTAKE_ON_DAY_ACCOUNT_INDEX_MAP};

/// Status of the staked account
#[derive(EnumString, Display, Debug, Clone, Serialize, Deserialize, CandidType, PartialEq)]
pub enum StakingAccountStatus {
  /// New status，at this timestake account已经创建，But there is no stake
  #[strum(serialize = "0")]
  Created,
  /// stake（stake time， Expiry time）
  #[strum(serialize = "1")]
  InStake,
  /// Released，Money transfers from the stake pool to the stake account（Release time）
  #[strum(serialize = "2")]
  Released,
  /// Dissolved，Money has been withdrawn to the payment center（Redemption time）
  #[strum(serialize = "3")]
  Dissolved,
}

/// Recoverable exceptions for staked accounts
#[derive(Debug, Clone, Serialize, Deserialize, CandidType, PartialEq)]
pub enum StakingAccountRecoverableError {
  /// When staking，Transfer from stake account to stake pool failed
  StakeTransferToPoolFailed(BlockIndex, u64),
  /// When dissolved，Payment center bookkeeping failed
  DissolvePayCenterFailed(BlockIndex),
  /// When unstake error，On-chain transactions in the stake pool failed
  EarlyUnstakePenaltyOnChainFailed(BlockIndex, TimestampNanos, E8S),
  /// 提前When unstaking，Payment center bookkeeping failed
  EarlyUnstakePenaltyPayCenterFailed(BlockIndex, BlockIndex, TimestampNanos, E8S),
}

/// stake account
/// Whenever a stake is initiated by a user，Create a new staked account
/// stake account，The balance in the staked account can be transferred to the user's main account at any time.
/// The balance in the staked account will only be transferred to the user's staked account when the user's stake expires.，at this time，The user's balance will increase，It will reduce the corresponding stake amount
#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct StakingAccount {
  /// stake accountID
  pub id: Option<EntityId>,
  /// Corresponding stake poolID
  pub pool_id: Option<EntityId>,
  /// The owner of the stake account Principal ID
  pub owner: Option<UserId>,
  /// The on-chain address of stake account 
  pub address: Option<String>,
  /// Released amount in the staked account
  pub released_amount: Option<E8S>,
  /// The amount currently staked by the staked account
  /// When a user initiates a stake
  pub staked_amount: Option<E8S>,
  /// Deduction amount for staked account
  pub penalty_amount: Option<E8S>,
  /// Total rewards received by the staked account
  pub accumulated_rewards: Option<E8S>,
  /// Status of the staked account
  pub status: Option<StakingAccountStatus>,
  /// Reward configuration for staked accounts
  pub reward_config: Option<RewardConfig>,
  pub stake_pay_center_onchain_tx_id: Option<u64>,
  /// Payment center transaction flow during stake ID
  pub stake_pay_center_tx_id: Option<u64>,
  /// When staking，Stake the account and the transaction on the chain from the stake pool ID
  pub stake_account_to_pool_onchain_tx_id: Option<u64>,
  pub release_onchain_tx_id: Option<u64>,
  /// When dissolved，On-chain transaction id
  pub dissolve_onchain_tx_id: Option<u64>,
  /// When dissolved，Payment Center Transactions ID
  pub dissolve_pay_center_tx_id: Option<u64>,
  pub penalty_onchain_tx_id: Option<u64>,
  pub penalty_pay_center_tx_id: Option<u64>,
  /// stake days
  pub total_staking_days: Option<u16>,
  /// The number of days to be de-pled in advance
  pub min_early_unstake_days: Option<u16>,
  /// stake start time
  pub stake_time: Option<TimestampNanos>,
  /// Time to de-punish in advance
  pub can_early_unstake_time: Option<TimestampNanos>,
  /// stake expiration time
  pub stake_deadline: Option<TimestampNanos>,
  /// Time to release money
  pub release_time: Option<TimestampNanos>,
  /// Time to transfer money to payment center
  pub dissolve_time: Option<TimestampNanos>,
  /// The last time of rewards
  pub last_reward_time: Option<TimestampNanos>,
  /// Meta information for staked accounts
  pub meta: Option<MetaData>,
  /// Abnormal state that can be restored
  pub recoverable_error: Option<StakingAccountRecoverableError>,
}

impl StakingAccount {
  /// Create a new staked account
  pub fn from_stake_dto_and_pool(stake_dto: &StakeDto, pool: &StakingPool) -> Self {
    let StakeDto { pool_id, staking_amount, staking_days } = *stake_dto;
    // Generate a stake poolID
    let id = STAKING_ACCOUNT_ID.with(|id_seq| new_entity_id(id_seq));
    let owner = ic_cdk::api::msg_caller().to_string();

    // Generate address on the stake pool chain
    let address = generate_staking_account_chain_address(id);

    Self {
      id: Some(id),
      pool_id: Some(pool_id),
      owner: Some(owner),
      address: Some(address),
      released_amount: None,
      staked_amount: Some(staking_amount),
      penalty_amount: None,
      accumulated_rewards: None,
      status: Some(StakingAccountStatus::Created),
      reward_config: Some(pool.get_reward_config()),
      stake_pay_center_onchain_tx_id: None,
      stake_pay_center_tx_id: None,
      stake_account_to_pool_onchain_tx_id: None,
      release_onchain_tx_id: None,
      dissolve_onchain_tx_id: None,
      dissolve_pay_center_tx_id: None,
      penalty_onchain_tx_id: None,
      penalty_pay_center_tx_id: None,
      total_staking_days: Some(staking_days),
      min_early_unstake_days: Some(pool.get_term_config().get_min_early_unstake_days()),
      stake_time: None,
      can_early_unstake_time: None,
      stake_deadline: None,
      release_time: None,
      dissolve_time: None,
      last_reward_time: None,
      meta: Some(MetaData::default()),
      recoverable_error: None,
    }
  }

  pub fn update_reward(reward: &StakingReward, last_reward_time: TimestampNanos) -> Self {
    STAKING_ACCOUNT_MAP.with(|map| {
      let mut map = map.borrow_mut();
      let mut account = map.get(&reward.get_account_id()).unwrap();

      account.accumulated_rewards = Some(account.get_accumulated_rewards() + reward.get_reward_amount());
      account.last_reward_time = Some(last_reward_time);
      account.update_meta();

      map.insert(account.get_id(), account.clone());

      account
    })
  }

  pub fn query_by_id(id: StakingAccountId) -> Result<Self, String> {
    STAKING_ACCOUNT_MAP.with(|map| {
      let account = map.borrow().get(&id);
      match account {
        Some(account) => Ok(account),
        None => Err(format!("Staking account not found: {}", id))
      }
    })
  }

  // Set the current staked account to a recoverable error state，Wait for timed task processing
  pub fn stable_to_recoverable_error(&self, error: StakingAccountRecoverableError) -> Self {
    STAKING_ACCOUNT_MAP.with(|map| {
      let mut map = map.borrow_mut();
      let mut account = map.get(&self.get_id()).unwrap();

      account.recoverable_error = Some(error);
      account.update_meta();

      map.insert(account.get_id(), account.clone());

      // Update the index that can recover errors
      STAKING_RECOVERABLE_ERROR_ACCOUNT_INDEX_MAP.with(|map| add_indexed_id(map, &account.get_pool_id(), account.get_id()));

      account
    })
  }

  /// Turn the staked account status from Created Change to InStake
  pub fn change_to_in_stake(
    &mut self,
    stake_pay_center_onchain_tx_id: u64,
    stake_pay_center_tx_id: u64,
    stake_account_to_pool_onchain_tx_id: u64,
  ) {
    if self.get_status() != StakingAccountStatus::Created {
      ic_cdk::trap("Staking account is not in Created status");
    }

    let now = ic_cdk::api::time();
    let stake_deadline = now + Duration::from_secs(self.get_total_staking_days() as u64 * 24 * 60 * 60).as_nanos() as u64;
    let can_early_unstake_time = now + Duration::from_secs(self.get_min_early_unstake_days() as u64 * 24 * 60 * 60).as_nanos() as u64;
    self.stake_pay_center_onchain_tx_id = Some(stake_pay_center_onchain_tx_id);
    self.stake_pay_center_tx_id = Some(stake_pay_center_tx_id);
    self.stake_account_to_pool_onchain_tx_id = Some(stake_account_to_pool_onchain_tx_id);
    self.status = Some(StakingAccountStatus::InStake);
    self.stake_time = Some(now);
    self.stake_deadline = Some(stake_deadline);
    self.can_early_unstake_time = Some(can_early_unstake_time);
    self.recoverable_error = None;
    self.update_meta();
  }

  /// Unsolicited Account
  pub fn change_to_un_stake(&self, unstake_tx_id: u64, release_amount: E8S, penalty_amount: E8S, now: TimestampNanos, penalty_onchain_tx_id: u64, penalty_pay_center_tx_id: u64) -> Result<Self, String> {
    STAKING_ACCOUNT_MAP.with(|map| {
      let mut map = map.borrow_mut();
      
      let mut account = map.get(&self.get_id()).ok_or("Staking account not found")?;
      account.status = Some(StakingAccountStatus::Released);
      account.release_onchain_tx_id = Some(unstake_tx_id);
      account.released_amount = Some(release_amount);
      account.penalty_amount = Some(penalty_amount);
      account.release_time = Some(now);
      account.penalty_onchain_tx_id = if penalty_onchain_tx_id != 0 { Some(penalty_onchain_tx_id)} else { None };
      account.penalty_pay_center_tx_id = if penalty_pay_center_tx_id != 0 { Some(penalty_pay_center_tx_id)} else { None };
      account.recoverable_error = None;
      account.update_meta();

      map.insert(account.get_id(), account.clone());

      // When unstake，Remove the staked account from the unstaked date index，Further improve the performance of timing tasks
      STAKING_UNSTAKE_ON_DAY_ACCOUNT_INDEX_MAP.with(|map| remove_indexed_id(map, &YearMonthDay::from(account.get_stake_deadline()), account.get_id()));

      Ok(account)
    })
  }

  /// Dissolve the account
  pub fn change_to_dissolved(&self, dissolve_tx_id: u64, pay_center_tx_id: u64) -> Result<Self, String> {
    STAKING_ACCOUNT_MAP.with(|map| {
      let mut map = map.borrow_mut();
      
      let mut account = map.get(&self.get_id()).ok_or("Staking account not found")?;
      account.status = Some(StakingAccountStatus::Dissolved);
      account.dissolve_onchain_tx_id = Some(dissolve_tx_id);
      account.dissolve_pay_center_tx_id = Some(pay_center_tx_id);
      account.dissolve_time = Some(ic_cdk::api::time());
      account.update_meta();
      account.recoverable_error = None;

      map.insert(account.get_id(), account.clone());

      Ok(account)
    })
  }

  fn update_meta(&mut self) {
    self.meta = Some(self.meta.clone().unwrap_or_default().update());
  }

  pub fn get_remaining_lockup_days(&self) -> u64 {
    let now = ic_cdk::api::time();
    let stake_deadline = self.get_stake_deadline();
    if stake_deadline > now {
      let diff_time = stake_deadline - now;
      let one_day = 24 * 60 * 60 * 1_000_000_000;
      let result = diff_time / one_day;
      if diff_time % one_day > 0 {
        result + 1
      } else {
        result
      }
    } else {
      0
    }
  }

  pub fn get_pool_id(&self) -> EntityId {
    self.pool_id.unwrap_or_default()
  }

  pub fn get_staked_amount(&self) -> E8S {
    self.staked_amount.unwrap_or_default()
  }

  pub fn get_status(&self) -> StakingAccountStatus {
    self.status.clone().unwrap_or(StakingAccountStatus::Created)
  }

  pub fn get_released_amount(&self) -> E8S {
    self.released_amount.unwrap_or_default()
  }

  pub fn get_id(&self) -> EntityId {
    self.id.unwrap_or_default()
  }

  pub fn get_owner(&self) -> UserId {
    self.owner.clone().unwrap_or_default()
  }

  pub fn get_onchain_address(&self) -> String {
    self.address.clone().unwrap_or_default()
  }

  pub fn get_accumulated_rewards(&self) -> E8S {
    self.accumulated_rewards.unwrap_or_default()
  }

  pub fn get_reward_config(&self) -> RewardConfig {
    self.reward_config.clone().unwrap_or_default()
  }
  pub fn get_penalty_amount(&self) -> E8S {
    self.penalty_amount.unwrap_or_default()
  }
  pub fn get_stake_pay_center_onchain_tx_id(&self) -> u64 {
    self.stake_pay_center_onchain_tx_id.unwrap_or_default()
  }

  pub fn get_stake_pay_center_tx_id(&self) -> u64 {
    self.stake_pay_center_tx_id.unwrap_or_default()
  }

  pub fn get_stake_account_to_pool_onchain_tx_id(&self) -> u64 {
    self.stake_account_to_pool_onchain_tx_id.unwrap_or_default()
  }
  pub fn get_release_onchain_tx_id(&self) -> u64 {
    self.release_onchain_tx_id.unwrap_or_default()
  }
  pub fn get_dissolve_onchain_tx_id(&self) -> u64 {
    self.dissolve_onchain_tx_id.unwrap_or_default()
  }
  pub fn get_dissolve_pay_center_tx_id(&self) -> u64 {
    self.dissolve_pay_center_tx_id.unwrap_or_default()
  }
  pub fn get_total_staking_days(&self) -> u16 {
    self.total_staking_days.unwrap_or_default()
  }

  pub fn get_min_early_unstake_days(&self) -> u16 {
    self.min_early_unstake_days.unwrap_or_default()
  }

  pub fn get_can_early_unstake_time(&self) -> TimestampNanos {
    self.can_early_unstake_time.unwrap_or_default()
  }

  pub fn get_release_time(&self) -> TimestampNanos {
    self.release_time.unwrap_or_default()
  }
  pub fn get_dissolve_time(&self) -> TimestampNanos {
    self.dissolve_time.unwrap_or_default()
  }
  pub fn get_last_reward_time(&self) -> TimestampNanos {
    self.last_reward_time.unwrap_or_default()
  }
  pub fn get_create_time(&self) -> TimestampNanos {
    self.meta.as_ref().and_then(|meta| meta.created_at).unwrap_or_default()
  }

  pub fn get_stake_time(&self) -> TimestampNanos {
    self.stake_time.unwrap_or_default()
  }

  pub fn get_stake_deadline(&self) -> TimestampNanos {
    self.stake_deadline.unwrap_or_default()
  }
}

impl Storable for StakingAccount {
  fn to_bytes(&self) -> Cow<[u8]> {
    Cow::Owned(Encode!(self).unwrap())
  }

  fn from_bytes(bytes: Cow<[u8]>) -> Self {
    Decode!(bytes.as_ref(), Self).unwrap()
  }

  const BOUND: Bound = Bound::Unbounded;
}
