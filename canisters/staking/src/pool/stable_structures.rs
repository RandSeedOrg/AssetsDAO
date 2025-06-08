use std::{borrow::Cow, str::FromStr};

use candid::{CandidType, Decode, Encode};
use ic_stable_structures::{storable::Bound, Storable};
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};
use types::{
  product::e8s_to_value,
  stable_structures::{new_entity_id, MetaData},
  Crypto, EntityId, TimestampNanos, E8S,
};

use crate::{
  account::{
    badge_utils::{add_staker_badge, remove_staker_badge},
    stable_structures::StakingAccount,
  },
  on_chain::address::generate_staking_pool_chain_address,
  pool_transaction_record::utils::record_stake_transaction,
};

use super::{transport_structures::StakingPoolAddDto, STAKING_POOL_ID, STAKING_POOL_MAP};

/// Staking pool data structure，Used to store financing amount、The amount of staked、Staking pool state
#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct StakingPool {
  /// Staking poolID
  pub id: Option<EntityId>,
  /// On-chain address of the stake pool
  pub address: Option<String>,
  /// Target financing amount，The maximum amount of stake that the stake pool can accommodate
  pub pool_size: Option<E8S>,
  /// The amount of the staked
  pub staked_amount: Option<E8S>,
  /// Locked staking pool capacity
  pub locked_size: Option<E8S>,
  /// Number of stakes in the stake pool
  pub staked_user_count: Option<u32>,
  /// The amount of funds occupied by the NNS neuron
  pub nns_neuron_occupies_funds: Option<E8S>,
  /// The amount of funds occupied by the jackpot
  pub jackpot_occupies_funds: Option<E8S>,
  /// Staking currency
  pub crypto: Option<Crypto>,
  /// Staking pool state
  pub status: Option<StakingPoolStatus>,
  /// The stake pool is inCEnd visibility
  pub client_visible: Option<bool>,
  /// Restricted configuration of staking pool
  pub limit_config: Option<LimitConfig>,
  /// Term configuration of stake pool
  pub term_config: Option<TermConfig>,
  /// Staking pool reward configuration
  pub reward_config: Option<RewardConfig>,
  /// Multiple reward configurations can be set
  pub reward_configs: Option<Vec<RewardConfig>>,
  /// Open staking time
  pub open_time: Option<TimestampNanos>,
  /// close staking time
  pub close_time: Option<TimestampNanos>,
  /// End time of the stake pool
  pub end_time: Option<TimestampNanos>,
  /// Meta information for staked accounts
  pub meta: Option<MetaData>,
}

impl StakingPool {
  pub fn from_add_dto(dto: &StakingPoolAddDto) -> Self {
    // Generate a stake poolID
    let id = STAKING_POOL_ID.with(|id_seq| new_entity_id(id_seq));

    // stake currency，Used by defaultICP
    let crypto = Crypto::from_str(&dto.crypto).unwrap_or(Crypto::ICP);

    // Staking pool reward currency，Used by defaultBONUS
    // let reward_crypto = RewardCrypto::from_str(&dto.reward_config.reward_crypto).unwrap_or(RewardCrypto::BONUS);

    // Generate a staking pool on-chain address
    let address = generate_staking_pool_chain_address(id);

    // Initialize the staking pool
    StakingPool {
      id: Some(id),
      address: Some(address),
      pool_size: Some(dto.pool_size),
      staked_amount: Some(0),
      locked_size: Some(0),
      staked_user_count: Some(0),
      crypto: Some(crypto),
      status: Some(StakingPoolStatus::Created),
      client_visible: Some(false),
      limit_config: Some(LimitConfig {
        min_stake_amount_per_user: Some(dto.limit_config.min_stake_amount_per_user),
        max_stake_amount_per_user: Some(dto.limit_config.max_stake_amount_per_user),
        step_amount: Some(dto.limit_config.step_amount),
      }),
      term_config: Some(TermConfig {
        min_term: Some(dto.term_config.min_term),
        max_term: Some(dto.term_config.max_term),
        min_early_unstake_days: Some(dto.term_config.min_early_unstake_days),
      }),
      reward_config: None,
      reward_configs: Some(
        dto
          .reward_configs
          .iter()
          .map(|config| RewardConfig {
            annualized_interest_rate: Some(config.annualized_interest_rate),
            daily_interest_rate: Some(config.daily_interest_rate),
            reward_crypto: Some(RewardCrypto::from_str(&config.reward_crypto).unwrap_or(RewardCrypto::BONUS)),
            min_stake_days: if config.min_stake_days == 0 { None } else { Some(config.min_stake_days) },
            max_stake_days: if config.max_stake_days == 0 { None } else { Some(config.max_stake_days) },
          })
          .collect(),
      ),
      open_time: None,
      close_time: None,
      end_time: None,
      meta: Some(MetaData::init_create_scene()),
      nns_neuron_occupies_funds: None,
      jackpot_occupies_funds: None,
    }
  }

  pub fn update(&mut self, dto: &StakingPoolAddDto) -> Option<String> {
    let status = self.get_status();
    if status != StakingPoolStatus::Created && status != StakingPoolStatus::Cancelled && status != StakingPoolStatus::Open {
      return Some("Only Created or Cancelled status pools can be updated".to_string());
    }

    if self.get_pool_size() > dto.pool_size && self.get_staked_amount() + self.get_locked_size() > dto.pool_size {
      return Some(format!(
        "Staking pool size is not enough, min can set size: {}, and new size: {}",
        e8s_to_value(self.get_staked_amount() + self.get_locked_size()).to_string(),
        e8s_to_value(dto.pool_size).to_string()
      ));
    }

    self.meta = Some(self.get_meta().update());
    self.pool_size = Some(dto.pool_size);
    self.limit_config = Some(LimitConfig {
      min_stake_amount_per_user: Some(dto.limit_config.min_stake_amount_per_user),
      max_stake_amount_per_user: Some(dto.limit_config.max_stake_amount_per_user),
      step_amount: Some(dto.limit_config.step_amount),
    });
    self.term_config = Some(TermConfig {
      min_term: Some(dto.term_config.min_term),
      max_term: Some(dto.term_config.max_term),
      min_early_unstake_days: Some(dto.term_config.min_early_unstake_days),
    });

    self.reward_config = None;

    self.reward_configs = Some(
      dto
        .reward_configs
        .iter()
        .map(|config| RewardConfig {
          annualized_interest_rate: Some(config.annualized_interest_rate),
          daily_interest_rate: Some(config.daily_interest_rate),
          reward_crypto: Some(RewardCrypto::from_str(&config.reward_crypto).unwrap_or(RewardCrypto::BONUS)),
          min_stake_days: if config.min_stake_days == 0 { None } else { Some(config.min_stake_days) },
          max_stake_days: if config.max_stake_days == 0 { None } else { Some(config.max_stake_days) },
        })
        .collect(),
    );

    self.crypto = Some(Crypto::from_str(&dto.crypto).unwrap_or(Crypto::ICP));

    // When updating, if the Staking pool state is Cancelled, then reset it to Created
    if self.get_status() == StakingPoolStatus::Cancelled {
      self.status = Some(StakingPoolStatus::Created);
    }

    self.meta = Some(self.get_meta().update());
    None
  }

  /// Verify whether the stake pool can currently accept stakes of the corresponding amount
  pub fn validate_and_lock_size(&mut self, staking_amount: E8S) -> Result<Self, String> {
    STAKING_POOL_MAP.with(|map| {
      let mut map = map.borrow_mut();
      let pool = map.get(&self.get_id());

      if pool.is_none() {
        return Err(format!("Staking pool with ID {} not found", self.get_id()));
      }

      let mut pool = pool.unwrap();

      let status = pool.get_status();
      let client_visible = pool.get_client_visible();

      // The staking pool is not visible on the client, or not in open state
      if status != StakingPoolStatus::Open || !client_visible {
        return Err(format!(
          "Staking pool is not open, current status: {:?}, and client visible is {}",
          status, client_visible
        ));
      }

      let remain_size = pool.get_pool_size() - pool.get_staked_amount() - pool.get_locked_size();

      // Try to Lock the staking pool amount
      if remain_size < staking_amount {
        return Err(format!(
          "Staking pool size is not enough, current remain size: {}, and staking amount: {}",
          e8s_to_value(remain_size).to_string(),
          e8s_to_value(staking_amount).to_string()
        ));
      }

      // Lock the stake pool amount
      pool.locked_size = Some(pool.get_locked_size() + staking_amount);
      // Update the stake pool information
      map.insert(pool.get_id(), pool.clone());

      Ok(pool.clone())
    })
  }

  // To restore locked stake pool amount
  pub fn restore_locked_size(&mut self, staking_amount: E8S) -> Result<Self, String> {
    STAKING_POOL_MAP.with(|map| {
      let mut map = map.borrow_mut();
      let pool = map.get(&self.get_id());

      if pool.is_none() {
        return Err(format!("Staking pool with ID {} not found", self.get_id()));
      }

      let mut pool = pool.unwrap();

      let new_locked_size = pool.get_locked_size() as i64 - staking_amount as i64;

      if new_locked_size < 0 {
        return Err(format!(
          "Staking pool locked size is not enough, current locked size: {}, and staking amount: {}",
          pool.get_locked_size(),
          staking_amount
        ));
      }

      // Release locked stake pool amount
      pool.locked_size = Some(new_locked_size as u64);
      // Update the stake pool information
      map.insert(pool.get_id(), pool.clone());

      Ok(pool.clone())
    })
  }

  // When there is a new stake account，update state of this staking pool
  pub fn add_stake_account(&mut self, account: &StakingAccount, user_already_in_stake_accounts: &Vec<StakingAccount>) -> Result<Self, String> {
    STAKING_POOL_MAP.with(|map| {
      let mut map = map.borrow_mut();
      let mut pool = match map.get(&self.get_id()) {
        Some(pool) => pool,
        None => {
          ic_cdk::println!("Staking pool with ID {} not found", self.get_id());
          return Err(format!("Staking pool with ID {} not found", self.get_id()));
        }
      };

      // Update the staked amount of the stake pool
      pool.staked_amount = Some(pool.get_staked_amount() + account.get_staked_amount());
      // Update the locked amount of the stake pool
      pool.locked_size = Some(pool.get_locked_size() - account.get_staked_amount());
      // Update the number of stakes in the stake pool
      if user_already_in_stake_accounts.is_empty() {
        pool.staked_user_count = Some(pool.get_staked_user_count() + 1);
      }

      pool.update_meta();

      map.insert(pool.get_id(), pool.clone());

      // Record the staking transaction of the staking pool
      record_stake_transaction(&account)?;

      let account_owner = account.get_owner();
      let account_id = account.get_id();

      ic_cdk::futures::spawn(async move {
        add_staker_badge(account_owner, account_id).await.unwrap_or_else(|e| {
          ic_cdk::println!("Failed to add staker badge: {:?}", e);
        });
      });

      Ok(pool)
    })
  }

  pub fn unstake_account(account: &StakingAccount, user_already_in_stake_accounts: &Vec<StakingAccount>) -> Result<Self, String> {
    STAKING_POOL_MAP.with(|map| {
      let mut map = map.borrow_mut();
      let pool_id = &account.get_pool_id();
      let mut pool = match map.get(pool_id) {
        Some(pool) => pool,
        None => {
          ic_cdk::println!("Staking pool with ID {} not found", pool_id);
          return Err(format!("Staking pool with ID {} not found", pool_id));
        }
      };

      // Update the staked amount of the stake pool
      pool.staked_amount = Some(pool.get_staked_amount() - account.get_staked_amount());

      if user_already_in_stake_accounts.len() == 1 {
        // If the user has only one staked account in the stake pool, then update the number of stakes in the stake pool
        pool.staked_user_count = Some(pool.get_staked_user_count() - 1);

        let account_owner = account.get_owner();
        ic_cdk::futures::spawn(async move {
          remove_staker_badge(account_owner).await.unwrap_or_else(|e| {
            ic_cdk::println!("Failed to remove staker badge: {:?}", e);
          });
        });
      }

      pool.update_meta();

      map.insert(pool.get_id(), pool.clone());

      Ok(pool)
    })
  }

  pub fn get_staked_user_count(&self) -> u32 {
    self.staked_user_count.unwrap_or_default()
  }

  pub fn update_meta(&mut self) {
    self.meta = Some(self.get_meta().update());
  }

  pub fn get_locked_size(&self) -> E8S {
    self.locked_size.unwrap_or_default()
  }

  pub fn get_id(&self) -> EntityId {
    self.id.unwrap_or_else(|| 1)
  }

  /// Get the on-chain address
  pub fn get_address(&self) -> String {
    self.address.clone().unwrap_or_default()
  }

  pub fn get_pool_size(&self) -> E8S {
    self.pool_size.unwrap_or_default()
  }

  pub fn get_staked_amount(&self) -> E8S {
    self.staked_amount.unwrap_or_default()
  }

  pub fn get_stake_user_count(&self) -> u32 {
    self.staked_user_count.unwrap_or_default()
  }

  pub fn get_crypto(&self) -> Crypto {
    self.crypto.clone().unwrap_or(Crypto::ICP)
  }

  pub fn get_status(&self) -> StakingPoolStatus {
    self.status.clone().unwrap_or(StakingPoolStatus::Created)
  }

  pub fn get_nns_neuron_occupies_funds(&self) -> E8S {
    self.nns_neuron_occupies_funds.unwrap_or_default()
  }

  fn set_nns_neuron_occupies_funds(&mut self, amount: E8S) {
    self.nns_neuron_occupies_funds = Some(amount);
  }

  pub fn add_nns_neuron_occupies_funds(&self, amount: E8S) -> Result<(), String> {
    STAKING_POOL_MAP.with(|map| {
      let mut map = map.borrow_mut();

      let pool = map.get(&self.get_id());

      if pool.is_none() {
        return Err("Staking pool not found".to_string());
      }

      let mut pool = pool.unwrap();

      let new_nns_neuron_occupies_funds = pool.get_nns_neuron_occupies_funds().checked_add(amount);

      if new_nns_neuron_occupies_funds.is_none() {
        return Err("Overflow when adding NNS neuron occupies funds".to_string());
      }

      pool.set_nns_neuron_occupies_funds(new_nns_neuron_occupies_funds.unwrap());

      map.insert(pool.get_id(), pool);

      Ok(())
    })
  }

  pub fn get_jackpot_occupies_funds(&self) -> E8S {
    self.jackpot_occupies_funds.unwrap_or_default()
  }

  pub fn get_available_funds(&self) -> Option<u64> {
    self
      .get_staked_amount()
      .checked_sub(self.get_nns_neuron_occupies_funds())?
      .checked_sub(self.get_jackpot_occupies_funds())
  }

  pub fn set_status(&mut self, status: StakingPoolStatus) -> Option<String> {
    let old_status = self.get_status();

    if old_status == StakingPoolStatus::Finished {
      // It's already the final state, can't change the state
      ic_cdk::println!("Attempt to change status from {:?} to {:?} is not allowed.", old_status, status);
      return Some("Cannot change status from Finished".to_string());
    }

    match status.clone() {
      StakingPoolStatus::Open => {
        if old_status != StakingPoolStatus::Created && old_status != StakingPoolStatus::Closed {
          ic_cdk::println!("Attempt to open pool from {:?} to {:?} is not allowed.", old_status, status);
          return Some("Only Created or Closed pools can be opened".to_string());
        }

        self.open_time = Some(ic_cdk::api::time());
      }
      StakingPoolStatus::Closed => {
        if old_status != StakingPoolStatus::Open {
          ic_cdk::println!("Attempt to close pool from {:?} to {:?} is not allowed.", old_status, status);
          return Some("Only Open pools can be closed".to_string());
        }

        self.close_time = Some(ic_cdk::api::time());
      }
      StakingPoolStatus::Finished => {
        if old_status != StakingPoolStatus::Closed {
          ic_cdk::println!("Attempt to finish pool from {:?} to {:?} is not allowed.", old_status, status);
          return Some("Only Closed pools can be finished".to_string());
        }
        //
        self.end_time = Some(ic_cdk::api::time());
      }
      StakingPoolStatus::Cancelled => {
        if old_status != StakingPoolStatus::Created {
          ic_cdk::println!("Attempt to cancel pool from {:?} to {:?} is not allowed.", old_status, status);
          return Some(format!("Only Created pools can be Cancelled."));
        }

        // Set the end time to the current time
        self.end_time = Some(ic_cdk::api::time());
      }
      StakingPoolStatus::Created => {
        if old_status != StakingPoolStatus::Cancelled {
          ic_cdk::println!("Attempt to set pool status back to Created from {:?} is not allowed.", old_status);
          return Some("Only Cancelled pools can be reset to Created".to_string());
        }
      }
    }

    self.status = Some(status.clone());
    self.meta = Some(self.get_meta().update());

    None
  }

  pub fn get_term_config(&self) -> TermConfig {
    self.term_config.clone().unwrap_or_default()
  }

  pub fn get_client_visible(&self) -> bool {
    self.client_visible.unwrap_or(false)
  }

  pub fn is_client_visible(&self) -> bool {
    self.get_client_visible()
  }

  pub fn set_client_visible(&mut self, visible: bool) -> Option<String> {
    let status = self.get_status();
    if status == StakingPoolStatus::Created || status == StakingPoolStatus::Cancelled {
      return Some("Cannot change visibility for Finished or Cancelled pools".to_string());
    }
    self.client_visible = Some(visible);
    self.meta = Some(self.get_meta().update());
    None
  }

  pub fn get_reward_config(&self) -> RewardConfig {
    self.reward_config.clone().unwrap_or_default()
  }

  pub fn get_reward_configs(&self) -> Cow<Vec<RewardConfig>> {
    if let Some(configs) = &self.reward_configs {
      return Cow::Borrowed(configs);
    } else {
      Cow::Owned(vec![self.get_reward_config()])
    }
  }

  pub fn get_limit_config(&self) -> LimitConfig {
    self.limit_config.clone().unwrap_or_default()
  }

  pub fn get_open_time(&self) -> TimestampNanos {
    self.open_time.unwrap_or_default()
  }

  pub fn get_clone_time(&self) -> TimestampNanos {
    self.close_time.unwrap_or_default()
  }

  pub fn get_end_time(&self) -> TimestampNanos {
    self.end_time.unwrap_or_default()
  }

  pub fn get_meta(&self) -> MetaData {
    self.meta.clone().unwrap_or(MetaData::init_create_scene())
  }
}

impl Storable for StakingPool {
  fn to_bytes(&self) -> Cow<[u8]> {
    Cow::Owned(Encode!(self).unwrap())
  }

  fn from_bytes(bytes: Cow<[u8]>) -> Self {
    Decode!(bytes.as_ref(), Self).unwrap()
  }

  const BOUND: Bound = Bound::Unbounded;
}

/// Staking pool reward configuration
#[derive(Debug, Clone, Serialize, Deserialize, CandidType, Default)]
pub struct RewardConfig {
  /// Annualized interest rate
  pub annualized_interest_rate: Option<E8S>,
  /// Daily interest rate
  pub daily_interest_rate: Option<E8S>,
  /// Staking pool reward currency
  pub reward_crypto: Option<RewardCrypto>,
  /// Minimum stake days
  pub min_stake_days: Option<u16>,
  /// Maximum stake days
  pub max_stake_days: Option<u16>,
}

impl RewardConfig {
  pub fn get_annualized_interest_rate(&self) -> E8S {
    self.annualized_interest_rate.unwrap_or_default()
  }

  pub fn get_daily_interest_rate(&self) -> E8S {
    self.daily_interest_rate.unwrap_or_default()
  }

  pub fn get_reward_crypto(&self) -> RewardCrypto {
    self.reward_crypto.clone().unwrap_or(RewardCrypto::BONUS)
  }

  pub fn get_min_stake_days(&self) -> u16 {
    self.min_stake_days.unwrap_or_default()
  }

  pub fn get_max_stake_days(&self) -> u16 {
    let max = self.max_stake_days.unwrap_or_default();

    if max == 0 {
      u16::MAX
    } else {
      max
    }
  }
}

/// Staking pool term configuration
#[derive(Debug, Clone, Serialize, Deserialize, CandidType, Default)]
pub struct TermConfig {
  /// Minimum term for stake pool，Unit is day
  pub min_term: Option<u16>,
  /// Staking pool term max limit，Unit is day
  pub max_term: Option<u16>,
  /// Minimum number of days to be deposable in advance
  pub min_early_unstake_days: Option<u16>,
}

impl TermConfig {
  pub fn get_min_term(&self) -> u16 {
    self.min_term.unwrap_or_default()
  }

  pub fn get_max_term(&self) -> u16 {
    self.max_term.unwrap_or_default()
  }

  pub fn get_min_early_unstake_days(&self) -> u16 {
    self.min_early_unstake_days.unwrap_or_default()
  }

  /// Verify the stake period
  pub fn validate_term(&self, term: u16) -> Result<(), String> {
    // Verify the stake period
    if self.get_min_term() > term {
      return Err(format!("Minimum staking term is {} days", self.get_min_term()));
    }

    // Verify the stake period
    if self.get_max_term() < term {
      return Err(format!("Maximum staking term is {} days", self.get_max_term()));
    }

    Ok(())
  }
}

#[derive(Debug, Clone, Serialize, Deserialize, CandidType, Default)]
pub struct LimitConfig {
  /// Minimum stake amount for stake pool，The unit isE8S
  pub min_stake_amount_per_user: Option<E8S>,
  /// The maximum amount of the stake pool，The unit isE8S
  pub max_stake_amount_per_user: Option<E8S>,
  /// Minimum stake amount for stake pool，The unit isE8S
  pub step_amount: Option<E8S>,
}

impl LimitConfig {
  pub fn get_min_stake_amount_per_user(&self) -> E8S {
    self.min_stake_amount_per_user.unwrap_or_default()
  }

  pub fn get_max_stake_amount_per_user(&self) -> E8S {
    self.max_stake_amount_per_user.unwrap_or_default()
  }

  pub fn get_step_amount(&self) -> E8S {
    self.step_amount.unwrap_or_default()
  }

  pub fn validate_stake_amount(&self, amount: E8S, current_user_in_stake_in_this_pool_accounts: &Vec<StakingAccount>) -> Result<(), String> {
    // Check the minimum stake amount
    if self.get_min_stake_amount_per_user() > amount {
      return Err(format!("Minimum staking amount is {} ICP", e8s_to_value(self.get_min_stake_amount_per_user())));
    }

    // Verify the maximum amount of stake
    if self.get_max_stake_amount_per_user() < amount {
      return Err(format!("Maximum staking amount is {} ICP", e8s_to_value(self.get_max_stake_amount_per_user())));
    }

    // Verify the stake amount step
    if (amount - self.get_min_stake_amount_per_user()) % self.get_step_amount() != 0 {
      return Err(format!(
        "The amount exceeding the minimum staking amount must be a multiple of {} ICP.",
        e8s_to_value(self.get_step_amount())
      ));
    }

    let already_staked_amount = current_user_in_stake_in_this_pool_accounts
      .iter()
      .map(|account| account.get_staked_amount())
      .sum::<E8S>();

    // Verify whether the sum of the amount that the user has staked in the Staking pool and the current staked amount exceeds the maximum staked amount
    if already_staked_amount + amount > self.get_max_stake_amount_per_user() {
      return Err(format!(
        "The total staking amount exceeds the maximum staking amount of {} ICP",
        e8s_to_value(self.get_max_stake_amount_per_user())
      ));
    }

    Ok(())
  }
}

/// Staking pool state
#[derive(EnumString, Display, Debug, Clone, Serialize, Deserialize, CandidType, PartialEq)]
pub enum StakingPoolStatus {
  /// Newly created, cannot be pledged, and is not visible to the client
  #[strum(serialize = "0")]
  Created,
  /// Open staking. At this time, if the staking pool is visible on the user side, staking can be performed.
  #[strum(serialize = "1")]
  Open,
  /// Close the pledge. At this time, the pledge pool cannot be pledged.
  #[strum(serialize = "2")]
  Closed,
  /// Finished indicates the final state of the pledge pool. Therefore, once this state is entered, the state cannot be changed.
  /// Finished: When the pledge pool has generated financing and the current remaining pledge amount is 0, the administrator can manually set it to Finished through the backend
  /// Only the pledge pool in the Closed state can be set to the Finished state
  #[strum(serialize = "3")]
  Finished,
  /// Cancelled: This can only be canceled if no one has ever staked in the staking pool. If you edit again after cancellation, it will enter the Created state.
  #[strum(serialize = "4")]
  Cancelled,
}

/// Rewards issued currency
#[derive(EnumString, Display, Debug, Clone, Serialize, Deserialize, CandidType)]
pub enum RewardCrypto {
  #[strum(serialize = "0")]
  BONUS,
}
