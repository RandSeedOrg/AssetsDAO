use std::borrow::Cow;

use bigdecimal::{BigDecimal, ToPrimitive};
use candid::{CandidType, Decode, Encode};
use ic_stable_structures::{storable::Bound, Storable};
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};
use types::{date::YearMonthDay, entities::add_indexed_id, product::e8s_to_value, stable_structures::{new_entity_id, MetaData}, EntityId, TimestampNanos, UserId, E8S};

use crate::{account::stable_structures::StakingAccount, pool::stable_structures::RewardCrypto, StakingAccountId, StakingRewardId};

use super::{STAKING_ACCOUNT_REWARD_INDEX_MAP, STAKING_POOL_REWARD_INDEX_MAP, STAKING_REWARD_ID, STAKING_REWARD_MAP, STAKING_USER_REWARD_INDEX_MAP};

/// Staking reward data structure，用于存储stake Reward的相关信息
#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct StakingReward {
  /// stake RewardID
  pub id: Option<EntityId>,
  /// Staking poolID
  pub pool_id: Option<EntityId>,
  /// stake accountID
  pub account_id: Option<EntityId>,
  pub tx_id: Option<EntityId>,
  /// The owner of the reward
  pub owner: Option<UserId>,
  /// The reward currency
  pub reward_crypto: Option<RewardCrypto>,
  /// The amount of stake reward
  pub reward_amount: Option<E8S>,
  pub status: Option<StakingRewardStatus>,
  pub meta: Option<MetaData>,
}

impl Storable for StakingReward {
  fn to_bytes(&self) -> Cow<[u8]> {
    Cow::Owned(Encode!(self).unwrap())
  }

  fn from_bytes(bytes: Cow<[u8]>) -> Self {
    Decode!(bytes.as_ref(), Self).unwrap()
  }

  const BOUND: Bound = Bound::Unbounded;
}

#[derive(EnumString, Display, Debug, Clone, Serialize, Deserialize, CandidType, PartialEq)]
pub enum StakingRewardStatus {
  /// New（Not received）
  #[strum(serialize = "0")]
  Created,
  /// Accounted（Received）
  #[strum(serialize = "1")]
  Received,
}

impl StakingReward {

  /// Create a new staking reward record
  pub fn reward_account(account: &StakingAccount)-> (StakingReward, StakingAccount) {
    // 1. generate a new ID for the reward record
    let id = STAKING_REWARD_ID.with(|id_seq| new_entity_id(id_seq));

    // 2. Calculate the reward amount
    // 2.1 Get reward configuration for staked accounts
    let reward_config = account.get_reward_config();
    // 2.2 Obtain the amount of the stake account
    let staked_amount = account.get_staked_amount();
    // 2.3. Calculate the reward amount
    let reward_amount = BigDecimal::from(staked_amount)  * e8s_to_value(reward_config.get_daily_interest_rate());
    
    
    // 3. Save stake reward records
    STAKING_REWARD_MAP.with(|map| {
      // 3.1 create a new reward record
      let reward = Self {
        id: Some(id),
        pool_id: Some(account.get_pool_id()),
        account_id: Some(account.get_id()),
        tx_id: None,
        owner: Some(account.get_owner()),
        reward_crypto: Some(account.get_reward_config().get_reward_crypto()),
        reward_amount: Some(reward_amount.to_u64().unwrap_or_default()),
        status: Some(StakingRewardStatus::Created),
        meta: Some(MetaData::init_create_scene()),
      };

      map.borrow_mut().insert(reward.get_id(), reward.clone());

      // 3.2 add index
      STAKING_POOL_REWARD_INDEX_MAP.with(|index_map| add_indexed_id(index_map, &reward.get_pool_id(), reward.get_id()));
      STAKING_ACCOUNT_REWARD_INDEX_MAP.with(|index_map| add_indexed_id(index_map, &reward.get_account_id(), reward.get_id()));
      STAKING_USER_REWARD_INDEX_MAP.with(|index_map| add_indexed_id(index_map, &reward.get_owner(), reward.get_id()));

      // 3.3 update the reward record for the account
      let updated_account = StakingAccount::update_reward(&reward, reward.get_create_at());

      (reward, updated_account)
    })
  }

  /// Update the reward record to received
  pub fn received(&self, tx_id: u64) -> Result<Self, String> {
     // Payment completed，renew reward record
    STAKING_REWARD_MAP.with(|map| {
      let mut mut_map = map.borrow_mut();
      let mut reward_record = mut_map.get(&self.get_id()).unwrap();

      reward_record.tx_id = Some(tx_id);
      reward_record.status = Some(StakingRewardStatus::Received);
      reward_record.update_meta();

      mut_map.insert(reward_record.get_id(), reward_record.clone());

      Ok(reward_record)
    })
  }

  pub fn get_pool_id(&self) -> StakingRewardId {
    self.pool_id.clone().unwrap_or_default()
  }

  pub fn get_reward_crypto(&self) -> RewardCrypto {
    self.reward_crypto.clone().unwrap_or(RewardCrypto::BONUS)
  }

  pub fn get_status(&self) -> StakingRewardStatus {
    self.status.clone().unwrap_or(StakingRewardStatus::Created)
  }

  pub fn get_owner(&self) -> UserId {
    self.owner.clone().unwrap_or_default()
  }

  pub fn get_create_at(&self) -> TimestampNanos {
    self.get_meta().get_created_at()
  }

  pub fn update_meta(&mut self) {
    self.meta = Some(self.get_meta().update());
  }

  pub fn get_id(&self) -> StakingRewardId {
    self.id.unwrap_or_default()
  }

  pub fn get_distribution_date(&self) -> YearMonthDay {
    YearMonthDay::from(self.get_meta().get_created_at())
  }

  /// Get metadata
  pub fn get_meta(&self) -> MetaData {
    self.meta.clone().unwrap_or(MetaData::init_create_scene())
  }

  pub fn get_reward_amount(&self) -> E8S {
    self.reward_amount.unwrap_or_default()
  }

  pub fn get_reward_amount_float(&self) -> f64 {
    e8s_to_value(self.get_reward_amount()).to_f64().unwrap_or_default()
  }

  pub fn get_tx_id(&self) -> u64 {
    self.tx_id.clone().unwrap_or_default()
  }

  pub fn get_account_id(&self) -> StakingAccountId {
    self.account_id.clone().unwrap_or_default()
  }
}