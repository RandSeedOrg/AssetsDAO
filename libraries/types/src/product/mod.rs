use bigdecimal::{BigDecimal, ToPrimitive};
use candid::CandidType;
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};

use crate::{E8S, EntityId};

pub mod base;
pub mod instant_win;

pub type BatchId = EntityId;
pub type ProductId = EntityId;

/// The base of the multiplier, keep four decimal places
pub const MULTIPLE_BASE: u32 = 10000;
pub const E8S_BASE: u64 = 100_000_000;

pub type E4S = u32;

pub fn calc_amount_multiple(amount: E8S, e4s: E4S) -> E8S {
  let result = BigDecimal::from(amount) * e4s_to_multiples(e4s);
  result.to_u64().unwrap_or_default()
}

pub fn e4s_to_multiples(e4s: u32) -> BigDecimal {
  BigDecimal::from(e4s) / BigDecimal::from(MULTIPLE_BASE)
}

pub fn multiples_to_e4s(multiples: BigDecimal) -> u32 {
  (multiples * BigDecimal::from(MULTIPLE_BASE)).to_u32().unwrap_or_default()
}

pub fn e8s_to_value(amount: E8S) -> BigDecimal {
  BigDecimal::from(amount) / BigDecimal::from(100_000_000)
}

pub fn value_to_e8s(value: BigDecimal) -> E8S {
  (value * BigDecimal::from(E8S_BASE)).to_u64().unwrap_or_default()
}

/// Corresponding dictionary encoding batch_state
#[derive(EnumString, Display, Debug, Clone, Serialize, Deserialize, CandidType, PartialEq, Eq)]
pub enum BatchState {
  /// New means the batch is created but not generated tickets
  #[strum(serialize = "0")]
  New,
  /// Initialized means the batch is generated but not started
  #[strum(serialize = "1")]
  Initialized,
  #[strum(serialize = "2")]
  Running,
  #[strum(serialize = "3")]
  Paused,
  #[strum(serialize = "4")]
  Finished,
  #[strum(serialize = "5")]
  Expired,
}

/// Corresponding dictionary encoding product_type
#[derive(EnumString, Display, Debug, Clone, Serialize, Deserialize, CandidType, PartialEq, Eq)]
pub enum ProductType {
  #[strum(serialize = "unknown")]
  Unknown,
  #[strum(serialize = "0")]
  InstantWin,
  #[strum(serialize = "1")]
  Daily4Balls,
}

/// Corresponding dictionary encoding product_status
#[derive(EnumString, Display, Debug, Clone, Serialize, Deserialize, CandidType, PartialEq, Eq)]
pub enum ProductStatus {
  #[strum(serialize = "1")]
  Available,
  #[strum(serialize = "0")]
  Delisted,
}

pub enum TxIdType {
  Play,
  Win,
}

pub fn generate_payment_transaction_id(id_type: TxIdType, product_id: ProductId, batch_id: BatchId, order_id: EntityId) -> Result<EntityId, String> {
  // Make sure each ID is within its own number of digits
  let tx_id_masked: u64 = match id_type {
    // 8-bit mask
    TxIdType::Play => 1,
    TxIdType::Win => 2,
  };
  let product_id_masked = product_id & 0xFFFFF; // 20-bit mask
  let batch_id_masked = batch_id & 0xFFFFF; // 20-bit mask
  let order_id_masked = order_id & 0xFFFF; // 16-bit mask

  if product_id_masked != product_id {
    return Err("Product ID is too large!".to_string());
  }

  if batch_id_masked != batch_id {
    return Err("Batch ID is too large!".to_string());
  }

  if order_id_masked != order_id {
    return Err("Order ID is too large!".to_string());
  }

  // Generate payment transaction ID
  Ok((tx_id_masked << 56) | (product_id_masked << 36) | (batch_id_masked << 16) | order_id_masked)
}

pub fn generate_staking_reward_payment_transaction_id(order_id: u64) -> Result<EntityId, String> {
  // Make sure each ID is within its own number of digits
  let tx_id_masked: u64 = 3; // 8-bit mask
  let order_id_masked = order_id & 0xFFFF_FFFF_FFFF_FFF; // 56-bit mask

  if order_id_masked != order_id {
    return Err("Order id is too large!".to_string());
  }

  // Generate payment transaction ID
  Ok((tx_id_masked << 56) | order_id_masked)
}
