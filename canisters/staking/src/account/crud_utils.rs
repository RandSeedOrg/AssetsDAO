use candid::Principal;
use types::{date::YearMonthDay, entities::{add_indexed_id, get_indexed_ids, remove_indexed_id}};

use crate::{event_log::staking_account_events::save_delete_staking_account_event_log, StakingAccountId, StakingPoolId};

use super::{stable_structures::{StakingAccount, StakingAccountStatus}, STAKING_ACCOUNT_MAP, STAKING_POOL_ACCOUNT_INDEX_MAP, STAKING_UNSTAKE_ON_DAY_ACCOUNT_INDEX_MAP, STAKING_USER_ACCOUNT_INDEX_MAP};


/// Query the list of staked accounts of the current session user in the stake pool
pub fn query_current_user_staking_accounts(pool_id: StakingPoolId) -> Vec<StakingAccount> {
  let user = ic_cdk::api::msg_caller();

  if user == Principal::anonymous() {
    return vec![]; // Anonymous user, no staking accounts
  }

  let user_id = user.to_string();

  query_user_staking_accounts(user_id)
    .into_iter()
    .filter(|account| account.get_pool_id() == pool_id)
    .collect()
}

/// Get the list of accounts that the current user is in a staked state in a stake pool
pub fn query_current_user_in_stake_accounts(pool_id: StakingPoolId) -> Vec<StakingAccount> {
  query_current_user_staking_accounts(pool_id)
    .into_iter()
    .filter(|account| account.get_status() == StakingAccountStatus::InStake)
    .collect()
}

/// Query the list of accounts that are currently in the stake pool.
pub fn query_user_in_stake_accounts(user_id: String, pool_id: StakingPoolId) -> Vec<StakingAccount> {
  query_user_staking_accounts(user_id)
    .into_iter()
    .filter(|account| account.get_pool_id() == pool_id && account.get_status() == StakingAccountStatus::InStake)
    .collect()
}

/// Query the list of staked accounts for the same user as the current staked account
pub fn query_user_staking_accounts(user_id: String) -> Vec<StakingAccount> {
  let user_account_index = STAKING_USER_ACCOUNT_INDEX_MAP.with(|map| get_indexed_ids(map, &user_id));

  if user_account_index.is_empty() {
    return vec![]; // No staking accounts for the user
  }

  
  let accounts = STAKING_ACCOUNT_MAP.with(|map| {
    let map = map.borrow();
    user_account_index
      .iter()
      .filter_map(|account_id| map.get(account_id))
      .collect::<Vec<StakingAccount>>()
  });

  // FIX: Temporary code，Wash out dirty data stored previously that does not belong to the current user

  let filter_accounts = accounts
    .into_iter()
    .filter(|account| account.get_owner() == user_id)
    .collect::<Vec<StakingAccount>>();

  STAKING_USER_ACCOUNT_INDEX_MAP.with(|map| {
    let mut map = map.borrow_mut();

    let mut index = map.get(&user_id).unwrap_or_default();
    let mut pending_remove_ids = vec![];

    // Clear the index of staked accounts that are not part of the current user
    for account_id in index.get_entity_ids() {
      if !filter_accounts.iter().any(|account| account.get_id() == account_id) {
        pending_remove_ids.push(account_id);
      }
    }

    for account_id in pending_remove_ids {
      index.remove_entity_id(&account_id);
    }

    // Update index
    map.insert(user_id.clone(), index);
  });

  filter_accounts
}

/// Save the staked account to stable memory
pub fn save_stake_account_to_stable_memory(staking_account: &StakingAccount) -> Result<StakingAccount, String> {
  STAKING_ACCOUNT_MAP.with(|map| {
    let mut map = map.borrow_mut();
    // Save the staked account to stable memory
    map.insert(staking_account.get_id(), staking_account.clone());

    STAKING_POOL_ACCOUNT_INDEX_MAP.with(|pool_map| add_indexed_id(pool_map, &staking_account.get_pool_id(), staking_account.get_id()));
    STAKING_USER_ACCOUNT_INDEX_MAP.with(|user_map| add_indexed_id(user_map, &staking_account.get_owner(), staking_account.get_id()));
    
    Ok(staking_account)
  }).cloned()
}


/// Query all staking account which status is in stake
pub fn query_all_in_stake_accounts() -> Vec<StakingAccount> {
  STAKING_ACCOUNT_MAP.with(|map| {
    let map = map.borrow();
    map.values()
      .filter(|account| (*account).get_status() == StakingAccountStatus::InStake)
      .collect::<Vec<StakingAccount>>()
  })
}

/// Inquiry of staked accounts that have expired in the past two days
pub fn query_unstake_near_two_days_account_ids() -> Vec<StakingAccountId> {
  let now = ic_cdk::api::time();
  let today = YearMonthDay::from(now);
  let yesterday = YearMonthDay::from(now - 24 * 60 * 60 * 1_000_000_000);

  STAKING_UNSTAKE_ON_DAY_ACCOUNT_INDEX_MAP.with(|map| {
    // Splice index
    let today_ids = get_indexed_ids(map, &today);
    let yesterday_ids = get_indexed_ids(map, &yesterday);
    let mut all_ids = today_ids;
    all_ids.extend(yesterday_ids);
    all_ids
  })
}

pub fn delete_staking_account(account_id: &StakingAccountId) -> Result<(), String> {
  STAKING_ACCOUNT_MAP.with(|map| {
    let mut map = map.borrow_mut();

    let staking_account = match map.get(account_id) {
      Some(account) => account,
      None => {
        return Err(format!("Staking account {} not found", account_id));
      }
    };

    // Staking accounts that are not newly created are not allowed to be deleted，Avoid mistaken deletion
    if staking_account.get_status() != StakingAccountStatus::Created {
      return Err(format!("Staking account {} is can not be delete, status is {}.", account_id, staking_account.get_status()));
    }

    // Delete the staking account
    map.remove(account_id);
    
    // Delete the index
    STAKING_POOL_ACCOUNT_INDEX_MAP.with(|pool_map| remove_indexed_id(pool_map, &staking_account.get_pool_id(), *account_id));
    STAKING_USER_ACCOUNT_INDEX_MAP.with(|user_map| remove_indexed_id(user_map, &staking_account.get_owner(), *account_id));

    save_delete_staking_account_event_log(account_id);
    Ok(())
  })
}