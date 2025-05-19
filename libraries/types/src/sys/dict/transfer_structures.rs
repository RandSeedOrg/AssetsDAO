use candid::CandidType;
use serde::{Deserialize, Serialize};

use crate::EntityId;

use super::DictCode;

#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct DictItemVo {
  pub label: String,
  pub value: String,
  pub description: String,
  pub sort: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct DictVo {
  pub id: EntityId,
  pub name: String,
  pub code: DictCode,
  pub description: String,
  pub items: Vec<DictItemVo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct AddDictDto {
  pub name: String,
  pub code: String,
  pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct UpdateDictDto {
  pub id: EntityId,
  pub name: String,
  pub code: String,
  pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct DictItemsDto {
  pub id: EntityId,
  pub items: Vec<DictItemVo>,
}
