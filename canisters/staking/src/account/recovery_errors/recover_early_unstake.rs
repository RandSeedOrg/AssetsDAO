use candid::Principal;
use common_canisters::pay_center::Result2;
use types::{entities::remove_indexed_id, sys::ExteralCanisterLabels};

use crate::{
  account::{
    STAKING_RECOVERABLE_ERROR_ACCOUNT_INDEX_MAP,
    crud_utils::query_user_in_stake_accounts,
    guard_keys::get_recovery_unstake_penalty_guard_key,
    stable_structures::{StakingAccount, StakingAccountRecoverableError, StakingAccountStatus},
    transport_structures::StakingAccountVo,
  },
  event_log::{
    stake_and_unstake_events::save_unstake_event,
    transfer_events::{
      save_dissolve_pay_center_receive_fail_event, save_unstake_penalty_pay_center_ok_event, save_unstake_penalty_pay_center_start_event,
      save_unstake_penalty_transfer_fail_event, save_unstake_penalty_transfer_ok_event, save_unstake_penalty_transfer_start_event,
    },
  },
  on_chain::transfer::transfer_from_staking_pool_to_pay_center,
  parallel_guard::EntryGuard,
  pool::stable_structures::StakingPool,
  system_configs::get_exteral_canister_id,
};

/// Handling of failed transfer on the unsolicited liquidated damages chain
pub async fn recover_unstake_penalty_onchain_error(account: &StakingAccount) -> Result<StakingAccountVo, String> {
  // Entrance guard
  let _entry_guard = EntryGuard::new(get_recovery_unstake_penalty_guard_key(account.get_id()))
    .map_err(|_| format!("Account is already in early unstake penalty recovery, account_id = {}", account.get_id()))?;

  if let Some(StakingAccountRecoverableError::EarlyUnstakePenaltyOnChainFailed(unstake_tx_id, unstake_time, penalty_amount)) =
    account.recoverable_error
  {
    // Verify the status of the staked account
    if account.get_status() != StakingAccountStatus::InStake {
      return Err("The staking account is not in stake".to_string());
    }

    // Calculate the actual redemption amount
    let release_amount = account.get_staked_amount() - penalty_amount;

    // Unstake:Transfer Event Log from the Stake Pool to the Payment Center-start
    save_unstake_penalty_transfer_start_event(account.get_id(), account.get_pool_id());

    // Execute on-chain transfer of unstaking liquidated damages
    let penalty_onchain_tx_id = match transfer_from_staking_pool_to_pay_center(account.get_pool_id(), penalty_amount).await {
      Ok(tx_id) => {
        ic_cdk::println!("On-chain transfer success: {}", tx_id);

        // Unstake:Transfer Event Log from the Stake Pool to the Payment Center-success
        save_unstake_penalty_transfer_ok_event(account.get_id(), account.get_pool_id(), tx_id);

        tx_id
      }
      Err(e) => {
        ic_cdk::println!("On-chain transfer failed: {:?}", e);

        // Unstake:Transfer Event Log from the Stake Pool to the Payment Center-fail
        save_unstake_penalty_transfer_fail_event(account.get_id(), account.get_pool_id(), e.clone());

        return Err(format!("On-chain transfer failed: {}", e));
      }
    };

    let pay_center_canister_id = get_exteral_canister_id(ExteralCanisterLabels::PayCenter);

    // Unstake:Payment Center Accounting Event Log-start
    save_unstake_penalty_pay_center_start_event(account.get_id(), pay_center_canister_id.to_string());

    let caller = Principal::from_text(account.get_owner()).map_err(|_| format!("Invalid account owner principal: {}", account.get_owner()))?;

    let pay_center = common_canisters::pay_center::Service(pay_center_canister_id);
    let stake_response = match pay_center
      .receive_early_unstake_penalty(caller, penalty_amount, account.get_pool_id(), account.get_id(), penalty_onchain_tx_id)
      .await
    {
      Ok(result) => result.0,
      Err(e) => {
        ic_cdk::println!("Pay center stake failed: {:?}", e);

        let error_message = format!("Pay center early unstake penalty failed: code = {:?}, message = {}", e.0, e.1);

        // Unstake:Payment Center Accounting Event Log-fail
        save_dissolve_pay_center_receive_fail_event(account.get_id(), pay_center_canister_id.to_string(), error_message.clone());
        account.stable_to_recoverable_error(StakingAccountRecoverableError::EarlyUnstakePenaltyPayCenterFailed(
          unstake_tx_id,
          penalty_onchain_tx_id,
          unstake_time,
          penalty_amount,
        ));

        return Err(error_message);
      }
    };

    let pay_center_tx_id = match stake_response {
      Result2::Ok(tx_id) => {
        ic_cdk::println!("Pay center early unstake penalty success: {}", tx_id);

        // Unstake:Payment Center Accounting Event Log-success
        save_unstake_penalty_pay_center_ok_event(account.get_id(), pay_center_canister_id.to_string(), tx_id);

        tx_id
      }
      Result2::Err(e) => {
        let error_message = format!("Pay center early unstake penalty failed: {}", e);

        ic_cdk::println!("{:?}", error_message.clone());

        // Unstake:Payment Center Accounting Event Log-fail
        save_dissolve_pay_center_receive_fail_event(account.get_id(), pay_center_canister_id.to_string(), error_message.clone());
        account.stable_to_recoverable_error(StakingAccountRecoverableError::EarlyUnstakePenaltyPayCenterFailed(
          unstake_tx_id,
          penalty_onchain_tx_id,
          unstake_time,
          penalty_amount,
        ));

        return Err(error_message);
      }
    };

    // Get the account of the current staked user in the stake pool
    let current_user_in_stake_accounts = query_user_in_stake_accounts(account.get_owner(), account.get_pool_id());

    // Unstake the account from the staking pool
    let pool = match StakingPool::unstake_account(&account, &current_user_in_stake_accounts) {
      Ok(pool) => pool,
      Err(e) => {
        ic_cdk::println!("Staking pool unstake failed: {:?}", e);
        return Err(e);
      }
    };

    // Update the status of the staked account，and save to stable memory
    let updated_account =
      match account.change_to_un_stake(unstake_tx_id, release_amount, penalty_amount, unstake_time, penalty_onchain_tx_id, pay_center_tx_id) {
        Ok(account) => account,
        Err(e) => {
          ic_cdk::println!("Staking account change to unstake failed: {:?}", e);
          return Err(e);
        }
      };

    // Save update the event log of staked account
    save_unstake_event(&pool, &updated_account);

    // Delete the recoverable error index of the staked account，Avoid repeated detection
    STAKING_RECOVERABLE_ERROR_ACCOUNT_INDEX_MAP.with(|map| remove_indexed_id(map, &account.get_pool_id(), account.get_id()));

    Ok(StakingAccountVo::from_staking_account(&account))
  } else {
    return Err("Account is not in recoverable unstake error state".to_string());
  }
}

pub async fn recover_unstake_penalty_pay_center_error(account: &StakingAccount) -> Result<StakingAccountVo, String> {
  // Entrance guard
  let _entry_guard = EntryGuard::new(get_recovery_unstake_penalty_guard_key(account.get_id()))
    .map_err(|_| format!("Account is already in early unstake penalty recovery, account_id = {}", account.get_id()))?;

  if let Some(StakingAccountRecoverableError::EarlyUnstakePenaltyPayCenterFailed(
    unstake_tx_id,
    penalty_onchain_tx_id,
    unstake_time,
    penalty_amount,
  )) = account.recoverable_error
  {
    // Verify the status of the staked account
    if account.get_status() != StakingAccountStatus::InStake {
      return Err("The staking account is not in stake".to_string());
    }

    // Calculate the actual redemption amount
    let release_amount = account.get_staked_amount() - penalty_amount;
    let pay_center_canister_id = get_exteral_canister_id(ExteralCanisterLabels::PayCenter);
    // Unstake:Payment Center Accounting Event Log-start
    save_unstake_penalty_pay_center_start_event(account.get_id(), pay_center_canister_id.to_string());

    let caller = Principal::from_text(account.get_owner()).map_err(|_| format!("Invalid account owner principal: {}", account.get_owner()))?;

    let pay_center = common_canisters::pay_center::Service(pay_center_canister_id);
    let stake_response = match pay_center
      .receive_early_unstake_penalty(caller, penalty_amount, account.get_pool_id(), account.get_id(), penalty_onchain_tx_id)
      .await
    {
      Ok(result) => result.0,
      Err(e) => {
        ic_cdk::println!("Pay center stake failed: {:?}", e);

        let error_message = format!("Pay center early unstake penalty failed: code = {:?}, message = {}", e.0, e.1);

        // Unstake:Payment Center Accounting Event Log-fail
        save_dissolve_pay_center_receive_fail_event(account.get_id(), pay_center_canister_id.to_string(), error_message.clone());

        return Err(error_message);
      }
    };

    let pay_center_tx_id = match stake_response {
      Result2::Ok(tx_id) => {
        ic_cdk::println!("Pay center early unstake penalty success: {}", tx_id);

        // Unstake:Payment Center Accounting Event Log-success
        save_unstake_penalty_pay_center_ok_event(account.get_id(), pay_center_canister_id.to_string(), tx_id);

        tx_id
      }
      Result2::Err(e) => {
        let error_message = format!("Pay center early unstake penalty failed: {}", e);

        ic_cdk::println!("{:?}", error_message.clone());

        // Unstake:Payment Center Accounting Event Log-fail
        save_dissolve_pay_center_receive_fail_event(account.get_id(), pay_center_canister_id.to_string(), error_message.clone());

        return Err(error_message);
      }
    };

    // Get the account of the current staked user in the stake pool
    let current_user_in_stake_accounts = query_user_in_stake_accounts(account.get_owner(), account.get_pool_id());

    // Unstake the account from the staking pool
    let pool = match StakingPool::unstake_account(&account, &current_user_in_stake_accounts) {
      Ok(pool) => pool,
      Err(e) => {
        ic_cdk::println!("Staking pool unstake failed: {:?}", e);
        return Err(e);
      }
    };

    // Update the status of the staked account，and save to stable memory
    let updated_account =
      match account.change_to_un_stake(unstake_tx_id, release_amount, penalty_amount, unstake_time, penalty_onchain_tx_id, pay_center_tx_id) {
        Ok(account) => account,
        Err(e) => {
          ic_cdk::println!("Staking account change to unstake failed: {:?}", e);
          return Err(e);
        }
      };

    // Save update the event log of staked account
    save_unstake_event(&pool, &updated_account);
    // Delete the recoverable error index of the staked account，Avoid repeated detection
    STAKING_RECOVERABLE_ERROR_ACCOUNT_INDEX_MAP.with(|map| remove_indexed_id(map, &account.get_pool_id(), account.get_id()));

    Ok(StakingAccountVo::from_staking_account(&account))
  } else {
    return Err("Account is not in recoverable unstake error state".to_string());
  }
}
