//! Account-indexed recovery. A missing index response is never evidence of absence.
use std::{borrow::Cow, cell::RefCell};

use candid::{CandidType, Decode, Encode, Principal};
use ic_ledger_types::{AccountIdentifier, Memo, Subaccount, Timestamp, Tokens, TransferArgs, TransferError, MAINNET_LEDGER_CANISTER_ID};
use ic_stable_structures::{memory_manager::MemoryId, storable::Bound, StableBTreeMap, Storable};
use serde::{Deserialize, Serialize};
use types::stable_structures::Memory;

use crate::{memory_ids, nns::utils::ledger_utils, MEMORY_MANAGER};

const PAGE_SIZE: u64 = 20;
const RETRY_DELAY: u64 = 5_000_000_000;

#[derive(Debug)]
pub enum StepError {
  Pending(String),
  Failed(String),
}
impl From<String> for StepError {
  fn from(value: String) -> Self {
    Self::Failed(value)
  }
}
pub type StepResult<T> = Result<T, StepError>;

/// Reservations include a possible archive callback, so actual calls never exceed 20.
pub struct Budget {
  calls: u8,
  pages: u8,
  pub pool_id: u64,
}
impl Budget {
  pub fn new(pool_id: u64) -> Self {
    Self {
      calls: 20,
      pages: 5,
      pool_id,
    }
  }
  pub fn check(&self) -> StepResult<()> {
    if super::get_job(self.pool_id)?.phase != super::WindDownPhase::Running {
      return Err(StepError::Pending("Wind-down paused".into()));
    }
    Ok(())
  }
  pub fn take(&mut self, calls: u8) -> StepResult<()> {
    self.check()?;
    if self.calls < calls {
      return Err(StepError::Pending("Execution budget reached; continue this batch".into()));
    }
    self.calls -= calls;
    Ok(())
  }
  fn page(&mut self) -> StepResult<()> {
    if self.pages == 0 {
      return Err(StepError::Pending("Account history page budget reached".into()));
    }
    self.take(1)?;
    self.pages -= 1;
    Ok(())
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, CandidType, Serialize, Deserialize)]
pub enum Stage {
  Release,
  Dissolve,
}
impl Stage {
  fn key(self) -> u8 {
    match self {
      Self::Release => 0,
      Self::Dissolve => 1,
    }
  }
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize, PartialEq, Eq)]
pub struct Spec {
  pub pool_id: u64,
  pub account_id: u64,
  pub stage: Stage,
  pub from_subaccount: [u8; 32],
  pub from: String,
  pub to: String,
  pub amount: u64,
  pub fee: u64,
  pub memo: u64,
  pub lower_bound: Option<u64>,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct Verified {
  pub block: u64,
  pub amount: u64,
  pub fee: u64,
  pub timestamp: u64,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
struct Search {
  upper: u64,
  next: u64,
  candidate: Option<u64>,
  examined: u64,
  complete: bool,
  synced: Option<u64>,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
struct Intent {
  created_at_time: u64,
  // Set before the external call, and committed with that call.
  attempted: bool,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
struct Operation {
  version: u16,
  spec: Spec,
  search: Option<Search>,
  intent: Option<Intent>,
  known: Option<u64>,
  verified: Option<Verified>,
}

#[derive(Debug, Clone, CandidType, Serialize, Deserialize)]
pub struct RecoveryProgress {
  pub account_id: u64,
  pub stage: Stage,
  pub state: String,
  pub examined_transactions: u64,
  pub next_start: Option<u64>,
  pub ledger_upper_bound: Option<u64>,
  pub index_blocks_synced: Option<u64>,
  pub candidate_blocks: Vec<u64>,
  pub retry_at: Option<u64>,
  pub last_error: Option<String>,
}

#[derive(Debug, Clone, Default, CandidType, Serialize, Deserialize)]
pub struct PoolAudit {
  pub version: u16,
  pub cursor: u64,
  pub complete: bool,
  pub repairs: Vec<crate::pool_transaction_record::utils::ReleaseRepair>,
  pub progress: Option<RecoveryProgress>,
}

macro_rules! stable {
  ($ty:ty) => {
    impl Storable for $ty {
      fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Owned(Encode!(self).unwrap())
      }
      fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
      }
      const BOUND: Bound = Bound::Unbounded;
    }
  };
}
stable!(Operation);
stable!(PoolAudit);
thread_local! {
  static OPERATIONS: RefCell<StableBTreeMap<(u64,u8), Operation, Memory>> = RefCell::new(StableBTreeMap::init(
    MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(memory_ids::STAKING_WIND_DOWN_RECOVERY)))));
  static AUDITS: RefCell<StableBTreeMap<u64, PoolAudit, Memory>> = RefCell::new(StableBTreeMap::init(
    MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(memory_ids::STAKING_WIND_DOWN_AUDIT)))));
}
fn save(op: &Operation) {
  OPERATIONS.with(|m| m.borrow_mut().insert((op.spec.account_id, op.spec.stage.key()), op.clone()));
}
pub fn audit(pool: u64) -> PoolAudit {
  AUDITS.with(|m| m.borrow().get(&pool)).unwrap_or_default()
}
pub fn save_audit(pool: u64, state: PoolAudit) {
  AUDITS.with(|m| m.borrow_mut().insert(pool, state));
}
pub fn progress(pool: u64) -> Option<RecoveryProgress> {
  audit(pool).progress
}
pub fn clear_progress(pool: u64) {
  let mut a = audit(pool);
  a.progress = None;
  save_audit(pool, a);
}
pub fn report(pool: u64, message: String, failed: bool) {
  let mut a = audit(pool);
  if let Some(p) = &mut a.progress {
    if failed {
      p.state = "ManualReview".into();
      p.retry_at = None;
    }
    p.last_error = Some(message);
  }
  save_audit(pool, a);
}
fn show(op: &Operation, state: &str) {
  let mut a = audit(op.spec.pool_id);
  let s = op.search.as_ref();
  a.progress = Some(RecoveryProgress {
    account_id: op.spec.account_id,
    stage: op.spec.stage,
    state: state.into(),
    examined_transactions: s.map_or(0, |s| s.examined),
    next_start: s.map(|s| s.next),
    ledger_upper_bound: s.map(|s| s.upper),
    index_blocks_synced: s.and_then(|s| s.synced),
    candidate_blocks: s.and_then(|s| s.candidate).or(op.known).into_iter().collect(),
    retry_at: Some(ic_cdk::api::time().saturating_add(RETRY_DELAY)),
    last_error: None,
  });
  save_audit(op.spec.pool_id, a);
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct IndexPage {
  pub balance: u64,
  pub transactions: Vec<IndexedTransaction>,
  pub oldest_tx_id: Option<u64>,
}
#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct IndexedTransaction {
  pub id: u64,
  pub transaction: IndexTransaction,
}
#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct IndexTransaction {
  pub memo: u64,
  pub operation: IndexOperation,
  pub created_at_time: Option<Timestamp>,
}
#[derive(Clone, Debug, CandidType, Deserialize)]
pub enum IndexOperation {
  Transfer {
    from: String,
    to: String,
    amount: Tokens,
    fee: Tokens,
    spender: Option<String>,
  },
  Mint {
    to: String,
    amount: Tokens,
  },
  Burn {
    from: String,
    amount: Tokens,
    spender: Option<String>,
  },
  Approve {
    from: String,
    spender: String,
    allowance: Tokens,
    expected_allowance: Option<Tokens>,
    expires_at: Option<Timestamp>,
    fee: Tokens,
  },
}
#[derive(CandidType, Deserialize)]
struct IndexError {
  message: String,
}
#[derive(CandidType)]
struct PageArgs {
  account_identifier: String,
  start: Option<u64>,
  max_results: u64,
}
#[derive(CandidType, Deserialize)]
struct IndexStatus {
  num_blocks_synced: u64,
}

fn index_id() -> Result<Principal, String> {
  let configured = crate::system_configs::get_dict_with_dict_code(&"system_config".to_string())
    .and_then(|d| d.items.into_iter().find(|i| i.label == "ICP Index").map(|i| i.value));
  Principal::from_text(configured.as_deref().unwrap_or("qhbym-qaaaa-aaaaa-aaafq-cai"))
    .map_err(|_| "Invalid ICP Index principal in system_config".into())
}
async fn index_call<T: for<'de> Deserialize<'de> + CandidType>(id: Principal, method: &str, args: impl CandidType) -> StepResult<T> {
  let response = ic_cdk::call::Call::unbounded_wait(id, method)
    .with_arg(args)
    .await
    .map_err(|e| StepError::Pending(format!("ICP Index unavailable: {e:?}")))?;
  response
    .candid::<T>()
    .map_err(|e| StepError::Failed(format!("Invalid ICP Index response: {e:?}")))
}
async fn index_noargs<T: for<'de> Deserialize<'de> + CandidType>(id: Principal, method: &str) -> StepResult<T> {
  let response = ic_cdk::call::Call::unbounded_wait(id, method)
    .await
    .map_err(|e| StepError::Pending(format!("ICP Index unavailable: {e:?}")))?;
  response
    .candid::<T>()
    .map_err(|e| StepError::Failed(format!("Invalid ICP Index response: {e:?}")))
}

fn same_address(a: &str, b: &str) -> Result<bool, String> {
  Ok(
    AccountIdentifier::from_hex(a).map_err(|_| "Invalid ICP account identifier".to_string())?
      == AccountIdentifier::from_hex(b).map_err(|_| "Invalid expected ICP account identifier".to_string())?,
  )
}
fn matches(spec: &Spec, tx: &IndexTransaction, created: Option<u64>) -> Result<bool, String> {
  if let IndexOperation::Transfer {
    from,
    to,
    amount,
    fee,
    spender,
  } = &tx.operation
  {
    if same_address(from, &spec.from)? && same_address(to, &spec.to)? && tx.memo == spec.memo {
      if amount.e8s() != spec.amount
        || fee.e8s() != spec.fee
        || spender.is_some()
        || created.is_some_and(|c| tx.created_at_time.map(|t| t.timestamp_nanos) != Some(c))
      {
        return Err("Conflicting transfer for this account and stage".into());
      }
      return Ok(true);
    }
  }
  Ok(false)
}

fn belongs_to_account(spec: &Spec, tx: &IndexTransaction) -> Result<bool, String> {
  let account = match spec.stage {
    Stage::Release => &spec.to,
    Stage::Dissolve => &spec.from,
  };
  match &tx.operation {
    IndexOperation::Transfer { from, to, spender, .. } => Ok(
      same_address(from, account)? | same_address(to, account)? | spender.as_deref().map(|s| same_address(s, account)).transpose()?.unwrap_or(false),
    ),
    IndexOperation::Mint { to, .. } => same_address(to, account),
    IndexOperation::Burn { from, spender, .. } => {
      Ok(same_address(from, account)? | spender.as_deref().map(|s| same_address(s, account)).transpose()?.unwrap_or(false))
    }
    IndexOperation::Approve { from, spender, .. } => Ok(same_address(from, account)? | same_address(spender, account)?),
  }
}

fn consume_page(op: &mut Operation, page: IndexPage) -> Result<(), String> {
  let s = op.search.as_mut().ok_or("Missing account search")?;
  if page.transactions.len() > PAGE_SIZE as usize {
    return Err("Oversized account history response".into());
  }
  if page.transactions.is_empty() {
    if page.oldest_tx_id.is_none() || page.oldest_tx_id.is_some_and(|oldest| oldest >= s.next) {
      s.complete = true;
      return Ok(());
    }
    return Err("Incomplete account history: index omitted older transactions".into());
  }
  let oldest = page.oldest_tx_id.ok_or("Index omitted oldest transaction ID")?;
  let mut previous = s.next;
  for item in &page.transactions {
    if item.id >= previous || item.id > s.upper || item.id < oldest {
      return Err("Invalid account history ordering or bounds".into());
    }
    if !belongs_to_account(&op.spec, &item.transaction)? {
      return Err("Index returned history for a different account".into());
    }
    previous = item.id;
    if op.spec.lower_bound.is_some_and(|lower| item.id <= lower) {
      s.complete = true;
      break;
    }
    s.examined = s.examined.checked_add(1).ok_or("Account history count overflow")?;
    if matches(&op.spec, &item.transaction, op.intent.as_ref().map(|i| i.created_at_time))? {
      if let Some(first) = s.candidate {
        return Err(format!("Multiple matching transfers: blocks {first} and {}", item.id));
      }
      s.candidate = Some(item.id);
    }
    if item.id == oldest {
      s.complete = true;
    }
  }
  s.next = previous;
  Ok(())
}

async fn verify(op: &mut Operation, block: u64, budget: &mut Budget) -> StepResult<Verified> {
  budget.take(2)?;
  let tx = ledger_utils::query_transaction_by_block_height(block).await.map_err(StepError::Pending)?;
  budget.check()?;
  let expected_from = AccountIdentifier::from_hex(&op.spec.from).map_err(|e| e.to_string())?;
  let expected_to = AccountIdentifier::from_hex(&op.spec.to).map_err(|e| e.to_string())?;
  if op.spec.lower_bound.is_some_and(|b| block <= b)
    || tx.operation_type != "Transfer"
    || tx.memo != op.spec.memo
    || tx.amount != op.spec.amount
    || tx.fee != op.spec.fee
    || tx.from != Some(expected_from)
    || tx.to != Some(expected_to)
    || op.intent.as_ref().is_some_and(|i| tx.created_at_time != Some(i.created_at_time))
  {
    return Err(StepError::Failed(format!("Ledger block {block} does not match the expected transfer")));
  }
  let verified = Verified {
    block,
    amount: tx.amount,
    fee: tx.fee,
    timestamp: tx.timestamp,
  };
  op.verified = Some(verified.clone());
  save(op);
  show(op, "AccountingPending");
  Ok(verified)
}

/// None means proven absence in a completed, synchronized account snapshot (audit only).
pub async fn resolve(spec: Spec, known: Option<u64>, send: bool, budget: &mut Budget) -> StepResult<Option<Verified>> {
  let key = (spec.account_id, spec.stage.key());
  let mut op = OPERATIONS.with(|m| m.borrow().get(&key)).unwrap_or(Operation {
    version: 1,
    spec: spec.clone(),
    search: None,
    intent: None,
    known,
    verified: None,
  });
  if op.version != 1 || op.spec != spec {
    return Err(StepError::Failed("Recovery specification changed; manual review required".into()));
  }
  show(&op, "CheckingHistory");
  if known.is_some() && op.known.is_some() && known != op.known {
    return Err(StepError::Failed("Conflicting persisted transfer receipts".into()));
  }
  if let Some(v) = &op.verified {
    show(&op, "AccountingPending");
    return Ok(Some(v.clone()));
  }
  op.known = op.known.or(known);
  if let Some(block) = op.known {
    return verify(&mut op, block, budget).await.map(Some);
  }

  if op.search.is_none() {
    budget.take(1)?;
    let upper = ledger_utils::tip_of_chain().await.map_err(StepError::Pending)?;
    budget.check()?;
    if spec.lower_bound.is_some_and(|lower| lower > upper) {
      return Err(StepError::Failed("Account lifecycle boundary is ahead of the Ledger tip".into()));
    }
    op.search = Some(Search {
      upper,
      next: upper.checked_add(1).ok_or("Ledger height overflow".to_string())?,
      candidate: None,
      examined: 0,
      complete: false,
      synced: None,
    });
    save(&op);
  }
  if !op.search.as_ref().unwrap().complete {
    let index = index_id()?;
    budget.take(1)?;
    let ledger: Principal = index_noargs(index, "ledger_id").await?;
    budget.check()?;
    if ledger != MAINNET_LEDGER_CANISTER_ID {
      return Err(StepError::Failed("ICP Index is connected to a different Ledger".into()));
    }
    budget.take(1)?;
    let status: IndexStatus = index_noargs(index, "status").await?;
    budget.check()?;
    let s = op.search.as_mut().unwrap();
    s.synced = Some(status.num_blocks_synced);
    let ready = status.num_blocks_synced > s.upper;
    save(&op);
    show(&op, if ready { "CheckingHistory" } else { "WaitingForIndex" });
    if !ready {
      return Err(StepError::Pending("Waiting for ICP Index to cover the fixed Ledger height".into()));
    }
    while !op.search.as_ref().unwrap().complete {
      budget.page()?;
      let page: Result<IndexPage, IndexError> = index_call(
        index,
        "get_account_identifier_transactions",
        PageArgs {
          account_identifier: crate::on_chain::address::generate_staking_account_account_identifier(spec.account_id).to_hex(),
          start: Some(op.search.as_ref().unwrap().next),
          max_results: PAGE_SIZE,
        },
      )
      .await?;
      budget.check()?;
      let page = page.map_err(|e| StepError::Failed(e.message))?;
      if let Err(error) = consume_page(&mut op, page.clone()) {
        // Preserve evidence for review without committing a partially consumed page.
        show(&op, "ManualReview");
        let mut state = audit(spec.pool_id);
        if let Some(progress) = &mut state.progress {
          let mut candidates: std::collections::BTreeSet<u64> = progress.candidate_blocks.iter().copied().collect();
          for item in page.transactions {
            if matches(&spec, &item.transaction, op.intent.as_ref().map(|i| i.created_at_time)).unwrap_or(false) {
              candidates.insert(item.id);
            }
          }
          progress.candidate_blocks = candidates.into_iter().collect();
        }
        save_audit(spec.pool_id, state);
        return Err(StepError::Failed(error));
      }
      save(&op);
      show(&op, "CheckingHistory");
    }
  }
  if let Some(block) = op.search.as_ref().and_then(|s| s.candidate) {
    op.known = Some(block);
    save(&op);
    return verify(&mut op, block, budget).await.map(Some);
  }
  if !send {
    return Ok(None);
  }
  budget.take(1)?;
  let now = ic_cdk::api::time();
  let intent = op.intent.get_or_insert(Intent {
    created_at_time: now,
    attempted: false,
  });
  // Conservative default for ICP. Never renew an ambiguous intent after expiry.
  if now.saturating_sub(intent.created_at_time) >= 86_400_000_000_000 {
    return Err(StepError::Failed(
      "Transfer intent expired without confirmed outcome; manual review required".into(),
    ));
  }
  let args = TransferArgs {
    memo: Memo(spec.memo),
    amount: Tokens::from_e8s(spec.amount),
    fee: Tokens::from_e8s(spec.fee),
    from_subaccount: Some(Subaccount(spec.from_subaccount)),
    to: AccountIdentifier::from_hex(&spec.to).map_err(|e| e.to_string())?,
    created_at_time: Some(Timestamp {
      timestamp_nanos: intent.created_at_time,
    }),
  };
  intent.attempted = true;
  // A trap in the reply must not resurrect an old proof of absence.
  op.search = None;
  save(&op);
  show(&op, "TransferPending");
  #[cfg(feature = "recovery-tests")]
  let inject_trap = super::fixture::take_fault(spec.stage);
  let response = ic_cdk::call::Call::unbounded_wait(MAINNET_LEDGER_CANISTER_ID, "transfer")
    .with_arg(args)
    .await
    .map_err(|e| StepError::Pending(format!("Transfer outcome awaiting confirmation: {e:?}")))?;
  #[cfg(feature = "recovery-tests")]
  if inject_trap {
    ic_cdk::trap("Injected trap after Ledger transfer");
  }
  let result: Result<u64, TransferError> = response
    .candid()
    .map_err(|e| StepError::Pending(format!("Transfer response undecodable: {e:?}")))?;
  let block = match result {
    Ok(block) | Err(TransferError::TxDuplicate { duplicate_of: block }) => block,
    Err(TransferError::TxTooOld { .. }) => return Err(StepError::Failed("Transfer intent too old; recover from Index or review manually".into())),
    Err(error) => return Err(StepError::Pending(format!("Ledger rejected the unchanged transfer intent: {error:?}"))),
  };
  op.known = Some(block);
  save(&op);
  budget.check()?;
  verify(&mut op, block, budget).await.map(Some)
}

#[cfg(test)]
mod tests {
  use super::*;
  fn operation() -> Operation {
    let from = AccountIdentifier::new(&Principal::anonymous(), &Subaccount([0; 32])).to_hex();
    let to = AccountIdentifier::new(&Principal::anonymous(), &Subaccount([1; 32])).to_hex();
    Operation {
      version: 1,
      spec: Spec {
        pool_id: 3,
        account_id: 147,
        stage: Stage::Release,
        from_subaccount: [0; 32],
        from,
        to,
        amount: 100_010_000,
        fee: 10_000,
        memo: 2,
        lower_bound: Some(10),
      },
      search: Some(Search {
        upper: 100,
        next: 101,
        candidate: None,
        examined: 0,
        complete: false,
        synced: Some(101),
      }),
      intent: None,
      known: None,
      verified: None,
    }
  }
  fn transaction(op: &Operation, id: u64, matched: bool) -> IndexedTransaction {
    IndexedTransaction {
      id,
      transaction: IndexTransaction {
        memo: if matched { 2 } else { 99 },
        created_at_time: Some(Timestamp { timestamp_nanos: 1 }),
        operation: IndexOperation::Transfer {
          from: op.spec.from.clone(),
          to: op.spec.to.clone(),
          amount: Tokens::from_e8s(op.spec.amount),
          fee: Tokens::from_e8s(op.spec.fee),
          spender: None,
        },
      },
    }
  }
  fn page(transactions: Vec<IndexedTransaction>, oldest: u64) -> IndexPage {
    IndexPage {
      balance: 0,
      transactions,
      oldest_tx_id: Some(oldest),
    }
  }
  #[test]
  fn short_page_does_not_prove_absence() {
    let mut op = operation();
    let tx = transaction(&op, 90, false);
    consume_page(&mut op, page(vec![tx], 1)).unwrap();
    assert!(!op.search.as_ref().unwrap().complete);
    assert_eq!(op.search.as_ref().unwrap().next, 90);
    assert!(consume_page(&mut op, page(vec![], 1)).is_err());
  }
  #[test]
  fn duplicate_matches_across_pages_are_rejected_after_stable_roundtrip() {
    let mut op = operation();
    let tx = transaction(&op, 90, true);
    consume_page(&mut op, page(vec![tx], 1)).unwrap();
    let mut restored = Operation::from_bytes(Cow::Owned(op.to_bytes().to_vec()));
    let duplicate = transaction(&restored, 80, true);
    assert!(consume_page(&mut restored, page(vec![duplicate], 1)).unwrap_err().contains("Multiple"));
  }
  #[test]
  fn covers_lower_boundary_and_excludes_earlier_lifecycle() {
    let mut op = operation();
    let txs = vec![transaction(&op, 90, true), transaction(&op, 10, true)];
    consume_page(&mut op, page(txs, 1)).unwrap();
    assert!(op.search.as_ref().unwrap().complete);
    assert_eq!(op.search.as_ref().unwrap().candidate, Some(90));
  }
  #[test]
  fn no_lower_bound_reads_to_oldest_and_validates_cursor() {
    let mut op = operation();
    op.spec.lower_bound = None;
    let tx = transaction(&op, 100, false);
    consume_page(&mut op, page(vec![tx.clone()], 1)).unwrap();
    assert!(consume_page(&mut op, page(vec![tx], 1)).is_err());
    let tx = transaction(&op, 1, false);
    consume_page(&mut op, page(vec![tx], 1)).unwrap();
    assert!(op.search.as_ref().unwrap().complete);
  }
  #[test]
  fn relevant_amount_fee_and_timestamp_conflicts_are_not_absence() {
    let op = operation();
    let mut tx = transaction(&op, 90, true).transaction;
    assert!(matches(&op.spec, &tx, None).unwrap());
    assert!(matches(&op.spec, &tx, Some(2)).is_err());
    if let IndexOperation::Transfer { fee, .. } = &mut tx.operation {
      *fee = Tokens::from_e8s(20_000);
    }
    assert!(matches(&op.spec, &tx, None).is_err());
    tx.memo = 3;
    assert!(!matches(&op.spec, &tx, None).unwrap());
  }
  #[test]
  fn history_for_another_account_cannot_prove_absence() {
    let mut op = operation();
    let mut tx = transaction(&op, 90, false);
    if let IndexOperation::Transfer { to, .. } = &mut tx.transaction.operation {
      *to = op.spec.from.clone();
    }
    assert!(consume_page(&mut op, page(vec![tx], 90)).unwrap_err().contains("different account"));
  }
}
