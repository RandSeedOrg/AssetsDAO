use std::borrow::Cow;

use candid::{CandidType, Decode, Encode};
use ic_stable_structures::{storable::Bound, Storable};
use serde::{Deserialize, Serialize};

use crate::{product::BatchState, stable_structures::MetaData, EntityId, Nanoseconds, TimestampNanos, UserId, E8S};

/// Batch describes a cycle of lottery, which is just an abstract description. 
/// The configuration of the specific lottery is passed in through the generic T
#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct Batch<T: CandidType> {
  pub id: Option<EntityId>,
  /// associated product id
  pub product_id: Option<EntityId>,
  /// Configuration information for specific lotteries is provided by the specific lotteries
  pub config: Option<T>,
  pub batch_state: Option<BatchState>,
  pub description: Option<String>,
  pub start_time: Option<TimestampNanos>,
  /// The current time of pause
  pub pause_time: Option<TimestampNanos>,
  /// The sum of all pauses
  pub accumulated_pause_time: Option<Nanoseconds>,
  pub end_time: Option<TimestampNanos>,
  pub meta: Option<MetaData>,
}

impl<T: CandidType + Serialize + for<'a> Deserialize<'a>> Storable for Batch<T> {
  fn to_bytes(&self) -> Cow<[u8]> {
    Cow::Owned(Encode!(self).unwrap())
  }

  fn from_bytes(bytes: Cow<[u8]>) -> Self {
    Decode!(bytes.as_ref(), Self).unwrap()
  }

  const BOUND: Bound = Bound::Unbounded;
}

/// TicketSalesOrder is a structure that describes the sales order of lottery tickets
#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct TicketSalesOrder<T: CandidType> {
  pub id: Option<EntityId>,
  pub product_id: Option<EntityId>,
  pub batch_id: Option<EntityId>,
  pub user_id: Option<UserId>,
  pub unit_price: Option<E8S>,
  pub total_price: Option<E8S>,
  pub tickets: Option<Vec<T>>,
  /// Payment serial number
  pub psn: Option<EntityId>,
  /// Additional information
  pub extra: Option<String>,
  pub meta: Option<MetaData>,
}

impl<T: CandidType + Serialize + for<'a> Deserialize<'a>> Storable for TicketSalesOrder<T> {
  fn to_bytes(&self) -> Cow<[u8]> {
    Cow::Owned(Encode!(self).unwrap())
  }

  fn from_bytes(bytes: Cow<[u8]>) -> Self {
    Decode!(bytes.as_ref(), Self).unwrap()
  }

  const BOUND: Bound = Bound::Unbounded;
}


/// RedemptionOrder is a structure that describes the redemption order of lottery tickets
#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct RedemptionOrder<T: CandidType> {
  pub id: Option<EntityId>,
  pub product_id: Option<EntityId>,
  pub batch_id: Option<EntityId>,
  pub user_id: Option<UserId>,
  pub prize_amount: Option<E8S>,
  /// Winning lottery information
  pub prize_tickets: Option<Vec<T>>,
  /// Payment serial number
  pub psn: Option<EntityId>,
  /// Additional information
  pub extra: Option<String>,
  pub meta: Option<MetaData>,
}

impl<T: CandidType + Serialize + for<'a> Deserialize<'a>> Storable for RedemptionOrder<T> {
  fn to_bytes(&self) -> Cow<[u8]> {
    Cow::Owned(Encode!(self).unwrap())
  }

  fn from_bytes(bytes: Cow<[u8]>) -> Self {
    Decode!(bytes.as_ref(), Self).unwrap()
  }

  const BOUND: Bound = Bound::Unbounded;
}

#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct ProductBatchIndex {
  pub product_id: Option<EntityId>,
  pub batch_ids: Option<Vec<EntityId>>,
}

impl Storable for ProductBatchIndex {
  fn to_bytes(&self) -> Cow<[u8]> {
      Cow::Owned(Encode!(self).unwrap())
  }

  fn from_bytes(bytes: Cow<[u8]>) -> Self {
      Decode!(bytes.as_ref(), Self).unwrap()
  }

  const BOUND: Bound = Bound::Unbounded;
}

#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct BatchData<T: CandidType> {
  pub batch_id: EntityId,
  pub data: Vec<T>,
}


impl<T: CandidType + Serialize + for<'a> Deserialize<'a>> Storable for BatchData<T> {
  fn to_bytes(&self) -> Cow<[u8]> {
    Cow::Owned(Encode!(self).unwrap())
  }

  fn from_bytes(bytes: Cow<[u8]>) -> Self {
    Decode!(bytes.as_ref(), Self).unwrap()
  }

  const BOUND: Bound = Bound::Unbounded;
}

impl<T: CandidType> BatchData<T> {
  pub fn new(batch_id: EntityId, data: Vec<T>) -> Self {
    Self {
      batch_id,
      data,
    }
  }
}
