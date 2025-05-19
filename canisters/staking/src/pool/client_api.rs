use super::{client_transport_structures::ClientStakingPoolVo, crud_utils::query_client_visible_staking_pools};

#[ic_cdk::query]
fn query_visible_staking_pools() -> Vec<ClientStakingPoolVo> {
  let visible_pools = query_client_visible_staking_pools();

  visible_pools
    .into_iter()
    .map(|pool| ClientStakingPoolVo::from_staking_pool(&pool))
    .collect()
}
