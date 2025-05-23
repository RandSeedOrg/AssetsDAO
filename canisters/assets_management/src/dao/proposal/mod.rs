use std::cell::RefCell;

use ic_stable_structures::{Cell, StableBTreeMap};
use stable_structures::Proposal;
use types::{assets_management::ProposalId, stable_structures::Memory, EntityId};

use crate::{
  memory_ids::{PROPOSAL_MAP_MEMORY_ID, PROPOSAL_SEQ_MEMORY_ID},
  MEMORY_MANAGER,
};

pub mod crud;
pub mod stable_structures;
pub mod transport_structures;

thread_local! {
  /// Proposal sequence stable storage
  pub static PROPOSAL_ID: RefCell<Cell<EntityId, Memory>> = RefCell::new(Cell::init(MEMORY_MANAGER.with(|m| m.borrow().get(PROPOSAL_SEQ_MEMORY_ID)), 0_u64).unwrap());

  /// Proposal stable storage
  pub static PROPOSAL_MAP: RefCell<StableBTreeMap<ProposalId, Proposal, Memory>> = RefCell::new(
    StableBTreeMap::init(
      MEMORY_MANAGER.with(|m| m.borrow().get(PROPOSAL_MAP_MEMORY_ID)),
    )
  );
}
