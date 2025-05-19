use candid::CandidType;
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};

/// The currency types for block chain
#[derive(EnumString, Display, Debug, Clone, Serialize, Deserialize, CandidType, PartialEq, Eq)]
pub enum Crypto {
  #[strum(serialize = "0")]
  ICP,
  #[strum(serialize = "1")]
  USDT,
}

/// The network types for block chain
#[derive(EnumString, Display, Debug, Clone, Serialize, Deserialize, CandidType, PartialEq, Eq)]
pub enum BlockChain {
  #[strum(serialize = "0")]
  ICP,
}
