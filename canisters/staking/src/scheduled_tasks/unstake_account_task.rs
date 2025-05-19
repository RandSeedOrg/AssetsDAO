use std::cell::RefCell;

use crate::account::{crud_utils::query_unstake_near_two_days_account_ids, operation_utils::maturity_unstake};

thread_local! {
  /// Unsolicited Account Task Lock
  static UNSTAKE_ACCOUNT_TASK_RUNNING: RefCell<bool> = RefCell::new(false);
}

/// Unsolicited Account Task Code
pub async fn unstake_accounts() {
  if UNSTAKE_ACCOUNT_TASK_RUNNING.with(|v| *v.borrow()) {
    ic_cdk::println!("Unstake accounts task is already running.");
    return;
  }

  let _guard = scopeguard::guard((), |_| {
    ic_cdk::println!("Unstake accounts task state recovery.");
    UNSTAKE_ACCOUNT_TASK_RUNNING.with(|v| *v.borrow_mut() = false);
  });

  UNSTAKE_ACCOUNT_TASK_RUNNING.with(|v| *v.borrow_mut() = true);
  ic_cdk::println!("Start run unstake accounts task...");

  let will_unstake_account_ids = query_unstake_near_two_days_account_ids();

  for account_id in will_unstake_account_ids {
    // Here you can add the logic of destaking accounts
    ic_cdk::println!("Unstaking account: {}", account_id);
    match maturity_unstake(account_id).await {
      Ok(_) => {
        ic_cdk::println!("Successfully unstaked account: {}", account_id);
      }
      Err(e) => {
        ic_cdk::println!("Failed to unstake account {}: {}", account_id, e);
      }
    };
  }

  ic_cdk::println!("Unstake accounts task completed.");
}
