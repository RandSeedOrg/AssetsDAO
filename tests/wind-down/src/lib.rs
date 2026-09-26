//! Runs real staking WASM with isolated Ledger, Index, and pay-center substitutes.
#[cfg(test)]
mod tests {
  use candid::{CandidType, Decode, Encode, Principal};
  use pocket_ic::{PocketIc, PocketIcBuilder};
  use serde::Deserialize;
  use std::{
    fs,
    path::PathBuf,
    time::{Duration, SystemTime},
  };
  #[derive(CandidType, Deserialize, Debug)]
  struct Progress {
    state: String,
    examined_transactions: u64,
    candidate_blocks: Vec<u64>,
  }
  #[derive(CandidType, Deserialize, Debug)]
  struct Status {
    phase: String,
    cursor: u64,
    completed_account_count: u64,
    completed_amount: u64,
    recovery: Option<Progress>,
  }
  #[derive(CandidType, Deserialize, Debug)]
  struct Batch {
    status: Status,
    processed_count: u32,
  }
  struct Test {
    pic: PocketIc,
    staking: Principal,
    ledger: Principal,
    pay: Principal,
    account: String,
  }
  fn wasm(name: &str, default: &str) -> Vec<u8> {
    fs::read(std::env::var(name).unwrap_or(default.into())).unwrap()
  }
  fn staking_wasm() -> Vec<u8> {
    wasm("STAKING_FIXTURE_WASM", "/tmp/wind-down-fixture-target/wasm32-unknown-unknown/release/staking.wasm")
  }
  impl Test {
    fn call(&self, id: Principal, name: &str, args: &str) -> Vec<u8> {
      self
        .pic
        .update_call(id, Principal::anonymous(), name, candid_parser::parse_idl_args(args).unwrap().to_bytes().unwrap())
        .unwrap()
    }
    fn new(release: bool, dissolve: bool, released: bool, extra: u64, duplicate: bool, tip: u64) -> Self {
      let binary = std::env::var("POCKET_IC_BIN").expect("Set POCKET_IC_BIN to PocketIC server 11");
      let pic = PocketIcBuilder::new()
        .with_server_binary(PathBuf::from(binary))
        .with_nns_subnet()
        .with_application_subnet()
        .build();
      pic.set_time((SystemTime::UNIX_EPOCH + Duration::from_secs(1_800_000_000)).into());
      let ledger = Principal::from_text("ryjl3-tyaaa-aaaaa-aaaba-cai").unwrap();
      let index = Principal::from_text("qhbym-qaaaa-aaaaa-aaafq-cai").unwrap();
      let pay = pic.create_canister();
      let staking = pic.create_canister();
      let mock = wasm("MOCK_WASM", "/tmp/wind-down-mock-target/wasm32-unknown-unknown/release/wind_down_mock.wasm");
      for id in [ledger, index] {
        pic.create_canister_with_id(None, None, id).unwrap();
      }
      for id in [ledger, index, pay, staking] {
        pic.add_cycles(id, 10_000_000_000_000);
        pic.install_canister(id, if id == staking { staking_wasm() } else { mock.clone() }, Encode!().unwrap(), None);
      }
      let mut t = Self {
        pic,
        staking,
        ledger,
        pay,
        account: String::new(),
      };
      t.configure();
      let args = Encode!(&pay, &released).unwrap();
      let raw = t.pic.update_call(staking, Principal::anonymous(), "fixture_seed", args).unwrap();
      let addresses = Decode!(&raw, Vec<String>).unwrap();
      t.account = addresses[1].clone();
      t.call(ledger,"seed",&format!("(record {{pool=\"{}\";account=\"{}\";pay=\"{}\";release={release};dissolve={dissolve};extra={extra}:nat64;duplicate={duplicate};tip={tip}:nat64}})",addresses[0],addresses[1],addresses[2]));
      t
    }
    fn configure(&self) {
      self.call(self.staking, "fixture_config", &format!("(principal \"{}\")", self.pay));
    }
    fn execute(&self) -> Result<Batch, String> {
      let r = self.call(self.staking, "execute_staking_wind_down_batch", "(3:nat64,20:nat32)");
      Decode!(&r,Result<Batch,String>).unwrap()
    }
    fn status(&self) -> Status {
      let r = self
        .pic
        .query_call(self.staking, Principal::anonymous(), "get_staking_wind_down", Encode!(&3u64).unwrap())
        .unwrap();
      Decode!(&r, Option<Status>).unwrap().unwrap()
    }
    fn counters(&self, id: Principal) -> (u64, u64, u64, u64) {
      let r = self.pic.query_call(id, Principal::anonymous(), "counters", Encode!().unwrap()).unwrap();
      Decode!(&r, u64, u64, u64, u64).unwrap()
    }
    fn controls(&self, id: Principal, synced: u64, reject: bool, lose: bool) {
      self.call(
        id,
        "controls",
        &format!("(record {{synced={synced}:nat64;reject_index={reject};hide_new=false;lose_pay_reply={lose}}})"),
      );
    }
    fn finish(&self) {
      for _ in 0..30 {
        if self.status().phase == "ResidualPending" {
          return;
        }
        self.execute().unwrap();
      }
      panic!("Did not finish: {:?}", self.status());
    }
    fn state(&self) -> (String, u64, u64, u64) {
      let r = self
        .pic
        .query_call(self.staking, Principal::anonymous(), "fixture_state", Encode!().unwrap())
        .unwrap();
      Decode!(&r, String, u64, u64, u64).unwrap()
    }
  }
  #[test]
  fn orphan_release_and_dissolve_recover_without_either_transfer() {
    let t = Test::new(true, true, false, 0, false, 2000);
    t.finish();
    assert_eq!(t.state(), ("Dissolved".into(), 100, 200, 317_214));
    assert_eq!(t.counters(t.ledger).0, 0);
    assert_eq!(t.counters(t.pay).3, 1);
    assert_eq!(t.status().cursor, 147);
    assert_eq!(t.status().completed_account_count, 17);
  }
  #[test]
  fn released_missing_accounting_is_repaired() {
    let t = Test::new(true, true, true, 0, false, 2000);
    t.finish();
    assert_eq!(t.state().3, 317_214);
    assert_eq!(t.counters(t.ledger).0, 0);
  }
  #[test]
  fn index_lag_then_rejection_never_sends() {
    let t = Test::new(true, true, false, 0, false, 2000);
    t.controls(t.ledger, 2000, false, false);
    assert_eq!(t.execute().unwrap().status.recovery.unwrap().state, "WaitingForIndex");
    assert_eq!(t.counters(t.ledger).0, 0);
    t.controls(t.ledger, 2001, true, false);
    assert!(t.execute().is_err());
    assert_eq!(t.counters(t.ledger).0, 0);
    t.controls(t.ledger, 2001, false, false);
    t.finish();
  }
  #[test]
  fn cross_page_duplicate_is_manual_review() {
    let t = Test::new(true, true, false, 21, true, 2000);
    let error = t.execute().unwrap_err();
    assert!(error.contains("Multiple"), "{error}");
    let recovery = t.status().recovery.unwrap();
    assert_eq!(recovery.state, "ManualReview");
    assert_eq!(recovery.candidate_blocks, vec![100, 1500]);
    assert_eq!(t.counters(t.ledger).0, 0);
  }
  #[test]
  fn bounded_account_pagination_survives_upgrade() {
    let t = Test::new(true, true, false, 125, false, 2_000_000_000);
    let b = t.execute().unwrap();
    assert_eq!(b.processed_count, 0);
    assert_eq!(t.counters(t.ledger).1, 5);
    t.pic.upgrade_canister(t.staking, staking_wasm(), Encode!().unwrap(), None).unwrap();
    t.configure();
    t.finish();
    assert_eq!(t.state().0, "Dissolved");
    assert_eq!(t.counters(t.ledger).0, 0);
    assert!(t.counters(t.ledger).1 <= 14);
  }
  #[test]
  fn trap_after_release_transfer_recovers_without_resending() {
    let t = Test::new(false, false, false, 0, false, 2000);
    t.call(t.staking, "fixture_fault", "(\"Release\")");
    let r = t.pic.update_call(
      t.staking,
      Principal::anonymous(),
      "execute_staking_wind_down_batch",
      Encode!(&3u64, &20u32).unwrap(),
    );
    assert!(r.is_err());
    assert_eq!(t.counters(t.ledger).0, 1);
    t.finish();
    assert_eq!(t.counters(t.ledger).0, 2);
    assert_eq!(t.state().0, "Dissolved");
  }
  #[test]
  fn pay_center_response_loss_does_not_credit_or_transfer_twice() {
    let t = Test::new(true, true, false, 0, false, 2000);
    t.controls(t.pay, 0, false, true);
    t.execute().unwrap();
    assert_eq!(t.counters(t.pay).3, 1);
    t.finish();
    assert_eq!(t.counters(t.pay).3, 1);
    assert_eq!(t.counters(t.ledger).0, 0);
  }

  #[derive(CandidType, Deserialize, Debug)]
  enum PayResult {
    #[serde(rename = "ok")]
    Ok(u64),
    #[serde(rename = "err")]
    Err(String),
  }
  fn pay_args(t: &Test, account: u64) -> Vec<u8> {
    Encode!(&Principal::anonymous(), &100_000_000u64, &200u64, &t.account, &account).unwrap()
  }
  fn pay_init(t: &Test) -> Vec<u8> {
    let id = t.ledger.to_text();
    Encode!(&id, &id, &id, &id, &id).unwrap()
  }
  fn pay_balance(t: &Test) -> candid::Int {
    let raw = t
      .pic
      .query_call(t.pay, Principal::anonymous(), "fixture_balance", Encode!().unwrap())
      .unwrap();
    Decode!(&raw, candid::Int).unwrap()
  }
  #[test]
  fn actual_pay_center_concurrent_replay_and_conflict() {
    let t = Test::new(true, true, false, 0, false, 2000);
    t.pic
      .reinstall_canister(t.pay, wasm("PAY_CENTER_WASM", "/tmp/wind-down-pay-center/current.wasm"), pay_init(&t), None)
      .unwrap();
    t.call(t.pay, "fixture_user", "(false,0:nat)");
    let a = t.pic.submit_call(t.pay, Principal::anonymous(), "dissolve", pay_args(&t, 147)).unwrap();
    let b = t.pic.submit_call(t.pay, Principal::anonymous(), "dissolve", pay_args(&t, 147)).unwrap();
    for id in [a, b] {
      let raw = t.pic.await_call(id).unwrap();
      assert!(matches!(Decode!(&raw, PayResult).unwrap(), PayResult::Ok(200)));
    }
    assert_eq!(pay_balance(&t), candid::Int::from(100_000_000u64));
    let raw = t.pic.update_call(t.pay, Principal::anonymous(), "dissolve", pay_args(&t, 148)).unwrap();
    assert!(matches!(Decode!(&raw, PayResult).unwrap(), PayResult::Err(_)));
    t.finish();
    assert_eq!(pay_balance(&t), candid::Int::from(100_000_000u64));
  }
  #[test]
  fn actual_pay_center_legacy_upgrade_uses_bounded_history_without_recredit() {
    let t = Test::new(true, true, false, 0, false, 2000);
    t.pic
      .reinstall_canister(
        t.pay,
        wasm("PREVIOUS_PAY_CENTER_WASM", "/tmp/wind-down-pay-center/previous.wasm"),
        pay_init(&t),
        None,
      )
      .unwrap();
    t.call(t.pay, "fixture_user", "(true,150:nat)");
    t.pic
      .upgrade_eop_canister(t.pay, wasm("PAY_CENTER_WASM", "/tmp/wind-down-pay-center/current.wasm"), pay_init(&t), None)
      .unwrap();
    let raw = t.pic.update_call(t.pay, Principal::anonymous(), "dissolve", pay_args(&t, 147)).unwrap();
    assert!(matches!(Decode!(&raw,PayResult).unwrap(),PayResult::Err(e) if e.contains("incomplete")));
    t.finish();
    assert_eq!(pay_balance(&t), candid::Int::from(100_000_150u64));
  }
  #[test]
  fn dissolve_callback_trap_is_recovered_without_a_second_payment() {
    let t = Test::new(true, false, false, 0, false, 2000);
    t.call(t.staking, "fixture_fault", "(\"Dissolve\")");
    let r = t.pic.update_call(
      t.staking,
      Principal::anonymous(),
      "execute_staking_wind_down_batch",
      Encode!(&3u64, &20u32).unwrap(),
    );
    assert!(r.is_err());
    t.finish();
    assert_eq!(t.counters(t.ledger).0, 1);
    assert_eq!(t.counters(t.pay).3, 1);
  }
  #[test]
  fn pause_during_batch_does_not_get_overwritten_by_callback() {
    let t = Test::new(true, true, false, 125, false, 2000);
    let running = t
      .pic
      .submit_call(
        t.staking,
        Principal::anonymous(),
        "execute_staking_wind_down_batch",
        Encode!(&3u64, &20u32).unwrap(),
      )
      .unwrap();
    t.pic.tick();
    t.call(t.staking, "pause_staking_wind_down", "(3:nat64)");
    let _ = t.pic.await_call(running);
    assert_eq!(t.status().phase, "Paused");
    assert_eq!(t.counters(t.ledger).0, 0);
    t.finish();
  }

  #[test]
  fn individual_archived_blocks_are_verified_without_range_scanning() {
    let t = Test::new(true, true, false, 0, false, 2000);
    t.call(t.ledger, "fault_mode", "(\"archive\")");
    t.finish();
    assert_eq!(t.counters(t.ledger).2, 4);
    assert_eq!(t.counters(t.ledger).0, 0);
  }
  #[test]
  fn mismatched_index_ledger_is_rejected_before_payment() {
    let t = Test::new(true, true, false, 0, false, 2000);
    t.call(Principal::from_text("qhbym-qaaaa-aaaaa-aaafq-cai").unwrap(), "fault_mode", "(\"wrong-ledger\")");
    assert!(t.execute().unwrap_err().contains("different Ledger"));
    assert_eq!(t.counters(t.ledger).0, 0);
  }
  // Even an incomplete Index returning a false absence must not bypass a fixed
  // transfer intent. The Ledger deduplicates within its window; expiry stops sends.
  fn ambiguous_release() -> Test {
    let t = Test::new(false, false, false, 0, false, 2000);
    t.call(t.staking, "fixture_fault", "(\"Release\")");
    assert!(t
      .pic
      .update_call(
        t.staking,
        Principal::anonymous(),
        "execute_staking_wind_down_batch",
        Encode!(&3u64, &20u32).unwrap()
      )
      .is_err());
    t.call(t.ledger, "fault_mode", "(\"omit-attempt\")");
    t
  }
  #[test]
  fn duplicate_response_reuses_original_block_and_fixed_parameters() {
    let t = ambiguous_release();
    t.finish();
    assert_eq!(t.state().1, 2001);
    assert_eq!(t.counters(t.ledger).0, 3);
    assert_eq!(t.counters(t.pay).3, 1);
  }
  #[test]
  fn expired_ambiguous_intent_never_renews_timestamp_or_resends() {
    let t = ambiguous_release();
    t.pic.advance_time(Duration::from_secs(25 * 3600));
    assert!(t.execute().unwrap_err().contains("expired"));
    assert_eq!(t.status().recovery.unwrap().state, "ManualReview");
    assert_eq!(t.counters(t.ledger).0, 1);
  }
  #[test]
  fn impossible_lifecycle_boundary_does_not_authorize_transfer() {
    let t = Test::new(false, false, false, 0, false, 9);
    assert!(t.execute().unwrap_err().contains("ahead of the Ledger tip"));
    assert_eq!(t.counters(t.ledger).0, 0);
  }
  #[test]
  fn pool_3_remaining_seven_accounts_finish_with_exact_counts_and_balance() {
    let t = Test::new(true, true, false, 0, false, 2000);
    let raw = t.call(t.staking, "fixture_remaining", "()");
    let addresses = Decode!(&raw, Vec<String>).unwrap();
    t.pic
      .update_call(t.ledger, Principal::anonymous(), "seed_remaining", Encode!(&addresses).unwrap())
      .unwrap();
    t.finish();
    assert_eq!(t.status().cursor, 153);
    assert_eq!(t.status().completed_account_count, 23);
    assert_eq!(t.status().completed_amount, 310_600_000);
    assert_eq!(t.state(), ("Dissolved".into(), 100, 200, 317_214));
    assert_eq!(t.counters(t.ledger).0, 6);
    assert_eq!(t.counters(t.pay).3, 7);
  }
}
