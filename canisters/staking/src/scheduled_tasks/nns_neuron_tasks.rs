use crate::nns::utils::nns_query::sync_nns_neuron;
use crate::pool::crud_utils::get_all_staking_pools;

pub async fn sync_nns_neuron_info_task() -> Result<(), String> {
  ic_cdk::println!("Starting NNS neuron check task...");

  // Get all staking pool IDs
  let staking_pools = get_all_staking_pools();

  ic_cdk::println!("Found {} staking pools", staking_pools.len());

  // For each staking pool, get/generate the corresponding neuron ID and check its dissolve status
  for pool in staking_pools {
    let pool_id = pool.get_id();
    ic_cdk::println!("Processing staking pool: {}", pool_id);

    match sync_nns_neuron(pool_id).await {
      Ok(_) => ic_cdk::println!("Successfully synced NNS neuron for pool {}", pool_id),
      Err(e) => ic_cdk::println!("Failed to sync NNS neuron for pool {}: {}", pool_id, e),
    }
  }

  ic_cdk::println!("NNS neuron check task completed");
  Ok(())
}
