use crate::event_log::staking_pool_events::save_create_staking_pool_event_log;

use super::{stable_structures::StakingPool, STAKING_POOL_MAP};

/// Added a stake pool
pub fn add_staking_pool_to_stable_memory(pool: &StakingPool) -> Option<String> {
  STAKING_POOL_MAP.with(|map| {
    let mut map = map.borrow_mut();

    let exist_pool = map.get(&pool.get_id());

    if exist_pool.is_some() {
      return Some(format!("Staking pool with ID {} already exists", pool.get_id()));
    }
    // Save the stake pool to stable memory
    map.insert(pool.get_id(), pool.clone());
    // Save the new staking pool event log to stable memory
    save_create_staking_pool_event_log(&pool);
    None
  })
}

/// Query all stake pool information visible to all clients
pub fn query_client_visible_staking_pools() -> Vec<StakingPool> {
  STAKING_POOL_MAP.with(|map| {
    let map = map.borrow();
    map.iter()
      .filter(|(_, pool)| {
        pool.is_client_visible()
      })
      .map(|(_, pool)| pool.clone())
      .collect()
  })
}

// According to the stake poolIDQuery stake pool information
pub fn query_staking_pool_by_id(pool_id: u64) -> Result<StakingPool, String> {
  STAKING_POOL_MAP.with(|map| {
    let map = map.borrow();
    map.get(&pool_id).ok_or_else(|| format!("Staking pool with ID {} not found", pool_id))
  })
}