use std::{cell::RefCell, collections::BTreeMap};

use ic_cdk::{query, update};
use ic_stable_structures::{memory_manager::MemoryId, StableBTreeMap};
use nns_governance_api::nns_governance_api::Neuron;
use stable_structures::NnsStakeExecuteRecord;
use system_configs_macro::has_permission_result;
use transport_structures::NnsStakeExecuteRecordVo;
use types::{assets_management::ProposalId, stable_structures::Memory, staking::StakingPoolId, E8S};
use utils::{nns_query::sync_nns_neuron, nns_update::refresh_nns_neuron_by_pool};

use crate::{
  guard_keys::get_stake_to_nns_guard_key,
  memory_ids::{NNS_STAKING_EXECUTE_RECORD, NNS_STAKING_POOL_NEURON_ID},
  on_chain::{address::generate_staking_pool_neuron_account, transfer::transfer_from_staking_pool_to_nns_neuron},
  parallel_guard::EntryGuard,
  pool::crud_utils::query_staking_pool_by_id,
  pool_transaction_record::utils::record_stake_to_neuron_transaction,
  MEMORY_MANAGER,
};

pub mod stable_structures;
pub mod transport_structures;
pub mod utils;

thread_local! {
  /// NNS staking execute record stable storage
  pub static NNS_STAKING_EXECUTE_RECORD_MAP: RefCell<StableBTreeMap<ProposalId, NnsStakeExecuteRecord, Memory>> = RefCell::new(
    StableBTreeMap::init(
      MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(NNS_STAKING_EXECUTE_RECORD))),
    )
  );

  pub static NNS_NEURON_MAP: RefCell<BTreeMap<StakingPoolId, Neuron>> = RefCell::new(
    BTreeMap::new()
  );

  pub static NNS_STAKING_POOL_NEURON_ID_MAP: RefCell<StableBTreeMap<StakingPoolId, u64, Memory>> = RefCell::new(
    StableBTreeMap::init(
      MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(NNS_STAKING_POOL_NEURON_ID))),
    )
  );
}

#[update]
#[has_permission_result("staking::nns::stake_to_nns_neuron")]
async fn stake_to_nns_neuron(proposal_id: ProposalId, pool_id: StakingPoolId, amount: E8S) -> Result<u64, String> {
  let mut execute_record = NnsStakeExecuteRecord::init_with(proposal_id, pool_id, amount);

  let _entry_guard = EntryGuard::new(get_stake_to_nns_guard_key(proposal_id));
  let pool = query_staking_pool_by_id(pool_id)?;
  let neuron_account = generate_staking_pool_neuron_account(pool_id);

  // Check if the execute record already has a transfer block index then skip transfer
  if execute_record.pool_to_neuron_transfer_block_index.is_none() {
    if amount < 1 {
      return Err("Amount must be greater than or equal 1 ICP".to_string());
    }

    let pool_available_funds = pool.get_available_funds().unwrap_or_default();

    if amount >= pool_available_funds {
      ic_cdk::println!(
        "Staking amount {} is greater than or equal to the pool's staked amount {}",
        amount,
        pool_available_funds
      );
      return Err("The staking pool does not have enough available funds.".to_string());
    }

    let nns_transfer_block_index = transfer_from_staking_pool_to_nns_neuron(pool_id, amount).await?;

    ic_cdk::println!(
      "Transferred {} ICP from staking pool {} to NNS neuron account: {}, block index: {}",
      amount,
      pool_id,
      neuron_account.to_hex(),
      nns_transfer_block_index
    );

    // Update the execute record with the transfer block index
    execute_record.set_pool_to_neuron_transfer_block_index(nns_transfer_block_index);
    execute_record.update_to_stable();

    // There should be no error here unless there is a serious flaw in the overall design
    pool.add_nns_neuron_occupies_funds(amount)?;
  }

  // update neuron status
  match refresh_nns_neuron_by_pool(pool_id).await {
    Ok(neuron_id) => {
      execute_record.update_to_success(neuron_id);
      record_stake_to_neuron_transaction(&execute_record)?;

      NNS_STAKING_POOL_NEURON_ID_MAP.with(|map| {
        let mut map = map.borrow_mut();
        map.insert(pool_id, neuron_id);
      });

      // Sync the NNS neuron to the local cache
      ic_cdk::futures::spawn(async move {
        sync_nns_neuron(pool_id).await.unwrap_or_else(|e| {
          ic_cdk::println!("Failed to sync NNS neuron for pool {}: {}", pool_id, e);
        });
      });

      Ok(neuron_id)
    }
    Err(e) => {
      ic_cdk::println!("Failed to refresh NNS neuron: {}", e);

      execute_record.update_to_error(neuron_account.to_hex(), e.to_string());
      Err(format!("Failed to refresh NNS neuron: {}", e))
    }
  }
}

#[query]
pub fn get_nns_staking_execute_record(proposal_id: ProposalId) -> Option<NnsStakeExecuteRecordVo> {
  NNS_STAKING_EXECUTE_RECORD_MAP.with(|map| {
    let record = map.borrow().get(&proposal_id)?;
    Some(NnsStakeExecuteRecordVo::from(record))
  })
}

#[query]
pub fn get_nns_neuron_by_pool_id(pool_id: StakingPoolId) -> Option<Neuron> {
  if let Some(neuron) = NNS_NEURON_MAP.with(|map| map.borrow().get(&pool_id).cloned()) {
    Some(neuron)
  } else {
    None
  }
}

#[update]
pub async fn sync_nns_neuron_by_pool_id(pool_id: StakingPoolId) -> Result<(), String> {
  refresh_nns_neuron_by_pool(pool_id).await?;
  sync_nns_neuron(pool_id).await
}

#[update]
#[has_permission_result("staking::nns::update_nns_neuron")]
async fn add_nns_hotkey(pool_id: StakingPoolId, hotkey: String) -> Result<(), String> {
  let neuron = NNS_NEURON_MAP.with(|map| {
    let map = map.borrow();
    map.get(&pool_id).cloned()
  });

  if let Some(neuron) = neuron {
    let neuron_id = neuron.id.map_or_else(
      || {
        ic_cdk::println!("Neuron ID is not set for pool ID: {}", pool_id);
        Err("Neuron ID is not set".to_string())
      },
      |id| Ok(id),
    )?;

    utils::nns_update::add_hot_key(neuron_id.id, hotkey).await
  } else {
    Err(format!("No NNS neuron found for pool ID: {}", pool_id))
  }
}

#[update]
#[has_permission_result("staking::nns::update_nns_neuron")]
async fn remove_nns_hotkey(pool_id: StakingPoolId, hotkey: String) -> Result<(), String> {
  let neuron = NNS_NEURON_MAP.with(|map| {
    let map = map.borrow();
    map.get(&pool_id).cloned()
  });
  if let Some(neuron) = neuron {
    let neuron_id = neuron.id.map_or_else(
      || {
        ic_cdk::println!("Neuron ID is not set for pool ID: {}", pool_id);
        Err("Neuron ID is not set".to_string())
      },
      |id| Ok(id),
    )?;

    utils::nns_update::remove_hot_key(neuron_id.id, hotkey).await
  } else {
    Err(format!("No NNS neuron found for pool ID: {}", pool_id))
  }
}

#[update]
#[has_permission_result("staking::nns::update_nns_neuron")]
async fn increase_nns_dissolve_delay(pool_id: StakingPoolId, additional_delay_seconds: u32) -> Result<(), String> {
  let neuron = NNS_NEURON_MAP.with(|map| {
    let map = map.borrow();
    map.get(&pool_id).cloned()
  });

  if let Some(neuron) = neuron {
    let neuron_id = neuron.id.map_or_else(
      || {
        ic_cdk::println!("Neuron ID is not set for pool ID: {}", pool_id);
        Err("Neuron ID is not set".to_string())
      },
      |id| Ok(id),
    )?;

    utils::nns_update::increase_dissolve_delay(neuron_id.id, additional_delay_seconds).await
  } else {
    Err(format!("No NNS neuron found for pool ID: {}", pool_id))
  }
}
