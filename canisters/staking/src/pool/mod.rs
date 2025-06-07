use std::{cell::RefCell, str::FromStr};

use crud_utils::add_staking_pool_to_stable_memory;
use ic_stable_structures::{memory_manager::MemoryId, Cell, StableBTreeMap};
use stable_structures::{StakingPool, StakingPoolStatus};
use system_configs_macro::{has_permission, has_permission_option};
use transport_structures::{StakingPoolAddDto, StakingPoolUpdateDto, StakingPoolVo};
use types::{stable_structures::Memory, staking::StakingPoolId, EntityId};

use crate::{
  event_log::staking_pool_events::{
    save_change_staking_pool_status_event_log, save_change_staking_pool_visible_event_log, save_update_staking_pool_event_log,
  },
  memory_ids::{STAKING_POOL, STAKING_POOL_SEQ},
  on_chain::address::{generate_staking_pool_account_identifier, generate_staking_pool_neuron_account},
  pool::transport_structures::StakingPoolAccountIds,
  MEMORY_MANAGER,
};

pub mod client_api;
pub mod client_transport_structures;
pub mod crud_utils;
pub mod stable_structures;
pub mod transport_structures;

thread_local! {
  /// The stake pool increases automaticallyIDGenerator
  pub static STAKING_POOL_ID: RefCell<Cell<EntityId, Memory>> = RefCell::new(Cell::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(STAKING_POOL_SEQ))), 0_u64).unwrap());

  /// Original data of the stake pool
  pub static STAKING_POOL_MAP: RefCell<StableBTreeMap<EntityId, StakingPool, Memory>> = RefCell::new(
    StableBTreeMap::init(
      MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(STAKING_POOL))),
    )
  );
}

/// Added a stake pool
#[ic_cdk::update]
#[has_permission_option("staking::pool::add")]
fn add_staking_pool(pool: StakingPoolAddDto) -> Option<String> {
  let staking_pool = StakingPool::from_add_dto(&pool);

  // Save the staking pool to stable memory
  add_staking_pool_to_stable_memory(&staking_pool)
}

/// Query all stake pools
#[ic_cdk::query]
#[has_permission("staking::pool::query")]
fn get_all_staking_pools() -> Vec<StakingPoolVo> {
  STAKING_POOL_MAP.with(|map| {
    let map = map.borrow();
    map.iter().map(|(_, pool)| StakingPoolVo::from_staking_pool(&pool)).collect()
  })
}

#[ic_cdk::update]
#[has_permission_option("staking::pool::update")]
fn set_staking_pool_client_visible(id: StakingPoolId, visible: bool) -> Option<String> {
  STAKING_POOL_MAP.with(|map| {
    let mut map = map.borrow_mut();
    let pool = map.get(&id);

    if pool.is_none() {
      return Some("Staking poll not found".to_string()); // Pool not found
    }

    let mut pool = pool.unwrap();

    let result = pool.set_client_visible(visible);

    if result.is_some() {
      return result; // If there was an error in setting visibility, return the error message
    }

    map.insert(pool.get_id(), pool.clone());

    // Save update stake pool event log to stable memory
    save_change_staking_pool_visible_event_log(id, visible);

    None
  })
}

#[ic_cdk::update]
#[has_permission_option("staking::pool::update")]
fn set_staking_pool_status(id: StakingPoolId, status: String) -> Option<String> {
  STAKING_POOL_MAP.with(|map| {
    let mut map = map.borrow_mut();
    let pool = map.get(&id);

    if pool.is_none() {
      return Some("Staking pool not found".to_string()); // Pool not found
    }

    let mut pool = pool.unwrap();

    match StakingPoolStatus::from_str(&status) {
      Ok(new_status) => {
        let result = pool.set_status(new_status);

        if result.is_some() {
          return result; // If there was an error in setting status, return the error message
        }

        map.insert(pool.get_id(), pool.clone());

        save_change_staking_pool_status_event_log(pool.get_id(), pool.get_status());
        None
      }
      Err(_) => Some("Invalid status".to_string()), // Invalid status string
    }
  })
}

#[ic_cdk::update]
#[has_permission_option("staking::pool::update")]
fn update_staking_pool(dto: StakingPoolUpdateDto) -> Option<String> {
  STAKING_POOL_MAP.with(|map| {
    let mut map = map.borrow_mut();
    let existing_pool = map.get(&dto.id);

    if existing_pool.is_none() {
      return Some("Staking pool not found".to_string()); // Pool not found
    }

    let mut staking_pool = existing_pool.unwrap();

    // Update the pool with new values from the DTO
    let result = staking_pool.update(&dto.add_dto);

    if result.is_some() {
      return result; // If there was an error in updating, return the error message
    }

    map.insert(staking_pool.get_id(), staking_pool.clone());

    save_update_staking_pool_event_log(&staking_pool);
    None
  })
}

#[ic_cdk::query]
fn query_pool_account_ids(pool_id: StakingPoolId) -> StakingPoolAccountIds {
  let nns_neuron_account_id = generate_staking_pool_neuron_account(pool_id).to_hex();
  let staking_pool_account_id = generate_staking_pool_account_identifier(pool_id).to_hex();

  StakingPoolAccountIds {
    pool_id,
    nns_neuron_account_id,
    staking_pool_account_id,
  }
}
