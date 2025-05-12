use crate::{account::{recovery_errors::recover_staking_account_error, STAKING_RECOVERABLE_ERROR_ACCOUNT_INDEX_MAP}, parallel_guard::EntryGuard};

/// Recovery task for error records caused by stake operations
/// This task checks for unprocessed error records every time it starts，And try to recover them

pub async fn recover_staking_account_errors() {
  let _entry_guard = EntryGuard::new("recover_staking_account_errors".to_string());

  // Check for unprocessed error records
  let recoverable_error_account_ids = STAKING_RECOVERABLE_ERROR_ACCOUNT_INDEX_MAP.with(|map| {
      let map = map.borrow();
      map.iter()
        .map(|(_, entity_index)| {
          entity_index.get_entity_ids()
        })
        .flatten()
        .collect::<Vec<_>>()
  });

  ic_cdk::println!("Recoverable error account IDs: {:?}", recoverable_error_account_ids);
  
  for account_id in recoverable_error_account_ids {
    // Try to restore error logs
    match recover_staking_account_error(account_id).await {
      Ok(_) => {
        ic_cdk::println!("Successfully recovered stake error for account: {}", account_id);
      }
      Err(e) => {
        ic_cdk::println!("Failed to recover stake error for account {}: {}", account_id, e);
      }
    }
  }

  ic_cdk::println!("Finished proceed recoverable error for accounts.");
}