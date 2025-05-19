use types::staking::StakingAccountId;

use super::stable_structures::{EventLog, EventType};
use crate::account::stable_structures::StakingAccount;

/// Added staking pool event log
pub fn save_create_staking_account_event_log(account: &StakingAccount) {
  EventLog::new(EventType::CreateStakingAccount(account.clone())).save_to_stable_memory()
}

pub fn save_update_staking_account_event_log(account: &StakingAccount) {
  EventLog::new(EventType::UpdateStakingAccount(account.clone())).save_to_stable_memory()
}

pub fn save_delete_staking_account_event_log(account_id: &StakingAccountId) {
  EventLog::new(EventType::DeleteStakingAccount(*account_id)).save_to_stable_memory()
}
