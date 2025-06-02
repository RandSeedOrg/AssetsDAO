use ic_ledger_types::{AccountIdentifier, BlockIndex, Memo, Subaccount, Tokens, TransferArgs, TransferError, MAINNET_LEDGER_CANISTER_ID};
use types::{
  staking::{StakingAccountId, StakingPoolId},
  sys::ExteralCanisterLabels,
  E8S,
};

use crate::system_configs::get_exteral_canister_id;

use super::address::{
  generate_staking_account_account_identifier, generate_staking_account_subaccount, generate_staking_pool_account_identifier,
  generate_staking_pool_neuron_account, generate_staking_pool_subaccount,
};

/// Parsing from string AccountIdentifier
fn parse_account_id(account_str: &str) -> Result<AccountIdentifier, String> {
  AccountIdentifier::from_hex(account_str).map_err(|e| format!("Invalid account address: {}", e))
}

/// Will E8S Convert to Tokens type
fn e8s_to_tokens(amount: E8S) -> Tokens {
  Tokens::from_e8s(amount)
}

pub const TRANSFER_SCENE_STAKE: u64 = 1;
pub const TRANSFER_SCENE_UNSTAKE: u64 = 2;
pub const TRANSFER_SCENE_PAY_CENTER: u64 = 3;
pub const TRANSFER_SCENE_UNSTAKE_PENALTY: u64 = 4;
pub const TRANSFER_SCENE_NNS_STAKE: u64 = 5;

pub async fn transfer_from_staking_pool_to_staking_account(
  pool_id: StakingPoolId,
  account_id: StakingAccountId,
  amount: E8S,
) -> Result<BlockIndex, String> {
  // Transfer out of the account
  let from_account = generate_staking_pool_subaccount(pool_id);

  // Transfer to the account
  let to_account = generate_staking_account_account_identifier(account_id);

  // Perform a transfer, the transfer amount needs to increase a handling fee, these amounts are paid in advance from the payment center
  transfer(&from_account, &to_account, amount + 10_000, Memo(TRANSFER_SCENE_UNSTAKE)).await
  // Use a valid u64 value for Memo
}

/// Transfer from staking account to staking pool (Stake)
pub async fn transfer_from_staking_account_to_staking_pool(
  account_id: StakingAccountId,
  pool_id: StakingPoolId,
  amount: E8S,
) -> Result<BlockIndex, String> {
  // Transfer out of the account
  let from_account = generate_staking_account_subaccount(account_id);

  // Transfer to the account
  let to_account = generate_staking_pool_account_identifier(pool_id);

  // Perform a transfer. The Transfer amount needs to add two handling fee amounts. These amounts are paid in advance from the payment center
  transfer(&from_account, &to_account, amount + 20_000, Memo(TRANSFER_SCENE_STAKE)).await
}

/// Transfer funds from the Pledge account to the payment center, that is, the released funds in the staking account are transferred to the payment center
pub async fn transfer_from_staking_account_to_pay_center(from_account_id: StakingAccountId, amount: E8S) -> Result<BlockIndex, String> {
  // Transfer out of the account
  let from_account = generate_staking_account_subaccount(from_account_id);

  let pay_center_canister_id = get_exteral_canister_id(ExteralCanisterLabels::PayCenter);
  let pay_center = common_canisters::pay_center::Service(pay_center_canister_id);

  let (pay_center_address,) = pay_center
    .get_address()
    .await
    .map_err(|e| format!("Failed to obtain payment center address: {:?}", e))?;

  // Transfer to the account
  let to_account = parse_account_id(&pay_center_address)?;

  // Perform a transfer
  transfer(&from_account, &to_account, amount, Memo(TRANSFER_SCENE_PAY_CENTER)).await
}

/// Transfer from the Pledge pool to the payment center. The current scenario is that when a user initiates early release of the Pledge, the resulting penalty is directly transferred to the payment center through the Pledge pool
pub async fn transfer_from_staking_pool_to_pay_center(pool_id: StakingPoolId, amount: E8S) -> Result<BlockIndex, String> {
  // Transfer out of the account
  let from_account = generate_staking_pool_subaccount(pool_id);

  let pay_center_canister_id = get_exteral_canister_id(ExteralCanisterLabels::PayCenter);
  let pay_center = common_canisters::pay_center::Service(pay_center_canister_id);

  let (pay_center_address,) = pay_center
    .get_address()
    .await
    .map_err(|e| format!("Failed to obtain payment center address: {:?}", e))?;

  // Transfer to the account
  let to_account = parse_account_id(&pay_center_address)?;

  // Perform a transfer
  transfer(&from_account, &to_account, amount - 10_000, Memo(TRANSFER_SCENE_UNSTAKE_PENALTY)).await
}

/// Transfer function
/// Transfer money from one sub-account to another
///
/// # Arguments
/// * `from_account` - Sub-accounts transferred out of the account
/// * `to_account` - Transfer to the account
/// * `amount` - Transfer amount，The unit is E8S
///
/// # Returns
/// Return one Result，Block index or error message containing transfers
///
/// # Errors
/// Return one Result，Block index or error message containing transfers
async fn transfer(from_account: &Subaccount, to_account: &AccountIdentifier, amount: E8S, memo: Memo) -> Result<BlockIndex, String> {
  // Call ICP Ledger canister Make a transfer
  let ledger_canister_id = MAINNET_LEDGER_CANISTER_ID;

  // Create transfer parameters
  let transfer_args = TransferArgs {
    memo, // Memo information can be set
    amount: e8s_to_tokens(amount),
    fee: e8s_to_tokens(10_000), // ICP The standard transfer fee is 10000 E8S
    from_subaccount: Some(from_account.clone()),
    to: to_account.clone(),
    created_at_time: None, // Current time of using the system
  };

  let result = ic_cdk::call::Call::unbounded_wait(ledger_canister_id, "transfer")
    .with_arg(transfer_args)
    .await
    .map_err(|e| format!("Call Ledger failed: {:?}", e))?;

  match result.candid_tuple::<(Result<BlockIndex, TransferError>,)>() {
    Ok((Ok(block_index),)) => Ok(block_index),
    Ok((Err(error),)) => Err(format!("Transfer failed: {:?}", error)),
    Err(error) => Err(format!("Transfer failed: {:?}", error)),
  }
}

/// Transfer from the Pledge pool to the payment center. The current scenario is that when a user initiates early release of the Pledge, the resulting penalty is directly transferred to the payment center through the Pledge pool
pub async fn transfer_from_staking_pool_to_nns_neuron(pool_id: StakingPoolId, amount: E8S) -> Result<BlockIndex, String> {
  // Transfer out of the account
  let from_account = generate_staking_pool_subaccount(pool_id);
  let to_account = generate_staking_pool_neuron_account(pool_id);

  // Perform a transfer
  transfer(&from_account, &to_account, amount, Memo(TRANSFER_SCENE_NNS_STAKE)).await
}
