use crate::nns::utils::nns_update::refresh_nns_neuron_by_pool;
use crate::pool::crud_utils::get_all_staking_pools;
use ic_cdk::update;
use nns_governance_api::get_governance;

/// Check all NNS neurons dissolve status.
#[update]
pub async fn nns_neuron_check_task() -> Result<(), String> {
  ic_cdk::println!("Starting NNS neuron check task...");

  // Get all staking pool IDs
  let staking_pools = get_all_staking_pools();

  ic_cdk::println!("Found {} staking pools", staking_pools.len());

  // For each staking pool, get/generate the corresponding neuron ID and check its dissolve status
  for pool in staking_pools {
    let pool_id = pool.get_id();
    ic_cdk::println!("Processing staking pool: {}", pool_id);

    // Get or refresh the neuron for this staking pool
    match refresh_nns_neuron_by_pool(pool_id).await {
      Ok(neuron_id) => {
        ic_cdk::println!("Got neuron ID {} for pool {}", neuron_id, pool_id);

        // Check the dissolve status of this neuron
        if let Err(e) = check_neuron_dissolve_status(neuron_id).await {
          ic_cdk::println!("Failed to check dissolve status for neuron {} (pool {}): {}", neuron_id, pool_id, e);
        }
      }
      Err(e) => {
        ic_cdk::println!("Failed to get neuron ID for pool {}: {}", pool_id, e);
      }
    }
  }

  ic_cdk::println!("NNS neuron check task completed");
  Ok(())
}

async fn check_neuron_dissolve_status(neuron_id: u64) -> Result<(), String> {
  let governance_service = get_governance();

  let (neuron_info,) = governance_service
    .get_full_neuron(neuron_id)
    .await
    .map_err(|e| format!("Failed to get NNS neuron info: {:?}", e))?;

  ic_cdk::println!("NNS neuron info: {:?}", neuron_info);

  // Here you can add logic to check the dissolve status and take actions if needed.

  Ok(())
}
