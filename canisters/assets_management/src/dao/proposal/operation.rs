use ic_cdk::update;
use types::{assets_management::ProposalId, sys::ExteralCanisterLabels};

use crate::system_configs::get_exteral_canister_id;

use super::{stable_structures::ProposalInstructionType, utils::query_proposal};

#[update]
fn execute_proposal(proposal_id: ProposalId) -> Result<(), String> {
  // Check if the proposal is valid
  if proposal_id == 0 {
    return Err("Invalid proposal ID".to_string());
  }

  let proposal = query_proposal(proposal_id)?;

  let instruction = proposal.get_proposal_instruction();

  let instruction_type = instruction.get_instruction_type();

  // Execute the proposal
  match instruction_type {
    ProposalInstructionType::NNSStake {
      pool_id,
      amount,
      duration,
      neuron_id,
    } => {
      // Execute transfer logic
      let staking_canister_id = get_exteral_canister_id(ExteralCanisterLabels::Staking);
      // ...
      Ok(())
    }
    ProposalInstructionType::JackpotInvestment { pool_id, jackpot_id, amount } => {
      // Execute jackpot investment logic
      // ...
      Ok(())
    }
    ProposalInstructionType::None => {
      // No action needed
      Ok(())
    }
  }
}
