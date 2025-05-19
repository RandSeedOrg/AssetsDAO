use std::cell::RefCell;

use ic_stable_structures::{Cell, StableBTreeMap, memory_manager::MemoryId};
use stable_key::StakingAccountUserRewardDateIndexKey;
use stable_structures::StakingReward;
use types::{
  EntityId, UserId,
  entities::EntityIndex,
  stable_structures::Memory,
  staking::{StakingAccountId, StakingPoolId, StakingRewardId},
};

use crate::{
  MEMORY_MANAGER,
  memory_ids::{
    STAKING_ACCOUNT_REWARD_INDEX, STAKING_POOL_REWARD_INDEX, STAKING_REWARD, STAKING_REWARD_SEQ, STAKING_USER_ACCOUNT_REWARD_DATE_INDEX,
    STAKING_USER_REWARD_INDEX,
  },
};

pub mod crud;
pub mod stable_key;
pub mod stable_structures;
pub mod transport_structures;
pub mod utils;

thread_local! {
  /// stake rewards increaseIDGenerator
  pub static STAKING_REWARD_ID: RefCell<Cell<EntityId, Memory>> = RefCell::new(Cell::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(STAKING_REWARD_SEQ))), 0_u64).unwrap());

  /// Staking rewards original data
  pub static STAKING_REWARD_MAP: RefCell<StableBTreeMap<EntityId, StakingReward, Memory>> = RefCell::new(
    StableBTreeMap::init(
      MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(STAKING_REWARD))),
    )
  );

  /// Reward index received in staked account
  pub static STAKING_ACCOUNT_REWARD_INDEX_MAP: RefCell<StableBTreeMap<StakingAccountId, EntityIndex<StakingAccountId>, Memory>> = RefCell::new(
    StableBTreeMap::init(
      MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(STAKING_ACCOUNT_REWARD_INDEX))),
    )
  );

  /// User Reward Index
  pub static STAKING_USER_REWARD_INDEX_MAP: RefCell<StableBTreeMap<UserId, EntityIndex<UserId>, Memory>> = RefCell::new(
    StableBTreeMap::init(
      MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(STAKING_USER_REWARD_INDEX))),
    )
  );

  /// Staking pool reward index
  pub static STAKING_POOL_REWARD_INDEX_MAP: RefCell<StableBTreeMap<StakingPoolId, EntityIndex<StakingPoolId>, Memory>> = RefCell::new(
    StableBTreeMap::init(
      MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(STAKING_POOL_REWARD_INDEX))),
    )
  );

  /// User stake account date reward index
  pub static STAKING_USER_ACCOUNT_REWARD_DATE_INDEX_MAP: RefCell<StableBTreeMap<StakingAccountUserRewardDateIndexKey, StakingRewardId, Memory>> = RefCell::new(
    StableBTreeMap::init(
      MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(STAKING_USER_ACCOUNT_REWARD_DATE_INDEX))),
    )
  );
}
