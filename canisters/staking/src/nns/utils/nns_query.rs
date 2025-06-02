use nns_governance_api::{get_governance, nns_governance_api::Neuron};
use types::staking::StakingPoolId;

use crate::nns::{NNS_NEURON_MAP, NNS_STAKING_POOL_NEURON_ID_MAP};

pub async fn query_nns_neuron_by_pool_id(neuron_id: u64) -> Result<Neuron, String> {
  let governance = get_governance();

  let (resp,) = governance.get_full_neuron(neuron_id).await.map_err(|e| {
    ic_cdk::println!("Failed to query NNS neuron by pool ID {}: {:?}", neuron_id, e);
    format!("Failed to query NNS neuron by pool ID {}: {:?}", neuron_id, e)
  })?;

  resp.map_err(|e| {
    ic_cdk::println!("Failed to get NNS neuron by pool ID {}: {:?}", neuron_id, e);
    format!("Failed to get NNS neuron by pool ID {}: {:?}", neuron_id, e)
  })
}

pub fn get_neuron_id_by_pool_id(pool_id: StakingPoolId) -> Option<u64> {
  NNS_STAKING_POOL_NEURON_ID_MAP.with(|map| {
    let map = map.borrow();
    map.get(&pool_id).map(|neuron_id| neuron_id)
  })
}

pub async fn sync_nns_neuron(pool_id: StakingPoolId) -> Result<(), String> {
  let neuron_id = get_neuron_id_by_pool_id(pool_id).ok_or_else(|| format!("No NNS neuron found for pool ID: {}", pool_id))?;
  let neuron = query_nns_neuron_by_pool_id(neuron_id).await?;

  NNS_NEURON_MAP.with(|map| {
    let mut map = map.borrow_mut();
    map.insert(pool_id, neuron);

    Ok(())
  })
}
