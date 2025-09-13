/*!
 * AssetsDAO
 * https://github.com/RandSeedOrg/AssetsDAO
 * Copyright (C) 2025 RandSeedOrg
 * https://github.com/RandSeedOrg/AssetsDAO/blob/master/LICENSE
 */

extern crate system_configs_macro;
system_configs_macro::system_configs!();
parallel_guard_macro::parallel_guard!();
identity_mapping_macro::identity_mapping!();

use std::cell::RefCell;

use ic_stable_structures::{memory_manager::MemoryManager, DefaultMemoryImpl};

pub mod dao;
pub mod guard_keys;
pub mod memory_ids;
pub mod transfer_address;
pub mod utils;

thread_local! {
  static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));
}

use candid::Principal;
use dao::proposal::transport_structures::AddProposalDto;
use dao::proposal::transport_structures::ProposalListParams;
use dao::proposal::transport_structures::ProposalVo;
use dao::proposal::transport_structures::UpdateProposalDto;
use transfer_address::transfer_structures::TransferAddressVo;
use types::assets_management::ProposalId;
use types::pagination::PageRequest;
use types::pagination::PageResponse;
use types::sys::config::SystemConfig;
use types::sys::config::UserRolePermissionVo;

ic_cdk::export_candid!();
