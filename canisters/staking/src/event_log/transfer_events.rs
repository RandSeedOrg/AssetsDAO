use ic_ledger_types::BlockIndex;
use types::staking::{StakingAccountId, StakingPoolId};

use super::stable_structures::{ErrorMessage, EventLog, EventType, PayCenterCanisterId};


/// Stake: Event log of Staking transfer initiated from the payment center to the Staking account-start
pub fn save_stake_pay_center_transfer_start_event(account_id: StakingAccountId, pay_center_canister_id: PayCenterCanisterId) {
  EventLog::new(EventType::StakePayCenterTransferStart(account_id, pay_center_canister_id))
      .save_to_stable_memory()
}

/// Stake: Event log of Staking transfer initiated from the payment center to the Staking account-success
pub fn save_stake_pay_center_transfer_ok_event(account_id: StakingAccountId, pay_center_canister_id: PayCenterCanisterId, tx_id: BlockIndex) {
  EventLog::new(EventType::StakePayCenterTransferOk(account_id, pay_center_canister_id, tx_id))
      .save_to_stable_memory()
}

/// Stake: Event log of Staking transfer initiated from the payment center to the Staking account-fail
pub fn save_stake_pay_center_transfer_fail_event(account_id: StakingAccountId, pay_center_canister_id: PayCenterCanisterId, error_massage: ErrorMessage) {
  EventLog::new(EventType::StakePayCenterTransferErr(account_id, pay_center_canister_id, error_massage))
      .save_to_stable_memory()
}

/// Stake: Log of transfer events from Staking account to Staking pool-start
pub fn save_stake_transfer_start_event(account_id: StakingAccountId, pool_id: StakingPoolId) {
  EventLog::new(EventType::StakeTransferStart(account_id, pool_id))
      .save_to_stable_memory()
}

/// Stake: Log of transfer events from Staking account to Staking pool-success
pub fn save_stake_transfer_ok_event(account_id: StakingAccountId, pool_id: StakingPoolId, tx_id: BlockIndex) {
  EventLog::new(EventType::StakeTransferOk(account_id, pool_id, tx_id))
      .save_to_stable_memory()
}

/// Stake: Log of transfer events from Staking account to Staking pool-fail
pub fn save_stake_transfer_fail_event(account_id: StakingAccountId, pool_id: StakingPoolId, error_massage: ErrorMessage) {
  EventLog::new(EventType::StakeTransferErr(account_id, pool_id, error_massage))
      .save_to_stable_memory()
}

/// Unstake：Transfer Event Log from Staking Pool to Staking Account-start
pub fn save_unstake_transfer_start_event(account_id: StakingAccountId, pool_id: StakingPoolId) {
  EventLog::new(EventType::UnstakeTransferStart(account_id, pool_id))
      .save_to_stable_memory()
}

/// Unstake：Transfer Event Log from Staking Pool to Staking Account-success
pub fn save_unstake_transfer_ok_event(account_id: StakingAccountId, pool_id: StakingPoolId, tx_id: BlockIndex) {
  EventLog::new(EventType::UnstakeTransferOk(account_id, pool_id, tx_id))
      .save_to_stable_memory()
}

/// Unstake：Transfer Event Log from Staking Pool to Staking Account-fail
pub fn save_unstake_transfer_fail_event(account_id: StakingAccountId, pool_id: StakingPoolId, error_massage: ErrorMessage) {
  EventLog::new(EventType::UnstakeTransferErr(account_id, pool_id, error_massage))
      .save_to_stable_memory()
}

/// Unstake：Transfer event log from Staking pool to payment center-start
pub fn save_unstake_penalty_transfer_start_event(account_id: StakingAccountId, pool_id: StakingPoolId) {
  EventLog::new(EventType::UnstakePenaltyTransferStart(account_id, pool_id))
      .save_to_stable_memory()
}

/// Unstake：Transfer event log from Staking pool to payment center-success
pub fn save_unstake_penalty_transfer_ok_event(account_id: StakingAccountId, pool_id: StakingPoolId, tx_id: BlockIndex) {
  EventLog::new(EventType::UnstakePenaltyTransferOk(account_id, pool_id, tx_id))
      .save_to_stable_memory()
}

/// Unstake：Transfer event log from Staking pool to payment center-fail
pub fn save_unstake_penalty_transfer_fail_event(account_id: StakingAccountId, pool_id: StakingPoolId, error_massage: ErrorMessage) {
  EventLog::new(EventType::UnstakePenaltyTransferErr(account_id, pool_id, error_massage))
      .save_to_stable_memory()
}

/// Unstake：Notify the payment center of accounting event log-start
pub fn save_unstake_penalty_pay_center_start_event(account_id: StakingAccountId, pay_center_canister_id: PayCenterCanisterId) {
  EventLog::new(EventType::UnstakePenaltyPayCenterStart(account_id, pay_center_canister_id))
      .save_to_stable_memory()
}

/// Unstake：Notify the payment center of accounting event log-success
pub fn save_unstake_penalty_pay_center_ok_event(account_id: StakingAccountId, pay_center_canister_id: PayCenterCanisterId, tx_id: BlockIndex) {
  EventLog::new(EventType::UnstakePenaltyPayCenterOk(account_id, pay_center_canister_id, tx_id))
      .save_to_stable_memory()
}

/// Unstake：Notify the payment center of accounting event log-fail
pub fn save_unstake_penalty_pay_center_fail_event(account_id: StakingAccountId, pay_center_canister_id: PayCenterCanisterId, error_massage: ErrorMessage) {
  EventLog::new(EventType::UnstakePenaltyPayCenterErr(account_id, pay_center_canister_id, error_massage))
      .save_to_stable_memory()
}

/// Dissolve：On-chain transfer event log from Staking account to payment center-start
pub fn save_dissolve_pay_center_transfer_start_event(account_id: StakingAccountId, pay_center_canister_id: PayCenterCanisterId) {
  EventLog::new(EventType::DissolvePayCenterTransferStart(account_id, pay_center_canister_id))
      .save_to_stable_memory()
}

/// Dissolve：On-chain transfer event log from Staking account to payment center-success
pub fn save_dissolve_pay_center_transfer_ok_event(account_id: StakingAccountId, pay_center_canister_id: PayCenterCanisterId, tx_id: BlockIndex) {
  EventLog::new(EventType::DissolvePayCenterTransferOk(account_id, pay_center_canister_id, tx_id))
      .save_to_stable_memory()
}

/// Dissolve：On-chain transfer event log from Staking account to payment center-fail
pub fn save_dissolve_pay_center_transfer_fail_event(account_id: StakingAccountId, pay_center_canister_id: PayCenterCanisterId, error_massage: ErrorMessage) {
  EventLog::new(EventType::DissolvePayCenterTransferErr(account_id, pay_center_canister_id, error_massage))
      .save_to_stable_memory()
}

/// Dissolve：Payment Center Accounting Event Log-start
pub fn save_dissolve_pay_center_receive_start_event(account_id: StakingAccountId, pay_center_canister_id: PayCenterCanisterId, tx_id: BlockIndex) {
  EventLog::new(EventType::DissolvePayCenterReceiveStart(account_id, pay_center_canister_id, tx_id))
      .save_to_stable_memory()
}

/// Dissolve：Payment Center Accounting Event Log-success
pub fn save_dissolve_pay_center_receive_ok_event(account_id: StakingAccountId, pay_center_canister_id: PayCenterCanisterId, tx_id: BlockIndex, pay_center_tx_id: u64) {
  EventLog::new(EventType::DissolvePayCenterReceiveOk(account_id, pay_center_canister_id, tx_id, pay_center_tx_id))
      .save_to_stable_memory()
}

/// Dissolve：Payment Center Accounting Event Log-fail
pub fn save_dissolve_pay_center_receive_fail_event(account_id: StakingAccountId, pay_center_canister_id: PayCenterCanisterId, error_massage: ErrorMessage) {
  EventLog::new(EventType::DissolvePayCenterReceiveErr(account_id, pay_center_canister_id, error_massage))
      .save_to_stable_memory()
}