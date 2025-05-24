use std::{cell::RefCell, str::FromStr};

use ic_cdk::api::msg_caller;
use ic_stable_structures::{memory_manager::MemoryId, Cell, StableBTreeMap};
use stable_structures::{StakingSubscription, SubscribeScene};
use system_configs_macro::has_permission;
use transport_structures::{StakingSubscribeAddDto, StakingSubscriptionVo, SubscriptionQueryParams, SubscriptionRequest, SubscriptionResponse};
use types::{stable_structures::Memory, staking::SubscriptionId, EntityId};

use crate::{
  event_log::transport_structures::SortType,
  memory_ids::{STAKING_SUBSCRIPTION, STAKING_SUBSCRIPTION_SEQ},
  MEMORY_MANAGER,
};

pub mod stable_structures;
pub mod transport_structures;

thread_local! {
  /// stake subscriptions are added IDGenerator
  pub static STAKING_SUBSCRIPTION_ID: RefCell<Cell<EntityId, Memory>> = RefCell::new(Cell::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(STAKING_SUBSCRIPTION_SEQ))), 0_u64).unwrap());

  /// Stake the original data subscription
  pub static STAKING_SUBSCRIPTION_MAP: RefCell<StableBTreeMap<SubscriptionId, StakingSubscription, Memory>> = RefCell::new(
    StableBTreeMap::init(
      MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(STAKING_SUBSCRIPTION))),
    )
  );
}

/// CAdd subscription notification message to the end
#[ic_cdk::update]
fn subscribe_notification(dto: StakingSubscribeAddDto) -> Option<String> {
  match StakingSubscription::add_staking_subscribe(&dto) {
    Ok(_) => None,
    Err(err) => Some(format!("Error creating Staking Subscribe: {}", err)),
  }
}

/// CCheck whether the current user has subscribed to a certain scenario notification
#[ic_cdk::query]
fn query_current_user_subscribed(scene: String) -> bool {
  let current_user = msg_caller().to_text();
  let scene = SubscribeScene::from_str(&scene).unwrap_or(SubscribeScene::CanStake);

  STAKING_SUBSCRIPTION_MAP.with(|map| {
    map
      .borrow()
      .iter()
      .any(|(_, subscribe)| subscribe.get_user_id() == current_user && subscribe.get_scene() == scene)
  })
}

/// Back-management paging query subscription list interface
#[ic_cdk::query]
#[has_permission("staking::subscription::query")]
fn query_subscriptions(request: SubscriptionRequest) -> SubscriptionResponse {
  let SubscriptionRequest {
    page,
    page_size,
    params: SubscriptionQueryParams {
      scene,
      email,
      user_id,
      id_sort,
      start_time,
      end_time,
    },
  } = request;

  let id_sort = SortType::from_str(&id_sort).unwrap_or(SortType::Desc);
  let subscribe_scene = match SubscribeScene::from_str(&scene) {
    Ok(scene) => Some(scene),
    Err(_) => None,
  };

  STAKING_SUBSCRIPTION_MAP.with(|map| {
    let map = map.borrow();

    let subscriptions: Vec<StakingSubscription> = map
      .values()
      .into_iter()
      .filter(|subscribe| {
        // Filter subscription scenarios
        if subscribe_scene.is_some() {
          subscribe.get_scene().to_string() == scene
        } else {
          true
        }
      })
      .filter(|subscribe| {
        // Filter subscription email
        if email.len() > 0 {
          subscribe.get_email().contains(&email)
        } else {
          true
        }
      })
      .filter(|subscribe| {
        // Filter subscribers id
        if user_id.len() > 0 {
          subscribe.get_user_id().contains(&user_id)
        } else {
          true
        }
      })
      .filter(|subscribe| {
        // Filter subscription time
        if start_time > 0 && end_time > 0 {
          subscribe.get_created_at() >= start_time && subscribe.get_created_at() <= end_time
        } else {
          true
        }
      })
      .collect();

    let total = subscriptions.len() as u32;
    let start: u32 = (page - 1) * page_size;

    let subscriptions = match id_sort {
      SortType::Asc => subscriptions
        .iter()
        .skip(start as usize)
        .take(page_size as usize)
        .map(|subscription| StakingSubscriptionVo::from_stable(subscription))
        .collect(),
      SortType::Desc => subscriptions
        .iter()
        .rev()
        .skip(start as usize)
        .take(page_size as usize)
        .map(|subscription| StakingSubscriptionVo::from_stable(subscription))
        .collect(),
    };

    SubscriptionResponse {
      total,
      page,
      page_size,
      records: subscriptions,
    }
  })
}
