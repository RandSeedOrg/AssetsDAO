use bigdecimal::{BigDecimal, ToPrimitive};
use candid::Principal;
use common_canisters::pay_center::{Result2, Result3};
use types::{
  date::YearMonthDay,
  entities::add_indexed_id,
  staking::{StakingAccountId, StakingPoolId},
  sys::ExteralCanisterLabels,
};

use crate::{
  account::{badge_utils::remove_staker_badge, crud_utils::delete_staking_account, stable_structures::StakingAccountRecoverableError},
  event_log::{
    stake_and_unstake_events::{save_dissolve_event, save_stake_event, save_unstake_event},
    staking_account_events::save_create_staking_account_event_log,
    transfer_events::{
      save_dissolve_pay_center_receive_fail_event, save_dissolve_pay_center_receive_ok_event, save_dissolve_pay_center_receive_start_event,
      save_dissolve_pay_center_transfer_ok_event, save_dissolve_pay_center_transfer_start_event, save_stake_pay_center_transfer_fail_event,
      save_stake_pay_center_transfer_ok_event, save_stake_pay_center_transfer_start_event, save_stake_transfer_fail_event,
      save_stake_transfer_ok_event, save_stake_transfer_start_event, save_unstake_penalty_pay_center_ok_event,
      save_unstake_penalty_pay_center_start_event, save_unstake_penalty_transfer_fail_event, save_unstake_penalty_transfer_ok_event,
      save_unstake_penalty_transfer_start_event, save_unstake_transfer_fail_event, save_unstake_transfer_ok_event, save_unstake_transfer_start_event,
    },
  },
  guard_keys::{get_dissolve_guard_key, get_stake_guard_key, get_unstake_guard_key},
  on_chain::transfer::{
    transfer_from_staking_account_to_pay_center, transfer_from_staking_account_to_staking_pool, transfer_from_staking_pool_to_pay_center,
    transfer_from_staking_pool_to_staking_account,
  },
  parallel_guard::EntryGuard,
  pool::{crud_utils::query_staking_pool_by_id, stable_structures::StakingPool},
  system_configs::get_exteral_canister_id,
};

use super::{
  client_transport_structures::{EarlyUnstakePreCheckVo, StakeDto},
  crud_utils::{query_current_user_in_stake_accounts, query_current_user_staking_accounts, save_stake_account_to_stable_memory},
  recovery_errors::{
    recover_dissolve::recover_dissolve_error,
    recover_early_unstake::{recover_unstake_penalty_onchain_error, recover_unstake_penalty_pay_center_error},
  },
  stable_structures::{StakingAccount, StakingAccountStatus},
  transport_structures::StakingAccountVo,
  STAKING_UNSTAKE_ON_DAY_ACCOUNT_INDEX_MAP,
};

/// The nanosecond value of 180 days
const ONE_HUNDRED_AND_EIGHTY_DAYS_OF_NANOSECONDS: u64 = 180 * 24 * 60 * 60 * 1_000_000_000;

/// User initiates a stake request
#[ic_cdk::update]
async fn stake(dto: StakeDto) -> Result<StakingAccountVo, String> {
  let caller: Principal = crate::identity_mapping::wl_caller();

  // Anonymous users cannot initiate stake requests
  if caller == Principal::anonymous() {
    return Err("Anonymous user cannot stake".to_string());
  }

  // Reentry protection
  let _entry_guard = EntryGuard::new(get_stake_guard_key(caller.to_string())).map_err(|_| {
    ic_cdk::println!("Stake entry guard failed");
    "You already have a stake in progress, please do not repeat the operation!".to_string()
  })?;

  let StakeDto {
    pool_id,
    staking_amount,
    staking_days,
  } = dto;

  let mut staking_pool = query_staking_pool_by_id(pool_id)?;

  // check term of the staking pool
  let term_config = staking_pool.get_term_config();
  term_config.validate_term(staking_days)?;

  // check staking amount
  let limit_config = staking_pool.get_limit_config();
  let current_user_in_stake_accounts = query_current_user_in_stake_accounts(pool_id);
  limit_config.validate_stake_amount(staking_amount, &current_user_in_stake_accounts)?;

  // Verify and lock the stake pool amount
  staking_pool = staking_pool.validate_and_lock_size(staking_amount)?;

  // Create a staked account
  let mut account = StakingAccount::from_stake_dto_and_pool(&dto, &staking_pool)?;

  // Save the staked account to stable memory
  save_stake_account_to_stable_memory(&account)?;
  // Save the event log of the new stake account
  save_create_staking_account_event_log(&account);

  let pay_center_canister_id = get_exteral_canister_id(ExteralCanisterLabels::PayCenter);
  let pay_center = common_canisters::pay_center::Service(pay_center_canister_id);

  // stake：Event logs for initiating a stake transfer from the payment center to the staked account-start
  save_stake_pay_center_transfer_start_event(account.get_id(), pay_center_canister_id.to_string());

  let stake_response = match pay_center
    .stake(
      caller,
      account.get_staked_amount(),
      account.get_onchain_address(),
      staking_pool.get_id(),
      account.get_id(),
    )
    .await
  {
    Ok(result) => result.0,
    Err(e) => {
      // End the staked account
      ic_cdk::println!("Pay center stake failed: {:?}", e);

      let error_message = format!("Pay center stake failed: code = {:?}, message = {}", e.0, e.1);

      // stake：Event logs for initiating a stake transfer from the payment center to the staked account-fail
      save_stake_pay_center_transfer_fail_event(account.get_id(), pay_center_canister_id.to_string(), error_message.clone());

      delete_staking_account(&account.get_id())?;
      staking_pool.restore_locked_size(staking_amount)?;

      return Err("A system error has occurred. Please try again. ".to_string());
    }
  };

  let result = match stake_response {
    Result3::Ok(result) => {
      ic_cdk::println!("Pay center stake success: {} {}", result.onchain_tx_id, result.pay_center_tx_id);

      // stake：Event logs for initiating a stake transfer from the payment center to the staked account-success
      save_stake_pay_center_transfer_ok_event(account.get_id(), pay_center_canister_id.to_string(), result.onchain_tx_id);

      result
    }
    Result3::Err(e) => {
      let error_message = format!("Pay center stake failed: {}", e);

      ic_cdk::println!("{:?}", error_message.clone());

      // stake：Event logs for initiating a stake transfer from the payment center to the staked account-fail
      save_stake_pay_center_transfer_fail_event(account.get_id(), pay_center_canister_id.to_string(), error_message.clone());

      delete_staking_account(&account.get_id())?;
      staking_pool.restore_locked_size(staking_amount)?;

      return Err("A system error has occurred. Please try again. ".to_string());
    }
  };

  // pay_center.
  let stake_pay_center_onchain_tx_id = result.onchain_tx_id;
  let stake_pay_center_tx_id = result.pay_center_tx_id;

  // stake：Event log of transfers from stake accounts to stake pools-start
  save_stake_transfer_start_event(account.get_id(), staking_pool.get_id());

  // Stake money in the account
  let staking_account_to_pool_tx_id =
    match transfer_from_staking_account_to_staking_pool(account.get_id(), staking_pool.get_id(), account.get_staked_amount()).await {
      Ok(tx_id) => {
        ic_cdk::println!("Transfer from staking account to pool success: {}", tx_id);

        // stake：Transfer Event Log from stake Account to stake Pool-success
        save_stake_transfer_ok_event(account.get_id(), staking_pool.get_id(), tx_id);

        tx_id
      }
      Err(e) => {
        ic_cdk::println!("Transfer from staking account to pool failed: {:?}", e);

        // stake：Transfer Event Log from stake Account to stake Pool-fail
        save_stake_transfer_fail_event(account.get_id(), staking_pool.get_id(), e.clone());
        account.stable_to_recoverable_error(StakingAccountRecoverableError::StakeTransferToPoolFailed(
          stake_pay_center_onchain_tx_id,
          stake_pay_center_tx_id,
        ));

        return Err("A system error has occurred. Please try again. ".to_string());
      }
    };

  // Update the status of the staked account，and save to stable memory
  account.change_to_in_stake(stake_pay_center_onchain_tx_id, stake_pay_center_tx_id, staking_account_to_pool_tx_id);
  save_stake_account_to_stable_memory(&account)?;
  // Add account index based on expiration date
  STAKING_UNSTAKE_ON_DAY_ACCOUNT_INDEX_MAP.with(|map| add_indexed_id(map, &YearMonthDay::from(account.get_stake_deadline()), account.get_id()));
  // Update the available amount of the stake pool
  let updated_pool = staking_pool.add_stake_account(&account, &current_user_in_stake_accounts)?;

  // Save stake events，When the stake is issued，The stake pool and stake account will be updated at the same time
  save_stake_event(&updated_pool, &account);

  Ok(StakingAccountVo::from_staking_account(&account))
}

/// Manually initiate a request to unstake，There will be a handling fee here
#[ic_cdk::update]
async fn early_unstake(account_id: StakingAccountId) -> Result<StakingAccountVo, String> {
  let caller: Principal = crate::identity_mapping::wl_caller();

  if caller == Principal::anonymous() {
    return Err("Anonymous user cannot stake".to_string());
  }

  // Reentry protection
  let _entry_guard = EntryGuard::new(get_unstake_guard_key(account_id)).map_err(|_| {
    ic_cdk::println!("Stake entry guard failed");
    "The current staking account is in the process of unstaking, please do not repeat the operation!".to_string()
  })?;

  let user_id = caller.to_string();

  // Query staked account
  let account = StakingAccount::query_by_id(account_id)?;

  // Verify the owner of the staked account
  if account.get_owner() != user_id {
    return Err("The caller is not the owner of the staking account".to_string());
  }

  // Verify the status of the staked account
  if account.get_status() != StakingAccountStatus::InStake {
    return Err("The staking account is not in stake".to_string());
  }

  match account.recoverable_error {
    Some(StakingAccountRecoverableError::EarlyUnstakePenaltyOnChainFailed(_, _, _)) => {
      // If the current staked account status is a restored error status，Indicates that the on-chain transfer has been completed，Payment Center failed to bookkeeping，Therefore, the error recovery process is directly followed
      return recover_unstake_penalty_onchain_error(&account).await;
    }
    Some(StakingAccountRecoverableError::EarlyUnstakePenaltyPayCenterFailed(_, _, _, _)) => {
      // If the current staked account status is a restored error status，Indicates that the on-chain transfer has been completed，Payment Center failed to bookkeeping，Therefore, the error recovery process is directly followed
      return recover_unstake_penalty_pay_center_error(&account).await;
    }
    _ => {}
  };

  let now = ic_cdk::api::time();
  let start_stake_time = account.get_stake_time();

  // Check if account can be unstake at this time
  if now < account.get_can_early_unstake_time() {
    return Err(format!(
      "The staking account cannot be released within {} days.",
      account.get_min_early_unstake_days()
    ));
  }

  // Calculate the penalty amount
  let mut penalty_amount = if now < start_stake_time + ONE_HUNDRED_AND_EIGHTY_DAYS_OF_NANOSECONDS {
    // The stake time of the stake account is less than 180 days，deduct 80% staking reward
    (BigDecimal::from(account.get_accumulated_rewards()) * BigDecimal::from(8) / BigDecimal::from(10)).to_u64()
  } else {
    // The stake time of the stake account is greater than 180 days，deduct 50% stake Reward
    (BigDecimal::from(account.get_accumulated_rewards()) * BigDecimal::from(5) / BigDecimal::from(10)).to_u64()
  }
  .unwrap_or_default();

  // If the penalty amount is less than or equal to 10,000(0.0001ICP), then set to 0
  if penalty_amount <= 10_000 {
    penalty_amount = 0;
  }

  // Calculate the actual redemption amount
  let released_amount = if penalty_amount >= account.get_staked_amount() {
    // Penalty fees great than the amount of the principal，the actual redemption amount is 0
    0
  } else {
    // Actual redemption amount = The amount of the stake - Penalty fees
    account.get_staked_amount() - penalty_amount
  };

  // Unstake：Transfer Event Log from stake Pool to stake Account-start
  save_unstake_transfer_start_event(account.get_id(), account.get_pool_id());

  let unstake_tx_id = if released_amount > 0 {
    // Unstake zero amount，On-chain transfer is required
    // Execute on-chain transfer of unstake
    match transfer_from_staking_pool_to_staking_account(account.get_pool_id(), account.get_id(), released_amount).await {
      Ok(tx_id) => {
        ic_cdk::println!("On-chain transfer success: {}", tx_id);

        // Unstake：Transfer Event Log from stake Pool to stake Account-success
        save_unstake_transfer_ok_event(account.get_id(), account.get_pool_id(), tx_id);

        tx_id
      }
      Err(e) => {
        ic_cdk::println!("On-chain transfer failed: {:?}", e);

        // Unstake：Transfer Event Log from stake Pool to stake Account-fail
        save_unstake_transfer_fail_event(account.get_id(), account.get_pool_id(), e.clone());

        return Err("A system error has occurred. Please try again. ".to_string());
      }
    }
  } else {
    // Unstake amount is 0，No on-chain transfer is required
    0
  };

  let (penalty_onchain_tx_id, penalty_pay_center_tx_id) = if penalty_amount > 0 {
    // Transfer event log from stake pool to payment center-start
    save_unstake_penalty_transfer_start_event(account.get_id(), account.get_pool_id());

    // Execute unstake penalty fee on-chain transfer
    let penalty_onchain_tx_id = match transfer_from_staking_pool_to_pay_center(account.get_pool_id(), penalty_amount).await {
      Ok(tx_id) => {
        ic_cdk::println!("On-chain transfer success: {}", tx_id);

        // Transfer event log from stake pool to payment center-success
        save_unstake_penalty_transfer_ok_event(account.get_id(), account.get_pool_id(), tx_id);

        tx_id
      }
      Err(e) => {
        ic_cdk::println!("On-chain transfer failed: {:?}", e);

        // Transfer event log from stake pool to payment center-fail
        save_unstake_penalty_transfer_fail_event(account.get_id(), account.get_pool_id(), e.clone());
        account.stable_to_recoverable_error(StakingAccountRecoverableError::EarlyUnstakePenaltyOnChainFailed(unstake_tx_id, now, penalty_amount));

        return Err(format!("On-chain transfer failed: {}", e));
      }
    };

    let pay_center_canister_id = get_exteral_canister_id(ExteralCanisterLabels::PayCenter);

    // Unstake：Payment Center Accounting Event Log-start
    save_unstake_penalty_pay_center_start_event(account.get_id(), pay_center_canister_id.to_string());

    // Execute the payment center's Unstake Penalty fees accounting request
    let pay_center = common_canisters::pay_center::Service(pay_center_canister_id);
    let stake_response = match pay_center
      .receive_early_unstake_penalty(caller, penalty_amount, account.get_pool_id(), account.get_id(), penalty_onchain_tx_id)
      .await
    {
      Ok(result) => result.0,
      Err(e) => {
        ic_cdk::println!("Pay center stake failed: {:?}", e);

        let error_message = format!("Pay center early unstake penalty failed: code = {:?}, message = {}", e.0, e.1);

        // Unstake：Payment Center Accounting Event Log-fail
        save_dissolve_pay_center_receive_fail_event(account.get_id(), pay_center_canister_id.to_string(), error_message.clone());
        account.stable_to_recoverable_error(StakingAccountRecoverableError::EarlyUnstakePenaltyPayCenterFailed(
          unstake_tx_id,
          penalty_onchain_tx_id,
          now,
          penalty_amount,
        ));

        return Err(error_message);
      }
    };

    let pay_center_tx_id = match stake_response {
      Result2::Ok(tx_id) => {
        ic_cdk::println!("Pay center early unstake penalty success: {}", tx_id);

        // Unstake：Payment Center Accounting Event Log-success
        save_unstake_penalty_pay_center_ok_event(account.get_id(), pay_center_canister_id.to_string(), tx_id);

        tx_id
      }
      Result2::Err(e) => {
        let error_message = format!("Pay center early unstake penalty failed: {}", e);

        ic_cdk::println!("{:?}", error_message.clone());

        // Unstake：Payment Center Accounting Event Log-fail
        save_dissolve_pay_center_receive_fail_event(account.get_id(), pay_center_canister_id.to_string(), error_message.clone());
        account.stable_to_recoverable_error(StakingAccountRecoverableError::EarlyUnstakePenaltyPayCenterFailed(
          unstake_tx_id,
          penalty_onchain_tx_id,
          now,
          penalty_amount,
        ));

        return Err(error_message);
      }
    };

    (penalty_onchain_tx_id, pay_center_tx_id)
  } else {
    // none Unstake Penalty fees，Then set to 0
    (0, 0)
  };

  let current_user_in_stake_accounts = query_current_user_in_stake_accounts(account.get_pool_id());

  // Unstake the staked account from the staking pool
  let pool = match StakingPool::unstake_account(&account, &current_user_in_stake_accounts) {
    Ok(pool) => pool,
    Err(e) => {
      ic_cdk::println!("Staking pool unstake failed: {:?}", e);
      return Err(e);
    }
  };

  // Update the status of the staked account，and save to stable memory
  let updated_account =
    match account.change_to_un_stake(unstake_tx_id, released_amount, penalty_amount, now, penalty_onchain_tx_id, penalty_pay_center_tx_id) {
      Ok(account) => account,
      Err(e) => {
        ic_cdk::println!("Staking account change to unstake failed: {:?}", e);
        return Err(e);
      }
    };

  // Save update the event log of staked account
  save_unstake_event(&pool, &updated_account);

  if current_user_in_stake_accounts.len() == 1 {
    let account_owner = account.get_owner();
    ic_cdk::futures::spawn(async move {
      remove_staker_badge(account_owner).await.unwrap_or_else(|e| {
        ic_cdk::println!("Failed to remove staker badge: {:?}", e);
      });
    });
  }

  Ok(StakingAccountVo::from_staking_account(&account))
}

#[ic_cdk::update]
async fn dissolve(account_id: StakingAccountId) -> Result<StakingAccountVo, String> {
  let caller: Principal = crate::identity_mapping::wl_caller();

  if caller == Principal::anonymous() {
    return Err("Anonymous user cannot stake".to_string());
  }

  // Reentry protection
  let _entry_guard = EntryGuard::new(get_dissolve_guard_key(account_id)).map_err(|_| {
    ic_cdk::println!("Stake entry guard failed");
    "The current staking account is being dissolved, please do not repeat the operation!".to_string()
  })?;

  let user_id = caller.to_string();

  // Query staked account
  let account = StakingAccount::query_by_id(account_id)?;

  // Verify the owner of the staked account
  if account.get_owner() != user_id {
    return Err("The caller is not the owner of the staking account".to_string());
  }

  // Verify the status of the staked account，Only accounts that have been de-staked can be dissolved
  if account.get_status() != StakingAccountStatus::Released {
    return Err("The staking account is not released".to_string());
  }

  // If the current staked account status is a restored error status，Indicates that the on-chain transfer has been completed，Payment Center failed to bookkeeping，Therefore, the error recovery process is directly followed
  if let Some(StakingAccountRecoverableError::DissolvePayCenterFailed(dissolve_tx_id)) = account.recoverable_error {
    return recover_dissolve_error(&account, dissolve_tx_id).await;
  }

  // Initiate a dissolving payment request from the payment center
  let pay_center_canister_id = get_exteral_canister_id(ExteralCanisterLabels::PayCenter);
  let pay_center = common_canisters::pay_center::Service(pay_center_canister_id);

  let (dissolve_tx_id, pay_center_tx_id) = if account.get_released_amount() == 0 {
    // Unstake the zero amount，No on-chain transfer is required
    ic_cdk::println!("The released amount is 0, no need to transfer on-chain");
    (0, 0)
  } else {
    // Unstake amount great than 0，On-chain transfer is required
    // stake dissolve：Transfer from the stake account to the payment center on-chain
    save_dissolve_pay_center_transfer_start_event(account.get_id(), pay_center_canister_id.to_string());

    // Perform on-dissolved chain transfers
    let dissolve_tx_id = match transfer_from_staking_account_to_pay_center(account.get_id(), account.get_released_amount()).await {
      Ok(tx_id) => {
        ic_cdk::println!("On-chain transfer success: {}", tx_id);

        // stake dissolve：Transfer from the stake account to the payment center on-chain success
        save_dissolve_pay_center_transfer_ok_event(account.get_id(), pay_center_canister_id.to_string(), tx_id);

        tx_id
      }
      Err(e) => {
        // End the dissolving account
        ic_cdk::println!("On-chain transfer failed: {:?}", e);

        // stake dissolve：Transfer from the stake account to the payment center on-chain fail
        save_stake_pay_center_transfer_fail_event(account.get_id(), pay_center_canister_id.to_string(), e.clone());

        return Err("A system error has occurred. Please try again. ".to_string());
      }
    };

    // stake dissolve：Payment Center Accounting Event Log-start
    save_dissolve_pay_center_receive_start_event(account.get_id(), pay_center_canister_id.to_string(), dissolve_tx_id);

    // Execute a few requests for dissolving the payment center
    let stake_response = match pay_center
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

        // stake dissolve：Payment Center Accounting Event Log-fail
        save_stake_pay_center_transfer_fail_event(account.get_id(), pay_center_canister_id.to_string(), error_message.clone());
        // Record the dissolution time，Recoverable status of payment center failure
        account.stable_to_recoverable_error(StakingAccountRecoverableError::DissolvePayCenterFailed(dissolve_tx_id));

        return Err("A system error has occurred. Please try again. ".to_string());
      }
    };

    let pay_center_tx_id = match stake_response {
      Result2::Ok(tx_id) => {
        ic_cdk::println!("Pay center dissolve success: tx_id = {}", tx_id);

        // stake dissolve：Payment Center Accounting Event Log-success
        save_dissolve_pay_center_receive_ok_event(account.get_id(), pay_center_canister_id.to_string(), dissolve_tx_id, tx_id);

        tx_id
      }
      Result2::Err(e) => {
        ic_cdk::println!("Pay center dissolve failed: {}", e);

        let error_message = format!("Pay center dissolve failed: {}", e);

        // stake dissolve：Payment Center Accounting Event Log-fail
        save_dissolve_pay_center_receive_fail_event(account.get_id(), pay_center_canister_id.to_string(), error_message.clone());

        // Record the dissolution time，Recoverable status of payment center failure
        account.stable_to_recoverable_error(StakingAccountRecoverableError::DissolvePayCenterFailed(dissolve_tx_id));

        return Err("A system error has occurred. Please try again. ".to_string());
      }
    };
    (dissolve_tx_id, pay_center_tx_id)
  };

  // Update the status of the staked account，and save to stable memory
  let updated_account = match account.change_to_dissolved(dissolve_tx_id, pay_center_tx_id) {
    Ok(account) => account,
    Err(e) => {
      ic_cdk::println!("Staking account change to dissolve failed: {}", e);
      return Err(e);
    }
  };

  // Save the event log of the dissolved account
  save_dissolve_event(&updated_account);

  Ok(StakingAccountVo::from_staking_account(&account))
}

/// Pre-resolution inspection of staked accounts
#[ic_cdk::query]
fn early_unstake_pre_check(account_id: StakingAccountId) -> Result<EarlyUnstakePreCheckVo, String> {
  let caller: Principal = crate::identity_mapping::wl_caller();

  if caller == Principal::anonymous() {
    return Err("Anonymous user cannot stake".to_string());
  }

  let user_id = caller.to_string();

  // Query staked account
  let account = StakingAccount::query_by_id(account_id)?;

  // Verify the owner of the staked account
  if account.get_owner() != user_id {
    return Err("The caller is not the owner of the staking account".to_string());
  }

  // Verify the status of the staked account
  if account.get_status() != StakingAccountStatus::InStake {
    return Err("The staking account is not in stake".to_string());
  }

  let now = ic_cdk::api::time();
  let start_stake_time = account.get_stake_time();

  // Check if account can be unstake at this time
  if now < account.get_can_early_unstake_time() {
    return Err(format!(
      "The staking account cannot be released within {} days.",
      account.get_min_early_unstake_days()
    ));
  }

  // Calculate the penalty amount
  let mut penalty_amount = if now < start_stake_time + ONE_HUNDRED_AND_EIGHTY_DAYS_OF_NANOSECONDS {
    // The stake time of the stake account is less than 180 days，deduct 80% stake Reward
    (BigDecimal::from(account.get_accumulated_rewards()) * BigDecimal::from(8) / BigDecimal::from(10)).to_u64()
  } else {
    // The stake time of the stake account is greater than 180 days，deduct 50% stake Reward
    (BigDecimal::from(account.get_accumulated_rewards()) * BigDecimal::from(5) / BigDecimal::from(10)).to_u64()
  }
  .unwrap_or_default();

  // If the penalty amount is less than or equal to 10,000(0.0001ICP), then set to 0
  if penalty_amount <= 10_000 {
    penalty_amount = 0;
  }

  // Calculate the actual redemption amount
  let released_amount = if penalty_amount >= account.get_staked_amount() {
    // Penalty fees great than the amount of the stake principal，The actual unstake amount is 0
    0
  } else {
    // Actual redemption amount = The amount of the stake - Penalty fees
    account.get_staked_amount() - penalty_amount
  };

  Ok(EarlyUnstakePreCheckVo {
    pool_id: account.get_pool_id(),
    staked_amount: account.get_staked_amount(),
    penalty_amount,
    released_amount,
    accumulated_rewards: account.get_accumulated_rewards(),
  })
}

/// Query the current user's staked account list
#[ic_cdk::query]
fn query_staking_accounts_with_pool_id(pool_id: StakingPoolId) -> Vec<StakingAccountVo> {
  query_current_user_staking_accounts(pool_id)
    .into_iter()
    .map(|account| StakingAccountVo::from_staking_account(&account))
    .collect::<Vec<StakingAccountVo>>()
}
