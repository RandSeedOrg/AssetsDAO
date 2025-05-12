use std::borrow::Cow;

use candid::{CandidType, Decode, Encode, Principal};
use ic_ledger_types::BlockIndex;
use ic_stable_structures::{storable::Bound, Storable};
use serde::{Deserialize, Serialize};
use types::{stable_structures::new_entity_id, staking::{StakingAccountId, StakingPoolId}, EntityId, TimestampNanos};

use crate::{account::stable_structures::StakingAccount, pool::stable_structures::{StakingPool, StakingPoolStatus}, reward::stable_structures::StakingReward};

use super::{STAKING_EVENT_LOG_ID, STAKING_EVENT_LOG_MAP};

/// Event log structure，Ability to store event logs，Used for querying、Analysis and replay
#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct EventLog {
  /// Event logID
  /// Event logIDIt's a self-increasingID
  pub id: Option<EntityId>,
  /// The person who triggered the event
  pub principal: Option<Principal>,
  /// Event Type
  pub event_type: Option<EventType>,
  /// Time of event occurrence
  pub event_time: Option<TimestampNanos>,
}

impl EventLog {
  /// Create event log
  pub fn new(event_type: EventType) -> Self {
    // Generate event logID
    let id = STAKING_EVENT_LOG_ID.with(|id_seq| new_entity_id(id_seq));

    Self {
      id: Some(id),
      principal: Some(ic_cdk::api::msg_caller()),
      event_type: Some(event_type),
      event_time: Some(ic_cdk::api::time()),
    }
  }

  pub fn get_id(&self) -> EntityId {
    self.id.unwrap_or_default()
  }

  pub fn get_event_type(&self) -> EventType {
    self.event_type.clone().unwrap()
  }

  pub fn get_trigger(&self) -> Principal {
    self.principal.unwrap_or(Principal::anonymous())
  }

  pub fn get_trigger_user_id(&self) -> String {
    self.get_trigger().to_text()
  }

  pub fn get_event_time(&self) -> TimestampNanos {
    self.event_time.unwrap_or_default()
  }

  pub fn save_to_stable_memory(&self) {
    // Save event logs to stable memory
   STAKING_EVENT_LOG_MAP.with(|map| {
      map.borrow_mut().insert(self.get_id(), self.clone());
    });
  }
}

/// On-chain address
pub type OnChainAddress = String;
pub type PayCenterCanisterId = String;
pub type ErrorMessage = String;

/// Event type enumeration
/// Event Type includes:
/// 1. Financing: Create a new stake pool
/// 2. Stake category: Add new stake, redeem stake, stake pool status change
/// 3. Reward distribution category: Rewards issued
#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub enum EventType {
  CreateStakingPool(StakingPool),
  UpdateStakingPool(StakingPool),
  
  CreateStakingAccount(StakingAccount),
  UpdateStakingAccount(StakingAccount),
  DeleteStakingAccount(StakingAccountId),

  ChangeStakingPoolStatus(StakingPoolId, StakingPoolStatus),
  ChangeStakingPoolClientVisible(StakingPoolId, bool),
  
  Stake(StakingPool, StakingAccount),
  Unstake(StakingPool, StakingAccount),
  Dissolve(StakingAccount),
  DistributeReward(StakingReward, StakingAccount),
  RewardReceived(StakingReward),

  /// The following event logs will not have substantial storage changes
  StakePayCenterTransferStart(StakingAccountId, PayCenterCanisterId),
  StakePayCenterTransferOk(StakingAccountId, PayCenterCanisterId, BlockIndex),
  StakePayCenterTransferErr(StakingAccountId, PayCenterCanisterId, ErrorMessage),
  StakeTransferStart(StakingAccountId, StakingPoolId),
  StakeTransferOk(StakingAccountId, StakingPoolId, BlockIndex),
  StakeTransferErr(StakingAccountId, StakingPoolId, ErrorMessage),

  UnstakeTransferStart(StakingAccountId, StakingPoolId),
  UnstakeTransferOk(StakingAccountId, StakingPoolId, BlockIndex),
  UnstakeTransferErr(StakingAccountId, StakingPoolId, ErrorMessage),
  UnstakePenaltyTransferStart(StakingAccountId, StakingPoolId),
  UnstakePenaltyTransferOk(StakingAccountId, StakingPoolId, BlockIndex),
  UnstakePenaltyTransferErr(StakingAccountId, StakingPoolId, ErrorMessage),
  UnstakePenaltyPayCenterStart(StakingAccountId, PayCenterCanisterId),
  UnstakePenaltyPayCenterOk(StakingAccountId, PayCenterCanisterId, BlockIndex),
  UnstakePenaltyPayCenterErr(StakingAccountId, PayCenterCanisterId, ErrorMessage),

  DissolvePayCenterTransferStart(StakingAccountId, PayCenterCanisterId),
  DissolvePayCenterTransferOk(StakingAccountId, PayCenterCanisterId, BlockIndex),
  DissolvePayCenterTransferErr(StakingAccountId, PayCenterCanisterId, ErrorMessage),
  DissolvePayCenterReceiveStart(StakingAccountId, PayCenterCanisterId, BlockIndex),
  DissolvePayCenterReceiveOk(StakingAccountId, PayCenterCanisterId, BlockIndex, u64),
  DissolvePayCenterReceiveErr(StakingAccountId, PayCenterCanisterId, ErrorMessage),
}

impl Storable for EventLog {
  fn to_bytes(&self) -> Cow<[u8]> {
    Cow::Owned(Encode!(self).unwrap())
  }

  fn from_bytes(bytes: Cow<[u8]>) -> Self {
    Decode!(bytes.as_ref(), Self).unwrap()
  }

  const BOUND: Bound = Bound::Unbounded;
}