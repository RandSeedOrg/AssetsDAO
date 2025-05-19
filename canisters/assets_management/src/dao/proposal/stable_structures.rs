use std::borrow::Cow;

use candid::{CandidType, Decode, Encode};
use ic_stable_structures::{Storable, storable::Bound};
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};
use types::{
  E8S, UserId,
  assets_management::{JackpotId, ProposalId},
  stable_structures::MetaData,
  staking::StakingPoolId,
};

#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct Proposal {
  pub id: Option<ProposalId>,
  pub title: Option<String>,
  pub description: Option<String>,
  pub status: Option<ProposalStatus>,
  pub proposal_initiator: Option<UserId>,
  pub proposal_instruction: Option<ProposalInstruction>,
  pub meta: Option<MetaData>,
}

#[derive(EnumString, Display, Debug, Clone, Serialize, Deserialize, CandidType)]
pub enum ProposalStatus {
  /// New creations can be edited, but cannot be changed until voting is open
  #[strum(serialize = "0")]
  Created,
  /// During the open voting phase, all staking accounts can vote until the voting ends.
  #[strum(serialize = "1")]
  OpenVoting,
  /// The vote is passed, and the proposal instructions can be executed at this time
  #[strum(serialize = "2")]
  Approved,
  /// Vote rejected. The proposal has not passed multi-signature and will not be executed. Only the status will be recorded.
  #[strum(serialize = "3")]
  Rejected,
  /// Executed, the proposal has been executed, and permanent changes will be made according to the proposal instructions
  #[strum(serialize = "4")]
  Executed,
}

/// Proposal instructions are used to accurately describe the actions to be performed.
/// The execution time, and the execution results of the proposal
#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct ProposalInstruction {
  pub instruction_type: Option<ProposalInstructionType>,
  pub instruction_status: Option<ProposalInstructionStatus>,
  pub meta: Option<MetaData>,
}

/// The Proposal Instruction Type, which describes the purpose of the instruction and the metadata required for the instruction to execute
#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub enum ProposalInstructionType {
  /// Stake the specified amount in the specified staking pool into the NNS neuron
  NNSStake { staking_pool_id: StakingPoolId, amount: E8S },
  /// Transfer the specified amount of funds in the staking pool to the jackpot account
  JackpotInvestment { jackpot_id: JackpotId, amount: E8S },
}

/// The status of the proposal instruction
#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub enum ProposalInstructionStatus {
  /// The proposal has not been voted on yet, so the instructions are not ready
  NotReady,
  /// The proposal has been voted through and is awaiting execution
  Pending,
  /// The instruction is being executed
  InProgress,
  /// The instruction was executed successfully
  Succeed,
  /// The instruction was executed but failed
  Failed,
}

impl Storable for Proposal {
  fn to_bytes(&self) -> Cow<[u8]> {
    Cow::Owned(Encode!(self).unwrap())
  }

  fn from_bytes(bytes: Cow<[u8]>) -> Self {
    Decode!(bytes.as_ref(), Self).unwrap()
  }

  const BOUND: Bound = Bound::Unbounded;
}
