use candid::CandidType;
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};

pub mod config;
pub mod dict;

#[derive(EnumString, Display, Debug, Clone, Serialize, Deserialize, CandidType)]
pub enum ExteralCanisterLabels {
  #[strum(serialize = "Pay center")]
  PayCenter,
  #[strum(serialize = "User center")]
  User,
  #[strum(serialize = "Product center")]
  ProductCenter,
  #[strum(serialize = "Marketing")]
  Marketing,
  #[strum(serialize = "Play")]
  Play,
  #[strum(serialize = "Messenger")]
  Messenger,
  #[strum(serialize = "Staking")]
  Staking,
}

/// System switch, map system_switches dictionary in system configuration
#[derive(EnumString, Display, Debug, Clone, Serialize, Deserialize, CandidType)]
pub enum SystemSwitches {
  #[strum(serialize = "Client Verification")]
  ClientVerification,
}
