use types::{UserId, staking::StakingAccountId};

/// Obtain the key for stake entrance
pub fn get_stake_guard_key(user_id: UserId) -> String {
  format!("stake_guard_{}", user_id)
}

/// Obtain the guard key for de-pending entry
pub fn get_unstake_guard_key(account_id: StakingAccountId) -> String {
  format!("unstake_guard_{}", account_id)
}

/// Get the key for dissolving the entrance guard
pub fn get_dissolve_guard_key(account_id: StakingAccountId) -> String {
  format!("dissolve_guard_{}", account_id)
}

/// Obtain the guard key for stake account stake recovery entry
pub fn get_recovery_stake_guard_key(account_id: StakingAccountId) -> String {
  format!("recovery_stake_guard_{}", account_id)
}

/// Obtain the key for the stake account dissolution recovery entry
pub fn get_recovery_dissolve_guard_key(account_id: StakingAccountId) -> String {
  format!("recovery_dissolve_guard_{}", account_id)
}

/// Obtain the guardian key for the issuance of rewards for staking account
pub fn get_distribute_reward_guard_key(account_id: StakingAccountId) -> String {
  format!("distribute_reward_guard_{}", account_id)
}

/// Obtain the guard key for unsolicited account recovery entry
pub fn get_recovery_unstake_penalty_guard_key(account_id: StakingAccountId) -> String {
  format!("recovery_unstake_penalty_guard_{}", account_id)
}
