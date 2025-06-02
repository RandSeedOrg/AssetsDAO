use ic_cdk::{query, update};
use system_configs_macro::has_permission_result;
use types::{
  assets_management::ProposalId,
  pagination::{PageRequest, PageResponse},
};

use super::{
  stable_structures::Proposal,
  transport_structures::{AddProposalDto, ProposalListParams, ProposalVo, UpdateProposalDto},
  PROPOSAL_MAP,
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

  PROPOSAL_MAP.with(|map| {
    map.borrow_mut().insert(proposal.get_id(), proposal.clone());
  });

  Ok(new_proposal_id)
}

#[update]
#[has_permission_result("assets_management::proposal::update")]
fn update_proposal(dto: UpdateProposalDto) -> Result<ProposalId, String> {
  let add_dto = &dto.add_dto;

  if add_dto.title.is_empty() {
    return Err("Title cannot be empty".to_string());
  }

  if add_dto.description.is_empty() {
    return Err("Description cannot be empty".to_string());
  }

  PROPOSAL_MAP.with(|map| {
    let mut map = map.borrow_mut();
    let mut proposal = map.get(&dto.id).ok_or("Proposal not found")?;

    proposal.update_with_dto(&dto);
    map.insert(dto.id, proposal.clone());

    Ok(proposal.get_id())
  })
}

#[query]
fn list_proposal(request: PageRequest<ProposalListParams>) -> PageResponse<ProposalVo> {
  let PageRequest {
    params: ProposalListParams { status },
    page,
    page_size,
  } = request;

  let proposals = PROPOSAL_MAP.with(|map| {
    if status.is_none() {
      return map.borrow().values().collect::<Vec<_>>();
    } else {
      let status = status.unwrap();
      return map
        .borrow()
        .values()
        .filter(|proposal| proposal.get_status().to_string() == status)
        .collect::<Vec<_>>();
    }
  });

  let total = proposals.len() as u32;
  let start = (page - 1) * page_size;

  PageResponse {
    total,
    page,
    page_size,
    records: proposals
      .iter()
      .rev()
      .skip(start as usize)
      .take(page_size as usize)
      .map(|proposal| ProposalVo::from_proposal(proposal))
      .collect(),
  }
}
