use candid::{CandidType, Decode, Encode};
use ic_stable_structures::{storable::Bound, Storable};
use serde::Deserialize;
use std::borrow::Cow;

use super::neuron::{BallotInfo, KnownNeuronData};

#[derive(CandidType, Deserialize, Debug)]
pub struct NeuronInfo {
  pub dissolve_delay_seconds: u64,
  pub recent_ballots: Vec<BallotInfo>,
  pub voting_power_refreshed_timestamp_seconds: Option<u64>,
  pub potential_voting_power: Option<u64>,
  pub neuron_type: Option<i32>,
  pub deciding_voting_power: Option<u64>,
  pub created_timestamp_seconds: u64,
  pub state: i32,
  pub stake_e8s: u64,
  pub joined_community_fund_timestamp_seconds: Option<u64>,
  pub retrieved_at_timestamp_seconds: u64,
  pub visibility: Option<i32>,
  pub known_neuron_data: Option<KnownNeuronData>,
  pub voting_power: u64,
  pub age_seconds: u64,
}

impl Storable for NeuronInfo {
  fn to_bytes(&self) -> Cow<[u8]> {
    Cow::Owned(Encode!(self).unwrap())
  }

  fn from_bytes(bytes: Cow<[u8]>) -> Self {
    Decode!(bytes.as_ref(), Self).unwrap()
  }

  const BOUND: Bound = Bound::Unbounded;
}
