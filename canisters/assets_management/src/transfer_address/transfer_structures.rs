use crate::transfer_address::stable_structures::{TransferAddress, TransferAddressType};

use candid::CandidType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct TransferAddressVo {
  pub id: u64,
  pub proposal_id: u64,
  pub name: String,
  pub usage: String,
  pub network: String,
  pub crypto: String,
  /// On-chain address
  pub address: String,
  pub status: String,
  pub address_type: TransferAddressType,
  pub created_at: u64,
  pub updated_at: u64,
  pub created_by: String,
  pub updated_by: String,
}

impl From<TransferAddress> for TransferAddressVo {
  fn from(address: TransferAddress) -> Self {
    let meta = address.get_meta();

    Self {
      id: address.get_id(),
      proposal_id: address.get_proposal_id(),
      name: address.get_name(),
      usage: address.get_usage(),
      network: address.get_network().to_string(),
      crypto: address.get_crypto().to_string(),
      address: address.get_address(),
      status: address.get_status().to_string(),
      address_type: address.get_address_type(),
      created_at: meta.get_created_at(), // Placeholder, should be set when creating the address
      updated_at: meta.get_updated_at(), // Placeholder, should be set when updating the address
      created_by: meta.get_created_by(), // Placeholder, should be set when creating the address
      updated_by: meta.get_updated_by(), // Placeholder, should be set when updating the address
    }
  }
}
