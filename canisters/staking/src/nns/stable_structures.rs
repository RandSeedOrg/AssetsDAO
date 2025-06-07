use std::borrow::Cow;

use super::NNS_STAKING_EXECUTE_RECORD_MAP;
use candid::{CandidType, Decode, Encode};
use ic_ledger_types::BlockIndex;
use ic_stable_structures::{storable::Bound, Storable};
use serde::{Deserialize, Serialize};
use types::{assets_management::ProposalId, stable_structures::MetaData, staking::StakingPoolId, E8S};

/// NNS neuron staking record status
#[derive(Debug, Clone, Serialize, Deserialize, CandidType, PartialEq)]
pub enum NnsStakeExecuteStatus {
  /// Error refreshing NNS neuron
  RefreshToNnsNeuronError(String, String),
  /// Proposal executed successfully
  Success,
}

/// NNS staking execute record, which is used to track the execution of staking operations
#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct NnsStakeExecuteRecord {
  /// Associated proposal ID
  pub proposal_id: Option<ProposalId>,
  /// Associated staking pool ID
  pub pool_id: Option<StakingPoolId>,
  /// NNS neuron ID
  pub neuron_id: Option<u64>,
  /// Block index of the transfer from the staking pool to the NNS neuron
  pub pool_to_neuron_transfer_block_index: Option<BlockIndex>,
  /// Staked amount (unit: e8s)
  pub amount: Option<E8S>,
  /// Record status
  pub status: Option<NnsStakeExecuteStatus>,
  /// Metadata
  pub meta: Option<MetaData>,
}

impl NnsStakeExecuteRecord {
  /// Creates a new NNS stake execute record
  pub fn init_with(proposal_id: ProposalId, pool_id: StakingPoolId, amount: E8S) -> Self {
    let record = Self::get_with_proposal(proposal_id);

    if let Some(existing_record) = record {
      return existing_record;
    }

    Self {
      proposal_id: Some(proposal_id),
      pool_id: Some(pool_id),
      neuron_id: None,
      pool_to_neuron_transfer_block_index: None,
      amount: Some(amount),
      status: None,
      meta: Some(MetaData::init_create_scene()),
    }
  }

  pub fn get_with_proposal(proposal_id: ProposalId) -> Option<Self> {
    NNS_STAKING_EXECUTE_RECORD_MAP.with(|map| map.borrow().get(&proposal_id))
  }

  pub fn set_pool_to_neuron_transfer_block_index(&mut self, block_index: BlockIndex) {
    self.pool_to_neuron_transfer_block_index = Some(block_index);
    self.update_meta();
  }

  pub fn update_to_success(&mut self, neuron_id: u64) {
    self.neuron_id = Some(neuron_id);
    self.status = Some(NnsStakeExecuteStatus::Success);
    self.update_meta();
    self.update_to_stable();
  }

  pub fn update_to_error(&mut self, neuron_account: String, err_info: String) {
    self.status = Some(NnsStakeExecuteStatus::RefreshToNnsNeuronError(neuron_account, err_info));
    self.update_meta();
    self.update_to_stable();
  }

  pub fn get_proposal_id(&self) -> ProposalId {
    self.proposal_id.unwrap_or_default()
  }

  fn update_meta(&mut self) {
    if let Some(meta) = &mut self.meta {
      self.meta = Some(meta.update());
    }
  }

  pub fn update_to_stable(&self) {
    NNS_STAKING_EXECUTE_RECORD_MAP.with(|map| {
      map.borrow_mut().insert(self.get_proposal_id(), self.clone());
    });
  }

  pub fn get_pool_id(&self) -> StakingPoolId {
    self.pool_id.unwrap_or_default()
  }

  pub fn get_amount(&self) -> E8S {
    self.amount.unwrap_or_default()
  }

  pub fn get_neuron_id(&self) -> u64 {
    self.neuron_id.unwrap_or_default()
  }

  pub fn get_status(&self) -> &NnsStakeExecuteStatus {
    self.status.as_ref().unwrap_or(&NnsStakeExecuteStatus::Success)
  }

  pub fn get_transfer_block_index(&self) -> BlockIndex {
    self.pool_to_neuron_transfer_block_index.unwrap_or_default()
  }

  pub fn get_meta(&self) -> Cow<MetaData> {
    match &self.meta {
      Some(meta) => Cow::Borrowed(meta),
      None => Cow::Owned(MetaData::init_create_scene()),
    }
  }

  pub fn get_updated_at(&self) -> u64 {
    self.get_meta().get_updated_at()
  }
}

impl Storable for NnsStakeExecuteRecord {
  fn to_bytes(&self) -> Cow<[u8]> {
    Cow::Owned(Encode!(self).unwrap())
  }

  fn from_bytes(bytes: Cow<[u8]>) -> Self {
    Decode!(bytes.as_ref(), Self).unwrap()
  }

  const BOUND: Bound = Bound::Unbounded;
}
