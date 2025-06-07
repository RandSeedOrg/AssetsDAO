use ic_cdk::update;
use system_configs_macro::has_permission_result;
use types::{assets_management::ProposalId, sys::ExteralCanisterLabels};

use crate::{
  guard_keys::get_execute_proposal_guard_key, parallel_guard::EntryGuard, system_configs::get_exteral_canister_id,
  transfer_address::stable_structures::TransferAddress,
};

use super::{
  stable_structures::{ProposalInstructionType, ProposalStatus},
  utils::query_proposal,
};

#[update]
#[has_permission_result("assets_management::proposal::execute")]
async fn execute_proposal(proposal_id: ProposalId) -> Result<u64, String> {
  if proposal_id == 0 {
    return Err("Invalid proposal ID".to_string());
  }

  let _entry_guard = EntryGuard::new(get_execute_proposal_guard_key(proposal_id))
    .map_err(|_| format!("Failed to acquire entry guard for proposal {}", proposal_id))?;

  let mut proposal = query_proposal(proposal_id)?;

  if proposal.get_status() != ProposalStatus::Passed {
    return Err(format!("Proposal {} is not in Passed status", proposal_id));
  }

  let instruction = proposal.get_proposal_instruction();

  match instruction {
    ProposalInstructionType::NNSStake {
      pool_id,
      amount,
      neuron_id: _,
    } => {
      let staking_canister_id = get_exteral_canister_id(ExteralCanisterLabels::Staking);

      let result = ic_cdk::call::Call::unbounded_wait(staking_canister_id, "stake_to_nns_neuron")
        .with_args(&(proposal_id, pool_id, amount))
        .await
        .map_err(|e| format!("Call Ledger failed: {:?}", e))?;

      match result.candid::<Result<u64, String>>() {
        Ok(Ok(neuron_id)) => {
          ic_cdk::println!("Successfully staked {} ICP to NNS neuron with ID: {}", amount, neuron_id);

          proposal.executed_nns_stake(neuron_id)?;

          Ok(neuron_id)
        }
        Ok(Err(error)) => Err(format!("Transfer failed: {:?}", error)),
        Err(error) => Err(format!("Transfer failed: {:?}", error)),
      }
    }
    ProposalInstructionType::JackpotInvestment {
      pool_id: _,
      jackpot_id: _,
      amount: _,
    } => Err("Jackpot investment not implemented".to_string()),
    ProposalInstructionType::AddTransferAddress {
      id: _,
      name,
      usage,
      network,
      crypto,
      address,
      address_type,
    } => {
      let transfer_address = TransferAddress::new(proposal_id, name, usage, network, crypto, address, address_type)?;

      proposal.executed_add_transfer_address(transfer_address.get_id())?;

      Ok(transfer_address.get_id())
    }
    ProposalInstructionType::None => Err("No action needed for None instruction".to_string()),
  }
}

#[update]
#[has_permission_result("assets_management::proposal::change_status")]
fn change_status(proposal_id: ProposalId, status: String) -> Result<(), String> {
  if proposal_id == 0 {
    return Err("Invalid proposal ID".to_string());
  }

  let mut proposal = query_proposal(proposal_id)?;

  let target_status = ProposalStatus::try_from(status.as_str()).map_err(|_| format!("Invalid proposal status: {}", status))?;

  let current_status = proposal.get_status();

  if current_status == target_status {
    return Err(format!("Proposal {} is already in status: {}", proposal_id, status));
  }

  if current_status != ProposalStatus::Created {
    return Err(format!("Proposal {} is not in Created status", proposal_id));
  }

  if target_status != ProposalStatus::Passed && target_status != ProposalStatus::Rejected {
    return Err(format!("Invalid target status: {}", target_status));
  }

  proposal.set_status(target_status);

  proposal.update_to_stable();

  Ok(())
}
