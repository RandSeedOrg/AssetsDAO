pub mod nns_governance_api;

use ic_ledger_types::MAINNET_GOVERNANCE_CANISTER_ID;
use nns_governance_api::Service;

pub fn get_governance() -> Service {
  Service(MAINNET_GOVERNANCE_CANISTER_ID)
}
