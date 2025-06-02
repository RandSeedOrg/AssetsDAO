use candid::CandidType;
use ic_principal::Principal;
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};

pub mod assets_management;
pub mod btree_set_entity_index;
pub mod date;
pub mod entities;
pub mod nns;
pub mod on_chain;
pub mod pagination;
pub mod product;
pub mod stable_structures;
pub mod staking;
pub mod sys;

pub type E8S = u64;
pub const E8S_PER_ICP: E8S = 100_000_000;

#[derive(CandidType, Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Tokens {
  e8s: E8S,
}

pub type Timestamp = u64;

pub type UserId = String;

pub type EntityId = u64;

pub type TicketNo = u32;

pub type CellIndex = u32;

pub type TransactionId = u64;

pub type AccessorId = Principal;
pub type CanisterId = Principal;
pub type Hash = [u8; 32];
pub type ICP = Tokens;
pub type Milliseconds = u64;
pub type Nanoseconds = u64;
pub type TimestampMillis = Timestamp;
pub type TimestampNanos = Timestamp;

/// Ranking factor, used for leaderboard sorting
pub type RankingFactor = u128;

/// The currency types for block chain
#[derive(EnumString, Display, Debug, Clone, Serialize, Deserialize, CandidType, PartialEq, Eq, Hash)]
pub enum Crypto {
  #[strum(serialize = "0")]
  ICP,
  #[strum(serialize = "1")]
  USDT,
}
