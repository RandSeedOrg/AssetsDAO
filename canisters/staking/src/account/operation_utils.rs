use types::staking::StakingAccountId;

use crate::{event_log::{stake_and_unstake_events::save_unstake_event, transfer_events::{save_unstake_transfer_fail_event, save_unstake_transfer_ok_event, save_unstake_transfer_start_event}}, on_chain::transfer::transfer_from_staking_pool_to_staking_account, parallel_guard::EntryGuard, pool::stable_structures::StakingPool};

use super::{crud_utils::query_user_in_stake_accounts, guard_keys::get_unstake_guard_key, stable_structures::{StakingAccount, StakingAccountStatus}, transport_structures::StakingAccountVo};

/// Depleted account due
pub async fn maturity_unstake(account_id: StakingAccountId) -> Result<StakingAccountVo, String> {
  // Reentry protection
  let _entry_guard = EntryGuard::new(get_unstake_guard_key(account_id)).map_err(|_| {
    ic_cdk::println!("Stake entry guard failed");
    "The current stake account is being unstaked!".to_string()
  })?;

  // Query staked account
  let account = StakingAccount::query_by_id(account_id)?;

  // Verify the status of the staked account
  if account.get_status() != StakingAccountStatus::InStake {
    return Err("The staking account is not in stake".to_string());
  }

  // Verify whether the staked account is expired
  let now = ic_cdk::api::time();
  if now < account.get_stake_deadline() {
    return Err("The staking account is not mature".to_string());
  }

  // Calculate the actual redemption amount
  let release_amount = account.get_staked_amount();

  // Unstake: Transfer Event Log from stake Pool to stake Account-start
  save_unstake_transfer_start_event(account.get_id(), account.get_pool_id());

  let unstake_tx_id = match transfer_from_staking_pool_to_staking_account(account.get_pool_id(), account.get_id(), release_amount).await {
    Ok(tx_id) => {
      // Unstake: Transfer Event Log from stake Pool to stake Account-success
      save_unstake_transfer_ok_event(account.get_id(), account.get_pool_id(), tx_id);
      tx_id
    },
    Err(e) => {
      ic_cdk::println!("On-chain transfer failed: {:?}", e);

      // Unstake: Transfer Event Log from stake Pool to stake Account-fail
      save_unstake_transfer_fail_event(account.get_id(), account.get_pool_id(), e.to_string());

      return Err(format!("On-chain transfer failed: {}", e));
    }
  };

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
  let updated_account = match account.change_to_un_stake(unstake_tx_id, release_amount, 0, now, 0, 0) {
    Ok(account) => account,
    Err(e) => {
      ic_cdk::println!("Staking account change to unstake failed: {:?}", e);
      return Err(e);
    }
  };

  // Save update the event log of staked account
  save_unstake_event(&pool, &updated_account);

  Ok(StakingAccountVo::from_staking_account(&account))
}

