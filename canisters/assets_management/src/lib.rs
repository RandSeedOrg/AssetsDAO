/*!
 * AssetsDAO
 * https://github.com/RandSeedOrg/AssetsDAO
 * Copyright (C) 2025 RandSeedOrg
 * https://github.com/RandSeedOrg/AssetsDAO/blob/master/LICENSE
 */

use std::cell::RefCell;

use ic_stable_structures::{DefaultMemoryImpl, memory_manager::MemoryManager};

pub mod dao;
pub mod memory_ids;
pub mod transfer_address;

thread_local! {
  static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));
}

ic_cdk::export_candid!();
