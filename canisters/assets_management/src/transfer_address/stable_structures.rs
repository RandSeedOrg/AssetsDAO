use types::{
  EntityId,
  assets_management::{ProposalId, TransferAddressId},
  on_chain::{BlockChain, Crypto},
  stable_structures::MetaData,
  staking::StakingPoolId,
};

use candid::CandidType;
use serde::{Deserialize, Serialize};

/// Asset transfer address
/// These accounts are officially certified accounts,
/// and the accounts that are guaranteed to be available are all secure on-chain addresses that have been officially confirmed through proposals or reliable procedures.
#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct TransferAddress {
  pub id: Option<TransferAddressId>,
  /// The creator of the asset transfer address (associated with the proposal to create this transfer address)
  pub proposal_id: Option<ProposalId>,
  pub name: Option<String>,
  pub usage: Option<String>,
  /// Blockchain network
  pub network: Option<BlockChain>,
  /// Currency on the blockchain
  pub crypto: Option<Crypto>,
  /// On-chain address
  pub address: Option<String>,
  pub status: Option<TransferAddressStatus>,
  /// Address type, used to distinguish the business module to which the address belongs.
  /// And save the unique identifier of the business module to accurately identify the source of the address
  pub address_type: Option<TransferAddressType>,
  pub meta: Option<MetaData>,
}

/// Asset transfer address status
#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub enum TransferAddressStatus {
  /// Activated: Only addresses in the activated state can transfer funds.
  Activated,
  /// If you actively propose a proposal to abandon the address, the address will become invalid after the proposal is passed
  Invalid,
}

/// Transfer address type
#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub enum TransferAddressType {
  /// This transfer address is an NNS neuron chain address
  NNSNeuron { id: EntityId },
  /// The on-chain address of the staking pool
  StakingPool { id: StakingPoolId },
  /// The on-chain address of the jackpot
  Jackpot { id: TransferAddressId },
}
