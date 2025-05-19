use candid::CandidType;
use serde::{Deserialize, Serialize};

use crate::{Crypto, E8S, EntityId, Nanoseconds, TicketNo, Timestamp, UserId, product::E4S, stable_structures::MetaData};

use super::stable_structures::{CloseMode, InstantWinConfig, Prize};

#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct InstantWinConfigVo {
  pub id: EntityId,
  pub name: String,
  // 0: ICP, 1: USDT
  pub crypto: String,
  pub ticket_price: E8S,
  pub total_ticket_count: u32,
  pub show_remaining_win_tickets: bool,
  pub prizes: Vec<PrizeVo>,
  pub duration: Nanoseconds,
  pub auto_start_next: bool,
  // 0: NoTicket, 1: NoWinTicket
  pub mode: String,
  pub description: String,
  pub total_prize_amounts: E8S,
  pub total_price: E8S,
  pub win_rate: String,
  pub payout: String,
  pub created_at: Timestamp,
  pub updated_at: Timestamp,
  pub created_by: UserId,
  pub updated_by: UserId,
}

impl InstantWinConfigVo {
  pub fn to_config(&self) -> InstantWinConfig {
    InstantWinConfig {
      id: Some(self.id),
      name: Some(self.name.clone()),
      crypto: Some(Crypto::ICP),
      ticket_price: Some(self.ticket_price),
      total_ticket_count: Some(self.total_ticket_count),
      show_remaining_win_tickets: Some(self.show_remaining_win_tickets),
      duration: Some(self.duration),
      auto_start_next: Some(self.auto_start_next),
      mode: Some(self.mode.parse().unwrap_or_else(|_| CloseMode::NoTicket)),
      description: Some(self.description.clone()),
      prizes: Some(
        self
          .prizes
          .clone()
          .iter()
          .map(|item| Prize {
            ticket_count: Some(item.ticket_count),
            multiples: Some(item.multiples),
            preset_tickets: Some(item.preset_tickets.clone()),
            col_span: Some(item.col_span),
          })
          .collect(),
      ),
      total_price: Some(self.total_price),
      payout: Some(self.payout.clone()),
      win_rate: Some(self.win_rate.clone()),
      total_prize_amounts: Some(self.total_prize_amounts),
      meta: Some(MetaData {
        created_at: Some(self.created_at),
        updated_at: Some(self.updated_at),
        created_by: Some(self.created_by.clone()),
        updated_by: Some(self.updated_by.clone()),
      }),
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct PrizeVo {
  pub ticket_count: u32,
  pub multiples: E4S,
  pub preset_tickets: Vec<TicketNo>,
  pub col_span: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct AddInstantWinConfigDto {
  pub name: String,
  // 0: ICP, 1: USDT
  pub crypto: String,
  pub ticket_price: E8S,
  pub total_ticket_count: u32,
  pub show_remaining_win_tickets: bool,
  pub prizes: Vec<PrizeVo>,
  pub duration: Nanoseconds,
  pub auto_start_next: bool,
  // 0: NoTicket, 1: NoWinTicket
  pub mode: String,
  pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct UpdateInstantWinConfigDto {
  pub id: EntityId,
  pub name: String,
  // 0: ICP, 1: USDT
  pub crypto: String,
  pub ticket_price: E8S,
  pub total_ticket_count: u32,
  pub show_remaining_win_tickets: bool,
  pub prizes: Vec<PrizeVo>,
  pub duration: Nanoseconds,
  pub auto_start_next: bool,
  // 0: NoTicket, 1: NoWinTicket
  pub mode: String,
  pub description: String,
}
