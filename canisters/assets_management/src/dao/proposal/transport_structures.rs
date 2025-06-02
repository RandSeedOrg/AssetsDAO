use candid::CandidType;
use serde::{Deserialize, Serialize};
use types::assets_management::ProposalId;

use super::stable_structures::{Proposal, ProposalInstructionType};

#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct AddProposalDto {
  pub title: String,
  pub description: String,
  pub instruction_type: ProposalInstructionType,
}

#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct UpdateProposalDto {
  pub id: ProposalId,
  pub add_dto: AddProposalDto,
}

#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct ProposalVo {
  pub id: ProposalId,
  pub title: String,
  pub description: String,
  pub instruction: ProposalInstructionType,
  pub status: String,
  pub proposal_initiator: String,
  pub created_by: String,
  pub updated_by: String,
  pub created_at: u64,
  pub updated_at: u64,
}

impl ProposalVo {
  pub fn from_proposal(proposal: &Proposal) -> Self {
    Self {
      id: proposal.get_id(),
      title: proposal.get_title(),
      description: proposal.get_description(),
      instruction: proposal.get_proposal_instruction(),
      status: proposal.get_status().to_string(),
      proposal_initiator: proposal.get_proposal_initiator().to_string(),
      created_by: proposal.get_meta().get_created_by().to_string(),
      updated_by: proposal.get_meta().get_updated_by().to_string(),
      created_at: proposal.get_meta().get_created_at(),
      updated_at: proposal.get_meta().get_updated_at(),
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct ProposalListParams {
  pub status: Option<String>,
}
