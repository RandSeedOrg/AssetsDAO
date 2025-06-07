use std::{borrow::Cow, collections::BTreeMap};

use candid::{CandidType, Decode, Encode, Principal};
use ic_ledger_types::BlockIndex;
use ic_stable_structures::{storable::Bound, Storable};
use serde::{Deserialize, Serialize};
use types::{
  pagination::PageResponse,
  product::ProductId,
  staking::{PoolTransactionRecordId, StakingAccountId, StakingPoolId},
  EntityId, TimestampNanos, E8S,
};

#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct PoolTransactionRecords {
  /// Transaction records of the staking pool, sorted by transaction time
  pub pool_id: Option<StakingPoolId>,
  /// Maximum transaction record id of the staking pool
  pub max_record_id: Option<PoolTransactionRecordId>,
  /// Transaction records of the staking pool
  pub transaction_records: Option<BTreeMap<PoolTransactionRecordId, PoolTransactionRecord>>,
}

impl PoolTransactionRecords {
  pub fn new_empty(pool_id: StakingPoolId) -> Self {
    PoolTransactionRecords {
      pool_id: Some(pool_id),
      max_record_id: Some(0),
      transaction_records: Some(BTreeMap::new()),
    }
  }

  pub fn get_transaction_records(&self) -> &BTreeMap<PoolTransactionRecordId, PoolTransactionRecord> {
    self.transaction_records.as_ref().unwrap()
  }

  pub fn get_pool_id(&self) -> StakingPoolId {
    self.pool_id.unwrap()
  }

  pub fn get_max_record_id(&self) -> PoolTransactionRecordId {
    self.max_record_id.unwrap()
  }

  pub fn get_newest_transaction_record(&self) -> Option<PoolTransactionRecord> {
    self
      .transaction_records
      .as_ref()
      .and_then(|records| records.get(&self.get_max_record_id()))
      .cloned()
  }

  pub fn add_record(&mut self, amount: i64, record_type: RecordType, block_index: BlockIndex, create_time: TimestampNanos) -> PoolTransactionRecord {
    let newest_record = self.get_newest_transaction_record();

    let new_record = if let Some(record) = newest_record {
      record.next_record(amount, record_type, block_index, create_time)
    } else {
      assert!(amount > 0, "The first transaction record must be a deposit");

      PoolTransactionRecord {
        id: Some(1),
        amount: Some(amount),
        balance: Some(amount as E8S),
        record_type: Some(record_type),
        block_index: Some(block_index),
        created_at: Some(create_time),
      }
    };

    let mut transaction_records = self.transaction_records.clone().unwrap();
    self.max_record_id = Some(new_record.get_id());
    transaction_records.insert(new_record.get_id(), new_record.clone());
    self.transaction_records = Some(transaction_records);

    new_record
  }

  pub fn get_page(&self, page: u32, page_size: u32) -> PageResponse<PoolTransactionRecord> {
    let records = self.get_transaction_records();
    let total_count = self.get_max_record_id() as u32;
    let start = (page - 1) * page_size;

    let paginated_records: Vec<PoolTransactionRecord> = records.values().rev().skip(start as usize).take(page_size as usize).cloned().collect();

    PageResponse::new(page, page_size, total_count, paginated_records)
  }

  pub fn get_page_by_ids(&self, page: u32, page_size: u32, ids: Vec<PoolTransactionRecordId>) -> PageResponse<PoolTransactionRecord> {
    let records = self.get_transaction_records();
    let total_count = ids.len() as u32;
    let start = (page - 1) * page_size;

    let paginated_records: Vec<PoolTransactionRecord> = ids
      .into_iter()
      .rev()
      .skip(start as usize)
      .take(page_size as usize)
      .filter_map(|id| records.get(&id).cloned())
      .collect();

    PageResponse::new(page, page_size, total_count, paginated_records)
  }
}

/// Virtual transaction records of the pledge pool can be reconciled with the on-chain
#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct PoolTransactionRecord {
  pub id: Option<PoolTransactionRecordId>,
  /// Transaction amount, positive number is for deposit, negative number is for withdrawal
  pub amount: Option<i64>,
  /// Account balance after this transaction
  pub balance: Option<E8S>,
  /// Type of transaction record
  pub record_type: Option<RecordType>,
  pub block_index: Option<BlockIndex>,
  pub created_at: Option<u64>,
}

impl PoolTransactionRecord {
  pub fn next_record(&self, amount: i64, record_type: RecordType, block_index: BlockIndex, create_time: TimestampNanos) -> PoolTransactionRecord {
    PoolTransactionRecord {
      id: Some(self.get_id() + 1),
      amount: Some(amount),
      balance: Some(if amount > 0 {
        self.get_balance().checked_add(amount as E8S).expect("Overflow when adding amount")
      } else {
        self.get_balance().checked_sub(-amount as E8S).expect("Overflow when subtracting amount")
      }),
      record_type: Some(record_type),
      block_index: Some(block_index),
      created_at: Some(create_time),
    }
  }

  pub fn get_id(&self) -> PoolTransactionRecordId {
    self.id.unwrap()
  }

  pub fn get_amount(&self) -> i64 {
    self.amount.unwrap()
  }

  pub fn get_balance(&self) -> E8S {
    self.balance.unwrap()
  }

  pub fn get_record_type(&self) -> RecordType {
    self.record_type.clone().unwrap()
  }
}

#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub enum RecordType {
  /// Transaction fees paid by the staking pool
  Fee(PoolTransactionRecordId),
  /// Transaction fees prepaid to the staking pool by external parties
  PrepaidFee(PoolTransactionRecordId),
  /// Transaction records generated when users staking
  Staking(StakingAccountId),
  /// Transaction records generated when users unstaking
  Unstaking(StakingAccountId),
  /// Transaction records generated when users unstaking with penalty
  EarlyUnstakePenalty(StakingAccountId),
  /// Transaction records generated when staking to nns neuron
  NNSNeuronStake { neuron_id: EntityId },
  /// Transaction records generated when unstaking from nns neuron
  NNSNeuronUnstake { neuron_id: EntityId },
  /// Transaction records generated when transferring to jackpot
  Jackpot { canister_id: Principal, product_id: ProductId },
}

#[derive(Debug, Clone, Serialize, Deserialize, CandidType, PartialEq, Eq, PartialOrd, Ord)]
pub enum RecordTypeKey {
  Fee,
  PrepaidFee,
  Staking,
  Unstaking,
  EarlyUnstakePenalty,
  NNSNeuronStake,
  NNSNeuronUnstake,
  Jackpot,
}

impl From<u8> for RecordTypeKey {
  fn from(index: u8) -> Self {
    match index {
      0 => RecordTypeKey::Fee,
      1 => RecordTypeKey::PrepaidFee,
      2 => RecordTypeKey::Staking,
      3 => RecordTypeKey::Unstaking,
      4 => RecordTypeKey::EarlyUnstakePenalty,
      5 => RecordTypeKey::NNSNeuronStake,
      6 => RecordTypeKey::NNSNeuronUnstake,
      7 => RecordTypeKey::Jackpot,
      _ => ic_cdk::trap(format!("Invalid RecordTypeKey index from u8 with value {}", index)),
    }
  }
}

impl From<&RecordType> for RecordTypeKey {
  fn from(record_type: &RecordType) -> Self {
    match record_type {
      RecordType::Fee(_) => RecordTypeKey::Fee,
      RecordType::PrepaidFee(_) => RecordTypeKey::PrepaidFee,
      RecordType::Staking(_) => RecordTypeKey::Staking,
      RecordType::Unstaking(_) => RecordTypeKey::Unstaking,
      RecordType::EarlyUnstakePenalty(_) => RecordTypeKey::EarlyUnstakePenalty,
      RecordType::NNSNeuronStake { neuron_id: _ } => RecordTypeKey::NNSNeuronStake,
      RecordType::NNSNeuronUnstake { neuron_id: _ } => RecordTypeKey::NNSNeuronUnstake,
      RecordType::Jackpot {
        canister_id: _,
        product_id: _,
      } => RecordTypeKey::Jackpot,
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize, CandidType, PartialEq, Eq, PartialOrd, Ord)]
pub struct RecordTypeIndexKey(pub StakingPoolId, pub RecordTypeKey);

impl Storable for PoolTransactionRecords {
  fn to_bytes(&self) -> Cow<[u8]> {
    Cow::Owned(Encode!(self).unwrap())
  }

  fn from_bytes(bytes: Cow<[u8]>) -> Self {
    Decode!(bytes.as_ref(), Self).unwrap()
  }

  const BOUND: Bound = Bound::Unbounded;
}

impl Storable for RecordTypeIndexKey {
  fn to_bytes(&self) -> Cow<[u8]> {
    Cow::Owned(Encode!(self).unwrap())
  }

  fn from_bytes(bytes: Cow<[u8]>) -> Self {
    Decode!(bytes.as_ref(), Self).unwrap()
  }

  const BOUND: Bound = Bound::Unbounded;
}
