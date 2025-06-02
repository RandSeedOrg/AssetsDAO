use candid::Principal;
use common_canisters::pay_center::Result2;
use ic_ledger_types::BlockIndex;
use types::{entities::remove_indexed_id, sys::ExteralCanisterLabels};

use crate::{
  account::{
    stable_structures::{StakingAccount, StakingAccountStatus},
    transport_structures::StakingAccountVo,
    STAKING_RECOVERABLE_ERROR_ACCOUNT_INDEX_MAP,
  },
  event_log::{
    stake_and_unstake_events::save_dissolve_event,
    transfer_events::{
      save_dissolve_pay_center_receive_fail_event, save_dissolve_pay_center_receive_ok_event, save_dissolve_pay_center_receive_start_event,
      save_stake_pay_center_transfer_fail_event,
    },
  },
  guard_keys::get_recovery_dissolve_guard_key,
  parallel_guard::EntryGuard,
  system_configs::get_exteral_canister_id,
};

/// Recover dissolution error
/// When the staked account is dissolved，Already transferred from the staked account to the payment center，But due to abnormal situation，Causes payment centers not。
pub async fn recover_dissolve_error(account: &StakingAccount, dissolve_tx_id: BlockIndex) -> Result<StakingAccountVo, String> {
  // Entrance guard
  let _entry_guard = EntryGuard::new(get_recovery_dissolve_guard_key(account.get_id()))
    .map_err(|_| format!("Account is already in dissolve recovery, account_id = {}", account.get_id()))?;

  if account.get_status() != StakingAccountStatus::Released {
    return Err("Account is not in recoverable error state".to_string());
  }

  // Initiate a dissolving payment request from the payment center
  let pay_center_canister_id = get_exteral_canister_id(ExteralCanisterLabels::PayCenter);
  let pay_center = common_canisters::pay_center::Service(pay_center_canister_id);
  let caller = Principal::from_text(account.get_owner()).unwrap();

  // stake dissolution：Payment Center Accounting Event Log-start
  save_dissolve_pay_center_receive_start_event(account.get_id(), pay_center_canister_id.to_string(), dissolve_tx_id);

  // Execute a few requests for dissolving the payment center
  let dissolve_response = match pay_center
    .dissolve(
      caller,
      account.get_released_amount(),
      dissolve_tx_id,
      account.get_onchain_address(),
      account.get_id(),
    )
    .await
  {
    Ok(result) => result.0,
    Err(e) => {
      ic_cdk::println!("Pay center stake failed: {:?}", e);

      let error_message = format!("Pay center dissolve failed: code = {:?}, message = {}", e.0, e.1);

      // stake dissolution：Payment Center Accounting Event Log-fail
      save_stake_pay_center_transfer_fail_event(account.get_id(), pay_center_canister_id.to_string(), error_message.clone());

      return Err(error_message);
    }
  };

  let pay_center_tx_id = match dissolve_response {
    Result2::Ok(tx_id) => {
      ic_cdk::println!("Pay center dissolve success: tx_id = {}", tx_id);

      // stake dissolution：Payment Center Accounting Event Log-success
      save_dissolve_pay_center_receive_ok_event(account.get_id(), pay_center_canister_id.to_string(), dissolve_tx_id, tx_id);

      tx_id
    }
    Result2::Err(e) => {
      ic_cdk::println!("Pay center dissolve failed: {}", e);

      let error_message = format!("Pay center dissolve failed: {}", e);

      // stake dissolution：Payment Center Accounting Event Log-fail
      save_dissolve_pay_center_receive_fail_event(account.get_id(), pay_center_canister_id.to_string(), error_message.clone());

      return Err(error_message);
    }
  };

  // Update the status of the staked account，and save to stable memory
  let updated_account = match account.change_to_dissolved(dissolve_tx_id, pay_center_tx_id) {
    Ok(account) => account,
    Err(e) => {
      ic_cdk::println!("Staking account change to dissolve failed: {}", e);
      return Err(e);
    }
  };

  // Save the event log of the dissolved staked account
  save_dissolve_event(&updated_account);

  // Delete the recoverable error index of the staked account，Avoid repeated detection
  STAKING_RECOVERABLE_ERROR_ACCOUNT_INDEX_MAP.with(|map| remove_indexed_id(map, &account.get_pool_id(), account.get_id()));

  Ok(StakingAccountVo::from_staking_account(&account))
}
