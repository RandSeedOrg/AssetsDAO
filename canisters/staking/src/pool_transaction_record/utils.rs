use ic_ledger_types::BlockIndex;
use types::{btree_set_entity_index::add_indexed_id, staking::StakingPoolId, TimestampNanos};

use crate::{account::stable_structures::StakingAccount, nns::stable_structures::NnsStakeExecuteRecord};

use super::{
  stable_structures::{PoolTransactionRecord, PoolTransactionRecords, RecordType, RecordTypeIndexKey, RecordTypeKey},
  STAKING_POOL_TRANSACTION_RECORD_MAP, STAKING_POOL_TRANSACTION_RECORD_TYPE_INDEX_MAP,
};

/// Record a transaction for a staking pool
fn record_transaction(
  pool_id: StakingPoolId,
  record_type: &RecordType,
  amount: i64,
  block_index: BlockIndex,
  create_time: TimestampNanos,
) -> Result<PoolTransactionRecord, String> {
  STAKING_POOL_TRANSACTION_RECORD_MAP.with(|map| {
    let mut map = map.borrow_mut();
    let mut records = map.get(&pool_id).unwrap_or_else(|| PoolTransactionRecords::new_empty(pool_id));

    let new_record = records.add_record(amount, record_type.clone(), block_index, create_time);
    map.insert(pool_id, records.clone());

    STAKING_POOL_TRANSACTION_RECORD_TYPE_INDEX_MAP
      .with(|index_map| add_indexed_id(index_map, &RecordTypeIndexKey(pool_id, RecordTypeKey::from(record_type)), new_record.get_id()));

    Ok(new_record)
  })
}

pub fn record_stake_transaction(account: &StakingAccount) -> Result<(), String> {
  // Record the staking transaction of the staking pool
  let staking_transaction = record_transaction(
    account.get_pool_id(),
    &RecordType::Staking(account.get_id()),
    account.get_staked_amount() as i64,
    account.get_stake_account_to_pool_onchain_tx_id(),
    account.get_stake_time(),
  )?;
  // Record the pay center prepaid fee transaction of the staking pool
  record_transaction(
    account.get_pool_id(),
    &RecordType::PrepaidFee(staking_transaction.get_id()),
    20_000,
    account.get_stake_account_to_pool_onchain_tx_id(),
    account.get_stake_time(),
  )?;

  Ok(())
}

pub fn record_unstake_transaction(account: &StakingAccount) -> Result<(), String> {
  let release_time = account.get_release_time();

  let unstaking_transaction = record_transaction(
    account.get_pool_id(),
    &RecordType::Unstaking(account.get_id()),
    -(account.get_released_amount() as i64),
    account.get_release_onchain_tx_id(),
    release_time,
  )?;
  record_transaction(
    account.get_pool_id(),
    &RecordType::Fee(unstaking_transaction.get_id()),
    -10_000,
    account.get_release_onchain_tx_id(),
    release_time,
  )?;

  if account.get_penalty_amount() > 0 {
    // Record the penalty transaction of the staking pool
    record_transaction(
      account.get_pool_id(),
      &RecordType::EarlyUnstakePenalty(account.get_id()),
      -(account.get_penalty_amount() as i64),
      account.get_release_onchain_tx_id(),
      release_time,
    )?;
  }

  Ok(())
}

/// Record a transaction for the NNS neuron stake
pub fn record_stake_to_neuron_transaction(execute_record: &NnsStakeExecuteRecord) -> Result<(), String> {
  let execute_time = execute_record.get_updated_at();

  let nns_neuron_transaction = record_transaction(
    execute_record.get_pool_id(),
    &RecordType::NNSNeuronStake {
      neuron_id: execute_record.get_neuron_id(),
    },
    -(execute_record.get_amount() as i64),
    execute_record.get_transfer_block_index(),
    execute_time,
  )?;

  record_transaction(
    execute_record.get_pool_id(),
    &RecordType::Fee(nns_neuron_transaction.get_id()),
    -10_000,
    execute_record.get_transfer_block_index(),
    execute_time,
  )?;

  Ok(())
}

pub fn record_nns_unstake_transaction(
  pool_id: StakingPoolId,
  neuron_id: u64,
  amount: u64,
  block_index: BlockIndex,
  distribute_time: TimestampNanos,
) -> Result<PoolTransactionRecord, String> {
  record_transaction(pool_id, &RecordType::NNSNeuronUnstake { neuron_id }, amount as i64, block_index, distribute_time)
}
