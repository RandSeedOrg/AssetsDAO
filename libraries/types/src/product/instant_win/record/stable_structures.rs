use std::borrow::Cow;

use candid::{CandidType, Decode, Encode};
use ic_stable_structures::{Storable, storable::Bound};
use serde::{Deserialize, Serialize};

use crate::{E8S, EntityId, TicketNo, UserId, product::E4S, stable_structures::MetaData};

use super::transport_structures::InstantWinPlayRecordVo;

#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct InstantWinPlayRecord {
  pub id: Option<EntityId>,
  pub sales_order_id: Option<EntityId>,
  pub redemption_order_id: Option<EntityId>,
  pub product_id: Option<EntityId>,
  pub batch_id: Option<EntityId>,
  pub user_id: Option<UserId>,
  pub ticket_no: Option<TicketNo>,
  pub prize_multiples: Option<E4S>,
  pub prize_amount: Option<E8S>,
  pub meta: Option<MetaData>,
}

impl Storable for InstantWinPlayRecord {
  fn to_bytes(&self) -> Cow<[u8]> {
    Cow::Owned(Encode!(self).unwrap())
  }

  fn from_bytes(bytes: Cow<[u8]>) -> Self {
    Decode!(bytes.as_ref(), Self).unwrap()
  }

  const BOUND: Bound = Bound::Unbounded;
}

impl InstantWinPlayRecord {
  pub fn to_vo(&self) -> InstantWinPlayRecordVo {
    InstantWinPlayRecordVo {
      id: self.id.unwrap_or_default(),
      sales_order_id: self.sales_order_id.unwrap_or_default(),
      redemption_order_id: self.redemption_order_id.unwrap_or_default(),
      product_id: self.product_id.unwrap_or_default(),
      batch_id: self.batch_id.unwrap_or_default(),
      user_id: self.user_id.clone().unwrap_or_default(),
      ticket_no: self.ticket_no.unwrap_or_default(),
      prize_multiple: self.prize_multiples.unwrap_or_default(),
      create_time: self.meta.clone().unwrap_or(MetaData::init_create_scene()).created_at.unwrap_or_default(),
    }
  }

  pub fn get_user_id(&self) -> UserId {
    self.user_id.clone().unwrap_or_default()
  }

  pub fn get_create_time(&self) -> u64 {
    self.meta.clone().unwrap_or(MetaData::init_create_scene()).created_at.unwrap_or_default()
  }

  pub fn get_ticket_no(&self) -> TicketNo {
    self.ticket_no.unwrap_or_default()
  }

  pub fn get_prize_multiples(&self) -> E4S {
    self.prize_multiples.unwrap_or_default()
  }

  pub fn get_prize_amount(&self) -> E8S {
    self.prize_amount.unwrap_or_default()
  }
}
