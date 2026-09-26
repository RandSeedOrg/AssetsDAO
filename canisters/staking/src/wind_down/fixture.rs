//! Fixture endpoints are compiled only for isolated PocketIC regression tests.
use super::*;
use crate::account::{STAKING_ACCOUNT_MAP, STAKING_POOL_ACCOUNT_INDEX_MAP, STAKING_USER_ACCOUNT_INDEX_MAP};
use types::entities::EntityIndex;
use types::sys::{
  config::{SystemConfig, UserRolePermissionVo},
  dict::transfer_structures::{DictItemVo, DictVo},
};
thread_local! {static FAULT:RefCell<String>=RefCell::new(String::new());}
pub fn take_fault(stage: Stage) -> bool {
  FAULT.with(|f| {
    let mut f = f.borrow_mut();
    if *f == format!("{stage:?}") {
      *f = String::new();
      true
    } else {
      false
    }
  })
}
#[ic_cdk::update]
pub fn fixture_fault(stage: String) {
  assert!(ic_cdk::api::is_controller(&ic_cdk::api::msg_caller()));
  FAULT.with(|f| *f.borrow_mut() = stage);
}
#[ic_cdk::update]
pub async fn fixture_config(pay_center: Principal) {
  let item = DictItemVo {
    label: "Pay center".into(),
    value: pay_center.to_text(),
    description: String::new(),
    sort: 0,
  };
  crate::system_configs::update_system_configs(SystemConfig {
    user_role_permissions: vec![UserRolePermissionVo::new(
      ic_cdk::api::msg_caller().to_text(),
      true,
      vec![],
      vec!["staking::pool::wind_down".into()],
    )],
    dicts: vec![DictVo {
      id: 1,
      name: "test".into(),
      code: "system_config".into(),
      description: String::new(),
      items: vec![item],
    }],
  })
  .await;
}
#[derive(CandidType)]
struct Empty {}
#[ic_cdk::update]
pub fn fixture_seed(pay_center: Principal, released: bool) -> Vec<String> {
  assert!(ic_cdk::api::is_controller(&ic_cdk::api::msg_caller()));
  let empty = Encode!(&Empty {}).unwrap();
  let mut account = Decode!(&empty, StakingAccount).unwrap();
  let address = generate_staking_account_account_identifier(147).to_hex();
  account.id = Some(147);
  account.pool_id = Some(3);
  account.owner = Some(Principal::anonymous().to_text());
  account.address = Some(address.clone());
  account.staked_amount = Some(100_000_000);
  account.stake_account_to_pool_onchain_tx_id = Some(10);
  account.status = Some(if released {
    StakingAccountStatus::Released
  } else {
    StakingAccountStatus::InStake
  });
  if released {
    account.released_amount = Some(100_000_000);
    account.release_onchain_tx_id = Some(100);
  }
  STAKING_ACCOUNT_MAP.with(|m| m.borrow_mut().insert(147, account));
  STAKING_POOL_ACCOUNT_INDEX_MAP.with(|m| m.borrow_mut().insert(3, EntityIndex::new(3, vec![147])));
  let user = Principal::anonymous().to_text();
  STAKING_USER_ACCOUNT_INDEX_MAP.with(|m| m.borrow_mut().insert(user.clone(), EntityIndex::new(user, vec![147])));
  let mut pool = Decode!(&empty, StakingPool).unwrap();
  pool.id = Some(3);
  pool.staked_amount = Some(if released { 0 } else { 100_000_000 });
  pool.staked_user_count = Some(if released { 0 } else { 1 });
  STAKING_POOL_MAP.with(|m| m.borrow_mut().insert(3, pool));
  let pay_address = ic_ledger_types::AccountIdentifier::new(&pay_center, &ic_ledger_types::Subaccount([0; 32])).to_hex();
  save_job(&WindDownJob {
    pool_id: 3,
    phase: WindDownPhase::Paused,
    cursor: 146,
    expected_account_count: 17,
    completed_account_count: 16,
    expected_amount: 230_600_000,
    completed_amount: 130_600_000,
    residual_amount: 0,
    residual_onchain_tx_id: 0,
    residual_pay_center_tx_id: 0,
    fee_dust: 0,
    nns_recovered: true,
    // Exercise byte-wise address validation independent of hex casing.
    pay_center_address: pay_address.to_uppercase(),
    updated_at: ic_cdk::api::time(),
  });
  crate::pool_transaction_record::utils::record_reconciliation_transaction(3, 80_260_000, 0, ic_cdk::api::time()).unwrap();
  vec![generate_staking_pool_account_identifier(3).to_hex(), address, pay_address]
}
#[ic_cdk::query]
pub fn fixture_state() -> (String, u64, u64, u64) {
  let a = StakingAccount::query_by_id(147).unwrap();
  (
    format!("{:?}", a.get_status()),
    a.get_release_onchain_tx_id(),
    a.get_dissolve_onchain_tx_id(),
    crate::pool_transaction_record::utils::get_virtual_balance(3),
  )
}

#[ic_cdk::update]
pub fn fixture_remaining() -> Vec<String> {
  assert!(ic_cdk::api::is_controller(&ic_cdk::api::msg_caller()));
  let empty = Encode!(&Empty {}).unwrap();
  let mut addresses = Vec::new();
  for id in 148..154 {
    let mut account = Decode!(&empty, StakingAccount).unwrap();
    let address = generate_staking_account_account_identifier(id).to_hex();
    account.id = Some(id);
    account.pool_id = Some(3);
    account.owner = Some(Principal::anonymous().to_text());
    account.address = Some(address.clone());
    account.status = Some(StakingAccountStatus::Released);
    account.staked_amount = Some(if id == 148 { 30_000_000 } else { 10_000_000 });
    account.released_amount = account.staked_amount;
    account.release_onchain_tx_id = Some(300 + id - 148);
    STAKING_ACCOUNT_MAP.with(|m| m.borrow_mut().insert(id, account));
    addresses.push(address);
  }
  STAKING_POOL_ACCOUNT_INDEX_MAP.with(|m| m.borrow_mut().insert(3, EntityIndex::new(3, (147..154).collect())));
  let user = Principal::anonymous().to_text();
  STAKING_USER_ACCOUNT_INDEX_MAP.with(|m| m.borrow_mut().insert(user.clone(), EntityIndex::new(user, (147..154).collect())));
  let mut job = get_job(3).unwrap();
  job.expected_account_count = 23;
  job.expected_amount = 310_600_000;
  save_job(&job);
  addresses
}
