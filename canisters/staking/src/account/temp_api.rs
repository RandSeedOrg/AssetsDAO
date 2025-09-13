use ic_cdk::{
  api::{is_controller, msg_caller},
  update,
};

use super::{badge_utils::add_staker_badge, crud_utils::query_all_in_stake_accounts};

#[update(hidden = true)]
async fn old_staker_badge_add() -> Result<(), String> {
  if !is_controller(&crate::identity_mapping::wl_caller()) {
    return Err("Only controller can call this api.".to_string());
  }

  let staking_accounts = query_all_in_stake_accounts();

  for account in staking_accounts {
    add_staker_badge(account.get_owner(), account.get_id())
      .await
      .map_err(|e| format!("Failed to add badge for {} account {}: {}", account.get_owner(), account.get_id(), e))?;
  }

  Ok(())
}
