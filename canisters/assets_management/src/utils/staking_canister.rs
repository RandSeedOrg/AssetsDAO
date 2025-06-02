use types::sys::ExteralCanisterLabels;

use crate::system_configs::get_exteral_canister_id;

pub async fn stake_to_nns_neuron(pool_id: u64, amount: u64) -> Result<u64, String> {
  let staking_canister_id = get_exteral_canister_id(ExteralCanisterLabels::Staking);

  let result = ic_cdk::call::Call::unbounded_wait(staking_canister_id, "stake_to_nns_neuron")
    .with_args(&(pool_id, amount))
    .await
    .map_err(|e| format!("Call staking method stake_to_nns_neuron failed: {:?}", e))?
    .candid::<Result<u64, String>>()
    .map_err(|e| format!("Candid decoding failed: {:?}", e))?;

  result
}
