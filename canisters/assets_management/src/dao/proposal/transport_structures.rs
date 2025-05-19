use candid::CandidType;
use serde::{Deserialize, Serialize};
use types::assets_management::ProposalId;

use super::stable_structures::ProposalInstructionType;

#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct AddProposalDto {
  pub title: String,
  pub description: String,
  pub instruction_type: ProposalInstructionType,
}

#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct UpdateProposalDto {
  pub id: ProposalId,
  pub title: String,
  pub description: String,
  pub instruction_type: ProposalInstructionType,
}
