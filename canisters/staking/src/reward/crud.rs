use system_configs_macro::has_permission;
use types::entities::get_indexed_ids;

use super::{
  STAKING_ACCOUNT_REWARD_INDEX_MAP, STAKING_POOL_REWARD_INDEX_MAP, STAKING_REWARD_MAP, STAKING_USER_REWARD_INDEX_MAP,
  transport_structures::{StakingRewardPageRequest, StakingRewardPageResponse, StakingRewardQueryParams, StakingRewardVo},
};

#[ic_cdk::query]
#[has_permission("staking::reward::query")]
fn query_staking_rewords(request: StakingRewardPageRequest) -> StakingRewardPageResponse {
  let StakingRewardPageRequest {
    page,
    page_size,
    params: StakingRewardQueryParams {
      pool_id,
      account_id,
      user_id,
      status,
      start_time,
      end_time,
    },
  } = request;

  let indexed_records = STAKING_REWARD_MAP.with(|map| {
    let map = map.borrow();

    // Filter the data according to the passed in index
    if user_id.len() == 63 {
      STAKING_USER_REWARD_INDEX_MAP
        .with(|m| get_indexed_ids(m, &user_id))
        .iter()
        .filter_map(|id| map.get(id))
        .collect::<Vec<_>>()
    } else if account_id > 0 {
      // If the account is passedID，Get the corresponding one from the account index queryID
      STAKING_ACCOUNT_REWARD_INDEX_MAP
        .with(|m| get_indexed_ids(m, &account_id))
        .iter()
        .filter_map(|id| map.get(id))
        .collect::<Vec<_>>()
    } else if pool_id > 0 {
      // If the stake pool is passedID，Get the corresponding one from the staking pool index queryID
      STAKING_POOL_REWARD_INDEX_MAP
        .with(|m| get_indexed_ids(m, &pool_id))
        .iter()
        .filter_map(|id| map.get(id))
        .collect::<Vec<_>>()
    } else {
      // If no indexes are passed in，Query from all reward data
      map.iter().map(|(_, record)| record).collect::<Vec<_>>()
    }
  });

  let filtered_records = indexed_records
    .into_iter()
    .filter(|record| {
      (pool_id == 0 || record.get_pool_id() == pool_id)
        && (account_id == 0 || record.get_account_id() == account_id)
        && (user_id.is_empty() || record.get_owner().contains(&user_id))
        && (status.is_empty() || record.get_status().to_string() == status)
        && (start_time == 0 || record.get_create_at() >= start_time)
        && (end_time == 0 || record.get_create_at() <= end_time)
    })
    .collect::<Vec<_>>();

  let total = filtered_records.len() as u32;
  let start = (page - 1) * page_size;

  StakingRewardPageResponse {
    total,
    page,
    page_size,
    records: filtered_records
      .iter()
      .rev()
      .skip(start as usize)
      .take(page_size as usize)
      .map(|record| StakingRewardVo::from_staking_reward(record))
      .collect(),
  }
}
