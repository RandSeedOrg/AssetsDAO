use std::cell::RefCell;

use ic_stable_structures::{memory_manager::MemoryId, StableBTreeMap};
use stable_structures::{PoolTransactionRecords, RecordTypeIndexKey};
use types::{btree_set_entity_index::BTreeSetEntityIndex, stable_structures::Memory, staking::StakingPoolId};

use crate::{
  memory_ids::{STAKING_POOL_TRANSACTION_RECORD, STAKING_POOL_TRANSACTION_RECORD_TYPE_INDEX},
  MEMORY_MANAGER,
};

pub mod stable_structures;
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
