use std::borrow::Cow;

use candid::{CandidType, Decode, Encode};
use ic_stable_structures::{storable::Bound, Storable};
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};
use types::{
  assets_management::{JackpotId, ProposalId},
  stable_structures::{new_entity_id, MetaData},
  staking::StakingPoolId,
  UserId, E8S,
};

use crate::transfer_address::stable_structures::TransferAddressType;

use super::{
  transport_structures::{AddProposalDto, UpdateProposalDto},
  PROPOSAL_ID, PROPOSAL_MAP,
};

#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct Proposal {
  pub id: Option<ProposalId>,
  pub title: Option<String>,
  pub description: Option<String>,
  pub status: Option<ProposalStatus>,
  pub proposal_initiator: Option<UserId>,
  pub proposal_instruction: Option<ProposalInstructionType>,
  pub meta: Option<MetaData>,
}

impl Proposal {
  pub fn from_add_dto(dto: &AddProposalDto) -> Self {
    let new_proposal_id = PROPOSAL_ID.with(|proposal_id| new_entity_id(proposal_id));
    let meta = MetaData::init_create_scene();
    Self {
      id: Some(new_proposal_id),
      title: Some(dto.title.clone()),
      description: Some(dto.description.clone()),
      // 这里的状态本该是Created，但现在暂不实现投票功能，因此直接将其状态设置为 Passed
      status: Some(ProposalStatus::Created),
      proposal_initiator: Some(ic_cdk::api::canister_self().to_text()),
      proposal_instruction: Some(dto.instruction_type.clone()),
      meta: Some(meta.clone()),
    }
  }

  pub fn update_with_dto(&mut self, dto: &UpdateProposalDto) {
    let add_dto = &dto.add_dto;
    self.title = Some(add_dto.title.clone());
    self.description = Some(add_dto.description.clone());
    self.proposal_instruction = Some(add_dto.instruction_type.clone());
    self.meta = Some(self.get_meta().update());
  }

  pub fn executed_nns_stake(&mut self, neuron_id: u64) -> Result<(), String> {
    self.status = Some(ProposalStatus::Executed);
    let mut instruction = self.get_proposal_instruction();

    if let ProposalInstructionType::NNSStake { .. } = instruction {
      instruction.set_neuron_id(neuron_id);
      self.proposal_instruction = Some(instruction);
      self.meta = Some(self.get_meta().update());
      self.update_to_stable();
      Ok(())
    } else {
      return Err("Proposal instruction is not NNSStake".to_string());
    }
  }

  pub fn executed_add_transfer_address(&mut self, transfer_address_id: u64) -> Result<(), String> {
    self.status = Some(ProposalStatus::Executed);
    let mut instruction = self.get_proposal_instruction();

    if let ProposalInstructionType::AddTransferAddress { ref mut id, .. } = instruction {
      if id.is_some() {
        return Err("Transfer address already exists".to_string());
      }

      *id = Some(transfer_address_id);
      self.proposal_instruction = Some(instruction);
      self.meta = Some(self.get_meta().update());
      self.update_to_stable();
      Ok(())
    } else {
      return Err("Proposal instruction is not AddTransferAddress".to_string());
    }
  }

  pub fn update_to_stable(&self) {
    PROPOSAL_MAP.with(|map| {
      map.borrow_mut().insert(self.get_id(), self.clone());
    });
  }

  pub fn get_id(&self) -> ProposalId {
    self.id.unwrap_or_default()
  }

  pub fn get_title(&self) -> String {
    self.title.clone().unwrap_or_default()
  }

  pub fn get_description(&self) -> String {
    self.description.clone().unwrap_or_default()
  }

  pub fn get_status(&self) -> ProposalStatus {
    self.status.clone().unwrap_or(ProposalStatus::Created)
  }

  pub fn set_status(&mut self, status: ProposalStatus) {
    self.status = Some(status);
    self.meta = Some(self.get_meta().update());
    self.update_to_stable();
  }

  pub fn get_proposal_initiator(&self) -> UserId {
    self.proposal_initiator.clone().unwrap_or_default()
  }

  pub fn get_proposal_instruction(&self) -> ProposalInstructionType {
    self.proposal_instruction.clone().unwrap_or(ProposalInstructionType::None)
  }

  pub fn get_meta(&self) -> MetaData {
    self.meta.clone().unwrap_or_default()
  }
}

#[derive(EnumString, Display, Debug, Clone, Serialize, Deserialize, CandidType, PartialEq, Eq)]
pub enum ProposalStatus {
  /// New creations can be edited, but cannot be changed until voting is open
  #[strum(serialize = "0")]
  Created,
  /// During the open voting phase, all staking accounts can vote until the voting ends.
  #[strum(serialize = "1")]
  Voting,
  /// The vote is passed, and the proposal instructions can be executed at this time
  #[strum(serialize = "2")]
  Passed,
  /// Vote rejected. The proposal has not passed multi-signature and will not be executed. Only the status will be recorded.
  #[strum(serialize = "3")]
  Rejected,
  /// Executed, the proposal has been executed, and permanent changes will be made according to the proposal instructions
  #[strum(serialize = "4")]
  Executed,
}

/// The Proposal Instruction Type, which describes the purpose of the instruction and the metadata required for the instruction to execute
#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub enum ProposalInstructionType {
  /// None, the proposal does not contain any instructions
  None,
  /// Stake the specified amount in the specified staking pool into the NNS neuron
  NNSStake {
    pool_id: StakingPoolId,
    amount: E8S,
    neuron_id: Option<u64>,
  },
  /// Transfer the specified amount of funds in the staking pool to the jackpot account
  JackpotInvestment {
    pool_id: StakingPoolId,
    jackpot_id: JackpotId,
    amount: E8S,
  },
  /// Add transfer address
  AddTransferAddress {
    id: Option<u64>,
    name: String,
    usage: String,
    network: String,
    crypto: String,
    address: String,
    address_type: TransferAddressType,
  },
}

impl ProposalInstructionType {
  pub fn get_pool_id(&self) -> StakingPoolId {
    match self {
      ProposalInstructionType::NNSStake { pool_id, .. } => *pool_id,
      ProposalInstructionType::JackpotInvestment { pool_id, .. } => *pool_id,
      ProposalInstructionType::None => StakingPoolId::default(),
      ProposalInstructionType::AddTransferAddress { .. } => StakingPoolId::default(),
    }
  }

  pub fn set_neuron_id(&mut self, neuron_id: u64) {
    if let ProposalInstructionType::NNSStake { neuron_id: ref mut id, .. } = self {
      *id = Some(neuron_id);
    }
  }

  pub fn get_amount(&self) -> E8S {
    match self {
      ProposalInstructionType::NNSStake { amount, .. } => *amount,
      ProposalInstructionType::JackpotInvestment { amount, .. } => *amount,
      ProposalInstructionType::None => 0,
      ProposalInstructionType::AddTransferAddress { .. } => 0,
    }
  }
}

impl Storable for Proposal {
  fn to_bytes(&self) -> Cow<[u8]> {
    Cow::Owned(Encode!(self).unwrap())
  }

  fn from_bytes(bytes: Cow<[u8]>) -> Self {
    Decode!(bytes.as_ref(), Self).unwrap()
  }

  const BOUND: Bound = Bound::Unbounded;
}
