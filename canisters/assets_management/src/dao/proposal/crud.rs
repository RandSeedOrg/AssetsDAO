use ic_cdk::update;
use system_configs_macro::has_permission_result;
use types::assets_management::ProposalId;

use super::{
  stable_structures::Proposal,
  transport_structures::{AddProposalDto, UpdateProposalDto},
  STAKING_ACCOUNT_MAP,
};

#[update]
#[has_permission_result("assets_management::proposal::create")]
fn create_proposal(dto: AddProposalDto) -> Result<ProposalId, String> {
  if dto.title.is_empty() {
    return Err("Title cannot be empty".to_string());
  }

  if dto.description.is_empty() {
    return Err("Description cannot be empty".to_string());
  }

  let proposal = Proposal::from_add_dto(&dto);
  let new_proposal_id = proposal.get_id();

  STAKING_ACCOUNT_MAP.with(|map| {
    map.borrow_mut().insert(proposal.get_id(), proposal.clone());
  });

  Ok(new_proposal_id)
}

#[update]
#[has_permission_result("assets_management::proposal::update")]
fn update_proposal(dto: UpdateProposalDto) -> Result<ProposalId, String> {
  if dto.title.is_empty() {
    return Err("Title cannot be empty".to_string());
  }

  if dto.description.is_empty() {
    return Err("Description cannot be empty".to_string());
  }

  STAKING_ACCOUNT_MAP.with(|map| {
    let mut map = map.borrow_mut();
    let mut proposal = map.get(&dto.id).ok_or("Proposal not found")?;

    proposal.update_with_dto(&dto);
    map.insert(dto.id, proposal.clone());

    Ok(proposal.get_id())
  })
}
