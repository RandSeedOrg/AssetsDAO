use candid::{CandidType, Deserialize, Principal};
use types::E8S;

const ICP_LEDGER_CANISTER_ID: &str = "ryjl3-tyaaa-aaaaa-aaaba-cai";

#[derive(CandidType, Deserialize, Debug, Clone)]
pub struct BlockIndex(pub u64);

#[derive(CandidType, Deserialize, Debug, Clone)]
pub struct Memo(pub u64);

#[derive(CandidType, Deserialize, Debug, Clone)]
pub struct TimeStamp {
  pub timestamp_nanos: u64,
}

#[derive(CandidType, Deserialize, Debug, Clone)]
pub struct AccountIdentifier {
  pub hash: [u8; 32],
}

#[derive(CandidType, Deserialize, Debug, Clone)]
pub struct Tokens {
  pub e8s: u64,
}

#[derive(CandidType, Deserialize, Debug, Clone)]
pub struct Transfer {
  pub memo: Memo,
  pub amount: Tokens,
  pub fee: Tokens,
  pub from: AccountIdentifier,
  pub to: AccountIdentifier,
  pub created_at_time: Option<TimeStamp>,
}

#[derive(CandidType, Deserialize, Debug, Clone)]
pub struct Mint {
  pub memo: Memo,
  pub amount: Tokens,
  pub to: AccountIdentifier,
  pub created_at_time: Option<TimeStamp>,
}

#[derive(CandidType, Deserialize, Debug, Clone)]
pub struct Burn {
  pub memo: Memo,
  pub amount: Tokens,
  pub from: AccountIdentifier,
  pub created_at_time: Option<TimeStamp>,
}

#[derive(CandidType, Deserialize, Debug, Clone)]
pub enum Operation {
  Mint(Mint),
  Burn(Burn),
  Transfer(Transfer),
}

#[derive(CandidType, Deserialize, Debug, Clone)]
pub struct Transaction {
  pub memo: Memo,
  pub operation: Option<Operation>,
  pub created_at_time: Option<TimeStamp>,
}

#[derive(CandidType, Deserialize, Debug, Clone)]
pub struct Block {
  pub parent_hash: Option<[u8; 32]>,
  pub transaction: Transaction,
  pub timestamp: TimeStamp,
}

#[derive(CandidType, Deserialize, Debug, Clone)]
pub struct EncodedBlock {
  pub block: Vec<u8>,
}

#[derive(CandidType, Deserialize, Debug, Clone)]
pub enum GetBlocksError {
  BadFirstBlockIndex {
    requested_index: BlockIndex,
    first_valid_index: BlockIndex,
  },
  Other {
    error_code: u64,
    error_message: String,
  },
}

#[derive(CandidType, Deserialize, Debug, Clone)]
pub struct GetBlocksArgs {
  pub start: BlockIndex,
  pub length: u64,
}

#[derive(CandidType, Deserialize, Debug, Clone)]
pub struct QueryBlocksResponse {
  pub chain_length: u64,
  pub certificate: Option<Vec<u8>>,
  pub blocks: Vec<EncodedBlock>,
  pub first_block_index: BlockIndex,
  pub archived_blocks: Vec<ArchivedBlocksRange>,
}

#[derive(CandidType, Deserialize, Debug, Clone)]
pub struct ArchivedBlocksRange {
  pub start: BlockIndex,
  pub length: u64,
  pub callback: QueryArchiveFn,
}

#[derive(CandidType, Deserialize, Debug, Clone)]
pub struct QueryArchiveFn {
  pub canister_id: Principal,
  pub method: String,
}

#[derive(CandidType, Deserialize, Debug, Clone)]
pub struct TransactionInfo {
  pub amount: E8S,
  pub fee: E8S,
  pub from: Option<AccountIdentifier>,
  pub to: Option<AccountIdentifier>,
  pub memo: u64,
  pub timestamp: u64,
  pub operation_type: String,
}

pub async fn query_transaction_by_block_height(block_height: u64) -> Result<TransactionInfo, String> {
  let ledger = Principal::from_text(ICP_LEDGER_CANISTER_ID).map_err(|e| format!("Invalid ledger canister ID: {}", e))?;

  let args = GetBlocksArgs {
    start: BlockIndex(block_height),
    length: 1,
  };

  let (response,): (Result<QueryBlocksResponse, GetBlocksError>,) = ic_cdk::call(ledger, "query_blocks", (args,))
    .await
    .map_err(|e| format!("Failed to call query_blocks: {:?}", e))?;

  let blocks_response = response.map_err(|e| format!("Query blocks error: {:?}", e))?;

  if blocks_response.blocks.is_empty() {
    return Err("Block not found".to_string());
  }

  let encoded_block = &blocks_response.blocks[0];
  let block = decode_block(&encoded_block.block)?;

  extract_transaction_info(&block)
}

fn decode_block(block_bytes: &[u8]) -> Result<Block, String> {
  candid::decode_one(block_bytes).map_err(|e| format!("Failed to decode block: {}", e))
}

fn extract_transaction_info(block: &Block) -> Result<TransactionInfo, String> {
  let transaction = &block.transaction;

  match &transaction.operation {
    Some(Operation::Transfer(transfer)) => Ok(TransactionInfo {
      amount: transfer.amount.e8s,
      fee: transfer.fee.e8s,
      from: Some(transfer.from.clone()),
      to: Some(transfer.to.clone()),
      memo: transaction.memo.0,
      timestamp: block.timestamp.timestamp_nanos,
      operation_type: "Transfer".to_string(),
    }),
    Some(Operation::Mint(mint)) => Ok(TransactionInfo {
      amount: mint.amount.e8s,
      fee: 0,
      from: None,
      to: Some(mint.to.clone()),
      memo: transaction.memo.0,
      timestamp: block.timestamp.timestamp_nanos,
      operation_type: "Mint".to_string(),
    }),
    Some(Operation::Burn(burn)) => Ok(TransactionInfo {
      amount: burn.amount.e8s,
      fee: 0,
      from: Some(burn.from.clone()),
      to: None,
      memo: transaction.memo.0,
      timestamp: block.timestamp.timestamp_nanos,
      operation_type: "Burn".to_string(),
    }),
    None => Err("No operation found in transaction".to_string()),
  }
}

pub async fn query_transactions_by_block_heights(block_heights: Vec<u64>) -> Result<Vec<TransactionInfo>, String> {
  let mut transactions = Vec::new();

  for block_height in block_heights {
    match query_transaction_by_block_height(block_height).await {
      Ok(tx_info) => transactions.push(tx_info),
      Err(e) => {
        ic_cdk::println!("Failed to query block {}: {}", block_height, e);

        continue;
      }
    }
  }

  Ok(transactions)
}

pub async fn query_transaction_range(start_block: u64, length: u64) -> Result<Vec<TransactionInfo>, String> {
  let ledger = Principal::from_text(ICP_LEDGER_CANISTER_ID).map_err(|e| format!("Invalid ledger canister ID: {}", e))?;

  let args = GetBlocksArgs {
    start: BlockIndex(start_block),
    length,
  };

  let (response,): (Result<QueryBlocksResponse, GetBlocksError>,) = ic_cdk::call(ledger, "query_blocks", (args,))
    .await
    .map_err(|e| format!("Failed to call query_blocks: {:?}", e))?;

  let blocks_response = response.map_err(|e| format!("Query blocks error: {:?}", e))?;

  let mut transactions = Vec::new();

  for encoded_block in blocks_response.blocks {
    let block = decode_block(&encoded_block.block)?;
    let tx_info = extract_transaction_info(&block)?;
    transactions.push(tx_info);
  }

  Ok(transactions)
}
