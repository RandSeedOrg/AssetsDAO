extern crate system_configs_macro;
system_configs_macro::system_configs!();
parallel_guard_macro::parallel_guard!();

use std::cell::RefCell;

use ic_stable_structures::{memory_manager::MemoryManager, DefaultMemoryImpl};
use types::EntityId;

pub mod account;
pub mod event_log;
pub mod guard_keys;
mod init;
pub mod memory_ids;
pub mod nns;
pub mod on_chain;
pub mod pool;
pub mod pool_transaction_record;
pub mod reward;
pub mod scheduled_tasks;
pub mod subscription;

thread_local! {
  static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));
}

use types::staking::StakingAccountId;
use types::staking::StakingPoolId;

use pool::client_transport_structures::ClientStakingPoolVo;
use pool::transport_structures::StakingPoolUpdateDto;
use types::sys::config::SystemConfig;
use types::sys::config::UserRolePermissionVo;

use account::client_transport_structures::StakeDto;
use account::transport_structures::StakingAccountPageRequest;
use account::transport_structures::StakingAccountPageResponse;
use account::transport_structures::StakingAccountVo;
use candid::Principal;
use pool::transport_structures::StakingPoolAddDto;
use pool::transport_structures::StakingPoolVo;

use account::client_transport_structures::EarlyUnstakePreCheckVo;
use event_log::transport_structures::StakingEventLogPageRequest;
use event_log::transport_structures::StakingEventLogPageResponse;
use nns::transport_structures::NnsStakeExecuteRecordVo;
use nns_governance_api::nns_governance_api::Neuron;
use pool_transaction_record::stable_structures::PoolTransactionRecord;
use pool_transaction_record::transport_structures::PoolTransactionQueryParams;
use reward::transport_structures::StakingRewardPageRequest;
use reward::transport_structures::StakingRewardPageResponse;
use subscription::transport_structures::StakingSubscribeAddDto;
use subscription::transport_structures::SubscriptionRequest;
use subscription::transport_structures::SubscriptionResponse;
use types::assets_management::ProposalId;
use types::pagination::PageRequest;
use types::pagination::PageResponse;
use types::E8S;

ic_cdk::export_candid!();
