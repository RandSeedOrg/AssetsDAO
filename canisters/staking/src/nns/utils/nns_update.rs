use candid::Principal;
use nns_governance_api::{
  get_governance,
  nns_governance_api::{
    AccountIdentifier, AddHotKey, By, ClaimOrRefresh, ClaimOrRefreshNeuronFromAccount, Command1, Configure, Disburse, DisburseToNeuron,
    IncreaseDissolveDelay, ManageNeuronCommandRequest, ManageNeuronRequest, NeuronId, NeuronIdOrSubaccount, Operation, RemoveHotKey,
  },
};
use types::{staking::StakingPoolId, E8S};

use crate::{
  nns::utils::ledger_utils::query_transaction_by_block_height,
  on_chain::address::{generate_staking_pool_account_identifier, generate_staking_pool_neuron_account},
};

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

pub async fn add_hot_key(neuron_id: u64, hotkey: String) -> Result<(), String> {
  let governance = get_governance();

  let (resp,) = governance
    .manage_neuron(ManageNeuronRequest {
      id: Some(NeuronId { id: neuron_id }),
      command: Some(ManageNeuronCommandRequest::Configure(Configure {
        operation: Some(Operation::AddHotKey(AddHotKey {
          new_hot_key: Some(Principal::from_text(&hotkey).map_err(|e| {
            ic_cdk::println!("Invalid hotkey principal: {}", e);
            "Invalid hotkey principal".to_string()
          })?),
        })),
      })),
      neuron_id_or_subaccount: None,
    })
    .await
    .map_err(|e| {
      ic_cdk::println!("Failed to add hotkey to neuron0: {:?}", e);
      "Failed to add hotkey to neuron".to_string()
    })?;

  if resp.command.is_none() {
    ic_cdk::println!("Failed to add hotkey to neuron1: No command returned");
    return Err("Failed to add hotkey to neuron: No command returned".to_string());
  }

  match resp.command.unwrap() {
    Command1::Configure {} => {
      ic_cdk::println!("Successfully added hotkey {} to neuron {}", hotkey, neuron_id);
      Ok(())
    }
    Command1::Error(err) => {
      ic_cdk::println!("Failed to add hotkey to neuron2: {:?}", err.clone());
      Err(format!("Failed to add hotkey to neuron"))
    }
    _ => {
      ic_cdk::println!("Failed to add hotkey to neuron3: {:?}", neuron_id);
      Err("Failed to add hotkey to neuron".to_string())
    }
  }
}

pub async fn remove_hot_key(neuron_id: u64, hotkey: String) -> Result<(), String> {
  let governance = get_governance();

  let (resp,) = governance
    .manage_neuron(ManageNeuronRequest {
      id: Some(NeuronId { id: neuron_id }),
      command: Some(ManageNeuronCommandRequest::Configure(Configure {
        operation: Some(Operation::RemoveHotKey(RemoveHotKey {
          hot_key_to_remove: Some(Principal::from_text(&hotkey).map_err(|e| {
            ic_cdk::println!("Invalid hotkey principal: {}", e);
            "Invalid hotkey principal".to_string()
          })?),
        })),
      })),
      neuron_id_or_subaccount: None,
    })
    .await
    .map_err(|e| {
      ic_cdk::println!("Failed to remove hotkey from neuron: {:?}", e);
      "Failed to remove hotkey from neuron".to_string()
    })?;

  if resp.command.is_none() {
    ic_cdk::println!("Failed to remove hotkey from neuron: No command returned");
    return Err("Failed to remove hotkey from neuron: No command returned".to_string());
  }

  match resp.command.unwrap() {
    Command1::Configure {} => {
      ic_cdk::println!("Successfully removed hotkey {} from neuron {}", hotkey, neuron_id);
      Ok(())
    }
    _ => {
      ic_cdk::println!("Failed to remove hotkey from neuron: {:?}", neuron_id);
      Err("Failed to remove hotkey from neuron".to_string())
    }
  }
}

pub async fn increase_dissolve_delay(neuron_id: u64, additional_delay_seconds: u32) -> Result<(), String> {
  let governance = get_governance();

  let (resp,) = governance
    .manage_neuron(ManageNeuronRequest {
      id: Some(NeuronId { id: neuron_id }),
      command: Some(ManageNeuronCommandRequest::Configure(Configure {
        operation: Some(Operation::IncreaseDissolveDelay(IncreaseDissolveDelay {
          additional_dissolve_delay_seconds: additional_delay_seconds,
        })),
      })),
      neuron_id_or_subaccount: None,
    })
    .await
    .map_err(|e| {
      ic_cdk::println!("Failed to increase dissolve delay for neuron: {:?}", e);
      "Failed to increase dissolve delay for neuron".to_string()
    })?;

  if resp.command.is_none() {
    ic_cdk::println!("Failed to increase dissolve delay for neuron: No command returned");
    return Err("Failed to increase dissolve delay for neuron: No command returned".to_string());
  }

  match resp.command.unwrap() {
    Command1::Configure {} => {
      ic_cdk::println!("Successfully increased dissolve delay for neuron {}", neuron_id);
      Ok(())
    }
    _ => {
      ic_cdk::println!("Failed to increase dissolve delay for neuron: {:?}", neuron_id);
      Err("Failed to increase dissolve delay for neuron".to_string())
    }
  }
}

pub async fn start_dissolve(neuron_id: u64) -> Result<(), String> {
  let governance = get_governance();

  let (resp,) = governance
    .manage_neuron(ManageNeuronRequest {
      id: Some(NeuronId { id: neuron_id }),
      command: Some(ManageNeuronCommandRequest::Configure(Configure {
        operation: Some(Operation::StartDissolving {}),
      })),
      neuron_id_or_subaccount: None,
    })
    .await
    .map_err(|e| {
      ic_cdk::println!("Failed to start dissolve for neuron: {:?}", e);
      "Failed to start dissolve for neuron".to_string()
    })?;

  if resp.command.is_none() {
    ic_cdk::println!("Failed to start dissolve for neuron: No command returned");
    return Err("Failed to start dissolve for neuron: No command returned".to_string());
  }

  match resp.command.unwrap() {
    Command1::Configure {} => {
      ic_cdk::println!("Successfully started dissolve for neuron {}", neuron_id);
      Ok(())
    }
    _ => {
      ic_cdk::println!("Failed to start dissolve for neuron: {:?}", neuron_id);
      Err("Failed to start dissolve for neuron".to_string())
    }
  }
}

pub async fn stop_dissolve(neuron_id: u64) -> Result<(), String> {
  let governance = get_governance();

  let (resp,) = governance
    .manage_neuron(ManageNeuronRequest {
      id: Some(NeuronId { id: neuron_id }),
      command: Some(ManageNeuronCommandRequest::Configure(Configure {
        operation: Some(Operation::StopDissolving {}),
      })),
      neuron_id_or_subaccount: None,
    })
    .await
    .map_err(|e| {
      ic_cdk::println!("Failed to stop dissolve for neuron: {:?}", e);
      "Failed to stop dissolve for neuron".to_string()
    })?;

  if resp.command.is_none() {
    ic_cdk::println!("Failed to stop dissolve for neuron: No command returned");
    return Err("Failed to stop dissolve for neuron: No command returned".to_string());
  }

  match resp.command.unwrap() {
    Command1::Configure {} => {
      ic_cdk::println!("Successfully stopped dissolve for neuron {}", neuron_id);
      Ok(())
    }
    _ => {
      ic_cdk::println!("Failed to stop dissolve for neuron: {:?}", neuron_id);
      Err("Failed to stop dissolve for neuron".to_string())
    }
  }
}

// Distribute the staked ICP in the neuron back to the staking pool account.
pub async fn nns_disburse(neuron_id: u64, pool_id: StakingPoolId) -> Result<(), String> {
  let governance = get_governance();

  let account_identifier = generate_staking_pool_account_identifier(pool_id);

  let (resp,) = governance
    .manage_neuron(ManageNeuronRequest {
      id: Some(NeuronId { id: neuron_id }),
      command: Some(ManageNeuronCommandRequest::Disburse(Disburse {
        to_account: Some(AccountIdentifier {
          hash: serde_bytes::ByteBuf::from(account_identifier.as_bytes()),
        }),
        amount: None,
      })),
      neuron_id_or_subaccount: None,
    })
    .await
    .map_err(|e| {
      ic_cdk::println!("Failed to disburse for neuron: {:?}", e);
      "Failed to disburse for neuron".to_string()
    })?;

  if resp.command.is_none() {
    ic_cdk::println!("Failed to disburse for neuron: No command returned");
    return Err("Failed to disburse for neuron: No command returned".to_string());
  }

  match resp.command.unwrap() {
    Command1::Disburse(disburse_resp) => {
      let tx_id = disburse_resp.transfer_block_height;

      ic_cdk::println!("Successfully disbursed for neuron {} with tx_id {}", neuron_id, tx_id);

      let tx_info = query_transaction_by_block_height(tx_id).await?;

      // Record the unstake transaction of the NNS neuron
      crate::pool_transaction_record::utils::record_nns_unstake_transaction(pool_id, neuron_id, tx_info.amount, tx_id, tx_info.timestamp)?;

      Ok(())
    }
    _ => {
      ic_cdk::println!("Failed to disburse for neuron: {:?}", neuron_id);
      Err("Failed to disburse for neuron".to_string())
    }
  }
}
