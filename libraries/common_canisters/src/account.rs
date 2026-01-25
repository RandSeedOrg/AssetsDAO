// This is an experimental feature to generate Rust binding from Candid.
// You may want to manually adjust some of the types.
#![allow(dead_code, unused_imports, deprecated)]
use candid::{self, CandidType, Deserialize, Principal};
use ic_cdk::api::call::CallResult as Result;

#[derive(CandidType, Deserialize)]
pub enum CommitmentLevel {
  #[serde(rename = "finalized")]
  Finalized,
  #[serde(rename = "confirmed")]
  Confirmed,
  #[serde(rename = "processed")]
  Processed,
}
#[derive(CandidType, Deserialize)]
pub enum Ed25519KeyName {
  MainnetTestKey1,
  LocalDevelopment,
  MainnetProdKey1,
}
#[derive(CandidType, Deserialize)]
pub struct AddInfoStr {
  pub value: String,
  pub name: String,
}
#[derive(CandidType, Deserialize)]
pub struct RpcEndpoint {
  pub url: String,
  pub headers: Option<Vec<AddInfoStr>>,
}
#[derive(CandidType, Deserialize)]
pub enum SolanaNetwork {
  Mainnet,
  Custom(RpcEndpoint),
  Devnet,
}
#[derive(CandidType, Deserialize)]
pub struct InitArg {
  pub solana_commitment_level: Option<CommitmentLevel>,
  pub ed25519_key_name: Option<Ed25519KeyName>,
  pub solana_network: Option<SolanaNetwork>,
  pub sol_rpc_canister_id: Option<Principal>,
}
#[derive(CandidType, Deserialize)]
pub enum EventStatus {
  Failed,
  Success,
  Processing,
  Pending,
}
#[derive(CandidType, Deserialize)]
pub enum EventType {
  Withdraw,
  Error,
  Deposit,
  TransferOut,
  SystemTransfer,
  TransferIn,
}
#[derive(CandidType, Deserialize)]
pub struct EventLog {
  pub fee: Option<u64>,
  pub status: EventStatus,
  pub slot: Option<u64>,
  pub time: u64,
  pub description: Option<String>,
  pub to_address: Option<String>,
  pub from_address: Option<String>,
  pub amount: Option<u64>,
  pub event_type: EventType,
}
#[derive(CandidType, Deserialize)]
pub enum Result_ {
  Ok,
  Err(String),
}
#[derive(CandidType, Deserialize)]
pub enum CurrencyType {
  #[serde(rename = "ICP")]
  Icp,
}
#[derive(CandidType, Deserialize)]
pub enum CurrencyUnit {
  E8S,
}
#[derive(CandidType, Deserialize)]
pub enum LedgerEventV3 {
  #[serde(rename = "FEE")]
  Fee,
  #[serde(rename = "WIN")]
  Win { times: candid::Nat },
  #[serde(rename = "STAKE")]
  Stake,
  #[serde(rename = "SHOP_SELL_TICKET")]
  ShopSellTicket,
  #[serde(rename = "TRANSFER")]
  Transfer,
  #[serde(rename = "BONUS")]
  Bonus,
  #[serde(rename = "PLAY")]
  Play,
  #[serde(rename = "DISSOLVE")]
  Dissolve,
  #[serde(rename = "MANUAL")]
  Manual,
  #[serde(rename = "WITHDRAW")]
  Withdraw,
  #[serde(rename = "PENALTY")]
  Penalty,
  #[serde(rename = "DEPOSIT")]
  Deposit,
  #[serde(rename = "ORIGIN")]
  Origin,
}
#[derive(CandidType, Deserialize)]
pub struct LedgerV5 {
  pub fee: candid::Int,
  pub ids: Vec<u64>,
  pub transaction_id: candid::Int,
  pub time: candid::Int,
  pub currency_type: CurrencyType,
  pub currency_unit: CurrencyUnit,
  pub event: LedgerEventV3,
  pub bonus_code: Option<String>,
  pub to_bonus: Option<candid::Int>,
  pub infos: Vec<String>,
  pub amount: candid::Int,
}
#[derive(CandidType, Deserialize)]
pub enum Event {
  #[serde(rename = "FEE")]
  Fee,
  #[serde(rename = "WIN")]
  Win {
    times: u64,
  },
  #[serde(rename = "STAKE")]
  Stake,
  #[serde(rename = "TRANSFER")]
  Transfer,
  #[serde(rename = "BONUS")]
  Bonus,
  #[serde(rename = "PLAY")]
  Play,
  #[serde(rename = "DISSOLVE")]
  Dissolve,
  #[serde(rename = "MANUAL")]
  Manual,
  BonusConvert,
  #[serde(rename = "WITHDRAW")]
  Withdraw,
  #[serde(rename = "PENALTY")]
  Penalty,
  ShopSellTicket,
  #[serde(rename = "DEPOSIT")]
  Deposit,
  #[serde(rename = "ORIGIN")]
  Origin,
}
#[derive(CandidType, Deserialize)]
pub enum Crypto {
  #[serde(rename = "ICP")]
  Icp,
  #[serde(rename = "GCOIN")]
  Gcoin,
  #[serde(rename = "USDC")]
  Usdc,
  #[serde(rename = "USDT")]
  Usdt,
}
#[derive(CandidType, Deserialize)]
pub struct AddInfoNum {
  pub value: u64,
  pub name: String,
}
#[derive(CandidType, Deserialize)]
pub struct Ledger {
  pub fee: Option<u64>,
  pub time: Option<u64>,
  pub blockchain_tx: Option<u64>,
  pub event: Option<Event>,
  pub crypto: Option<Crypto>,
  pub bonus_code: Option<String>,
  pub to_bonus: Option<i64>,
  pub amount: Option<i64>,
  pub add_info_num: Option<Vec<AddInfoNum>>,
  pub add_info_str: Option<Vec<AddInfoStr>>,
}
#[derive(CandidType, Deserialize)]
pub enum Result1 {
  Ok(String),
  Err(String),
}
#[derive(CandidType, Deserialize)]
pub struct UserDepositAddress {
  pub username: String,
  pub solana_wallet_address: Option<String>,
  pub error: Option<String>,
  pub solana_ata_address: Option<String>,
  pub principal_id: String,
}
#[derive(CandidType, Deserialize)]
pub enum Result2 {
  Ok(Vec<UserDepositAddress>),
  Err(String),
}
#[derive(CandidType, Deserialize)]
pub enum Result3 {
  Ok(bool, String),
  Err(String),
}
#[derive(CandidType, Deserialize)]
pub enum FundConditionNew {
  RealSpend {
    bonus_deduction: Option<u64>,
    deliver_count: Option<u8>,
    min_amount: u64,
    end_time: u64,
    start_time: u64,
    currency: String,
  },
  HasBalance {
    currency: String,
    amount: u64,
  },
  HasDeposit {
    min_amount: u64,
    end_time: u64,
    start_time: u64,
    is_first_deposit: Option<bool>,
    currency: String,
    max_amount: Option<u64>,
  },
}
#[derive(CandidType, Deserialize)]
pub enum PlayConditionNew {
  PlayCount { end_time: u64, min_count: u64, start_time: u64 },
}
#[derive(CandidType, Deserialize)]
pub struct UserBlacklistEntry {
  pub withdrawal_address: String,
  pub principal_id: String,
}
#[derive(CandidType, Deserialize)]
pub enum UserConditionNew {
  VerifiedUser,
  EarlyUser(u32),
  AllUser,
  SpecificUser(Vec<String>),
  RegisterActivityUser,
  AnonymousUser,
  UserBlacklist(Vec<UserBlacklistEntry>),
}
#[derive(CandidType, Deserialize)]
pub enum ConditionNew {
  Fund(FundConditionNew),
  Play(PlayConditionNew),
  User(UserConditionNew),
}
#[derive(CandidType, Deserialize)]
pub struct DepositCheckResult {
  pub principal_id: String,
  pub is_over_threshold: bool,
  pub total_deposit: i64,
}
#[derive(CandidType, Deserialize)]
pub enum Result4 {
  Ok(DepositCheckResult),
  Err(String),
}
#[derive(CandidType, Deserialize)]
pub struct PlayCheckResult {
  pub play_count: u64,
  pub principal_id: String,
  pub is_over_threshold: bool,
}
#[derive(CandidType, Deserialize)]
pub enum Result5 {
  Ok(PlayCheckResult),
  Err(String),
}
#[derive(CandidType, Deserialize)]
pub enum RealSpendingType {
  BonusSpender,
  AirdropSpender,
  LotterySpender,
}
#[derive(CandidType, Deserialize)]
pub struct SpentCheckResult {
  pub real_spending: i64,
  pub principal_id: String,
  pub is_over_threshold: bool,
}
#[derive(CandidType, Deserialize)]
pub enum Result6 {
  Ok(SpentCheckResult),
  Err(String),
}
#[derive(CandidType, Deserialize)]
pub enum Result7 {
  Ok(u64),
  Err(String),
}
#[derive(CandidType, Deserialize)]
pub struct TokenAta {
  pub ata_address: String,
  pub token_name: String,
}
#[derive(CandidType, Deserialize)]
pub struct WithdrawAddress {
  pub address: String,
  pub notes: String,
  pub timestamp: u64,
}
#[derive(CandidType, Deserialize)]
pub struct Account {
  pub principal: Option<Principal>,
  pub pending_withdrawal_amount: Option<u64>,
  pub evm_address: Option<String>,
  pub solana_wallet_address: Option<String>,
  pub coin_balance: Option<u64>,
  pub last_checked_slot: Option<u64>,
  pub pending_pool_time: Option<u64>,
  pub created_at: Option<u64>,
  pub ckusd_subaccount: Option<serde_bytes::ByteBuf>,
  pub bonus_balance: Option<u64>,
  pub solana_ata_address: Option<String>,
  pub icp_bonus_balance: Option<u64>,
  pub pending_withdrawal_time: Option<u64>,
  pub token_atas: Option<Vec<TokenAta>>,
  pub pending_pool_amount_spl: Option<u64>,
  pub withdraw_addresses: Option<Vec<WithdrawAddress>>,
  pub icp_balance: Option<u64>,
}
#[derive(CandidType, Deserialize)]
pub enum Result8 {
  Ok(Account),
  Err(String),
}
#[derive(CandidType, Deserialize)]
pub struct TokenConfig {
  pub value: u64,
  pub token_name: String,
}
#[derive(CandidType, Deserialize)]
pub struct SolanaConsensusConfig {
  pub min: u8,
  pub total: Option<u8>,
  pub commitment_level: CommitmentLevel,
}
#[derive(CandidType, Deserialize)]
pub struct AccountConfigs {
  pub ens: Option<Vec<(String, u8)>>,
  pub min_withdraw_amount: Option<u64>,
  pub token_withdrawal_fees: Option<Vec<TokenConfig>>,
  pub solana_high_consensus: Option<SolanaConsensusConfig>,
  pub max_pengding_pool_time: Option<u64>,
  pub solana_usd_mint_address: Option<String>,
  pub solana_middle_consensus: Option<SolanaConsensusConfig>,
  pub withdrawal_fee: Option<u64>,
  pub min_deposit_amount: Option<u64>,
  pub solana_low_consensus: Option<SolanaConsensusConfig>,
  pub token_min_withdraw_amounts: Option<Vec<TokenConfig>>,
  pub spl_pool_threshold: Option<u64>,
  pub ata_creation_fee_usdc: Option<u64>,
  pub exchange_rate: Option<f64>,
}
#[derive(CandidType, Deserialize)]
pub enum Result9 {
  Ok(AccountConfigs),
  Err(String),
}
#[derive(CandidType, Deserialize)]
pub struct Ledgers {
  pub ledgers: Option<Vec<Ledger>>,
}
#[derive(CandidType, Deserialize)]
pub enum Result10 {
  Ok(Ledgers),
  Err(String),
}
#[derive(CandidType, Deserialize)]
pub enum Result11 {
  Ok(Vec<Principal>),
  Err(String),
}
#[derive(CandidType, Deserialize)]
pub enum CoinType {
  #[serde(rename = "ETH")]
  Eth,
  #[serde(rename = "SOL")]
  Sol,
  #[serde(rename = "WLT")]
  Wlt,
  #[serde(rename = "USDC")]
  Usdc,
  #[serde(rename = "USDT")]
  Usdt,
}
#[derive(CandidType, Deserialize)]
pub struct SolanaAccountInfo {
  pub id: u8,
  pub solana_wallet_address: Option<String>,
  pub solana_chain_balance: Option<Vec<(CoinType, u64)>>,
  pub solana_ata_address: Option<String>,
  pub all_users_chain_balance: Option<u64>,
}
#[derive(CandidType, Deserialize)]
pub enum Result12 {
  Ok(Vec<SolanaAccountInfo>),
  Err(String),
}
#[derive(CandidType, Deserialize)]
pub enum Result13 {
  Ok(f64),
  Err(String),
}
#[derive(CandidType, Deserialize)]
pub struct AccountBalances {
  pub solana_ata_balance: Option<u64>,
  pub ckusdt_balance: Option<(candid::Nat, u8)>,
  pub wlt_balance: Option<u64>,
}
#[derive(CandidType, Deserialize)]
pub enum Result14 {
  Ok(AccountBalances),
  Err(String),
}
#[derive(CandidType, Deserialize)]
pub enum RealSpendingType1 {
  BonusSpender,
  AirdropSpender,
  LotterySpender,
}
#[derive(CandidType, Deserialize)]
pub struct SpendingRankEntry {
  pub real_spending: f64,
  pub principal_id: String,
}
#[derive(CandidType, Deserialize)]
pub struct SpendingRankResult {
  pub rankings: Vec<SpendingRankEntry>,
  pub sum_usd: f64,
  pub total_users_with_spending: candid::Nat,
}
#[derive(CandidType, Deserialize)]
pub enum Result15 {
  Ok(SpendingRankResult),
  Err(String),
}
#[derive(CandidType, Deserialize)]
pub struct SigStatus {
  pub err: Option<String>,
  pub signature: String,
  pub block_time: Option<i64>,
  pub slot: u64,
  pub confirmation_status: Option<String>,
}
#[derive(CandidType, Deserialize)]
pub enum Result16 {
  Ok(Vec<SigStatus>),
  Err(String),
}
#[derive(CandidType, Deserialize)]
pub enum Result17 {
  Ok(candid::Nat),
  Err(String),
}
#[derive(CandidType, Deserialize)]
pub struct SolanaTokenBalance {
  pub updated_at: u64,
  pub token: String,
  pub balance: u64,
}
#[derive(CandidType, Deserialize)]
pub struct AccountVo {
  pub principal: Option<Principal>,
  pub evm_address: Option<String>,
  pub solana_wallet_address: Option<String>,
  pub coin_balance: Option<u64>,
  pub last_checked_slot: Option<u64>,
  pub pending_pool_time: Option<u64>,
  pub created_at: Option<u64>,
  pub ckusd_subaccount: Option<serde_bytes::ByteBuf>,
  pub solana_token_balances: Option<Vec<SolanaTokenBalance>>,
  pub bonus_balance: Option<u64>,
  pub solana_ata_address: Option<String>,
  pub icp_bonus_balance: Option<u64>,
  pub token_atas: Option<Vec<TokenAta>>,
  pub pending_pool_amount_spl: Option<u64>,
  pub withdraw_addresses: Option<Vec<WithdrawAddress>>,
  pub icp_balance: Option<u64>,
}
#[derive(CandidType, Deserialize)]
pub enum Result18 {
  Ok(AccountVo),
  Err(String),
}
#[derive(CandidType, Deserialize)]
pub struct TokenAmountVo {
  pub decimals: u8,
  pub ui_amount: Option<f64>,
  pub ui_amount_string: String,
  pub amount: String,
}
#[derive(CandidType, Deserialize)]
pub enum Result19 {
  Ok(TokenAmountVo),
  Err(String),
}
#[derive(CandidType, Deserialize)]
pub struct TransactionDetails {
  pub fee: u64,
  pub token_amount: u64,
  pub block_time: Option<i64>,
  pub slot: u64,
  pub to_address: String,
  pub from_address: String,
  pub success: bool,
}
#[derive(CandidType, Deserialize)]
pub enum Result20 {
  Ok(TransactionDetails),
  Err(String),
}
#[derive(CandidType, Deserialize)]
pub struct UserInfo {
  pub account: Option<Account>,
  pub ledgers: Option<Ledgers>,
}
#[derive(CandidType, Deserialize)]
pub enum Result21 {
  Ok(UserInfo),
  Err(String),
}
#[derive(CandidType, Deserialize)]
pub struct UserInfoWithIcp {
  pub icp: Option<i64>,
  pub usd: Option<i64>,
  pub icp_rate: Option<f64>,
  pub gcoin: Option<i64>,
  pub bonus: Option<i64>,
  pub ledgers: Option<Ledgers>,
  pub gcoin_rate: Option<f64>,
}
#[derive(CandidType, Deserialize)]
pub enum Result22 {
  Ok(UserInfoWithIcp),
  Err(String),
}
#[derive(CandidType, Deserialize)]
pub struct UserRolePermissionVo {
  pub permission_codes: Vec<String>,
  pub principal_id: String,
  pub role_codes: Vec<String>,
  pub is_controller: bool,
}
#[derive(CandidType, Deserialize)]
pub enum Result23 {
  Ok(Vec<Vec<Principal>>),
  Err(String),
}
#[derive(CandidType, Deserialize)]
pub enum SortBy {
  UpdatedAt,
  Balance,
}
#[derive(CandidType, Deserialize)]
pub enum SortOrder {
  Asc,
  Desc,
}
#[derive(CandidType, Deserialize)]
pub struct BalanceQueryResult {
  pub updated_at: u64,
  pub principal: Principal,
  pub token: String,
  pub balance: u64,
}
#[derive(CandidType, Deserialize)]
pub struct BalancesResponse {
  pub total: u64,
  pub balances: Vec<BalanceQueryResult>,
}
#[derive(CandidType, Deserialize)]
pub enum Result24 {
  Ok(BalancesResponse),
  Err(String),
}
#[derive(CandidType, Deserialize)]
pub enum Result25 {
  Ok(u64),
  Err(String),
}
#[derive(CandidType, Deserialize)]
pub enum AddressType {
  #[serde(rename = "WITHDRAW")]
  Withdraw,
  #[serde(rename = "DEPOSIT")]
  Deposit,
}
#[derive(CandidType, Deserialize)]
pub enum Result26 {
  Ok(Vec<Account>),
  Err(String),
}
#[derive(CandidType, Deserialize)]
pub struct EventLogs {
  pub logs: Option<Vec<EventLog>>,
}
#[derive(CandidType, Deserialize)]
pub enum Result27 {
  Ok(EventLogs),
  Err(String),
}
#[derive(CandidType, Deserialize)]
pub struct EventStatUser {
  pub amount_total: i64,
  pub crypto: Crypto,
  pub principal_id: String,
}
#[derive(CandidType, Deserialize)]
pub struct QueryEventsByTypeResult {
  pub rankings: Vec<EventStatUser>,
  pub total_users: u64,
  pub sum_usd: f64,
}
#[derive(CandidType, Deserialize)]
pub enum Result28 {
  Ok(QueryEventsByTypeResult),
  Err(String),
}
#[derive(CandidType, Deserialize)]
pub struct FrequentPlayUser {
  pub total_spending: f64,
  pub play_count: u64,
  pub principal_id: String,
}
#[derive(CandidType, Deserialize)]
pub struct QueryFrequentPlayUsersResult {
  pub rankings: Vec<FrequentPlayUser>,
  pub total_users: u64,
  pub sum_usd: f64,
}
#[derive(CandidType, Deserialize)]
pub enum Result29 {
  Ok(QueryFrequentPlayUsersResult),
  Err(String),
}
#[derive(CandidType, Deserialize)]
pub struct HighDepositUser {
  pub deposit_count: u64,
  pub principal_id: String,
  pub total_deposit: f64,
}
#[derive(CandidType, Deserialize)]
pub struct HighDepositUsersResult {
  pub rankings: Vec<HighDepositUser>,
  pub total_users: u64,
  pub sum_usd: f64,
}
#[derive(CandidType, Deserialize)]
pub enum Result30 {
  Ok(HighDepositUsersResult),
  Err(String),
}
#[derive(CandidType, Deserialize)]
pub struct AccountStatNoCountEntry {
  pub principal_id: String,
  pub amount: f64,
}
#[derive(CandidType, Deserialize)]
pub struct AccountsStatsNoCountResult {
  pub total: u64,
  pub rankings: Vec<AccountStatNoCountEntry>,
  pub sum_usd: f64,
}
#[derive(CandidType, Deserialize)]
pub enum Result31 {
  Ok(AccountsStatsNoCountResult),
  Err(String),
}
#[derive(CandidType, Deserialize)]
pub enum QueryType {
  #[serde(rename = "BONUS")]
  Bonus,
  #[serde(rename = "WITHDRAW")]
  Withdraw { mode: u64 },
}
#[derive(CandidType, Deserialize)]
pub struct AccountStatEntry {
  pub count: Option<u64>,
  pub timestamp: Option<i64>,
  pub principal_id: String,
  pub amount: f64,
}
#[derive(CandidType, Deserialize)]
pub struct AccountsStatsResult {
  pub total: u64,
  pub rankings: Vec<AccountStatEntry>,
  pub sum_usd: f64,
}
#[derive(CandidType, Deserialize)]
pub enum Result32 {
  Ok(AccountsStatsResult),
  Err(String),
}
#[derive(CandidType, Deserialize)]
pub struct UserAllInfo {
  pub solana_wallet_address: Option<String>,
  pub last_play_time: Option<u64>,
  pub wallet_balance_gcoin: Option<f64>,
  pub sum_deposit_usdc: Option<f64>,
  pub playtimes: Option<u64>,
  pub sum_withdraw_usdc: Option<f64>,
  pub sum_bonus_gcoin: Option<f64>,
  pub solana_ata_address: Option<String>,
  pub bonus_balance_gcoin: Option<f64>,
  pub account_withdrawal_addresses: Option<Vec<WithdrawAddress>>,
}
#[derive(CandidType, Deserialize)]
pub enum Result33 {
  Ok(UserAllInfo),
  Err(String),
}
#[derive(CandidType, Deserialize)]
pub enum SortField {
  Withdraw,
  Coin,
  Deposit,
  Play,
}
#[derive(CandidType, Deserialize)]
pub enum WithdrawalTxStatus {
  Failed(String),
  Success,
  Processing,
}
#[derive(CandidType, Deserialize)]
pub struct WithdrawalTxRecord {
  pub status: WithdrawalTxStatus,
  pub updated_at: u64,
  pub signature: Option<String>,
  pub tx_id: u64,
  pub user: Principal,
  pub created_at: u64,
  pub recipient_address: String,
  pub amount: u64,
}
#[derive(CandidType, Deserialize)]
pub enum Result34 {
  Ok(WithdrawalTxRecord),
  Err(String),
}
#[derive(CandidType, Deserialize)]
pub enum Result35 {
  Ok(f64, f64),
  Err(String),
}
#[derive(CandidType, Deserialize)]
pub enum Result36 {
  Ok(bool),
  Err(String),
}
#[derive(CandidType, Deserialize)]
pub struct AccountConfigsUpdateDto {
  pub ens: Option<Option<Vec<(String, u8)>>>,
  pub min_withdraw_amount: Option<Option<u64>>,
  pub token_withdrawal_fees: Option<Option<Vec<TokenConfig>>>,
  pub solana_high_consensus: Option<Option<SolanaConsensusConfig>>,
  pub max_pengding_pool_time: Option<Option<u64>>,
  pub solana_usd_mint_address: Option<Option<String>>,
  pub solana_middle_consensus: Option<Option<SolanaConsensusConfig>>,
  pub withdrawal_fee: Option<Option<u64>>,
  pub min_deposit_amount: Option<Option<u64>>,
  pub solana_low_consensus: Option<Option<SolanaConsensusConfig>>,
  pub token_min_withdraw_amounts: Option<Option<Vec<TokenConfig>>>,
  pub spl_transfer_threshold: Option<Option<u64>>,
  pub ata_creation_fee_usdc: Option<Option<u64>>,
  pub exchange_rate: Option<Option<f64>>,
}
#[derive(CandidType, Deserialize)]
pub struct DictItemVo {
  pub value: String,
  pub sort: u16,
  pub description: String,
  pub label: String,
}
#[derive(CandidType, Deserialize)]
pub struct DictVo {
  pub id: u64,
  pub code: String,
  pub name: String,
  pub description: String,
  pub items: Vec<DictItemVo>,
}
#[derive(CandidType, Deserialize)]
pub struct SystemConfig {
  pub dicts: Vec<DictVo>,
  pub user_role_permissions: Vec<UserRolePermissionVo>,
}

pub struct Service(pub Principal);
impl Service {
  pub async fn add_event_log_unified(&self, arg0: String, arg1: EventLog) -> Result<(Result_,)> {
    ic_cdk::call(self.0, "add_event_log_unified", (arg0, arg1)).await
  }
  pub async fn add_sol_transfer_record(&self, arg0: Principal) -> Result<(Result_,)> {
    ic_cdk::call(self.0, "add_sol_transfer_record", (arg0,)).await
  }
  pub async fn add_user_ledger_from_paycenter(&self, arg0: Principal, arg1: LedgerV5) -> Result<(Result_,)> {
    ic_cdk::call(self.0, "add_user_ledger_from_paycenter", (arg0, arg1)).await
  }
  pub async fn add_user_ledger_record(&self, arg0: Principal, arg1: Ledger) -> Result<(Result_,)> {
    ic_cdk::call(self.0, "add_user_ledger_record", (arg0, arg1)).await
  }
  pub async fn add_withdraw_address(&self, arg0: String, arg1: String) -> Result<(Result1,)> {
    ic_cdk::call(self.0, "add_withdraw_address", (arg0, arg1)).await
  }
  pub async fn associated_token_account(&self, arg0: Option<Principal>, arg1: String) -> Result<(String,)> {
    ic_cdk::call(self.0, "associated_token_account", (arg0, arg1)).await
  }
  pub async fn batch_query_sol_deposit_addresses(&self, arg0: String) -> Result<(Result2,)> {
    ic_cdk::call(self.0, "batch_query_sol_deposit_addresses", (arg0,)).await
  }
  pub async fn calculate_ata_address(&self, arg0: String, arg1: String) -> Result<(Result1,)> {
    ic_cdk::call(self.0, "calculate_ata_address", (arg0, arg1)).await
  }
  pub async fn check_ata_exists(&self, arg0: Option<String>, arg1: Option<String>, arg2: Option<String>) -> Result<(Result3,)> {
    ic_cdk::call(self.0, "check_ata_exists", (arg0, arg1, arg2)).await
  }
  pub async fn check_fund_and_play_conditions(&self, arg0: Vec<ConditionNew>, arg1: Principal) -> Result<(Result_,)> {
    ic_cdk::call(self.0, "check_fund_and_play_conditions", (arg0, arg1)).await
  }
  pub async fn check_user_deposit_over_threshold(&self, arg0: String, arg1: i64, arg2: i64, arg3: f64) -> Result<(Result4,)> {
    ic_cdk::call(self.0, "check_user_deposit_over_threshold", (arg0, arg1, arg2, arg3)).await
  }
  pub async fn check_user_play_over_threshold(&self, arg0: String, arg1: i64, arg2: i64, arg3: u64) -> Result<(Result5,)> {
    ic_cdk::call(self.0, "check_user_play_over_threshold", (arg0, arg1, arg2, arg3)).await
  }
  pub async fn check_user_spent_over_threshold(
    &self,
    arg0: String,
    arg1: i64,
    arg2: i64,
    arg3: f64,
    arg4: RealSpendingType,
    arg5: Option<i64>,
  ) -> Result<(Result6,)> {
    ic_cdk::call(self.0, "check_user_spent_over_threshold", (arg0, arg1, arg2, arg3, arg4, arg5)).await
  }
  pub async fn clear_accounts(&self) -> Result<(Result_,)> {
    ic_cdk::call(self.0, "clear_accounts", ()).await
  }
  pub async fn create_ata_for_main_account(&self, arg0: bool, arg1: Option<String>) -> Result<(Result_,)> {
    ic_cdk::call(self.0, "create_ata_for_main_account", (arg0, arg1)).await
  }
  pub async fn del_withdraw_address(&self, arg0: String) -> Result<(Result1,)> {
    ic_cdk::call(self.0, "del_withdraw_address", (arg0,)).await
  }
  pub async fn delete_account(&self, arg0: Principal) -> Result<(Result_,)> {
    ic_cdk::call(self.0, "delete_account", (arg0,)).await
  }
  pub async fn delete_backup_solana_balance(&self, arg0: u8) -> Result<(Result7,)> {
    ic_cdk::call(self.0, "delete_backup_solana_balance", (arg0,)).await
  }
  pub async fn delete_ledgers(&self, arg0: Principal) -> Result<(Result_,)> {
    ic_cdk::call(self.0, "delete_ledgers", (arg0,)).await
  }
  pub async fn delete_solana_balance(&self, arg0: Principal) -> Result<(Result7,)> {
    ic_cdk::call(self.0, "delete_solana_balance", (arg0,)).await
  }
  pub async fn deposit(&self, arg0: Option<u64>, arg1: String) -> Result<(Result8,)> {
    ic_cdk::call(self.0, "deposit", (arg0, arg1)).await
  }
  pub async fn fix_user_ledger_timestamps(&self, arg0: Principal) -> Result<(Result1,)> {
    ic_cdk::call(self.0, "fix_user_ledger_timestamps", (arg0,)).await
  }
  pub async fn get_account_configs(&self) -> Result<(Result9,)> {
    ic_cdk::call(self.0, "get_account_configs", ()).await
  }
  pub async fn get_account_ledger(&self, arg0: Principal) -> Result<(Result10,)> {
    ic_cdk::call(self.0, "get_account_ledger", (arg0,)).await
  }
  pub async fn get_all_migrated_users(&self) -> Result<(Vec<Principal>,)> {
    ic_cdk::call(self.0, "get_all_migrated_users", ()).await
  }
  pub async fn get_all_sol_transfer_records(&self) -> Result<(Result11,)> {
    ic_cdk::call(self.0, "get_all_sol_transfer_records", ()).await
  }
  pub async fn get_all_solana_accounts(&self, arg0: bool) -> Result<(Result12,)> {
    ic_cdk::call(self.0, "get_all_solana_accounts", (arg0,)).await
  }
  pub async fn get_gcoin_exchange_rate(&self) -> Result<(Result13,)> {
    ic_cdk::call(self.0, "get_gcoin_exchange_rate", ()).await
  }
  pub async fn get_my_account_balances(&self) -> Result<(Result14,)> {
    ic_cdk::call(self.0, "get_my_account_balances", ()).await
  }
  pub async fn get_real_spending_ranking(
    &self,
    arg0: i64,
    arg1: i64,
    arg2: Option<f64>,
    arg3: Option<f64>,
    arg4: RealSpendingType1,
  ) -> Result<(Result15,)> {
    ic_cdk::call(self.0, "get_real_spending_ranking", (arg0, arg1, arg2, arg3, arg4)).await
  }
  pub async fn get_signatures_for_address(&self, arg0: String, arg1: Option<u64>, arg2: Option<u32>) -> Result<(Result16,)> {
    ic_cdk::call(self.0, "get_signatures_for_address", (arg0, arg1, arg2)).await
  }
  pub async fn get_sol_balance_admin(&self, arg0: String) -> Result<(Result17,)> {
    ic_cdk::call(self.0, "get_sol_balance_admin", (arg0,)).await
  }
  pub async fn get_spec_subaccount_info(&self, arg0: Option<Principal>) -> Result<(Result18,)> {
    ic_cdk::call(self.0, "get_spec_subaccount_info", (arg0,)).await
  }
  pub async fn get_spl_token_balance_admin(&self, arg0: Option<String>, arg1: Option<String>) -> Result<(Result19,)> {
    ic_cdk::call(self.0, "get_spl_token_balance_admin", (arg0, arg1)).await
  }
  pub async fn get_subaccount_info(&self) -> Result<(Result18,)> {
    ic_cdk::call(self.0, "get_subaccount_info", ()).await
  }
  pub async fn get_transaction(&self, arg0: String) -> Result<(Result20,)> {
    ic_cdk::call(self.0, "get_transaction", (arg0,)).await
  }
  pub async fn get_user_info(&self, arg0: Principal) -> Result<(Result21,)> {
    ic_cdk::call(self.0, "get_user_info", (arg0,)).await
  }
  pub async fn get_user_info_with_icp(&self, arg0: Principal) -> Result<(Result22,)> {
    ic_cdk::call(self.0, "get_user_info_with_icp", (arg0,)).await
  }
  pub async fn get_user_role_permissions(&self) -> Result<(Vec<UserRolePermissionVo>,)> {
    ic_cdk::call(self.0, "get_user_role_permissions", ()).await
  }
  pub async fn get_users_by_withdraw_address(&self, arg0: String) -> Result<(Result11,)> {
    ic_cdk::call(self.0, "get_users_by_withdraw_address", (arg0,)).await
  }
  pub async fn get_users_by_withdraw_addresses(&self, arg0: Vec<String>) -> Result<(Result23,)> {
    ic_cdk::call(self.0, "get_users_by_withdraw_addresses", (arg0,)).await
  }
  pub async fn init_main_account(&self) -> Result<(Result_,)> {
    ic_cdk::call(self.0, "init_main_account", ()).await
  }
  pub async fn list_all_balances(&self, arg0: SortBy, arg1: SortOrder, arg2: u64, arg3: u64) -> Result<(Result24,)> {
    ic_cdk::call(self.0, "list_all_balances", (arg0, arg1, arg2, arg3)).await
  }
  pub async fn play(&self, arg0: Principal, arg1: u64, arg2: Option<u8>, arg3: u64, arg4: u64, arg5: Option<u64>, arg6: u64) -> Result<(Result25,)> {
    ic_cdk::call(self.0, "play", (arg0, arg1, arg2, arg3, arg4, arg5, arg6)).await
  }
  pub async fn query_account_all_info(&self, arg0: String) -> Result<(Result22,)> {
    ic_cdk::call(self.0, "query_account_all_info", (arg0,)).await
  }
  pub async fn query_account_by_address(&self, arg0: String, arg1: AddressType) -> Result<(Result8,)> {
    ic_cdk::call(self.0, "query_account_by_address", (arg0, arg1)).await
  }
  pub async fn query_all_accounts(&self) -> Result<(Result26,)> {
    ic_cdk::call(self.0, "query_all_accounts", ()).await
  }
  pub async fn query_event_logs(&self, arg0: Option<Principal>) -> Result<(Result27,)> {
    ic_cdk::call(self.0, "query_event_logs", (arg0,)).await
  }
  pub async fn query_event_logs_unified(&self, arg0: String) -> Result<(Result27,)> {
    ic_cdk::call(self.0, "query_event_logs_unified", (arg0,)).await
  }
  pub async fn query_events_by_type(&self, arg0: Event, arg1: i64, arg2: i64) -> Result<(Result28,)> {
    ic_cdk::call(self.0, "query_events_by_type", (arg0, arg1, arg2)).await
  }
  pub async fn query_frequent_play_users(&self, arg0: i64, arg1: i64, arg2: u64) -> Result<(Result29,)> {
    ic_cdk::call(self.0, "query_frequent_play_users", (arg0, arg1, arg2)).await
  }
  pub async fn query_high_deposit_users(&self, arg0: i64, arg1: i64, arg2: f64) -> Result<(Result30,)> {
    ic_cdk::call(self.0, "query_high_deposit_users", (arg0, arg1, arg2)).await
  }
  pub async fn query_identity_mapping(&self) -> Result<(Vec<(Principal, Principal)>,)> {
    ic_cdk::call(self.0, "query_identity_mapping", ()).await
  }
  pub async fn query_positive_balance_accounts(&self) -> Result<(Result31,)> {
    ic_cdk::call(self.0, "query_positive_balance_accounts", ()).await
  }
  pub async fn query_spl_operations_cycle_costs(&self) -> Result<(String,)> {
    ic_cdk::call(self.0, "query_spl_operations_cycle_costs", ()).await
  }
  pub async fn query_stats_by_type(&self, arg0: QueryType, arg1: i64, arg2: i64) -> Result<(Result32,)> {
    ic_cdk::call(self.0, "query_stats_by_type", (arg0, arg1, arg2)).await
  }
  pub async fn query_user_account_all_info(&self, arg0: Principal) -> Result<(Result33,)> {
    ic_cdk::call(self.0, "query_user_account_all_info", (arg0,)).await
  }
  pub async fn query_users_by_field(
    &self,
    arg0: Option<u64>,
    arg1: Option<u64>,
    arg2: SortField,
    arg3: SortOrder,
    arg4: u64,
    arg5: u64,
  ) -> Result<(Result26,)> {
    ic_cdk::call(self.0, "query_users_by_field", (arg0, arg1, arg2, arg3, arg4, arg5)).await
  }
  pub async fn query_withdraw_address_sorted(&self) -> Result<(Vec<WithdrawAddress>,)> {
    ic_cdk::call(self.0, "query_withdraw_address_sorted", ()).await
  }
  pub async fn query_withdrawal_status(&self, arg0: u64) -> Result<(Result34,)> {
    ic_cdk::call(self.0, "query_withdrawal_status", (arg0,)).await
  }
  pub async fn redeem_points(&self, arg0: Principal, arg1: u64, arg2: Crypto, arg3: String, arg4: f64) -> Result<(Result35,)> {
    ic_cdk::call(self.0, "redeem_points", (arg0, arg1, arg2, arg3, arg4)).await
  }
  pub async fn remove_ata_from_cache(&self, arg0: String) -> Result<(Result36,)> {
    ic_cdk::call(self.0, "remove_ata_from_cache", (arg0,)).await
  }
  pub async fn send_sol_admin(&self, arg0: Option<Principal>, arg1: String, arg2: candid::Nat) -> Result<(Result25,)> {
    ic_cdk::call(self.0, "send_sol_admin", (arg0, arg1, arg2)).await
  }
  pub async fn send_spl_token_admin(&self, arg0: Option<Principal>, arg1: Option<u8>, arg2: String, arg3: candid::Nat) -> Result<(Result25,)> {
    ic_cdk::call(self.0, "send_spl_token_admin", (arg0, arg1, arg2, arg3)).await
  }
  pub async fn setup_subscribe(&self, arg0: Principal) -> Result<(String,)> {
    ic_cdk::call(self.0, "setup_subscribe", (arg0,)).await
  }
  pub async fn sign_sol_transfer(
    &self,
    arg0: Option<Principal>,
    arg1: Option<u8>,
    arg2: String,
    arg3: u64,
    arg4: String,
    arg5: String,
    arg6: bool,
  ) -> Result<(Result1,)> {
    ic_cdk::call(self.0, "sign_sol_transfer", (arg0, arg1, arg2, arg3, arg4, arg5, arg6)).await
  }
  pub async fn subscribe_identity_mapping_change(&self, arg0: Principal) -> Result<(String,)> {
    ic_cdk::call(self.0, "subscribe_identity_mapping_change", (arg0,)).await
  }
  pub async fn update_account_bonus(
    &self,
    arg0: Principal,
    arg1: i64,
    arg2: Crypto,
    arg3: Option<u64>,
    arg4: String,
    arg5: Option<u64>,
    arg6: Option<u64>,
    arg7: Option<Vec<AddInfoStr>>,
  ) -> Result<(Result25,)> {
    ic_cdk::call(self.0, "update_account_bonus", (arg0, arg1, arg2, arg3, arg4, arg5, arg6, arg7)).await
  }
  pub async fn update_account_configs(&self, arg0: AccountConfigsUpdateDto) -> Result<(Result9,)> {
    ic_cdk::call(self.0, "update_account_configs", (arg0,)).await
  }
  pub async fn update_icp_usd_rate(&self, arg0: f64) -> Result<(Result_,)> {
    ic_cdk::call(self.0, "update_icp_usd_rate", (arg0,)).await
  }
  pub async fn update_identity_mapping(&self, arg0: Vec<(Principal, Principal)>) -> Result<()> {
    ic_cdk::call(self.0, "update_identity_mapping", (arg0,)).await
  }
  pub async fn update_system_configs(&self, arg0: SystemConfig) -> Result<()> {
    ic_cdk::call(self.0, "update_system_configs", (arg0,)).await
  }
  pub async fn update_withdraw_address(&self, arg0: String, arg1: String) -> Result<(Result1,)> {
    ic_cdk::call(self.0, "update_withdraw_address", (arg0, arg1)).await
  }
  pub async fn win(&self, arg0: Principal, arg1: u64, arg2: Option<u8>, arg3: u64, arg4: u64, arg5: u64, arg6: u64) -> Result<(Result25,)> {
    ic_cdk::call(self.0, "win", (arg0, arg1, arg2, arg3, arg4, arg5, arg6)).await
  }
  pub async fn withdraw(&self, arg0: String, arg1: u64, arg2: String, arg3: String, arg4: Option<String>) -> Result<(Result25,)> {
    ic_cdk::call(self.0, "withdraw", (arg0, arg1, arg2, arg3, arg4)).await
  }
}
