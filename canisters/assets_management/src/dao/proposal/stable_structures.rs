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

use super::{
  transport_structures::{AddProposalDto, UpdateProposalDto},
  PROPOSAL_ID,
};

#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct Proposal {
  pub id: Option<ProposalId>,
  pub title: Option<String>,
  pub description: Option<String>,
  pub status: Option<ProposalStatus>,
  pub proposal_initiator: Option<UserId>,
  pub proposal_instruction: Option<ProposalInstruction>,
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
      status: Some(ProposalStatus::Created),
      proposal_initiator: Some(ic_cdk::api::canister_self().to_text()),
      proposal_instruction: Some(ProposalInstruction {
        instruction_type: Some(dto.instruction_type.clone()),
        instruction_status: Some(ProposalInstructionStatus::NotReady),
        meta: Some(meta.clone()),
      }),
      meta: Some(meta.clone()),
    }
  }

  pub fn update_with_dto(&mut self, dto: &UpdateProposalDto) {
    let add_dto = &dto.add_dto;
    self.title = Some(add_dto.title.clone());
    self.description = Some(add_dto.description.clone());
    self.proposal_instruction = Some(ProposalInstruction {
      instruction_type: Some(add_dto.instruction_type.clone()),
      instruction_status: Some(ProposalInstructionStatus::NotReady),
      meta: Some(self.get_proposal_instruction().get_meta().update()),
    });
    self.meta = Some(self.get_meta().update());
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

  pub fn get_proposal_initiator(&self) -> UserId {
    self.proposal_initiator.clone().unwrap_or_default()
  }

  pub fn get_proposal_instruction(&self) -> ProposalInstruction {
    self.proposal_instruction.clone().unwrap_or(ProposalInstruction {
      instruction_type: None,
      instruction_status: None,
      meta: None,
    })
  }

  pub fn get_meta(&self) -> MetaData {
    self.meta.clone().unwrap_or_default()
  }
}

#[derive(EnumString, Display, Debug, Clone, Serialize, Deserialize, CandidType)]
pub enum ProposalStatus {
  /// New creations can be edited, but cannot be changed until voting is open
  #[strum(serialize = "0")]
  Created,
  /// During the open voting phase, all staking accounts can vote until the voting ends.
  #[strum(serialize = "1")]
  OpenVoting,
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

/// Proposal instructions are used to accurately describe the actions to be performed.
/// The execution time, and the execution results of the proposal
#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct ProposalInstruction {
  pub instruction_type: Option<ProposalInstructionType>,
  pub instruction_status: Option<ProposalInstructionStatus>,
  pub meta: Option<MetaData>,
}

impl ProposalInstruction {
  pub fn get_instruction_type(&self) -> ProposalInstructionType {
    self.instruction_type.clone().unwrap_or(ProposalInstructionType::None)
  }

  pub fn get_instruction_status(&self) -> ProposalInstructionStatus {
    self.instruction_status.clone().unwrap_or(ProposalInstructionStatus::NotReady)
  }

  pub fn get_meta(&self) -> MetaData {
    self.meta.clone().unwrap_or_default()
  }
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
    duration: u16,
    neuron_id: Option<u64>,
  },
  /// Transfer the specified amount of funds in the staking pool to the jackpot account
  JackpotInvestment {
    pool_id: StakingPoolId,
    jackpot_id: JackpotId,
    amount: E8S,
  },
}

/// The status of the proposal instruction
#[derive(EnumString, Display, Debug, Clone, Serialize, Deserialize, CandidType)]
pub enum ProposalInstructionStatus {
  /// The proposal has not been voted on yet, so the instructions are not ready
  #[strum(serialize = "0")]
  NotReady,
  /// The proposal has been voted through and is awaiting execution
  #[strum(serialize = "1")]
  Pending,
  /// The instruction is being executed
  #[strum(serialize = "2")]
  InProgress,
  /// The instruction was executed successfully
  #[strum(serialize = "3")]
  Succeed,
  /// The instruction was executed but failed
  #[strum(serialize = "4")]
  Failed,
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
