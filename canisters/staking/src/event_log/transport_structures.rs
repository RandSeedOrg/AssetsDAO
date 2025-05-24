use candid::CandidType;
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};
use types::{
  pagination::{PageRequest, PageResponse},
  TimestampNanos,
};

use super::stable_structures::{EventLog, EventType};

#[derive(EnumString, Display, Debug, Clone, Serialize, Deserialize, CandidType, PartialEq)]
pub enum EventTypeCode {
  /// Undefined event type
  #[strum(serialize = "-1")]
  Undefined,
  /// Create a new staking pool
  #[strum(serialize = "0")]
  CreateStakingPool,
  /// Update the staking pool
  #[strum(serialize = "1")]
  UpdateStakingPool,
  /// Add a staking account
  #[strum(serialize = "2")]
  CreateStakingAccount,
  /// Delete the staking account
  #[strum(serialize = "3")]
  DeleteStakingAccount,
  /// Update staking account
  #[strum(serialize = "4")]
  UpdateStakingAccount,
  /// Update the state of staking pool
  #[strum(serialize = "5")]
  ChangeStakingPoolStatus,
  /// Change client visible of staking pool
  #[strum(serialize = "6")]
  ChangeStakingPoolClientVisible,
  /// Staking
  #[strum(serialize = "7")]
  Stake,
  /// Unstake
  #[strum(serialize = "8")]
  Unstake,
  /// Dissolve staking account
  #[strum(serialize = "9")]
  Dissolve,
  /// Rewards issued
  #[strum(serialize = "10")]
  DistributeReward,
  /// Rewards are credited
  #[strum(serialize = "11")]
  RewardReceived,
}

impl EventTypeCode {
  pub fn is_match(&self, event_type: &EventType) -> bool {
    match self {
      EventTypeCode::CreateStakingPool => match event_type {
        EventType::CreateStakingPool(_) => true,
        _ => false,
      },
      EventTypeCode::UpdateStakingPool => match event_type {
        EventType::UpdateStakingPool(_) => true,
        _ => false,
      },
      EventTypeCode::CreateStakingAccount => match event_type {
        EventType::CreateStakingAccount(_) => true,
        _ => false,
      },
      EventTypeCode::DeleteStakingAccount => match event_type {
        EventType::DeleteStakingAccount(_) => true,
        _ => false,
      },
      EventTypeCode::UpdateStakingAccount => match event_type {
        EventType::UpdateStakingAccount(_) => true,
        _ => false,
      },
      EventTypeCode::ChangeStakingPoolStatus => match event_type {
        EventType::ChangeStakingPoolStatus(_, _) => true,
        _ => false,
      },
      EventTypeCode::ChangeStakingPoolClientVisible => match event_type {
        EventType::ChangeStakingPoolClientVisible(_, _) => true,
        _ => false,
      },
      EventTypeCode::Stake => match event_type {
        EventType::Stake(_, _) => true,
        _ => false,
      },
      EventTypeCode::Unstake => match event_type {
        EventType::Unstake(_, _) => true,
        _ => false,
      },
      EventTypeCode::Dissolve => match event_type {
        EventType::Dissolve(_) => true,
        _ => false,
      },
      EventTypeCode::DistributeReward => match event_type {
        EventType::DistributeReward(_, _) => true,
        _ => false,
      },
      EventTypeCode::RewardReceived => match event_type {
        EventType::RewardReceived(_) => true,
        _ => false,
      },
      EventTypeCode::Undefined => true,
    }
  }
}

#[derive(EnumString, Display, Debug, Clone, Serialize, Deserialize, CandidType, PartialEq)]
pub enum SortType {
  /// Ascending order
  #[strum(serialize = "0")]
  Asc,
  /// descending order
  #[strum(serialize = "1")]
  Desc,
}

#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct EventLogQueryParams {
  /** Event Type */
  pub event_type: String,
  /** Start time */
  pub start_time: TimestampNanos,
  /** End time */
  pub end_time: TimestampNanos,
}

pub type StakingEventLogPageRequest = PageRequest<EventLogQueryParams>;
pub type StakingEventLogPageResponse = PageResponse<EventLog>;
