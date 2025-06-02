use std::cell::RefCell;

use ic_cdk::query;
use ic_stable_structures::{memory_manager::MemoryId, StableBTreeMap};
use stable_structures::{PoolTransactionRecord, PoolTransactionRecords, RecordTypeIndexKey, RecordTypeKey};
use transport_structures::PoolTransactionQueryParams;
use types::{
  btree_set_entity_index::BTreeSetEntityIndex,
  pagination::{PageRequest, PageResponse},
  stable_structures::Memory,
  staking::StakingPoolId,
  EntityId,
};

use crate::{
  memory_ids::{STAKING_POOL_TRANSACTION_RECORD, STAKING_POOL_TRANSACTION_RECORD_TYPE_INDEX},
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
