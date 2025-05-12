use std::borrow::Cow;

use candid::{CandidType, Decode, Encode};
use ic_stable_structures::{storable::Bound, Storable};
use serde::{Deserialize, Serialize};
use types::{date::YearMonthDay, UserId};

use crate::StakingAccountId;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, CandidType, Default)]
pub struct StakingAccountUserRewardDateIndexKey(StakingAccountId, UserId, YearMonthDay);
impl StakingAccountUserRewardDateIndexKey {
  pub fn new(account_id: StakingAccountId, user_id: UserId, date: YearMonthDay) -> Self {
    Self(account_id, user_id, date)
  }
}

impl StakingAccountUserRewardDateIndexKey {
  pub fn get_account_id(&self) -> &StakingAccountId {
    &self.0
  }
  pub fn get_user_id(&self) -> &UserId {
    &self.1
  }
  pub fn get_reward_date(&self) -> &YearMonthDay {
    &self.2
  }
}

impl Storable for StakingAccountUserRewardDateIndexKey {
  fn to_bytes(&self) -> Cow<[u8]> {
    Cow::Owned(Encode!(self).unwrap())
  }

  fn from_bytes(bytes: Cow<[u8]>) -> Self {
    Decode!(bytes.as_ref(), Self).unwrap()
  }

  const BOUND: Bound = Bound::Unbounded;
}