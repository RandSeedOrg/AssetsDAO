use crate::{account::stable_structures::StakingAccount, pool::stable_structures::StakingPool};

use super::stable_structures::{EventLog, EventType};

/// Event log information during stake
pub fn save_stake_event(staking_pool: &StakingPool, staking_account: &StakingAccount) {
  EventLog::new(EventType::Stake(staking_pool.clone(), staking_account.clone())).save_to_stable_memory()
}

/// Event log information when destaking
pub fn save_unstake_event(staking_pool: &StakingPool, staking_account: &StakingAccount) {
  EventLog::new(EventType::Unstake(staking_pool.clone(), staking_account.clone())).save_to_stable_memory()
}

/// Event log when dissolving a staked account
pub fn save_dissolve_event(staking_account: &StakingAccount) {
  EventLog::new(EventType::Dissolve(staking_account.clone())).save_to_stable_memory()
}
