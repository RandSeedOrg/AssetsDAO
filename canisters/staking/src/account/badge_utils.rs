use common_canisters::messenger::{BadgeWithProps, PropertiesDto, PropertyValue};
use types::{staking::StakingAccountId, sys::ExteralCanisterLabels};

use crate::system_configs::get_exteral_canister_id;

/// from messenger/src/common/stable_structures.rs
const STAKER_BADGE: u64 = 2;

/// Add staker badge to user
/// This function is used when the user stakes
pub async fn add_staker_badge(user_principal: String, account_id: StakingAccountId) -> Result<(), String> {
  let messenger_canister_id = get_exteral_canister_id(ExteralCanisterLabels::Messenger);
  let messenger = common_canisters::messenger::Service(messenger_canister_id);

  messenger
    .update_user_badges(
      user_principal,
      STAKER_BADGE,
      false,
      Some(vec![BadgeWithProps {
        badge: STAKER_BADGE,
        props: Some(PropertiesDto {
          prop: PropertyValue::U64(account_id),
          timestamp: ic_cdk::api::time(),
        }),
      }]),
    )
    .await
    .map_err(|e| format!("Failed to add staker badge: {:?}", e))?;

  Ok(())
}

/// Remove staker badge from user
/// This function is used when the user unstakes
pub async fn remove_staker_badge(user_principal: String) -> Result<(), String> {
  let messenger_canister_id = get_exteral_canister_id(ExteralCanisterLabels::Messenger);
  let messenger = common_canisters::messenger::Service(messenger_canister_id);

  messenger
    .update_user_badges(user_principal, STAKER_BADGE, true, None)
    .await
    .map_err(|e| format!("Failed to remove staker badge: {:?}", e))?;

  Ok(())
}
