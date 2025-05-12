use candid::CandidType;
use serde::{Deserialize, Serialize};
use types::E8S;

use crate::{account::{crud_utils::query_current_user_staking_accounts, stable_structures::StakingAccountStatus}, StakingPoolId};

use super::{stable_structures::StakingPool, transport_structures::{LimitConfigVo, RewardConfigVo, TermConfigVo}};

/// CThe return data structure of the end user querying the stake pool information
#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct ClientStakingPoolVo {
  /// Staking poolID
  pub id: StakingPoolId,
  /// Staking pool on-chain address
  pub address: String,
  /// Staking pool maximum capacity
  pub pool_size: E8S,
  /// Staked amount of the staking pool
  pub pool_staked_amount: E8S,
  /// Remainable deposit amount
  pub pool_remain_amount: E8S,
  /// The current user's staked amount
  pub my_staked_amount: E8S,
  /// Remainable deposit amount for the current user
  pub my_remain_stake_amount: E8S,
  /// The total reward amount that the user has received
  pub my_rewards: E8S,
  /// Remaining lock-up: xxx days （weighted average days： （Amount1*Lock date1+Amount2*Lock date2）/（Amount1+Amount2）
  pub my_remaining_lockup: String,
  /// Released Amount:  xxx ICP，illustrate：amount available for unstake （Money from staking address Transfer to user deposit account）
  pub my_released_amount: E8S,
  /// Number of stakes
  pub stakers: u32,
  /// Staking pool status，refer to StakingPoolStatus enumerate
  pub status: String,
  /// stake currency，refer to Crypto enumerate
  pub crypto: String,
  /// Staking pool reward configuration
  pub reward_config: RewardConfigVo,
  /// Staking pool limit configuration
  pub limit_config: LimitConfigVo,
  /// Staking pool term configuration
  pub term_config: TermConfigVo,
}

impl ClientStakingPoolVo {
  /// Convert the Staking pool to the Staking pool information visible to the client
  pub fn from_staking_pool(pool: &StakingPool) -> Self {
    let pool_size = pool.get_pool_size();
    let pool_staked_amount = pool.get_staked_amount();

    let current_user_accounts = query_current_user_staking_accounts(pool.get_id());

    let my_in_stake_accounts = current_user_accounts
      .iter()
      .filter(|account| {
        match account.get_status() {
          StakingAccountStatus::InStake => true,
          _ => false,
        }
      })
      .collect::<Vec<_>>();

    let my_staked_amount = my_in_stake_accounts
      .iter()
      .map(|account| account.get_staked_amount())
      .sum::<E8S>();

    let my_rewards = current_user_accounts
      .iter()
      .map(|account| account.get_accumulated_rewards())
      .sum::<E8S>();

    let my_released_amount = current_user_accounts
      .iter()
      .filter(|account| {
        match account.get_status() {
          StakingAccountStatus::Released => true,
          _ => false,
        }
      })
      .map(|account| account.get_released_amount())
      .sum::<E8S>();

    let my_remaining_lockup = if my_staked_amount > 0 {
      let my_remaining_lockup_actor1 = my_in_stake_accounts
        .iter()
        .map(|account| account.get_staked_amount() * account.get_remaining_lockup_days())
        .sum::<E8S>();
      format!("{}", my_remaining_lockup_actor1 / my_staked_amount)
    } else {
      "0".to_string()
    };

    let pool_max_stake_amount_per_user = pool.get_limit_config().get_max_stake_amount_per_user();

    Self {
      id: pool.get_id(),
      address: pool.get_address(),
      pool_size,
      pool_staked_amount,
      pool_remain_amount: pool_size - pool_staked_amount,
      my_staked_amount: my_staked_amount,
      my_remain_stake_amount: pool_max_stake_amount_per_user - my_staked_amount,
      my_rewards,
      my_remaining_lockup,
      my_released_amount,
      stakers: pool.get_stake_user_count(),
      status: pool.get_status().to_string(),
      crypto: pool.get_crypto().to_string(),
      reward_config: RewardConfigVo::from_config(&pool.get_reward_config()),
      limit_config: LimitConfigVo::from_config(&pool.get_limit_config()),
      term_config: TermConfigVo::from_config(&pool.get_term_config()),
    }
  }
}