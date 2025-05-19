use candid::CandidType;
use serde::{Deserialize, Serialize};

use crate::{EntityId, TicketNo, UserId};

#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct InstantWinPlayRecordVo {
  pub id: EntityId,
  pub sales_order_id: EntityId,
  pub redemption_order_id: EntityId,
  pub product_id: EntityId,
  pub batch_id: EntityId,
  pub user_id: UserId,
  pub ticket_no: TicketNo,
  pub prize_multiple: u32,
  pub create_time: u64,
}
