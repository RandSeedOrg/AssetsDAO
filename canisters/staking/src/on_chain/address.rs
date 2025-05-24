use candid::Principal;
use ic_ledger_types::{AccountIdentifier, Subaccount};
use types::{
  staking::{StakingAccountId, StakingPoolId},
  EntityId,
};

const STAKING_POOL_ADDRESS_PREFIX: &str = "staking_pool_";

const STAKING_ACCOUNT_ADDRESS_PREFIX: &str = "staking_account_";

/// Generate staking pool on-chain address
///
/// # Arguments
/// * `pool_id` - Staking pool ID
///
/// # Returns
/// Returns a standard ICP Account address string
pub fn generate_staking_pool_chain_address(pool_id: StakingPoolId) -> String {
  let canister_id = ic_cdk::api::canister_self();
  let sub_account_id = format!("{}{}", STAKING_POOL_ADDRESS_PREFIX, pool_id);
  generate_address(canister_id, &sub_account_id)
}

/// Staking account on-chain address
///
/// # Arguments
/// * `account_id` - Staking account ID
///
/// # Returns
/// Returns a standard ICP Account address string
pub fn generate_staking_account_chain_address(account_id: StakingAccountId) -> String {
  let canister_id = ic_cdk::api::canister_self();
  let sub_account_id = format!("{}{}", STAKING_ACCOUNT_ADDRESS_PREFIX, account_id);
  generate_address(canister_id, &sub_account_id)
}

pub fn generate_staking_account_account_identifier(account_id: StakingAccountId) -> AccountIdentifier {
  let canister_id = ic_cdk::api::canister_self();
  let sub_account_id = format!("{}{}", STAKING_ACCOUNT_ADDRESS_PREFIX, account_id);
  generate_account_identifier(canister_id, &sub_account_id)
}

pub fn generate_staking_pool_account_identifier(pool_id: StakingPoolId) -> AccountIdentifier {
  let canister_id = ic_cdk::api::canister_self();
  let sub_account_id = format!("{}{}", STAKING_POOL_ADDRESS_PREFIX, pool_id);
  generate_account_identifier(canister_id, &sub_account_id)
}

pub fn generate_staking_pool_subaccount(pool_id: EntityId) -> Subaccount {
  generate_subaccount(&format!("{}{}", STAKING_POOL_ADDRESS_PREFIX, pool_id))
}

pub fn generate_staking_account_subaccount(account_id: EntityId) -> Subaccount {
  generate_subaccount(&format!("{}{}", STAKING_ACCOUNT_ADDRESS_PREFIX, account_id))
}

fn generate_account_identifier(principal: Principal, account_id: &str) -> AccountIdentifier {
  AccountIdentifier::new(&principal, &generate_subaccount(&account_id))
}

/// Generate a standard ICP Account address
///
/// # Arguments
/// * `principal` - main body ID，Usually canister ID
/// * `sub_account_id` - Sub-account ID，Can be a number or string identifier
///
/// # Returns
/// Returns a standard ICP Account address string
fn generate_address(principal: Principal, account_id: &str) -> String {
  generate_account_identifier(principal, account_id).to_hex()
}

fn generate_subaccount(subaccount_id: &str) -> Subaccount {
  let mut subaccount = [0; 32];

  // According to different types sub_account_id Fill in sub-accounts
  if let Ok(id) = subaccount_id.parse::<u64>() {
    // If it's a number，Fill in the sub-account with the number
    let bytes = id.to_be_bytes();
    subaccount[0..8].copy_from_slice(&bytes);
  } else {
    // If it is a string，Populate subaccounts with its hash value
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(subaccount_id.as_bytes());
    let result = hasher.finalize();
    subaccount[0..32].copy_from_slice(&result[0..32]);
  }

  // Create a sub-account object
  Subaccount(subaccount)
}
