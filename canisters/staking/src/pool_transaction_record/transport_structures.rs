use candid::CandidType;
use serde::{Deserialize, Serialize};
use types::staking::StakingPoolId;

#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct PoolTransactionQueryParams {
  pub pool_id: StakingPoolId,
  pub record_type: Option<u8>,
}
