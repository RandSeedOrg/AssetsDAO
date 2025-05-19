use std::borrow::Cow;

use candid::{CandidType, Decode, Encode};
use ic_stable_structures::{Storable, storable::Bound};
use serde::{Deserialize, Serialize};

use crate::{
  EntityId,
  stable_structures::{EntityIdGenerator, new_entity_id},
};

use super::{
  DictCode,
  transfer_structures::{AddDictDto, DictItemVo, DictItemsDto, DictVo, UpdateDictDto},
};

#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct Dict {
  pub id: Option<EntityId>,
  pub name: Option<String>,
  pub code: Option<DictCode>,
  pub description: Option<String>,
  pub items: Option<Vec<DictItem>>,
}

impl Storable for Dict {
  fn to_bytes(&self) -> Cow<[u8]> {
    Cow::Owned(Encode!(self).unwrap())
  }

  fn from_bytes(bytes: Cow<[u8]>) -> Self {
    Decode!(bytes.as_ref(), Self).unwrap()
  }

  const BOUND: Bound = Bound::Unbounded;
}

impl Dict {
  pub fn new(id_gen: &EntityIdGenerator, add_dto: AddDictDto) -> Self {
    let id = new_entity_id(id_gen);

    Dict {
      id: Some(id),
      name: Some(add_dto.name),
      code: Some(add_dto.code),
      description: Some(add_dto.description),
      items: Some(vec![]),
    }
  }

  pub fn from_update_dto(update_dto: &UpdateDictDto) -> Self {
    Dict {
      id: Some(update_dto.id),
      name: Some(update_dto.name.clone()),
      code: Some(update_dto.code.clone()),
      description: Some(update_dto.description.clone()),
      items: Some(vec![]),
    }
  }

  pub fn update(&self, update_dto: UpdateDictDto) -> Self {
    Dict {
      id: self.id,
      name: Some(update_dto.name),
      code: Some(update_dto.code),
      description: Some(update_dto.description),
      items: self.items.clone(),
    }
  }

  pub fn update_items(&self, items_dto: DictItemsDto) -> Self {
    Dict {
      id: self.id,
      name: self.name.clone(),
      code: self.code.clone(),
      description: self.description.clone(),
      items: Some(
        items_dto
          .items
          .iter()
          .map(|item| DictItem {
            label: Some(item.label.clone()),
            value: Some(item.value.clone()),
            description: Some(item.description.clone()),
            sort: Some(item.sort),
          })
          .collect::<Vec<_>>(),
      ),
    }
  }

  pub fn to_vo(&self) -> DictVo {
    DictVo {
      id: self.id.unwrap_or_default(),
      name: self.name.clone().unwrap_or_default(),
      code: self.code.clone().unwrap_or_default(),
      description: self.description.clone().unwrap_or_default(),
      items: self
        .items
        .clone()
        .unwrap_or_default()
        .iter()
        .map(|item| DictItemVo {
          label: item.label.clone().unwrap_or_default(),
          value: item.value.clone().unwrap_or_default(),
          description: item.description.clone().unwrap_or_default(),
          sort: item.sort.unwrap_or_default(),
        })
        .collect::<Vec<_>>(),
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct DictItem {
  pub label: Option<String>,
  pub value: Option<String>,
  pub description: Option<String>,
  pub sort: Option<u16>,
}

impl Storable for DictItem {
  fn to_bytes(&self) -> Cow<[u8]> {
    Cow::Owned(Encode!(self).unwrap())
  }

  fn from_bytes(bytes: Cow<[u8]>) -> Self {
    Decode!(bytes.as_ref(), Self).unwrap()
  }

  const BOUND: Bound = Bound::Unbounded;
}

impl From<&DictItemVo> for DictItem {
  fn from(vo: &DictItemVo) -> Self {
    DictItem {
      label: Some(vo.label.clone()),
      value: Some(vo.value.clone()),
      description: Some(vo.description.clone()),
      sort: Some(vo.sort),
    }
  }
}
