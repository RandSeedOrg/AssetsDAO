use candid::{CandidType, Principal};
use ic_ledger_types::*;
use serde::Deserialize;
use std::{cell::RefCell, collections::BTreeMap};
#[derive(Default)]
struct State {
  blocks: BTreeMap<u64, Block>,
  balance: u64,
  tip: u64,
  synced: u64,
  sends: u64,
  pages: u64,
  verified: u64,
  credits: u64,
  credited: BTreeMap<u64, (Principal, u64, String, u64)>,
  duplicates: Vec<(TransferArgs, u64)>,
  reject_index: bool,
  hide_new: bool,
  lose_pay_reply: bool,
  archived: bool,
  wrong_ledger: bool,
}
thread_local! {static STATE:RefCell<State>=RefCell::new(State::default());}
#[derive(CandidType, Deserialize)]
struct Seed {
  pool: String,
  account: String,
  pay: String,
  release: bool,
  dissolve: bool,
  extra: u64,
  duplicate: bool,
  tip: u64,
}
fn address(a: &str) -> AccountIdentifier {
  AccountIdentifier::from_hex(a).unwrap()
}
fn block(from: &str, to: &str, amount: u64, memo: u64, created: u64) -> Block {
  Block {
    parent_hash: None,
    timestamp: Timestamp { timestamp_nanos: created },
    transaction: Transaction {
      memo: Memo(memo),
      operation: Some(Operation::Transfer {
        from: address(from),
        to: address(to),
        amount: Tokens::from_e8s(amount),
        fee: Tokens::from_e8s(10_000),
      }),
      created_at_time: Timestamp { timestamp_nanos: created },
      icrc1_memo: None,
    },
  }
}
#[ic_cdk::update]
fn seed(seed: Seed) {
  STATE.with(|s| {
    let mut s = s.borrow_mut();
    *s = State::default();
    s.tip = seed.tip;
    s.synced = seed.tip + 1;
    s.balance = if seed.release { 317_214 } else { 100_337_214 };
    s.blocks.insert(10, block(&seed.account, &seed.pool, 100_020_000, 1, 1));
    if seed.release {
      s.blocks.insert(100, block(&seed.pool, &seed.account, 100_010_000, 2, 2));
    }
    if seed.dissolve {
      s.blocks.insert(200, block(&seed.account, &seed.pay, 100_000_000, 3, 3));
    }
    for i in 0..seed.extra {
      s.blocks.insert(1000 + i, block(&seed.pay, &seed.account, 1, 99, 4));
    }
    if seed.duplicate {
      s.blocks.insert(1500, block(&seed.pool, &seed.account, 100_010_000, 2, 5));
    }
  });
}
#[derive(CandidType, Deserialize)]
struct Controls {
  synced: u64,
  reject_index: bool,
  hide_new: bool,
  lose_pay_reply: bool,
}
#[ic_cdk::update]
fn controls(c: Controls) {
  STATE.with(|s| {
    let mut s = s.borrow_mut();
    s.synced = c.synced;
    s.reject_index = c.reject_index;
    s.hide_new = c.hide_new;
    s.lose_pay_reply = c.lose_pay_reply;
  });
}
#[derive(CandidType)]
struct Tip {
  tip_index: u64,
  certification: Option<Vec<u8>>,
}
#[ic_cdk::update]
fn tip_of_chain() -> Tip {
  Tip {
    tip_index: STATE.with(|s| s.borrow().tip),
    certification: None,
  }
}
#[ic_cdk::update]
fn account_balance(_: AccountBalanceArgs) -> Tokens {
  Tokens::from_e8s(STATE.with(|s| s.borrow().balance))
}
#[ic_cdk::update]
fn query_blocks(args: GetBlocksArgs) -> QueryBlocksResponse {
  assert_eq!(args.length, 1, "Global ledger scanning is forbidden");
  STATE.with(|s| {
    let mut s = s.borrow_mut();
    s.verified += 1;
    if s.archived {
      return QueryBlocksResponse {
        chain_length: s.tip + 1,
        certificate: None,
        first_block_index: s.tip + 1,
        blocks: vec![],
        archived_blocks: vec![ArchivedBlockRange {
          start: args.start,
          length: 1,
          callback: candid::Func {
            principal: ic_cdk::api::canister_self(),
            method: "archive".into(),
          }
          .into(),
        }],
      };
    }
    QueryBlocksResponse {
      chain_length: s.tip + 1,
      certificate: None,
      first_block_index: args.start,
      blocks: s.blocks.get(&args.start).cloned().into_iter().collect(),
      archived_blocks: vec![],
    }
  })
}
#[ic_cdk::update]
fn transfer(args: TransferArgs) -> Result<u64, TransferError> {
  STATE.with(|s| {
    let mut s = s.borrow_mut();
    s.sends += 1;
    if let Some((_, id)) = s
      .duplicates
      .iter()
      .find(|(a, _)| candid::encode_one(a).unwrap() == candid::encode_one(&args).unwrap())
    {
      return Err(TransferError::TxDuplicate { duplicate_of: *id });
    }
    let created = args.created_at_time.unwrap().timestamp_nanos;
    if ic_cdk::api::time().saturating_sub(created) > 86_400_000_000_000 {
      return Err(TransferError::TxTooOld {
        allowed_window_nanos: 86_400_000_000_000,
      });
    }
    s.tip += 1;
    let id = s.tip;
    let from = AccountIdentifier::new(&ic_cdk::api::msg_caller(), &args.from_subaccount.unwrap_or(Subaccount([0; 32]))).to_hex();
    s.blocks
      .insert(id, block(&from, &args.to.to_hex(), args.amount.e8s(), args.memo.0, created));
    if args.memo.0 == 2 {
      s.balance -= args.amount.e8s() + args.fee.e8s();
    }
    s.duplicates.push((args, id));
    if !s.hide_new {
      s.synced = s.tip + 1;
    }
    Ok(id)
  })
}
#[ic_cdk::update]
fn ledger_id() -> Principal {
  STATE.with(|s| {
    if s.borrow().wrong_ledger {
      Principal::anonymous()
    } else {
      MAINNET_LEDGER_CANISTER_ID
    }
  })
}
#[derive(CandidType, Deserialize)]
struct IndexStatus {
  num_blocks_synced: u64,
}
#[ic_cdk::update]
async fn status() -> IndexStatus {
  ic_cdk::call::Call::unbounded_wait(MAINNET_LEDGER_CANISTER_ID, "index_status")
    .await
    .unwrap()
    .candid()
    .unwrap()
}
#[ic_cdk::update]
fn index_status() -> IndexStatus {
  STATE.with(|s| IndexStatus {
    num_blocks_synced: s.borrow().synced,
  })
}
#[derive(CandidType, Deserialize)]
struct PageArgs {
  account_identifier: String,
  start: Option<u64>,
  max_results: u64,
}
#[derive(CandidType, Deserialize)]
struct Page {
  balance: u64,
  transactions: Vec<Indexed>,
  oldest_tx_id: Option<u64>,
}
#[derive(CandidType, Deserialize)]
struct Indexed {
  id: u64,
  transaction: Tx,
}
#[derive(CandidType, Deserialize)]
struct Tx {
  memo: u64,
  operation: IndexOp,
  created_at_time: Option<Timestamp>,
}
#[derive(CandidType, Deserialize)]
enum IndexOp {
  Transfer {
    from: String,
    to: String,
    amount: Tokens,
    fee: Tokens,
    spender: Option<String>,
  },
}
#[derive(CandidType, Deserialize)]
struct IndexError {
  message: String,
}
#[ic_cdk::update]
async fn get_account_identifier_transactions(args: PageArgs) -> Result<Page, IndexError> {
  ic_cdk::call::Call::unbounded_wait(MAINNET_LEDGER_CANISTER_ID, "history")
    .with_arg(args)
    .await
    .unwrap()
    .candid()
    .unwrap()
}
#[ic_cdk::update]
fn history(args: PageArgs) -> Result<Page, IndexError> {
  STATE.with(|s| {let mut s=s.borrow_mut();s.pages+=1;
 if s.reject_index {return Err(IndexError {message:"Injected index failure".into()});}
 assert!(args.max_results<=20);
 let all:Vec<_>=s.blocks.iter().filter(|(id,_)| !s.hide_new || !s.duplicates.iter().any(|(_,sent)| sent==*id)).filter(|(_,b)|matches!(b.transaction.operation,Some(Operation::Transfer {from,to,..}) if from.to_hex()==args.account_identifier || to.to_hex()==args.account_identifier)).collect();
 let oldest_tx_id=all.first().map(|(id,_)|**id);
 let transactions=all.into_iter().rev().filter(|(id,_)|**id<args.start.unwrap_or(u64::MAX)).take(args.max_results as usize).map(|(id,b)| {
 let Operation::Transfer {from,to,amount,fee}=b.transaction.operation.clone().unwrap() else {unreachable!()};
 Indexed {id:*id,transaction:Tx {memo:b.transaction.memo.0,operation:IndexOp::Transfer {from:from.to_hex(),to:to.to_hex(),amount,fee,spender:None},created_at_time:Some(b.transaction.created_at_time)}}
 }).collect();Ok(Page {balance:0,transactions,oldest_tx_id})
 })
}
#[derive(CandidType, Deserialize)]
enum PayResult {
  #[serde(rename = "ok")]
  Ok(u64),
  #[serde(rename = "err")]
  Err(String),
}
// Candid Rust rename uses serde attributes only with Deserialize derive.
#[ic_cdk::update]
async fn dissolve(user: Principal, amount: u64, tx: u64, source: String, account: u64) -> PayResult {
  let result = STATE.with(|s| {
    let mut s = s.borrow_mut();
    let request = (user, amount, source, account);
    if let Some(previous) = s.credited.get(&tx) {
      return if *previous == request {
        PayResult::Ok(tx)
      } else {
        PayResult::Err("Conflict".into())
      };
    }
    s.credited.insert(tx, request);
    s.credits += 1;
    PayResult::Ok(tx)
  });
  let lose = STATE.with(|s| {
    let mut s = s.borrow_mut();
    let v = s.lose_pay_reply;
    s.lose_pay_reply = false;
    v
  });
  if lose {
    let _ = ic_cdk::call::Call::unbounded_wait(ic_cdk::api::canister_self(), "checkpoint").await;
    ic_cdk::trap("Injected loss after pay center credit");
  }
  result
}
#[ic_cdk::update]
fn checkpoint() {}
#[ic_cdk::query]
fn counters() -> (u64, u64, u64, u64) {
  STATE.with(|s| {
    let s = s.borrow();
    (s.sends, s.pages, s.verified, s.credits)
  })
}

#[ic_cdk::update]
fn fault_mode(mode: String) {
  STATE.with(|s| {
    let mut s = s.borrow_mut();
    match mode.as_str() {
      "archive" => s.archived = true,
      "wrong-ledger" => s.wrong_ledger = true,
      "omit-attempt" => {
        s.hide_new = true;
        s.synced = s.tip + 1;
      }
      _ => panic!("Unknown fixture mode"),
    }
  });
}
#[ic_cdk::update]
fn archive(args: GetBlocksArgs) -> GetBlocksResult {
  assert_eq!(args.length, 1);
  STATE.with(|s| {
    let mut s = s.borrow_mut();
    s.verified += 1;
    Ok(BlockRange {
      blocks: s.blocks.get(&args.start).cloned().into_iter().collect(),
    })
  })
}

#[ic_cdk::update]
fn seed_remaining(accounts: Vec<String>) {
  STATE.with(|s| {
    let mut s = s.borrow_mut();
    let Some(Operation::Transfer { from, .. }) = s.blocks.get(&100).unwrap().transaction.operation else {
      panic!("Missing seed release")
    };
    for (offset, to) in accounts.iter().enumerate() {
      let principal = if offset == 0 { 30_000_000 } else { 10_000_000 };
      s.blocks.insert(300 + offset as u64, block(&from.to_hex(), to, principal + 10_000, 2, 6));
    }
  });
}
