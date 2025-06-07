use std::cell::RefCell;

use ic_cdk::{
  api::{is_controller, msg_caller},
  query, update,
};
use ic_stable_structures::{memory_manager::MemoryId, StableBTreeMap};
use stable_structures::{PoolTransactionRecord, PoolTransactionRecords, RecordTypeIndexKey, RecordTypeKey};
use transport_structures::PoolTransactionQueryParams;
use types::{
  btree_set_entity_index::BTreeSetEntityIndex,
  pagination::{PageRequest, PageResponse},
  stable_structures::Memory,
  staking::StakingPoolId,
  EntityId, TimestampNanos,
};

use crate::{
  account::{
    crud_utils::query_staking_account_with_pool_id,
    stable_structures::{StakingAccount, StakingAccountStatus},
  },
  memory_ids::{STAKING_POOL_TRANSACTION_RECORD, STAKING_POOL_TRANSACTION_RECORD_TYPE_INDEX},
  pool_transaction_record::utils::{record_stake_transaction, record_unstake_transaction},
  MEMORY_MANAGER,
};

pub mod stable_structures;
pub mod transport_structures;
pub mod utils;

thread_local! {
  /// Original data of staked account
  pub static STAKING_POOL_TRANSACTION_RECORD_MAP: RefCell<StableBTreeMap<StakingPoolId, PoolTransactionRecords, Memory>> = RefCell::new(
    StableBTreeMap::init(
      MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(STAKING_POOL_TRANSACTION_RECORD))),
    )
  );

  /// Transaction record type index
  pub static STAKING_POOL_TRANSACTION_RECORD_TYPE_INDEX_MAP: RefCell<StableBTreeMap<RecordTypeIndexKey, BTreeSetEntityIndex<RecordTypeIndexKey>, Memory>> = RefCell::new(
    StableBTreeMap::init(
      MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(STAKING_POOL_TRANSACTION_RECORD_TYPE_INDEX))),
    )
  );
}

#[query]
fn get_staking_pool_transaction_record_page(request: PageRequest<PoolTransactionQueryParams>) -> PageResponse<PoolTransactionRecord> {
  let PageRequest {
    page,
    page_size,
    params: PoolTransactionQueryParams { pool_id, record_type },
  } = request;

  if record_type.is_some() {
    let record_type_key = RecordTypeKey::from(record_type.unwrap());

    let entry_ids = STAKING_POOL_TRANSACTION_RECORD_TYPE_INDEX_MAP.with(|index_map| {
      let index_map = index_map.borrow();
      let key = RecordTypeIndexKey(pool_id, record_type_key);
      if let Some(index) = index_map.get(&key) {
        let ids = index.get_entity_ids();
        ids.iter().copied().collect::<Vec<EntityId>>()
      } else {
        vec![]
      }
    });

    if entry_ids.is_empty() {
      return PageResponse::new_empty(page, page_size);
    }

    STAKING_POOL_TRANSACTION_RECORD_MAP.with(|map| {
      let map = map.borrow();
      if let Some(records) = map.get(&pool_id) {
        records.get_page_by_ids(page, page_size, entry_ids)
      } else {
        PageResponse::new_empty(page, page_size)
      }
    })
  } else {
    STAKING_POOL_TRANSACTION_RECORD_MAP.with(|map| {
      let map = map.borrow();
      if let Some(records) = map.get(&pool_id) {
        records.get_page(page, page_size)
      } else {
        PageResponse::new_empty(page, page_size)
      }
    })
  }
}

enum AccountCategory {
  Staking,
  Unstaking,
}

struct StakingAccountSortHelper {
  pub sort_time: TimestampNanos,
  pub category: AccountCategory,
  pub account: StakingAccount,
}

#[update(hidden = true)]
fn refactor_transaction_records(pool_id: StakingPoolId) -> Option<String> {
  if !is_controller(&msg_caller()) {
    return Some("Only the controller can call this method".to_string());
  }

  let staking_accounts = query_staking_account_with_pool_id(pool_id);

  if staking_accounts.is_empty() {
    return Some("No staking accounts found for the given pool ID".to_string());
  }

  let staking_sort_helpers = staking_accounts
    .clone()
    .into_iter()
    .map(|account| {
      let sort_helper = StakingAccountSortHelper {
        sort_time: account.get_stake_time(),
        category: AccountCategory::Staking,
        account,
      };

      sort_helper
    })
    .collect::<Vec<_>>();

  let unstaking_sort_helpers = staking_accounts
    .into_iter()
    .filter(|account| account.get_status() == StakingAccountStatus::Released || account.get_status() == StakingAccountStatus::Dissolved)
    .map(|account| {
      let sort_helper = StakingAccountSortHelper {
        sort_time: account.get_release_time(),
        category: AccountCategory::Unstaking,
        account,
      };

      sort_helper
    })
    .collect::<Vec<_>>();

  let mut combined_helpers: Vec<StakingAccountSortHelper> = staking_sort_helpers.into_iter().chain(unstaking_sort_helpers.into_iter()).collect();

  combined_helpers.sort_by_key(|helper| helper.sort_time);

  STAKING_POOL_TRANSACTION_RECORD_MAP.with(|map| {
    let mut map = map.borrow_mut();
    map.remove(&pool_id);
  });
  for helper in combined_helpers {
    let account = &helper.account;
    match helper.category {
      AccountCategory::Staking => {
        record_stake_transaction(account).expect("Failed to record stake transaction");
      }
      AccountCategory::Unstaking => {
        record_unstake_transaction(account).expect("Failed to record unstake transaction");
      }
    };
  }

  None
}
