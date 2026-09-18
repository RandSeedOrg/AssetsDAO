use bigdecimal::{BigDecimal, ToPrimitive};

use types::TimestampNanos;

use super::stable_structures::StakingAccount;

const ONE_HUNDRED_AND_EIGHTY_DAYS_OF_NANOSECONDS: u64 = 180 * 24 * 60 * 60 * 1_000_000_000;
const MINIMUM_PENALTY_E8S: u64 = 10_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnstakeAmounts {
  pub penalty_amount: u64,
  pub released_amount: u64,
}

/// Calculate the amounts for a user initiated unstake.
///
/// Once the account reaches its configured deadline it is treated as a mature
/// unstake, so no penalty is charged regardless of how long it has remained in
/// the pool. All arithmetic is performed in e8s and the released amount is
/// never allowed to underflow.
pub fn calculate_unstake_amounts(account: &StakingAccount, now: TimestampNanos) -> UnstakeAmounts {
  let penalty_amount = if now >= account.get_stake_deadline() {
    0
  } else {
    let reward_ratio = if now < account.get_stake_time() + ONE_HUNDRED_AND_EIGHTY_DAYS_OF_NANOSECONDS {
      (8, 10)
    } else {
      (5, 10)
    };
    let penalty = (BigDecimal::from(account.get_accumulated_rewards()) * BigDecimal::from(reward_ratio.0) / BigDecimal::from(reward_ratio.1))
      .to_u64()
      .unwrap_or_default();
    if penalty <= MINIMUM_PENALTY_E8S {
      0
    } else {
      penalty
    }
  };

  UnstakeAmounts {
    penalty_amount,
    released_amount: account.get_staked_amount().saturating_sub(penalty_amount),
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn account(staked_amount: u64, rewards: u64, stake_time: u64, deadline: u64) -> StakingAccount {
    StakingAccount {
      id: Some(1),
      pool_id: Some(1),
      owner: Some("owner".to_string()),
      address: Some("address".to_string()),
      released_amount: None,
      staked_amount: Some(staked_amount),
      penalty_amount: None,
      accumulated_rewards: Some(rewards),
      status: None,
      reward_config: None,
      stake_pay_center_onchain_tx_id: None,
      stake_pay_center_tx_id: None,
      stake_account_to_pool_onchain_tx_id: None,
      release_onchain_tx_id: None,
      dissolve_onchain_tx_id: None,
      dissolve_pay_center_tx_id: None,
      penalty_onchain_tx_id: None,
      penalty_pay_center_tx_id: None,
      total_staking_days: Some(180),
      min_early_unstake_days: Some(1),
      stake_time: Some(stake_time),
      can_early_unstake_time: Some(stake_time),
      stake_deadline: Some(deadline),
      release_time: None,
      dissolve_time: None,
      last_reward_time: None,
      meta: None,
      recoverable_error: None,
    }
  }

  #[test]
  fn mature_account_has_no_penalty() {
    let account = account(39_000_000_000, 3_510_000_000, 1_000, 2_000);

    assert_eq!(
      calculate_unstake_amounts(&account, 2_000),
      UnstakeAmounts {
        penalty_amount: 0,
        released_amount: 39_000_000_000,
      }
    );
  }

  #[test]
  fn immature_account_deducts_penalty_from_principal_once() {
    let account = account(39_000_000_000, 3_510_000_000, 1_000, u64::MAX);

    assert_eq!(
      calculate_unstake_amounts(&account, 2_000),
      UnstakeAmounts {
        penalty_amount: 2_808_000_000,
        released_amount: 36_192_000_000,
      }
    );
  }
}
