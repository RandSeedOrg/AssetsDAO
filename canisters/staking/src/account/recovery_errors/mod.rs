use recover_early_unstake::{recover_unstake_penalty_onchain_error, recover_unstake_penalty_pay_center_error};
use types::staking::StakingAccountId;

use super::{stable_structures::StakingAccountRecoverableError, STAKING_ACCOUNT_MAP};

pub mod recover_dissolve;
pub mod recover_stake;
pub mod recover_early_unstake;

/// stake account error recovery
pub async fn recover_staking_account_error(account_id: StakingAccountId) -> Result<(), String> {
  // 1. Get a staked account
  let account = STAKING_ACCOUNT_MAP.with(|map| {
    let map = map.borrow();
    map.get(&account_id).ok_or("Account not found")
  })?;

  if account.recoverable_error.is_none() {
    return Err("Account is not in recoverable error state".to_string());
  }

  let recoverable_error = account.recoverable_error.clone().unwrap();

  // 2. According to error type，Call the corresponding recovery function
  match recoverable_error {
    StakingAccountRecoverableError::StakeTransferToPoolFailed(pay_center_onchain_tx_id, pay_center_tx_id) => {
      match recover_stake::recover_stake_error(&account, pay_center_onchain_tx_id, pay_center_tx_id).await {
        Ok(_) => Ok(()),
        Err(e) => {
          // 5. Recovery failed，Return error message
          Err(format!("Failed to recover stake: account_id = {}, error = {}", account_id, e))
        }
      }
    }
    StakingAccountRecoverableError::DissolvePayCenterFailed(dissolve_tx_id) => {
      match recover_dissolve::recover_dissolve_error(&account, dissolve_tx_id).await {
        Ok(_) => Ok(()),
        Err(e) => {
          // 5. Recovery failed，Return error message
          Err(format!("Failed to recover dissolve: account_id = {}, error = {}", account_id, e))
        }
      }
    }
    StakingAccountRecoverableError::EarlyUnstakePenaltyOnChainFailed(_, _, _) => {
      // If the current staked account status is a restored error status，Indicates that the on-chain transfer has been completed，Payment Center failed to bookkeeping，Therefore, the error recovery process is directly followed
      match recover_unstake_penalty_onchain_error(&account).await {
        Ok(_) => Ok(()),
        Err(e) => {
          Err(format!("Failed to recover early unstake penalty on-chain: account_id = {}, error = {}", account_id, e))
        }
      }
    }
    StakingAccountRecoverableError::EarlyUnstakePenaltyPayCenterFailed(_, _, _, _) => {
      // If the current staked account status is a restored error status，Indicates that the on-chain transfer has been completed，Payment Center failed to bookkeeping，Therefore, the error recovery process is directly followed
      match recover_unstake_penalty_pay_center_error(&account).await {
        Ok(_) => Ok(()),
        Err(e) => {
          Err(format!("Failed to recover early unstake penalty pay center: account_id = {}, error = {}", account_id, e))
        }
      }
    }
  }
}