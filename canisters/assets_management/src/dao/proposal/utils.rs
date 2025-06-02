use types::assets_management::ProposalId;

use super::{stable_structures::Proposal, PROPOSAL_MAP};

pub fn query_proposal(proposal_id: ProposalId) -> Result<Proposal, String> {
  PROPOSAL_MAP.with(|map| {
    let map = map.borrow();
    map.get(&proposal_id).ok_or_else(|| "Proposal not found".to_string())
  })
}
