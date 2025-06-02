use std::time::Duration;

use crate::scheduled_tasks::{
  nns_neuron_tasks::sync_nns_neuron_info_task, reward_distribution_task::distribute_staking_rewards,
  stake_error_recovery_task::recover_staking_account_errors, unstake_account_task::unstake_accounts,
};

#[ic_cdk::init]
fn init() {
  // Check stake rewards and stake maturity every minute
  let interval = Duration::from_secs(60);
  ic_cdk::println!("Starting a periodic task with interval {interval:?}");
  ic_cdk_timers::set_timer_interval(interval, || {
    // Perform stake reward distribution tasks every minute
    ic_cdk::futures::spawn(async { distribute_staking_rewards().await });
    // Perform unstaked account tasks every minute
    ic_cdk::futures::spawn(async { unstake_accounts().await });
    // Perform staked account error recovery tasks every minute
    ic_cdk::futures::spawn(async { recover_staking_account_errors().await });
  });

  // Sync NNS neuron info every hour
  let nns_neuron_sync_interval = Duration::from_secs(60 * 60); // 1 hour
  ic_cdk_timers::set_timer_interval(nns_neuron_sync_interval, || {
    // Perform NNS neuron sync tasks every hour
    ic_cdk::futures::spawn(async {
      sync_nns_neuron_info_task()
        .await
        .unwrap_or_else(|e| ic_cdk::println!("Failed to sync NNS neuron info: {}", e));
    });
  });

  ic_cdk_timers::set_timer(Duration::from_secs(1), || {
    ic_cdk::futures::spawn(async {
      // Initial sync of NNS neuron info
      sync_nns_neuron_info_task()
        .await
        .unwrap_or_else(|e| ic_cdk::println!("Failed to sync NNS neuron info: {}", e));
    });
  });
}

#[ic_cdk::post_upgrade]
fn post_upgrade() {
  init();
}
