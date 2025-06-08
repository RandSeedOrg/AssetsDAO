use candid::Principal;
use common_canisters::pay_center::Result2;
use types::{date::YearMonthDay, product::generate_staking_reward_payment_transaction_id, sys::ExteralCanisterLabels};

use crate::{
  account::stable_structures::StakingAccount,
  event_log::stake_reward_events::{save_reward_distribute_event, save_reward_received_event},
  guard_keys::get_distribute_reward_guard_key,
  parallel_guard::EntryGuard,
  system_configs::get_exteral_canister_id,
};

use super::{
  stable_key::StakingAccountUserRewardDateIndexKey,
  stable_structures::{StakingReward, StakingRewardStatus},
  STAKING_REWARD_MAP, STAKING_USER_ACCOUNT_REWARD_DATE_INDEX_MAP,
};

/// Update the reward distribution record of the current account on a certain day
fn update_account_distributed_the_day(reward: &StakingReward, day: YearMonthDay) {
  let index_key = StakingAccountUserRewardDateIndexKey::new(reward.get_account_id(), reward.get_owner(), day);

  STAKING_USER_ACCOUNT_REWARD_DATE_INDEX_MAP.with(|v| {
    let mut map = v.borrow_mut();
    map.insert(index_key, reward.get_id());
  });
}

/// Obtain a stake reward distribution record for a certain day
pub fn get_account_distributed_the_day(account: &StakingAccount, day: YearMonthDay) -> Option<StakingReward> {
  let index_key = StakingAccountUserRewardDateIndexKey::new(account.get_id(), account.get_owner(), day);

  let reward_id = STAKING_USER_ACCOUNT_REWARD_DATE_INDEX_MAP.with(|v| {
    let map = v.borrow();
    map.get(&index_key)
  });

  reward_id.and_then(|reward_id| {
    STAKING_REWARD_MAP.with(|v| {
      let map = v.borrow();
      map.get(&reward_id)
    })
  })
}

/// Issuing stake rewards
pub async fn distribute_reward(account: &StakingAccount, day: YearMonthDay) -> Result<StakingReward, String> {
  // Reentry protection
  let _entry_guard = EntryGuard::new(get_distribute_reward_guard_key(account.get_id()))
    .map_err(|_| format!("Failed to acquire distribute reward entry guard for account {}", account.get_id()))?;

  // If the current account is staked today，No reward will be issued
  if YearMonthDay::from(account.get_stake_time()) == day {
    return Err(format!("Account {} is stake today, no reward to distribute", account.get_id()));
  }

  // If the expiration time of the current account is less than today(Expired)，No reward will be issued
  if YearMonthDay::from(account.get_stake_deadline()) < day {
    return Err(format!("Account {} is expired, no reward to distribute", account.get_id()));
  }

  // 1. Create a stake reward record，If already exists，Then no longer create
  let (reward, updated_staking_account) = match get_account_distributed_the_day(account, day) {
    Some(reward) => (reward, account.clone()), // Get the reward distribution record of the current account in today's reward distribution record，If so, return the reward record and account information
    None => StakingReward::reward_account(account), // If not，Create a new reward record
  };

  if reward.get_status() == StakingRewardStatus::Created {
    // 1.1 If the staking account is in a new state，Update today's reward distribution index
    update_account_distributed_the_day(&reward, day);
  } else {
    // 1.2 The stake reward for the day has been issued，No reissue
    return Err(format!(
      "Account {} has already been distributed on {}-{}-{}",
      account.get_id(),
      day.year(),
      day.month(),
      day.day()
    ));
  }

  // Save Reward Distribution Events
  save_reward_distribute_event(&reward, &updated_staking_account);

  let user_principal = match Principal::from_text(account.get_owner()) {
    Ok(principal) => principal,
    Err(_) => {
      return Err(format!("Invalid user principal: {}", account.get_owner()));
    }
  };

  // Initiate a stake reward issuance request from the payment center
  let pay_center_canister_id = get_exteral_canister_id(ExteralCanisterLabels::PayCenter);
  let pay_center = common_canisters::pay_center::Service(pay_center_canister_id);
  let tx_id = generate_staking_reward_payment_transaction_id(reward.get_id()).unwrap();
  let (response,) = match pay_center
    .update_account_bonus(
      user_principal,
      reward.get_reward_amount_float(),
      Some(tx_id),
      "Staking Rewards".to_string(),
      Some(reward.get_account_id()),
      Some(reward.get_id()),
      Vec::new(),
    )
    .await
  {
    Ok(response) => response,
    Err(e) => {
      return Err(format!("Failed to update account bonus: {}", e.1));
    }
  };

  let pay_center_tx_id = match response {
    Result2::Ok(tx_id) => tx_id,
    Result2::Err(e) => {
      return Err(format!("Failed to update account bonus: {}", e));
    }
  };

  let updated_reward = reward.received(pay_center_tx_id)?;

  save_reward_received_event(&updated_reward);

  Ok(updated_reward)
}
