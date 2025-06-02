use std::cell::RefCell;

use ic_cdk::{query, update};
use ic_stable_structures::{memory_manager::MemoryId, StableBTreeMap};
use stable_structures::NnsStakeExecuteRecord;
use transport_structures::NnsStakeExecuteRecordVo;
use types::{assets_management::ProposalId, stable_structures::Memory, staking::StakingPoolId, E8S};
use utils::nns_update::refresh_nns_neuron_by_pool;

use crate::{
  guard_keys::get_stake_to_nns_guard_key,
  memory_ids::NNS_STAKING_EXECUTE_RECORD,
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
}

#[update]
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
