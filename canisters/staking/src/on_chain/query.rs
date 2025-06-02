use ic_ledger_types::{account_balance, AccountBalanceArgs, AccountIdentifier, MAINNET_LEDGER_CANISTER_ID};

/// Fetches the balance of a given account on the mainnet ledger.
pub async fn balance_of(target: &AccountIdentifier) -> Result<u64, String> {
  account_balance(MAINNET_LEDGER_CANISTER_ID, &AccountBalanceArgs { account: *target })
    .await
    .map_err(|e| format!("Failed to get balance: {:?}", e))
    .and_then(|balance| Ok(balance.e8s()))
}
