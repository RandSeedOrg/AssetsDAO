use std::cell::RefCell;

use ic_stable_structures::{memory_manager::MemoryId, Cell, StableBTreeMap};
use stable_structures::StakingAccount;
use system_configs_macro::has_permission;
use transport_structures::{StakingAccountPageRequest, StakingAccountPageResponse, StakingAccountQueryParams, StakingAccountVo};
use types::{date::YearMonthDay, entities::{get_indexed_ids, EntityIndex}, stable_structures::Memory, EntityId, UserId};

use crate::{memory_ids::{STAKING_ACCOUNT, STAKING_ACCOUNT_SEQ, STAKING_POOL_ACCOUNT_INDEX, STAKING_RECOVERABLE_ERROR_ACCOUNT_INDEX, STAKING_UNSTAKE_ON_DAY_ACCOUNT_INDEX, STAKING_USER_ACCOUNT_INDEX}, StakingAccountId, StakingPoolId, MEMORY_MANAGER};

pub mod stable_structures;
pub mod crud_utils;
pub mod transport_structures;
pub mod client_api;
pub mod client_transport_structures;
pub mod operation_utils;
pub mod guard_keys;
pub mod recovery_errors;

thread_local! {
  /// stake account increasesIDGenerator
  pub static STAKING_ACCOUNT_ID: RefCell<Cell<EntityId, Memory>> = RefCell::new(Cell::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(STAKING_ACCOUNT_SEQ))), 0_u64).unwrap());

  /// Original data of staked account
  pub static STAKING_ACCOUNT_MAP: RefCell<StableBTreeMap<StakingAccountId, StakingAccount, Memory>> = RefCell::new(
    StableBTreeMap::init(
      MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(STAKING_ACCOUNT))),
    )
  );

  /// Index of user staked accounts
  pub static STAKING_USER_ACCOUNT_INDEX_MAP: RefCell<StableBTreeMap<UserId, EntityIndex<UserId>, Memory>> = RefCell::new(
    StableBTreeMap::init(
      MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(STAKING_USER_ACCOUNT_INDEX))),
    )
  );

  /// stake pool stake account index
  pub static STAKING_POOL_ACCOUNT_INDEX_MAP: RefCell<StableBTreeMap<StakingPoolId, EntityIndex<StakingPoolId>, Memory>> = RefCell::new(
    StableBTreeMap::init(
      MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(STAKING_POOL_ACCOUNT_INDEX))),
    )
  );

  /// A staked account index was generated that could recover errors
  pub static STAKING_RECOVERABLE_ERROR_ACCOUNT_INDEX_MAP: RefCell<StableBTreeMap<StakingPoolId, EntityIndex<StakingPoolId>, Memory>> = RefCell::new(
    StableBTreeMap::init(
      MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(STAKING_RECOVERABLE_ERROR_ACCOUNT_INDEX))),
    )
  );

  /// Accounts that are staked and indexed by expiration date
  pub static STAKING_UNSTAKE_ON_DAY_ACCOUNT_INDEX_MAP: RefCell<StableBTreeMap<YearMonthDay, EntityIndex<YearMonthDay>, Memory>> = RefCell::new(
    StableBTreeMap::init(
      MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(STAKING_UNSTAKE_ON_DAY_ACCOUNT_INDEX))),
    )
  );
}

/// Paging query stake account list
#[ic_cdk::query]
#[has_permission("staking::account::query")]
fn query_staking_accounts(request: StakingAccountPageRequest) -> StakingAccountPageResponse {
  let StakingAccountPageRequest { 
    page, 
    page_size, 
    params: StakingAccountQueryParams { 
      pool_id, 
      user_id, 
      onchain_address, 
      status 
    } 
  } = request;

  // List of accounts filtered by conditions
  let filtered_accounts: Vec<StakingAccount> = if pool_id > 0 {
    let account_ids = STAKING_POOL_ACCOUNT_INDEX_MAP.with(|map| get_indexed_ids(map, &pool_id));

    STAKING_ACCOUNT_MAP.with(|map| {
      let map = map.borrow();
      account_ids
        .iter()
        .filter_map(|account_id| {
          map.get(account_id)
        })
        .filter(|account| {
          if status.len() > 0 {
            account.get_status().to_string() == status
          } else {
            true
          }
        })
        .filter(|account| {
          if user_id.len() > 0 {
            account.get_owner().contains(&user_id)
          } else {
            true
          }
        })
        .filter(|account| {
          if onchain_address.len() > 0 {
            account.get_onchain_address().contains(&onchain_address)
          } else {
            true
          }
        })
        .collect()
    })
  } else {
    STAKING_ACCOUNT_MAP.with(|map| {
      let map = map.borrow();
      
      map.iter()
        .map(|(_, account)| account.clone())
        .filter(|account| {
          if status.len() > 0 {
            account.get_status().to_string() == status
          } else {
            true
          }
        })
        .filter(|account| {
          if user_id.len() > 0 {
            account.get_owner().contains(&user_id)
          } else {
            true
          }
        })
        .filter(|account| {
          if onchain_address.len() > 0 {
            account.get_onchain_address().contains(&onchain_address)
          } else {
            true
          }
        })
        .collect()
    })
  };

  let total = filtered_accounts.len() as u32;
  let start = (page - 1) * page_size;

  let records = filtered_accounts
    .iter()
    .rev()
    .skip(start as usize)
    .take(page_size as usize)
    .map(|account| {
      StakingAccountVo::from_staking_account(account)
    })
    .collect::<Vec<StakingAccountVo>>();

  StakingAccountPageResponse {
    total,
    page: page as u32,
    page_size: page_size as u32,
    records,
  } 
}