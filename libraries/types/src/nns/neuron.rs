use candid::{CandidType, Decode, Encode, Principal};
use ic_stable_structures::{storable::Bound, Storable};
use serde::Deserialize;
use std::borrow::Cow;

#[derive(CandidType, Deserialize, Debug, Clone)]
pub struct NeuronId {
  pub id: u64,
}

#[derive(CandidType, Deserialize, Debug, Clone)]
pub struct BallotInfo {
  pub proposal_id: Option<ProposalId>,
  pub vote: i32,
}

#[derive(CandidType, Deserialize, Debug, Clone)]
pub struct ProposalId {
  pub id: u64,
}

#[derive(CandidType, Deserialize, Debug, Clone)]
pub struct MaturityDisbursement {
  pub timestamp_of_disbursement_seconds: u64,
  pub amount_e8s: u64,
  pub account: Option<serde_bytes::ByteBuf>,
}

#[derive(CandidType, Deserialize, Debug, Clone)]
pub enum DissolveState {
  DissolveTimestampSeconds(u64),
  WhenDissolvedTimestampSeconds(u64),
}

#[derive(CandidType, Deserialize, Debug, Clone)]
pub struct Followees {
  pub followees: Vec<NeuronId>,
}

#[derive(CandidType, Deserialize, Debug, Clone)]
pub struct NeuronStakeTransfer {
  pub transfer_timestamp: u64,
  pub from: Option<Principal>,
  pub from_subaccount: Vec<u8>,
  pub to_subaccount: Vec<u8>,
  pub neuron_stake_e8s: u64,
  pub block_height: u64,
  pub memo: u64,
}

#[derive(CandidType, Deserialize, Debug, Clone)]
pub struct KnownNeuronData {
  pub name: String,
  pub description: Option<String>,
}

#[derive(CandidType, Deserialize, Debug, Clone)]
pub struct Neuron {
  pub id: Option<NeuronId>,
  pub staked_maturity_e8s_equivalent: Option<u64>,
  pub controller: Option<Principal>,
  pub recent_ballots: Vec<BallotInfo>,
  pub voting_power_refreshed_timestamp_seconds: Option<u64>,
  pub kyc_verified: bool,
  pub potential_voting_power: Option<u64>,
  pub neuron_type: Option<i32>,
  pub not_for_profit: bool,
  pub maturity_e8s_equivalent: u64,
  pub deciding_voting_power: Option<u64>,
  pub cached_neuron_stake_e8s: u64,
  pub created_timestamp_seconds: u64,
  pub auto_stake_maturity: Option<bool>,
  pub aging_since_timestamp_seconds: u64,
  pub hot_keys: Vec<Principal>,
  pub account: serde_bytes::ByteBuf,
  pub joined_community_fund_timestamp_seconds: Option<u64>,
  pub maturity_disbursements_in_progress: Option<Vec<MaturityDisbursement>>,
  pub dissolve_state: Option<DissolveState>,
  pub followees: Vec<(i32, Followees)>,
  pub neuron_fees_e8s: u64,
  pub visibility: Option<i32>,
  pub transfer: Option<NeuronStakeTransfer>,
  pub known_neuron_data: Option<KnownNeuronData>,
  pub spawn_at_timestamp_seconds: Option<u64>,
}

impl Storable for Neuron {
  fn to_bytes(&self) -> Cow<[u8]> {
    Cow::Owned(Encode!(self).unwrap())
  }

  fn from_bytes(bytes: Cow<[u8]>) -> Self {
    Decode!(bytes.as_ref(), Self).unwrap()
  }

  const BOUND: Bound = Bound::Unbounded;
}
