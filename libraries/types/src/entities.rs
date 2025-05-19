use std::{borrow::Cow, cell::RefCell};

use candid::{CandidType, Decode, Encode};
use ic_stable_structures::{StableBTreeMap, Storable, storable::Bound};
use serde::{Deserialize, Serialize};

use crate::{EntityId, stable_structures::Memory};

/// Common entity index data structure
#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct EntityIndex<T: CandidType> {
  pub id: Option<T>,
  pub entity_ids: Option<Vec<EntityId>>,
}

impl<T: CandidType + Serialize + for<'a> Deserialize<'a>> Storable for EntityIndex<T> {
  fn to_bytes(&self) -> Cow<[u8]> {
    Cow::Owned(Encode!(self).unwrap())
  }

  fn from_bytes(bytes: Cow<[u8]>) -> Self {
    Decode!(bytes.as_ref(), Self).unwrap()
  }

  const BOUND: Bound = Bound::Unbounded;
}

impl<T: CandidType> EntityIndex<T> {
  pub fn new(id: T, entity_ids: Vec<EntityId>) -> Self {
    Self {
      id: Some(id),
      entity_ids: Some(entity_ids),
    }
  }

  pub fn add_entity_id(&mut self, entity_id: EntityId) {
    if self.has_entity_id(&entity_id) {
      return;
    }

    if let Some(ref mut ids) = self.entity_ids {
      ids.push(entity_id);
    } else {
      self.entity_ids = Some(vec![entity_id]);
    }
  }

  pub fn remove_entity_id(&mut self, entity_id: &EntityId) {
    if let Some(ref mut ids) = self.entity_ids {
      ids.retain(|&id| id != *entity_id);
    }
  }

  pub fn has_entity_id(&self, entity_id: &EntityId) -> bool {
    if let Some(ref ids) = self.entity_ids {
      ids.contains(entity_id)
    } else {
      false
    }
  }

  pub fn get_entity_ids(&self) -> Vec<EntityId> {
    self.entity_ids.clone().unwrap_or_default()
  }

  pub fn is_empty(&self) -> bool {
    self.get_entity_ids().is_empty()
  }
}

impl<T: CandidType> Default for EntityIndex<T> {
  fn default() -> Self {
    EntityIndex {
      id: None,
      entity_ids: Some(vec![]), // Default to an empty vector
    }
  }
}

impl<T: CandidType + Default + Clone> EntityIndex<T> {
  pub fn get_id(&self) -> T {
    self.id.clone().unwrap_or_default()
  }
}

/// Get the id in the index, tool function
pub fn get_indexed_ids<T>(stable_map: &RefCell<StableBTreeMap<T, EntityIndex<T>, Memory>>, key: &T) -> Vec<EntityId>
where
  T: CandidType + Storable + Ord + Clone + Serialize + for<'a> Deserialize<'a>,
{
  stable_map.borrow().get(&key).map_or_else(|| vec![], |index| index.get_entity_ids())
}

/// Add a new entity id to the id in the index, tool function
pub fn add_indexed_id<T>(stable_map: &RefCell<StableBTreeMap<T, EntityIndex<T>, Memory>>, key: &T, entity_id: EntityId)
where
  T: CandidType + Storable + Ord + Clone + Serialize + for<'a> Deserialize<'a>,
{
  let mut map = stable_map.borrow_mut();

  let mut index = match map.get(&key) {
    Some(index) => index,
    None => EntityIndex::new(key.clone(), vec![]),
  };

  index.add_entity_id(entity_id);

  map.insert(key.clone(), index.clone());
}

/// Delete the id in the index, tool function
pub fn remove_indexed_id<T>(stable_map: &RefCell<StableBTreeMap<T, EntityIndex<T>, Memory>>, key: &T, entity_id: EntityId)
where
  T: CandidType + Storable + Ord + Clone + Serialize + for<'a> Deserialize<'a>,
{
  let mut map = stable_map.borrow_mut();

  let mut index = match map.get(&key) {
    Some(index) => index,
    None => EntityIndex::new(key.clone(), vec![]),
  };

  index.remove_entity_id(&entity_id);

  map.insert(key.clone(), index.clone());
}
