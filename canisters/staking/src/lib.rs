extern crate system_configs_macro;
system_configs_macro::system_configs!();
parallel_guard_macro::parallel_guard!();

use std::cell::RefCell;

use ic_stable_structures::{memory_manager::MemoryManager, DefaultMemoryImpl};
use types::EntityId;

pub mod account;
pub mod pool;
pub mod event_log;
pub mod reward;
pub mod memory_ids;
pub mod on_chain;
pub mod scheduled_tasks;
mod init;
pub mod subscription;

thread_local! {
  static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));
}

use types::staking::StakingAccountId;
use types::staking::StakingPoolId;

use types::sys::config::SystemConfig;
use types::sys::config::UserRolePermissionVo;
use pool::transport_structures::StakingPoolUpdateDto;
use pool::client_transport_structures::ClientStakingPoolVo;

use pool::transport_structures::StakingPoolAddDto;
use pool::transport_structures::StakingPoolVo;
use candid::Principal;
use account::transport_structures::StakingAccountPageRequest;
use account::transport_structures::StakingAccountPageResponse;
use account::client_transport_structures::StakeDto;
use account::transport_structures::StakingAccountVo;

use reward::transport_structures::StakingRewardPageRequest;
use reward::transport_structures::StakingRewardPageResponse;
use event_log::transport_structures::StakingEventLogPageResponse;
use event_log::transport_structures::StakingEventLogPageRequest;
use subscription::transport_structures::StakingSubscribeAddDto;
use subscription::transport_structures::SubscriptionResponse;
use subscription::transport_structures::SubscriptionRequest;
use account::client_transport_structures::EarlyUnstakePreCheckVo;

ic_cdk::export_candid!();