use std::borrow::Cow;

use ic_stable_structures::{storable::Bound, Storable};
use types::{
  assets_management::{ProposalId, TransferAddressId},
  on_chain::{BlockChain, Crypto},
  stable_structures::{new_entity_id, MetaData},
  staking::StakingPoolId,
  EntityId,
};

use candid::{CandidType, Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::transfer_address::{TRANSFER_ADDRESS_ID, TRANSFER_ADDRESS_MAP};

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

impl TransferAddress {
  /// Creates a new transfer address
  pub fn new(
    proposal_id: ProposalId,
    name: String,
    usage: String,
    network: String,
    crypto: String,
    address: String,
    address_type: TransferAddressType,
  ) -> Result<Self, String> {
    let meta = MetaData::init_create_scene();
    let mut instance = Self {
      id: None,
      proposal_id: Some(proposal_id),
      name: Some(name),
      usage: Some(usage),
      network: Some(BlockChain::try_from(network.as_ref()).map_err(|_| format!("Invalid blockchain network: {}", network))?),
      crypto: Some(Crypto::try_from(crypto.as_ref()).map_err(|_| format!("Invalid crypto currency: {}", crypto))?),
      address: Some(address),
      status: Some(TransferAddressStatus::Activated),
      address_type: Some(address_type),
      meta: Some(meta),
    };

    let id = TRANSFER_ADDRESS_ID.with(|transfer_address_id| new_entity_id(transfer_address_id));

    instance.id = Some(id);

    TRANSFER_ADDRESS_MAP.with(|map| {
      let mut map = map.borrow_mut();
      map.insert(id, instance.clone());
      Ok(instance)
    })
  }

  /// Returns the ID of the transfer address
  pub fn get_id(&self) -> TransferAddressId {
    self.id.unwrap_or_default()
  }
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
  /// The transfer address is the Neuron on-chain address of the staking pool to which it belongs
  StakingPoolNNSNeuron { pool_id: EntityId },
  /// The on-chain address of the staking pool
  StakingPool { pool_id: StakingPoolId },
  /// The on-chain address of the jackpot
  Jackpot { jackpot_id: TransferAddressId },
  /// Any other on-chain address
  Other,
}

impl Storable for TransferAddress {
  fn to_bytes(&self) -> Cow<[u8]> {
    Cow::Owned(Encode!(self).unwrap())
  }

  fn from_bytes(bytes: Cow<[u8]>) -> Self {
    Decode!(bytes.as_ref(), Self).unwrap()
  }

  const BOUND: Bound = Bound::Unbounded;
}
