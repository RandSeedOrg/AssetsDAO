use crate::{account::stable_structures::StakingAccount, reward::stable_structures::StakingReward};

use super::stable_structures::{EventLog, EventType};

/// Reward event log
pub fn save_reward_distribute_event(reward: &StakingReward, account: &StakingAccount) {
  EventLog::new(EventType::DistributeReward(reward.clone(), account.clone()))
    .save_to_stable_memory()
}

/// Rewards Incident
pub fn save_reward_received_event(reward: &StakingReward) {
  EventLog::new(EventType::RewardReceived(reward.clone()))
    .save_to_stable_memory()
}