use candid::CandidType;
use ic_ledger_types::BlockIndex;
use serde::{Deserialize, Serialize};
use types::{assets_management::ProposalId, stable_structures::MetaData, staking::StakingPoolId, E8S};

use super::stable_structures::{NnsStakeExecuteRecord, NnsStakeExecuteStatus};

/// NNS staking execute record for transfer layer (without Option wrappers)
#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct NnsStakeExecuteRecordVo {
  /// Associated proposal ID
  pub proposal_id: ProposalId,
  /// Associated staking pool ID
  pub pool_id: StakingPoolId,
  /// NNS neuron ID
  pub neuron_id: u64,
  /// Block index of the transfer from the staking pool to the NNS neuron
  pub pool_to_neuron_transfer_block_index: BlockIndex,
  /// Staked amount (unit: e8s)
  pub amount: E8S,
  /// Record status
  pub status: NnsStakeExecuteStatus,
  /// Metadata
  pub meta: MetaData,
}

impl From<NnsStakeExecuteRecord> for NnsStakeExecuteRecordVo {
  fn from(record: NnsStakeExecuteRecord) -> Self {
    Self {
      proposal_id: record.proposal_id.unwrap_or_default(),
      pool_id: record.pool_id.unwrap_or_default(),
      neuron_id: record.neuron_id.unwrap_or_default(),
      pool_to_neuron_transfer_block_index: record.pool_to_neuron_transfer_block_index.unwrap_or_default(),
      amount: record.amount.unwrap_or_default(),
      status: record.status.unwrap_or(NnsStakeExecuteStatus::Success),
      meta: record.meta.unwrap_or_default(),
    }
  }
}
