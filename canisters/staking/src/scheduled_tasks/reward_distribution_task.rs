use std::cell::RefCell;

use types::date::YearMonthDay;

use crate::{account::crud_utils::query_all_in_stake_accounts, reward::utils::distribute_reward};

thread_local! {
  /// stake Reward Distribution Task Lock
  static REWARD_DISTRIBUTION_TASK_RUNNING: RefCell<bool> = RefCell::new(false);
}

/// Staking Reward Distribution Task Code
pub async fn distribute_staking_rewards() {
  if REWARD_DISTRIBUTION_TASK_RUNNING.with(|v| *v.borrow()) {
    ic_cdk::println!("Reward distribution task is already running.");
    return;
  }

  let _guard = scopeguard::guard((), |_| {
    ic_cdk::println!("Reward distribution task state recovery.");
    REWARD_DISTRIBUTION_TASK_RUNNING.with(|v| *v.borrow_mut() = false);
  });

  REWARD_DISTRIBUTION_TASK_RUNNING.with(|v| *v.borrow_mut() = true);

  ic_cdk::println!("Starting reward distribution task...");

  // Get the current date
  let current_date = YearMonthDay::from(ic_cdk::api::time());

  let all_in_stake_accounts = query_all_in_stake_accounts();

  for in_stake_account in all_in_stake_accounts {
    match distribute_reward(&in_stake_account, current_date).await {
      Ok(_) => (),
      Err(e) => {
        ic_cdk::println!("Reward distribute error: {}", e);
      }
    };
  }

  ic_cdk::println!("Reward distribution task completed.");
}
