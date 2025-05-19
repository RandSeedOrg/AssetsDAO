use candid::CandidType;
use serde::{Deserialize, Serialize};

use crate::{
  E8S, EntityId, TimestampNanos, UserId,
  product::{BatchId, E4S},
};

#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct QuickQuidExtraConfigDto {
  // All bonus code information on the current Batch
  pub bonus_codes: Vec<String>,
  // The background image of all cards in the current batch
  pub cards: Vec<CardDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct CardDto {
  // Number of grids on the card
  pub cell_count: u16,
  // Card background image
  pub background_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct QuickQuidBatchExtraVo {
  pub batch_id: BatchId,
  // Additional information configuration
  pub config: QuickQuidExtraConfigDto,
  // Extra information at runtime will change as the game runs
  pub runtime: QuickQuidExtraRuntimeVo,
}

#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct QuickQuidExtraRuntimeVo {
  pub cards: Vec<RuntimeCardVo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct RuntimeCardVo {
  pub ordinal: u32,
  // Grid on the card
  pub cells: Vec<RuntimeCellVo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct RuntimeCellVo {
  // Grid index
  pub index: u32,
  // Award information, if not drawn, none
  pub prize: Option<CardCellPrizeVo>,
  // Bonus code tied to the current grid
  pub bonus_code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct CardCellPrizeVo {
  /// player id
  pub user_id: UserId,
  /// Win amount
  pub prize_amount: E8S,
  /// Winning multiple
  pub prize_multiples: E4S,
  /// Winning time
  pub create_time: TimestampNanos,
  /// The corresponding play record id
  pub play_record_id: EntityId,
}
