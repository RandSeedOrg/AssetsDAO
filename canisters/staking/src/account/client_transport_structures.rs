use candid::CandidType;
use serde::{Deserialize, Serialize};
use types::{E8S, staking::StakingPoolId};

/// The client that initiates the stake requests the data transfer object
#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct StakeDto {
  pub pool_id: StakingPoolId,
  pub staking_amount: E8S,
  pub staking_days: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct EarlyUnstakePreCheckVo {
  pub pool_id: StakingPoolId,
  pub staked_amount: E8S,
  pub penalty_amount: E8S,
  pub released_amount: E8S,
  pub accumulated_rewards: E8S,
}
