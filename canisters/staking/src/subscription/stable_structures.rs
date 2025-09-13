use std::{borrow::Cow, str::FromStr};

use candid::{CandidType, Decode, Encode, Principal};
use ic_stable_structures::{storable::Bound, Storable};
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};
use types::{stable_structures::new_entity_id, EntityId, TimestampNanos, UserId};

use super::{transport_structures::StakingSubscribeAddDto, STAKING_SUBSCRIPTION_ID, STAKING_SUBSCRIPTION_MAP};

/// Subscribe to scenarios
#[derive(EnumString, Display, Debug, Clone, Serialize, Deserialize, CandidType, PartialEq)]
pub enum SubscribeScene {
  /// Can be staked
  #[strum(serialize = "0")]
  CanStake,
  #[strum(serialize = "1")]
  PreSale,
}

/// Subscription history
#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct StakingSubscription {
  pub id: Option<EntityId>,
  pub user_id: Option<UserId>,
  pub business_id: Option<EntityId>,
  pub email: Option<String>,
  pub scene: Option<SubscribeScene>,
  pub created_at: Option<TimestampNanos>,
}

impl StakingSubscription {
  pub fn get_id(&self) -> EntityId {
    self.id.unwrap_or_default()
  }

  pub fn get_user_id(&self) -> UserId {
    self.user_id.clone().unwrap_or(String::from(""))
  }
  pub fn get_business_id(&self) -> EntityId {
    self.business_id.unwrap_or_default()
  }
  pub fn get_email(&self) -> String {
    self.email.clone().unwrap_or_default()
  }

  pub fn get_scene(&self) -> SubscribeScene {
    self.scene.clone().unwrap_or(SubscribeScene::CanStake)
  }

  pub fn get_created_at(&self) -> u64 {
    self.created_at.unwrap_or_default()
  }

  pub fn add_staking_subscribe(dto: &StakingSubscribeAddDto) -> Result<Self, String> {
    let id = STAKING_SUBSCRIPTION_ID.with(|id_seq| new_entity_id(id_seq));

    let current_user = crate::identity_mapping::wl_caller();

    if current_user == Principal::anonymous() {
      return Err("Anonymous user can not subscribe".to_string());
    }

    let user_id = current_user.to_text();
    let pool_id = dto.business_id.clone();
    let email = dto.email.clone();
    let scene = SubscribeScene::from_str(&dto.scene).unwrap_or(SubscribeScene::CanStake);

    let subscribe = StakingSubscription {
      id: Some(id),
      user_id: Some(user_id),
      business_id: Some(pool_id),
      email: Some(email),
      scene: Some(scene),
      created_at: Some(ic_cdk::api::time()),
    };

    STAKING_SUBSCRIPTION_MAP.with(|map| {
      let mut map = map.borrow_mut();

      map.insert(subscribe.get_id(), subscribe.clone());

      Ok(subscribe)
    })
  }
}

impl Storable for StakingSubscription {
  fn to_bytes(&self) -> Cow<[u8]> {
    Cow::Owned(Encode!(self).unwrap())
  }

  fn from_bytes(bytes: Cow<[u8]>) -> Self {
    Decode!(bytes.as_ref(), Self).unwrap()
  }

  const BOUND: Bound = Bound::Unbounded;
}
