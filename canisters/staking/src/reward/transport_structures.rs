use candid::CandidType;
use serde::{Deserialize, Serialize};
use types::{pagination::{PageRequest, PageResponse}, TimestampNanos, E8S};

use crate::{StakingAccountId, StakingPoolId, StakingRewardId};

use super::stable_structures::StakingReward;


#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct StakingRewardVo {
  pub id: StakingRewardId,
  pub pool_id: StakingAccountId,
  pub account_id: StakingAccountId,
  pub tx_id: u64,
  pub owner: String,
  pub reward_crypto: String,
  pub reward_amount: E8S,
  pub status: String,
  pub created_at: TimestampNanos,
  pub updated_at: TimestampNanos,
}

#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct StakingRewardQueryParams {
  pub pool_id: StakingPoolId,
  pub account_id: StakingAccountId,
  pub user_id: String,
  pub status: String,
  pub start_time: TimestampNanos,
  pub end_time: TimestampNanos,
}


pub type StakingRewardPageRequest = PageRequest<StakingRewardQueryParams>;
pub type StakingRewardPageResponse = PageResponse<StakingRewardVo>;


impl StakingRewardVo {
  pub fn from_staking_reward(reward: &StakingReward) -> Self {
    let  meta = reward.get_meta();
    Self {
      id: reward.id.unwrap_or_default(),
      pool_id: reward.pool_id.unwrap_or_default(),
      account_id: reward.account_id.unwrap_or_default(),
      tx_id: reward.tx_id.unwrap_or_default(),
      owner: reward.owner.clone().unwrap_or_default(),
      reward_crypto: reward.get_reward_crypto().to_string(),
      reward_amount: reward.get_reward_amount(),
      status: reward.get_status().to_string(),
      created_at: meta.get_created_at(),
      updated_at: meta.get_updated_at(),
    }
  }
    
}