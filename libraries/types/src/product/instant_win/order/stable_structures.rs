use candid::CandidType;
use serde::{Deserialize, Serialize};

use crate::{product::{base::batch::stable_structures::{RedemptionOrder, TicketSalesOrder}, instant_win::batch::transport_structures::InstantWinBatchVo}, stable_structures::MetaData, EntityId, TicketNo, UserId, E8S};

/// InstantWinSalesOrder is a ticket order for instant win lottery
pub type InstantWinSalesOrder = TicketSalesOrder<TicketNo>;

impl InstantWinSalesOrder {
  pub fn new(batch: &InstantWinBatchVo, user_id: &UserId, tickets: Vec<TicketNo>, extra: &str) -> Self {
    let total_price = batch.config.ticket_price * tickets.len() as u64;

    Self {
      id: None,
      product_id: Some(batch.product_id),
      batch_id: Some(batch.id),
      user_id: Some(user_id.to_string()),
      unit_price: Some(batch.config.ticket_price),
      total_price: Some(total_price),
      tickets: Some(tickets),
      psn: None,
      extra: Some(extra.to_string()),
      meta: Some(MetaData::init_create_scene()),
    }
  }

  pub fn update_psn(&mut self, psn: u64) -> &mut Self {
    self.psn = Some(psn);
    self.meta = Some(self.meta.clone().unwrap_or(MetaData::init_create_scene()).update());
    self
  }

  pub fn set_id(&mut self, id: EntityId) -> &mut Self {
    self.id = Some(id);
    self
  }
}

#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct InstantWinPrizeTicket {
  pub ticket: Option<TicketNo>,
  pub multiples: Option<u32>,
  pub prize: Option<E8S>,
}

/// InstantWinRedemptionOrder is a redemption order for instant win lottery
pub type InstantWinRedemptionOrder = RedemptionOrder<InstantWinPrizeTicket>;


impl InstantWinRedemptionOrder {
  pub fn new(batch: &InstantWinBatchVo, user_id: &UserId, tickets: Vec<InstantWinPrizeTicket>, prize_amount: E8S, extra: &str) -> Self {

    Self {
      id: None,
      product_id: Some(batch.product_id),
      batch_id: Some(batch.id),
      user_id: Some(user_id.to_string()),
      prize_amount: Some(prize_amount),
      prize_tickets: Some(tickets),
      psn: None,
      extra: Some(extra.to_string()),
      meta: Some(MetaData::init_create_scene()),
    }
  }

  pub fn update_psn(&mut self, psn: u64) -> &mut Self {
    self.psn = Some(psn);
    self.meta = Some(self.meta.clone().unwrap_or(MetaData::init_create_scene()).update());
    self 
  }
}