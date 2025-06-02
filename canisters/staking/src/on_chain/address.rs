use candid::Principal;
use ic_ledger_types::{AccountIdentifier, Subaccount, MAINNET_GOVERNANCE_CANISTER_ID};
use sha2::{Digest, Sha256};
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

/// Computes the bytes of the subaccount to which neuron staking transfers are made. This
/// function must be kept in sync with the Nervous System UI equivalent.
/// This code comes from the IC repo:
/// https://github.com/dfinity/ic/blob/master/rs/nervous_system/common/src/ledger.rs#L211
fn compute_neuron_staking_subaccount_bytes(controller: Principal, nonce: u64) -> [u8; 32] {
  const DOMAIN: &[u8] = b"neuron-stake";
  const DOMAIN_LENGTH: [u8; 1] = [0x0c];

  let mut hasher = Sha256::new();
  hasher.update(DOMAIN_LENGTH);
  hasher.update(DOMAIN);
  hasher.update(controller.as_slice());
  hasher.update(nonce.to_be_bytes());
  hasher.finalize().into()
}

/// Generate a nns neuron account identifier for staking pool
pub fn generate_staking_pool_neuron_account(pool_id: StakingPoolId) -> AccountIdentifier {
  let canister_id = ic_cdk::api::canister_self();
  let account_buf = compute_neuron_staking_subaccount_bytes(canister_id, pool_id);
  AccountIdentifier::new(&MAINNET_GOVERNANCE_CANISTER_ID, &Subaccount(account_buf))
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
