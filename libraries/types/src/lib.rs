use ic_principal::Principal;
use candid::CandidType;
use serde::{Serialize, Deserialize};
use strum_macros::{Display, EnumString};

pub mod stable_structures;
pub mod product;
pub mod sys;
pub mod date;
pub mod entities;
pub mod pagination;
pub mod on_chain;
pub mod staking;
pub mod assets_management;


pub type E8S = u64;


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

/// 排名因子，用于排行榜排序
pub type RankingFactor = u128;

/// The currency types for block chain
#[derive(EnumString, Display, Debug, Clone, Serialize, Deserialize, CandidType, PartialEq, Eq, Hash)]
pub enum Crypto {
  #[strum(serialize = "0")]
  ICP,
  #[strum(serialize = "1")]
  USDT,
}
