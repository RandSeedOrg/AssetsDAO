use std::borrow::Cow;
use std::cell::RefCell;
use ic_cdk::api::{time, msg_caller};

use candid::{Decode, Encode};
use ic_stable_structures::memory_manager::VirtualMemory;
use ic_stable_structures::storable::Bound;
use ic_stable_structures::Cell;
use ic_stable_structures::DefaultMemoryImpl;
use serde::{Serialize, Deserialize};
use candid::CandidType;

use crate::EntityId;
use crate::Timestamp;
use crate::UserId;
use ic_stable_structures::Storable;

#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct MetaData {
  pub created_at: Option<Timestamp>,
  pub updated_at: Option<Timestamp>,
  pub created_by: Option<UserId>,
  pub updated_by: Option<UserId>,
}

impl Storable for MetaData {
  fn to_bytes(&self) -> Cow<[u8]> {
      Cow::Owned(Encode!(self).unwrap())
  }

  fn from_bytes(bytes: Cow<[u8]>) -> Self {
      Decode!(bytes.as_ref(), Self).unwrap()
  }

  const BOUND: Bound = Bound::Unbounded;
}

impl Default for MetaData {
  fn default() -> Self {
    MetaData {
      created_at: None,
      updated_at: None,
      created_by: None,
      updated_by: None,
    }
  }
}

impl MetaData {
  pub fn init_create_scene() -> Self {
    MetaData {
      created_at: Some(time()),
      updated_at: None,
      created_by: Some(msg_caller().to_string()),
      updated_by: None,
    }
  }
  
  pub fn update(&self) -> Self {
    MetaData {
      created_at: self.created_at.clone(),
      updated_at: Some(time()),
      created_by: self.created_by.clone(),
      updated_by: Some(msg_caller().to_string()),
    }
  } 

  pub fn get_created_at(&self) -> Timestamp {
    self.created_at.unwrap_or_default()
  }

  pub fn get_updated_at(&self) -> Timestamp {
    self.updated_at.unwrap_or_default()
  }

  pub fn get_created_by(&self) -> UserId {
    self.created_by.clone().unwrap_or_default()
  }
  
  pub fn get_updated_by(&self) -> UserId {
    self.updated_by.clone().unwrap_or_default()
  }
}

pub type Memory = VirtualMemory<DefaultMemoryImpl>;
pub type EntityIdGenerator = RefCell<Cell<EntityId, Memory>>;

pub fn new_entity_id(id_seq: &EntityIdGenerator) -> EntityId {
  let id = id_seq.borrow().get() + 1;
  if let Err(e) = id_seq.borrow_mut().set(id) {
    ic_cdk::println!("Failed to set product ID: {:?}", e);
  }
  id
}

pub fn update_entity_id(id_seq: &EntityIdGenerator, id: EntityId) -> () {
  let seq = *id_seq.borrow().get();
  if id > seq {
    if let Err(e) = id_seq.borrow_mut().set(id) {
      ic_cdk::println!("Failed to set product ID: {:?}", e);
    }
  }
}