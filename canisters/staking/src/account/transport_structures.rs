use candid::CandidType;
use serde::{Deserialize, Serialize};
use types::{
  pagination::{PageRequest, PageResponse},
  staking::{StakingAccountId, StakingPoolId},
  UserId, E8S,
};

use crate::pool::transport_structures::RewardConfigVo;

use super::stable_structures::StakingAccount;

#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct StakingAccountQueryParams {
  /// Staking poolID
  pub pool_id: StakingPoolId,
  /// userID
  pub user_id: String,
  /// Address on the chain of stake account
  pub onchain_address: String,
  pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct StakingAccountVo {
  /// stake accountID
  pub id: StakingAccountId,
  /// Staking poolID
  pub pool_id: StakingPoolId,
  /// The owner of the stake account Principal ID
  pub owner: UserId,
  /// Address on the chain of stake account
  pub onchain_address: String,
  /// The amount released in the staked account
  pub released_amount: E8S,
  /// The amount of the stake account
  pub staked_amount: E8S,
  /// Deduction amount for staked account
  pub penalty_amount: E8S,
  /// The total amount of rewards received
  pub accumulated_rewards: E8S,
  /// stake account status,refer to StakingAccountStatus enumerate
  pub status: String,
  /// Reward configuration for staked accounts
  pub reward_configs: Vec<RewardConfigVo>,
  /// When staking，On-chain transaction ID of the payment center
  pub stake_pay_center_onchain_tx_id: u64,
  /// Payment center transaction flow during stake ID
  pub stake_pay_center_tx_id: u64,
  /// When staking，Stake the account and the transaction on the chain from the stake pool ID
  pub stake_account_to_pool_onchain_tx_id: u64,
  /// On-chain transactions at release ID
  pub release_onchain_tx_id: u64,
  /// On-chain transactions id when dissolved
  pub dissolve_onchain_tx_id: u64,
  /// When dissolved，Payment Center Transactions ID
  pub dissolve_pay_center_tx_id: u64,
  /// minimum unstake days
  pub min_early_unstake_days: u16,
  /// stake days
  pub total_staking_days: u16,
  /// stake start time
  pub stake_time: u64,
  /// stake expiration time
  pub stake_deadline: u64,
  /// Release time for stake account
  pub release_time: u64,
  pub dissolve_time: u64,
  pub last_reward_time: u64,
  /// Stake account creation time
  pub create_time: u64,
}

pub type StakingAccountPageRequest = PageRequest<StakingAccountQueryParams>;
pub type StakingAccountPageResponse = PageResponse<StakingAccountVo>;

impl StakingAccountVo {
  /// Convert staked account to staked account information visible to the client
  pub fn from_staking_account(account: &StakingAccount) -> Self {
    Self {
      id: account.get_id(),
      pool_id: account.get_pool_id(),
      owner: account.get_owner(),
      onchain_address: account.get_onchain_address(),
      released_amount: account.get_released_amount(),
      staked_amount: account.get_staked_amount(),
      penalty_amount: account.get_penalty_amount(),
      accumulated_rewards: account.get_accumulated_rewards(),
      status: account.get_status().to_string(),
      reward_configs: account.get_reward_configs().iter().map(RewardConfigVo::from_config).collect(),
      stake_pay_center_onchain_tx_id: account.get_stake_pay_center_onchain_tx_id(),
      stake_pay_center_tx_id: account.get_stake_pay_center_tx_id(),
      stake_account_to_pool_onchain_tx_id: account.get_stake_account_to_pool_onchain_tx_id(),
      release_onchain_tx_id: account.get_release_onchain_tx_id(),
      dissolve_onchain_tx_id: account.get_dissolve_onchain_tx_id(),
      dissolve_pay_center_tx_id: account.get_dissolve_pay_center_tx_id(),
      min_early_unstake_days: account.get_min_early_unstake_days(),
      total_staking_days: account.get_total_staking_days(),
      stake_time: account.get_stake_time(),
      stake_deadline: account.get_stake_deadline(),
      release_time: account.get_release_time(),
      dissolve_time: account.get_dissolve_time(),
      last_reward_time: account.get_last_reward_time(),
      create_time: account.get_create_time(),
    }
  }
}
