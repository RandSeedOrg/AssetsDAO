use types::assets_management::ProposalId;

/// Obtain the key for stake entrance
pub fn get_execute_proposal_guard_key(proposal_id: ProposalId) -> String {
  format!("execute_proposal_guard_{}", proposal_id)
}
