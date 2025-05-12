use candid::CandidType;
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};

pub mod dict;
pub mod config;

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
}

/// System switch, map system_switches dictionary in system configuration
#[derive(EnumString, Display, Debug, Clone, Serialize, Deserialize, CandidType)]
pub enum SystemSwitches {
  #[strum(serialize = "Client Verification")]
  ClientVerification,
}