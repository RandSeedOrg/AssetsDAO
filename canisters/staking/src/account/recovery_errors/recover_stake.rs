use ic_ledger_types::BlockIndex;
use types::{
  date::YearMonthDay,
  entities::{add_indexed_id, remove_indexed_id},
};

use crate::{
  account::{
    STAKING_RECOVERABLE_ERROR_ACCOUNT_INDEX_MAP, STAKING_UNSTAKE_ON_DAY_ACCOUNT_INDEX_MAP,
    crud_utils::{query_user_in_stake_accounts, save_stake_account_to_stable_memory},
    guard_keys::get_recovery_stake_guard_key,
    stable_structures::{StakingAccount, StakingAccountStatus},
    transport_structures::StakingAccountVo,
  },
  event_log::{
    stake_and_unstake_events::save_stake_event,
    transfer_events::{save_stake_transfer_fail_event, save_stake_transfer_ok_event, save_stake_transfer_start_event},
  },
  on_chain::transfer::transfer_from_staking_account_to_staking_pool,
  parallel_guard::EntryGuard,
  pool::crud_utils::query_staking_pool_by_id,
};

// During the stake process, the payment has been transferred from the payment center to the stake account, but due to absolute situation, resulting in the stake account not succeeding and transferring funds to the stake pool.
// In this case, the stake account will be in a recoverable error state
// You need to transfer the funds of the stake account to the stake pool and change the status of the stake account to normal.
pub async fn recover_stake_error(
  account: &StakingAccount,
  pay_center_onchain_tx_id: BlockIndex,
  pay_center_tx_id: u64,
) -> Result<StakingAccountVo, String> {
  // Entrance guard
  let _entry_guard = EntryGuard::new(get_recovery_stake_guard_key(account.get_id()))
    .map_err(|_| format!("Account is already in stake recovery, account_id = {}", account.get_id()))?;

  if account.get_status() != StakingAccountStatus::Created {
    return Err("Account is not in recoverable error state".to_string());
  }

  // 1. Get the stake pool
  let mut staking_pool = query_staking_pool_by_id(account.get_pool_id())?;
  let current_user_in_stake_accounts = query_user_in_stake_accounts(account.get_owner(), account.get_pool_id());

  // 1. Get the stake pool
  // stake: Event log of transfer from stake account to stake pool -start
  save_stake_transfer_start_event(account.get_id(), staking_pool.get_id());

  // 2. Transfer the funds from the stake account to the stake pool
  let staking_account_to_pool_tx_id =
    match transfer_from_staking_account_to_staking_pool(account.get_id(), staking_pool.get_id(), account.get_staked_amount()).await {
      Ok(tx_id) => {
        ic_cdk::println!("Transfer from staking account to pool success: {}", tx_id);

        // stake：Transfer event log from stake account to stake pool -success
        save_stake_transfer_ok_event(account.get_id(), staking_pool.get_id(), tx_id);

        tx_id
      }
      Err(e) => {
        ic_cdk::println!("Transfer from staking account to pool failed: {:?}", e);

        // stake：Transfer event log from stake account to stake pool-fail
        save_stake_transfer_fail_event(account.get_id(), staking_pool.get_id(), e.clone());

        return Err(format!("Transfer from staking account to pool failed: {}", e));
      }
    };

  let mut account = account.clone();

  // 3. Update the status to in stake of the stake account
  account.change_to_in_stake(pay_center_onchain_tx_id, pay_center_tx_id, staking_account_to_pool_tx_id);

  save_stake_account_to_stable_memory(&account)?;

  STAKING_UNSTAKE_ON_DAY_ACCOUNT_INDEX_MAP.with(|map| add_indexed_id(map, &YearMonthDay::from(account.get_stake_deadline()), account.get_id()));

  let updated_pool = staking_pool.add_stake_account(&account, &current_user_in_stake_accounts)?;

  save_stake_event(&updated_pool, &account);

  STAKING_RECOVERABLE_ERROR_ACCOUNT_INDEX_MAP.with(|map| remove_indexed_id(map, &account.get_pool_id(), account.get_id()));

  Ok(StakingAccountVo::from_staking_account(&account))
}
