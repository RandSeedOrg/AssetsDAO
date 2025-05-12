use candid::CandidType;
use serde::{Deserialize, Serialize};
use types::{pagination::{PageRequest, PageResponse}, TimestampNanos, UserId};

use crate::EntityId;

use super::stable_structures::StakingSubscription;


/// Stake Subscription
#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct StakingSubscriptionVo {
  pub id: EntityId,
  pub user_id: UserId,
  pub business_id: EntityId,
  pub email: String,
  pub scene: String,
  pub created_at: TimestampNanos,
}

impl StakingSubscriptionVo {
  pub fn from_stable(entity: &StakingSubscription) -> Self {
    Self {
      id: entity.get_id(),
      user_id: entity.get_user_id(),
      business_id: entity.get_business_id(),
      email: entity.get_email(),
      scene: entity.get_scene().to_string(),
      created_at: entity.get_created_at(),
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct StakingSubscribeAddDto {
  /// businessID
  pub business_id: EntityId,
  /// Subscribe to email
  pub email: String,
  /// Subscribe to scenarios: refer to SubscribeScene
  /// Can be staked: 0
  /// Pre-sale: 1
  pub scene: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct SubscriptionQueryParams {
  /// Subscribe to scenarios: refer to SubscribeScene
  /// Can be staked: 0
  /// Pre-sale: 1
  pub scene: String,
  /// userID
  pub user_id: UserId,
  /// Mail
  pub email: String,
  /// IDSort
  pub id_sort: String,
  /// Start time
  pub start_time: TimestampNanos,
  /// End time
  pub end_time: TimestampNanos,
}

pub type SubscriptionRequest = PageRequest<SubscriptionQueryParams>;
pub type SubscriptionResponse = PageResponse<StakingSubscriptionVo>;