use candid::CandidType;
use ic_ledger_types::BlockIndex;
use serde::{Deserialize, Serialize};
use types::{
  btree_set_entity_index::add_indexed_id,
  staking::{PoolTransactionRecordId, StakingPoolId},
  TimestampNanos, E8S,
};

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

    let new_record = records.add_record(amount, record_type.clone(), block_index, create_time)?;
    map.insert(pool_id, records.clone());

    STAKING_POOL_TRANSACTION_RECORD_TYPE_INDEX_MAP
      .with(|index_map| add_indexed_id(index_map, &RecordTypeIndexKey(pool_id, RecordTypeKey::from(record_type)), new_record.get_id()));

    Ok(new_record)
  })
}

fn find_record(pool_id: StakingPoolId, predicate: impl Fn(&RecordType) -> bool) -> Option<PoolTransactionRecord> {
  STAKING_POOL_TRANSACTION_RECORD_MAP.with(|map| {
    map.borrow().get(&pool_id).and_then(|records| {
      records
        .get_transaction_records()
        .values()
        .find(|record| predicate(&record.get_record_type()))
        .cloned()
    })
  })
}

pub fn get_virtual_balance(pool_id: StakingPoolId) -> E8S {
  STAKING_POOL_TRANSACTION_RECORD_MAP
    .with(|map| {
      map
        .borrow()
        .get(&pool_id)
        .and_then(|records| records.get_newest_transaction_record())
        .map(|record| record.get_balance())
    })
    .unwrap_or_default()
}

pub fn has_unstake_transaction(pool_id: StakingPoolId, account_id: u64) -> bool {
  find_record(pool_id, |record_type| matches!(record_type, RecordType::Unstaking(id) if *id == account_id)).is_some()
}

pub fn get_latest_nns_unstake_block_index(pool_id: StakingPoolId) -> Option<BlockIndex> {
  STAKING_POOL_TRANSACTION_RECORD_MAP.with(|map| {
    map.borrow().get(&pool_id).and_then(|records| {
      records
        .get_transaction_records()
        .values()
        .rev()
        .find(|record| matches!(record.get_record_type(), RecordType::NNSNeuronUnstake { .. }))
        .and_then(|record| record.get_block_index())
    })
  })
}

pub fn record_reconciliation_transaction(
  pool_id: StakingPoolId,
  amount: E8S,
  block_index: BlockIndex,
  create_time: TimestampNanos,
) -> Result<(), String> {
  if amount == 0 {
    return Ok(());
  }

  let amount = i64::try_from(amount).map_err(|_| "Reconciliation amount exceeds signed transaction range".to_string())?;
  record_transaction(pool_id, &RecordType::Reconciliation, amount, block_index, create_time).map(|_| ())
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
  let released_amount = i64::try_from(account.get_released_amount()).map_err(|_| "Release exceeds signed record range")?;
  let penalty_amount = i64::try_from(account.get_penalty_amount()).map_err(|_| "Penalty exceeds signed record range")?;

  let pool_id = account.get_pool_id();
  let unstaking_transaction = find_record(pool_id, |record_type| matches!(record_type, RecordType::Unstaking(id) if *id == account.get_id()))
    .map(Ok)
    .unwrap_or_else(|| {
      record_transaction(
        pool_id,
        &RecordType::Unstaking(account.get_id()),
        -released_amount,
        account.get_release_onchain_tx_id(),
        release_time,
      )
    })?;
  let unstaking_record_id: PoolTransactionRecordId = unstaking_transaction.get_id();
  let recorded_fee = STAKING_POOL_TRANSACTION_RECORD_MAP.with(|map| {
    map
      .borrow()
      .get(&pool_id)
      .map(|records| {
        records
          .get_transaction_records()
          .values()
          .filter(|record| matches!(record.get_record_type(), RecordType::Fee(id) if id == unstaking_record_id))
          .try_fold(0u64, |sum, record| {
            if record.get_amount() >= 0 {
              return Err("Invalid positive unstake fee".to_string());
            }
            sum
              .checked_add(record.get_amount().unsigned_abs())
              .ok_or_else(|| "Unstake fee overflow".to_string())
          })
      })
      .unwrap_or(Ok(0))
  })?;
  let expected_fee: u64 = if account.get_released_amount() > 0 { 20_000 } else { 0 };
  let missing_fee = expected_fee
    .checked_sub(recorded_fee)
    .ok_or("Unstake fee exceeds the actual transfer cost")?;
  if missing_fee > 0 {
    record_transaction(
      pool_id,
      &RecordType::Fee(unstaking_record_id),
      -(missing_fee as i64),
      account.get_release_onchain_tx_id(),
      release_time,
    )?;
  }

  if account.get_penalty_amount() > 0 {
    // Record the penalty transaction of the staking pool
    if find_record(
      pool_id,
      |record_type| matches!(record_type, RecordType::EarlyUnstakePenalty(id) if *id == account.get_id()),
    )
    .is_none()
    {
      record_transaction(
        pool_id,
        &RecordType::EarlyUnstakePenalty(account.get_id()),
        -penalty_amount,
        account.get_release_onchain_tx_id(),
        release_time,
      )?;
    }
  }

  Ok(())
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct ReleaseRepair {
  pub account_id: u64,
  pub block: u64,
  pub principal: u64,
  pub transfer_amount: u64,
  pub ledger_fee: u64,
  pub timestamp: u64,
}

fn missing_release_debit(records: &PoolTransactionRecords, repair: &ReleaseRepair) -> Result<(Option<u64>, u64, u64), String> {
  let principal = i64::try_from(repair.principal).map_err(|_| "Principal exceeds signed record range")?;
  let expected_fee = repair
    .transfer_amount
    .checked_sub(repair.principal)
    .and_then(|extra| extra.checked_add(repair.ledger_fee))
    .ok_or("Invalid verified release cost")?;
  let matches: Vec<_> = records
    .get_transaction_records()
    .values()
    .filter(|r| matches!(r.get_record_type(),RecordType::Unstaking(id) if id==repair.account_id))
    .collect();
  if matches.len() > 1 {
    return Err("Duplicate virtual unstaking records".into());
  }
  let Some(unstake) = matches.first() else {
    return Ok((None, repair.principal, expected_fee));
  };
  if unstake.get_amount() != -principal || unstake.get_block_index() != Some(repair.block) {
    return Err(format!("Unstaking record conflicts with verified block for account {}", repair.account_id));
  }
  let fee = records
    .get_transaction_records()
    .values()
    .filter(|r| matches!(r.get_record_type(),RecordType::Fee(id) if id==unstake.get_id()))
    .try_fold(0u64, |total, r| {
      if r.get_amount() >= 0 || r.get_block_index() != Some(repair.block) {
        return Err("Invalid historical release fee".to_string());
      }
      total
        .checked_add(r.get_amount().unsigned_abs())
        .ok_or_else(|| "Historical fee overflow".into())
    })?;
  Ok((
    Some(unstake.get_id()),
    0,
    expected_fee.checked_sub(fee).ok_or("Recorded fee exceeds verified cost")?,
  ))
}

/// Build all changes before touching stable state. The audit includes every confirmed
/// pending release in this pool, not just the currently processed account.
fn build_release_repairs(
  mut records: PoolTransactionRecords,
  repairs: &[ReleaseRepair],
  balance: u64,
  now: u64,
) -> Result<(PoolTransactionRecords, Vec<PoolTransactionRecord>), String> {
  let mut seen = std::collections::BTreeSet::new();
  let mut pending = 0u64;
  for repair in repairs {
    if !seen.insert(repair.account_id) {
      return Err("Duplicate repair account".into());
    }
    let (_, principal, fee) = missing_release_debit(&records, repair)?;
    pending = pending
      .checked_add(principal)
      .and_then(|n| n.checked_add(fee))
      .ok_or("Release audit overflow")?;
  }
  let target = balance.checked_add(pending).ok_or("Reconciliation overflow")?;
  let virtual_balance = records.get_newest_transaction_record().map_or(0, |r| r.get_balance());
  let difference = target
    .checked_sub(virtual_balance)
    .ok_or_else(|| format!("Unexplained virtual balance excess: virtual={virtual_balance}, actual={balance}, pending={pending}"))?;
  let mut added = Vec::new();
  if difference > 0 {
    added.push(records.add_record(
      i64::try_from(difference).map_err(|_| "Reconciliation exceeds signed range")?,
      RecordType::Reconciliation,
      0,
      now,
    )?);
  }
  for repair in repairs {
    let (id, principal, fee) = missing_release_debit(&records, repair)?;
    let id = match id {
      Some(id) => id,
      None => {
        let record = records.add_record(
          -i64::try_from(principal).map_err(|_| "Principal exceeds signed range")?,
          RecordType::Unstaking(repair.account_id),
          repair.block,
          repair.timestamp,
        )?;
        let id = record.get_id();
        added.push(record);
        id
      }
    };
    if fee > 0 {
      added.push(records.add_record(
        -i64::try_from(fee).map_err(|_| "Fee exceeds signed range")?,
        RecordType::Fee(id),
        repair.block,
        repair.timestamp,
      )?);
    }
  }
  Ok((records, added))
}

pub fn reconcile_release_repairs(pool_id: u64, repairs: &[ReleaseRepair], balance: u64, now: u64) -> Result<(), String> {
  let records = STAKING_POOL_TRANSACTION_RECORD_MAP
    .with(|m| m.borrow().get(&pool_id))
    .unwrap_or_else(|| PoolTransactionRecords::new_empty(pool_id));
  let (records, added) = build_release_repairs(records, repairs, balance, now)?;
  STAKING_POOL_TRANSACTION_RECORD_MAP.with(|m| m.borrow_mut().insert(pool_id, records));
  STAKING_POOL_TRANSACTION_RECORD_TYPE_INDEX_MAP.with(|m| {
    for record in added {
      add_indexed_id(m, &RecordTypeIndexKey(pool_id, RecordTypeKey::from(&record.get_record_type())), record.get_id());
    }
  });
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
  disburse_time: TimestampNanos,
) -> Result<(), String> {
  let nns_unstake_transaction = record_transaction(
    pool_id,
    &RecordType::NNSNeuronUnstake { neuron_id },
    (amount + 10_000) as i64,
    block_index,
    disburse_time,
  )?;

  record_transaction(pool_id, &RecordType::Fee(nns_unstake_transaction.get_id()), -10_000, block_index, disburse_time)?;
  Ok(())
}

#[cfg(test)]
mod repair_tests {
  use super::*;
  fn repair() -> ReleaseRepair {
    ReleaseRepair {
      account_id: 147,
      block: 100,
      principal: 100_000_000,
      transfer_amount: 100_010_000,
      ledger_fee: 10_000,
      timestamp: 1,
    }
  }
  fn funded(balance: i64) -> PoolTransactionRecords {
    let mut r = PoolTransactionRecords::new_empty(3);
    r.add_record(balance, RecordType::Reconciliation, 0, 1).unwrap();
    r
  }
  #[test]
  fn orphan_reconciles_then_records_full_debit_exactly_once() {
    let (r, added) = build_release_repairs(funded(80_260_000), &[repair()], 317_214, 2).unwrap();
    assert_eq!(added[0].get_amount(), 20_077_214);
    assert_eq!(r.get_newest_transaction_record().unwrap().get_balance(), 317_214);
    let (_, added) = build_release_repairs(r, &[repair()], 317_214, 3).unwrap();
    assert!(added.is_empty());
  }
  #[test]
  fn fixes_missing_or_partial_fee_without_rewriting_principal() {
    for paid in [0i64, 10_000, 20_000] {
      let mut r = funded(100_337_214);
      let id = r.add_record(-100_000_000, RecordType::Unstaking(147), 100, 1).unwrap().get_id();
      if paid > 0 {
        r.add_record(-paid, RecordType::Fee(id), 100, 1).unwrap();
      }
      let (r, added) = build_release_repairs(r, &[repair()], 317_214, 2).unwrap();
      assert_eq!(r.get_newest_transaction_record().unwrap().get_balance(), 317_214);
      assert_eq!(added.len(), usize::from(paid < 20_000));
      assert!(added.iter().all(|r| matches!(r.get_record_type(), RecordType::Fee(_))));
    }
  }
  #[test]
  fn refuses_unexplained_deficits_and_conflicting_blocks() {
    assert!(build_release_repairs(funded(100_337_215), &[repair()], 317_214, 2).is_err());
    let mut r = funded(100_337_214);
    r.add_record(-100_000_000, RecordType::Unstaking(147), 99, 1).unwrap();
    assert!(build_release_repairs(r, &[repair()], 317_214, 2).is_err());
  }
}
