use candid::CandidType;
use serde::{Deserialize, Serialize};
use types::{EntityId, TimestampNanos, E8S};

use super::stable_structures::{LimitConfig, RewardConfig, StakingPool, TermConfig};

#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct StakingPoolUpdateDto {
  /// Staking poolID
  pub id: EntityId,
  /// Additional attributes when added
  pub add_dto: StakingPoolAddDto,
}

/// Create the Staking Pool transfer object
#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct StakingPoolAddDto {
  /// Target financing amount，The maximum amount of stake that the stake pool can accommodate
  pub pool_size: E8S,
  /// Staking currency
  pub crypto: String,
  /// Staking pool term configuration
  pub term_config: TermConfigVo,
  /// Staking pool reward configuration
  pub reward_config: RewardConfigVo,
  /// Staking pool limit configuration
  pub limit_config: LimitConfigVo,
}

#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct TermConfigVo {
  /// The minimum deadline of staking pool，Unit is day
  pub min_term: u16,
  /// The maximum deadline of staking pool，Unit is day
  pub max_term: u16,
  /// Minimum number of days to be deposable in advance
  pub min_early_unstake_days: u16,
}

impl TermConfigVo {
  pub fn from_config(config: &TermConfig) -> Self {
    Self {
      min_term: config.get_min_term(),
      max_term: config.get_max_term(),
      min_early_unstake_days: config.get_min_early_unstake_days(),
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct RewardConfigVo {
  /// Annualized interest rate
  pub annualized_interest_rate: E8S,
  /// Daily interest rate
  pub daily_interest_rate: E8S,
  /// The reward currency of the staking pool，refer to RewardCrypto enumerate
  pub reward_crypto: String,
}

impl RewardConfigVo {
  pub fn from_config(config: &RewardConfig) -> Self {
    Self {
      annualized_interest_rate: config.get_annualized_interest_rate(),
      daily_interest_rate: config.get_daily_interest_rate(),
      reward_crypto: config.get_reward_crypto().to_string(),
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct LimitConfigVo {
  /// The minimum staking amount of the staking pool，The unit isE8S
  pub min_stake_amount_per_user: E8S,
  /// The maximum staking amount of the staking pool，The unit isE8S
  pub max_stake_amount_per_user: E8S,
  /// The minimum step add amount of staking pool，The unit isE8S
  pub step_amount: E8S,
}

impl LimitConfigVo {
  pub fn from_config(config: &LimitConfig) -> Self {
    Self {
      min_stake_amount_per_user: config.get_min_stake_amount_per_user(),
      max_stake_amount_per_user: config.get_max_stake_amount_per_user(),
      step_amount: config.get_step_amount(),
    }
  }
}

/// Staking pool information visible to the client
#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct StakingPoolVo {
  /// Staking poolID，Unique ID
  pub id: EntityId,
  /// Staking pool on-chain address
  pub address: String,
  /// Target financing amount，The maximum amount of stake that the stake pool can accommodate
  pub pool_size: E8S,
  /// Staked amount of the staking pool
  pub staked_amount: E8S,
  /// The amount of funds occupied by the NNS neuron
  pub nns_neuron_occupies_funds: E8S,
  /// The amount of funds occupied by the jackpot
  pub jackpot_occupies_funds: E8S,
  /// Available funds in the staking pool
  pub available_funds: E8S,
  /// Number of users who have staked
  pub stake_user_count: u32,
  /// stake currency，refer to Crypto enumerate
  pub crypto: String,
  /// Staking pool status，refer to StakingPoolStatus enumerate
  pub status: String,
  /// Term configuration information
  pub term_config: TermConfigVo,
  /// stake reward configuration information
  pub reward_config: RewardConfigVo,
  /// Staking pool limit configuration information
  pub limit_config: LimitConfigVo,
  /// Is it visible to the client
  pub client_visible: bool,
  /// Opening hours
  pub open_time: i64,
  /// Close time
  pub close_time: i64,
  /// End time
  pub end_time: i64,
  /// Created by
  pub creator: String,
  /// Creation time
  pub create_time: TimestampNanos,
  /// Updater
  pub last_update_person: String,
  /// Last updated time
  pub last_update_time: TimestampNanos,
}

impl StakingPoolVo {
  pub fn from_staking_pool(pool: &StakingPool) -> Self {
    let meta = pool.get_meta();

    Self {
      id: pool.get_id(),
      address: pool.get_address(),
      pool_size: pool.get_pool_size(),
      staked_amount: pool.get_staked_amount(),
      stake_user_count: pool.get_stake_user_count(),
      crypto: pool.get_crypto().to_string(),
      status: pool.get_status().to_string(),
      term_config: TermConfigVo::from_config(&pool.get_term_config()),
      reward_config: RewardConfigVo::from_config(&pool.get_reward_config()),
      limit_config: LimitConfigVo::from_config(&pool.get_limit_config()),
      client_visible: pool.get_client_visible(),
      open_time: {
        let time = pool.get_open_time();
        if time == 0 {
          -1
        } else {
          time as i64 / 1_000_000
        }
      },
      close_time: {
        let time = pool.get_clone_time();
        if time == 0 {
          -1
        } else {
          time as i64 / 1_000_000
        }
      },
      end_time: {
        let time = pool.get_end_time();
        if time == 0 {
          -1
        } else {
          time as i64 / 1_000_000
        }
      },
      creator: meta.get_created_by(),
      create_time: meta.get_created_at(),
      last_update_person: meta.get_updated_by(),
      last_update_time: meta.get_updated_at(),
      nns_neuron_occupies_funds: pool.get_nns_neuron_occupies_funds(),
      jackpot_occupies_funds: pool.get_jackpot_occupies_funds(),
      available_funds: pool.get_available_funds().unwrap_or_default(),
    }
  }
}
