#![allow(deprecated)]
use crate::nns::utils::ledger_canister::Service as LedgerService;
use candid::{CandidType, Deserialize, Func, Principal};
use ic_ledger_types::{ArchivedBlockRange, Block, BlockRange, GetBlocksArgs, GetBlocksResult, Operation}; // removed GetBlocksResult
use types::E8S;

const ICP_LEDGER_CANISTER_ID: &str = "ryjl3-tyaaa-aaaaa-aaaba-cai";

#[derive(CandidType, Deserialize, Debug, Clone)]
pub struct TransactionInfo {
  pub amount: E8S,
  pub fee: E8S,
  pub from: Option<ic_ledger_types::AccountIdentifier>,
  pub to: Option<ic_ledger_types::AccountIdentifier>,
  pub memo: u64,
  pub timestamp: u64,
  pub operation_type: String,
}

pub async fn query_transaction_by_block_height(block_height: u64) -> Result<TransactionInfo, String> {
  let ledger_principal = Principal::from_text(ICP_LEDGER_CANISTER_ID).map_err(|e| format!("Invalid ledger canister ID: {}", e))?;
  let service = LedgerService(ledger_principal);
  let args = GetBlocksArgs {
    start: block_height,
    length: 1,
  };
  ic_cdk::println!("[ledger_utils] Query start height={} length=1", block_height);
  let (result,) = service
    .query_blocks(args)
    .await
    .map_err(|e| format!("Failed to call query_blocks: code={:?} msg={}", e.0, e.1))?;

  let primary = result;

  ic_cdk::println!(
    "[ledger_utils] first_block_index={} chain_length={} returned_blocks={} archived_ranges={}",
    primary.first_block_index,
    primary.chain_length,
    primary.blocks.len(),
    primary.archived_blocks.len()
  );

  let chain_end_exclusive = primary.first_block_index + primary.chain_length;
  if block_height >= chain_end_exclusive {
    return Err(format!(
      "Block height {} out of range (>= chain_end {}). first={}, length={}",
      block_height, chain_end_exclusive, primary.first_block_index, primary.chain_length
    ));
  }
  if block_height < primary.first_block_index {
    ic_cdk::println!(
      "[ledger_utils] height {} < first_block_index {}, searching archives",
      block_height,
      primary.first_block_index
    );
    if primary.archived_blocks.is_empty() {
      return Err(format!("Requested archived block {} but no archived ranges returned", block_height));
    }
  }
  if !primary.blocks.is_empty() {
    let block = &primary.blocks[0];
    return extract_transaction_info(block);
  }
  if let Some(info) = fetch_from_archives(block_height, &primary.archived_blocks).await? {
    return Ok(info);
  }

  if !primary.archived_blocks.is_empty() {
    let mut covered = false;
    for r in &primary.archived_blocks {
      if block_height >= r.start && block_height < r.start + r.length {
        covered = true;
        break;
      }
    }
    if !covered {
      return Err(format!(
        "Block {} not in any archived range (ranges count={})",
        block_height,
        primary.archived_blocks.len()
      ));
    }
  }
  Err(format!("Block {} not found in primary or archives", block_height))
}

async fn fetch_from_archives(block_height: u64, archived: &Vec<ArchivedBlockRange>) -> Result<Option<TransactionInfo>, String> {
  for range in archived {
    let start = range.start;
    let end_exclusive = start + range.length;
    if block_height >= start && block_height < end_exclusive {
      let archive_args = GetBlocksArgs {
        start: block_height,
        length: 1,
      };

      let func: Func = range.callback.clone().into();

      ic_cdk::println!(
        "Block {} is archived. Calling archive canister={} method={}",
        block_height,
        func.principal,
        func.method
      );
      let (archived_resp,): (GetBlocksResult,) = ic_cdk::call(func.principal, &func.method, (archive_args,))
        .await
        .map_err(|e| format!("Failed to call archive callback: code={:?} msg={}", e.0, e.1))?;

      match archived_resp {
        GetBlocksResult::Ok(BlockRange { blocks }) => {
          if blocks.is_empty() {
            return Err("Archived block not found".to_string());
          }

          return extract_transaction_info(&blocks[0]).map(Some);
        }
        GetBlocksResult::Err(err) => {
          return Err(format!("Archive canister returned error: {:?}", err));
        }
      }
    }
  }
  Ok(None)
}

fn extract_transaction_info(block: &Block) -> Result<TransactionInfo, String> {
  let txn = &block.transaction;
  match &txn.operation {
    Some(Operation::Transfer { from, to, amount, fee }) => Ok(TransactionInfo {
      amount: amount.e8s(),
      fee: fee.e8s(),
      from: Some(*from),
      to: Some(*to),
      memo: txn.memo.0,
      timestamp: block.timestamp.timestamp_nanos,
      operation_type: "Transfer".to_string(),
    }),
    Some(Operation::Mint { amount, to }) => Ok(TransactionInfo {
      amount: amount.e8s(),
      fee: 0,
      from: None,
      to: Some(*to),
      memo: txn.memo.0,
      timestamp: block.timestamp.timestamp_nanos,
      operation_type: "Mint".to_string(),
    }),
    Some(Operation::Burn { amount, from }) => Ok(TransactionInfo {
      amount: amount.e8s(),
      fee: 0,
      from: Some(*from),
      to: None,
      memo: txn.memo.0,
      timestamp: block.timestamp.timestamp_nanos,
      operation_type: "Burn".to_string(),
    }),
    Some(Operation::Approve { from, spender, fee, .. }) => Ok(TransactionInfo {
      amount: 0,
      fee: fee.e8s(),
      from: Some(*from),
      to: Some(*spender),
      memo: txn.memo.0,
      timestamp: block.timestamp.timestamp_nanos,
      operation_type: "Approve".to_string(),
    }),
    Some(Operation::TransferFrom { from, to, amount, fee, .. }) => Ok(TransactionInfo {
      amount: amount.e8s(),
      fee: fee.e8s(),
      from: Some(*from),
      to: Some(*to),
      memo: txn.memo.0,
      timestamp: block.timestamp.timestamp_nanos,
      operation_type: "TransferFrom".to_string(),
    }),
    None => Err("No operation found in transaction".to_string()),
  }
}

pub async fn query_transactions_by_block_heights(block_heights: Vec<u64>) -> Result<Vec<TransactionInfo>, String> {
  let mut out = Vec::with_capacity(block_heights.len());
  for h in block_heights {
    match query_transaction_by_block_height(h).await {
      Ok(info) => out.push(info),
      Err(e) => {
        ic_cdk::println!("Skip block {}: {}", h, e);
      }
    }
  }
  Ok(out)
}

pub async fn query_transaction_range(start_block: u64, length: u64) -> Result<Vec<TransactionInfo>, String> {
  if length == 0 {
    return Ok(vec![]);
  }
  let mut result = Vec::with_capacity(length as usize);
  for h in start_block..start_block + length {
    match query_transaction_by_block_height(h).await {
      Ok(info) => result.push(info),
      Err(e) => {
        ic_cdk::println!("Skip block {} in range query: {}", h, e);
      }
    }
  }
  Ok(result)
}
