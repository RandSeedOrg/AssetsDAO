use std::cell::RefCell;

use ic_cdk::query;
use ic_stable_structures::{Cell, StableBTreeMap};
use types::{assets_management::TransferAddressId, stable_structures::Memory, EntityId};

use crate::{
  memory_ids::{TRANSFER_ADDRESS_MEMORY_ID, TRANSFER_ADDRESS_SEQ_MEMORY_ID},
  transfer_address::{stable_structures::TransferAddress, transfer_structures::TransferAddressVo},
  MEMORY_MANAGER,
};

pub mod stable_structures;
pub mod transfer_structures;

thread_local! {
  /// Transfer address sequence stable storage
  pub static TRANSFER_ADDRESS_ID: RefCell<Cell<EntityId, Memory>> = RefCell::new(Cell::init(MEMORY_MANAGER.with(|m| m.borrow().get(TRANSFER_ADDRESS_SEQ_MEMORY_ID)), 0_u64).unwrap());

  /// Transfer address stable storage
  pub static TRANSFER_ADDRESS_MAP: RefCell<StableBTreeMap<TransferAddressId, TransferAddress, Memory>> = RefCell::new(
    StableBTreeMap::init(
      MEMORY_MANAGER.with(|m| m.borrow().get(TRANSFER_ADDRESS_MEMORY_ID)),
    )
  );
}

#[query]
fn query_transfer_addresses() -> Vec<TransferAddressVo> {
  TRANSFER_ADDRESS_MAP.with(|map| map.borrow().values().into_iter().map(|address| address.into()).collect())
}
