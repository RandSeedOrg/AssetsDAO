use nns_governance_api::{
  get_governance,
  nns_governance_api::{
    By, ClaimOrRefresh, ClaimOrRefreshNeuronFromAccount, Command1, DisburseToNeuron, ManageNeuronCommandRequest, ManageNeuronRequest, NeuronId,
    NeuronIdOrSubaccount,
  },
};
use types::{staking::StakingPoolId, E8S};

use crate::on_chain::address::generate_staking_pool_neuron_account;

pub async fn refresh_nns_neuron_by_pool(pool_id: StakingPoolId) -> Result<u64, String> {
  let governance = get_governance();

  let (resp,) = governance
    .manage_neuron(ManageNeuronRequest {
      id: None,
      command: Some(ManageNeuronCommandRequest::ClaimOrRefresh(ClaimOrRefresh {
        by: Some(By::MemoAndController(ClaimOrRefreshNeuronFromAccount {
          controller: Some(ic_cdk::api::canister_self()),
          memo: pool_id,
        })),
      })),
      neuron_id_or_subaccount: Some(NeuronIdOrSubaccount::Subaccount(serde_bytes::ByteBuf::from(
        generate_staking_pool_neuron_account(pool_id).as_bytes(),
      ))),
    })
    .await
    .map_err(|e| {
      ic_cdk::println!("Failed to create NNS neuron by pool: code = {:?}, msg = {}", e.0, e.1);
      format!("Failed to create NNS neuron by pool: code = {:?}, msg = {}", e.0, e.1)
    })?;

  ic_cdk::println!("NNS manage neuron resp: {:?}", resp);

  match resp.command {
    Some(Command1::ClaimOrRefresh(resp)) => {
      ic_cdk::println!("Successfully created NNS neuron by pool: {:?}", pool_id);

      match resp.refreshed_neuron_id {
        Some(neuron_id) => {
          ic_cdk::println!("Neuron ID: {}", neuron_id.id);
          Ok(neuron_id.id)
        }
        None => {
          ic_cdk::println!("No refreshed neuron ID returned");
          Err("No refreshed neuron ID returned".to_string())
        }
      }
    }
    None => {
      ic_cdk::println!("Failed to create NNS neuron none by pool: {}", pool_id);
      Err("Failed to create NNS neuron none".to_string())
    }
    _ => {
      ic_cdk::println!("Failed to create NNS neuron by pool: {}", pool_id);
      Err("Failed to create NNS neuron".to_string())
    }
  }
}

pub async fn disburse_to_neuron(neuron_id: u64, amount: E8S, dissolve_delay_seconds: u64, pool_id: StakingPoolId) -> Result<(), String> {
  let governance = get_governance();

  let (resp,) = governance
    .manage_neuron(ManageNeuronRequest {
      id: Some(NeuronId { id: neuron_id }),
      command: Some(ManageNeuronCommandRequest::DisburseToNeuron(DisburseToNeuron {
        dissolve_delay_seconds,
        kyc_verified: false,
        amount_e8s: amount,
        new_controller: None,
        nonce: pool_id,
      })),
      neuron_id_or_subaccount: Some(NeuronIdOrSubaccount::NeuronId(NeuronId { id: neuron_id })),
    })
    .await
    .map_err(|e| {
      ic_cdk::println!("Failed to disburse to neuron: {:?}", e);
      "Failed to disburse to neuron".to_string()
    })?;

  if resp.command.is_none() {
    ic_cdk::println!("Failed to disburse to neuron: No command returned");
    return Err("Failed to disburse to neuron: No command returned".to_string());
  }

  match resp.command.unwrap() {
    Command1::DisburseToNeuron(_) => {
      ic_cdk::println!("Successfully disbursed {} to neuron {}", amount, neuron_id);
      Ok(())
    }
    _ => {
      ic_cdk::println!("Failed to disburse to neuron: {:?}", neuron_id);
      Err("Failed to disburse to neuron".to_string())
    }
  }
}
