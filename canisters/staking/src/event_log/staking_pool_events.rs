use types::staking::StakingPoolId;

use crate::pool::stable_structures::{StakingPool, StakingPoolStatus};

use super::stable_structures::{EventLog, EventType};

/// Added staking pool event log
pub fn save_create_staking_pool_event_log(pool: &StakingPool) {
  EventLog::new(EventType::CreateStakingPool(pool.clone())).save_to_stable_memory()
}

/// Update the staking pool
pub fn save_update_staking_pool_event_log(pool: &StakingPool) {
  EventLog::new(EventType::UpdateStakingPool(pool.clone())).save_to_stable_memory()
}

/// Update the client visible of staking pool
pub fn save_change_staking_pool_visible_event_log(id: StakingPoolId, visible: bool) {
  EventLog::new(EventType::ChangeStakingPoolClientVisible(id, visible)).save_to_stable_memory()
}

/// Update the status of staking pool
pub fn save_change_staking_pool_status_event_log(id: StakingPoolId, status: StakingPoolStatus) {
  EventLog::new(EventType::ChangeStakingPoolStatus(id, status)).save_to_stable_memory()
}
