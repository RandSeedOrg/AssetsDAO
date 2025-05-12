// This is an experimental feature to generate Rust binding from Candid.
// You may want to manually adjust some of the types.
#![allow(dead_code, unused_imports)]
use candid::{self, CandidType, Deserialize, Principal, Encode, Decode};
use ic_cdk::api::call::CallResult as Result;

#[derive(CandidType, Deserialize)]
pub enum Result2 {
  #[serde(rename="ok")]
  Ok(u64),
  #[serde(rename="err")]
  Err(String),
}

#[derive(CandidType, Deserialize)]
pub enum TransferError {
  GenericError{ message: String, error_code: candid::Nat },
  TemporarilyUnavailable,
  BadBurn{ min_burn_amount: candid::Nat },
  Duplicate{ duplicate_of: candid::Nat },
  BadFee{ expected_fee: candid::Nat },
  CreatedInFuture{ ledger_time: u64 },
  TooOld,
  InsufficientFunds{ balance: candid::Nat },
}

#[derive(CandidType, Deserialize)]
pub enum TransferResult { Ok(candid::Nat), Err(TransferError) }

#[derive(CandidType, Deserialize)]
pub struct StakeResult { pub pay_center_tx_id: u64, pub onchain_tx_id: u64 }

#[derive(CandidType, Deserialize)]
pub enum Result3 {
  #[serde(rename="ok")]
  Ok(StakeResult),
  #[serde(rename="err")]
  Err(String),
}

pub struct Service(pub Principal);
impl Service {
  pub async fn dissolve(
    &self,
    arg0: Principal,
    arg1: u64,
    arg2: u64,
    arg3: String,
    arg4: u64,
  ) -> Result<(Result2,)> {
    ic_cdk::call(self.0, "dissolve", (arg0,arg1,arg2,arg3,arg4,)).await
  }
  pub async fn get_address(&self) -> Result<(String,)> {
    ic_cdk::call(self.0, "get_address", ()).await
  }
  pub async fn receive_early_unstake_penalty(
    &self,
    arg0: Principal,
    arg1: u64,
    arg2: u64,
    arg3: u64,
    arg4: u64,
  ) -> Result<(Result2,)> {
    ic_cdk::call(self.0, "receive_early_unstake_penalty", (
      arg0,arg1,arg2,arg3,arg4,
    )).await
  }
  pub async fn stake(
    &self,
    arg0: Principal,
    arg1: u64,
    arg2: String,
    arg3: u64,
    arg4: u64,
  ) -> Result<(Result3,)> {
    ic_cdk::call(self.0, "stake", (arg0,arg1,arg2,arg3,arg4,)).await
  }
  pub async fn update_account_bonus(
    &self,
    arg0: Principal,
    arg1: f64,
    arg2: Option<u64>,
    arg3: String,
    arg4: Option<u64>,
    arg5: Option<u64>,
    arg6: Vec<String>,
  ) -> Result<(Result2,)> {
    ic_cdk::call(self.0, "update_account_bonus", (
      arg0,arg1,arg2,arg3,arg4,arg5,arg6,
    )).await
  }
}
