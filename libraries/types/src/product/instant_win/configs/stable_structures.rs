use std::borrow::Cow;

use bigdecimal::{BigDecimal, ToPrimitive};
use candid::{CandidType, Decode, Encode};
use ic_stable_structures::{Storable, storable::Bound};
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};

use crate::{
  Crypto, E8S, EntityId, Nanoseconds, TicketNo,
  product::{E4S, e4s_to_multiples},
  stable_structures::{EntityIdGenerator, MetaData, new_entity_id},
};

use super::transport_structures::{AddInstantWinConfigDto, InstantWinConfigVo, PrizeVo, UpdateInstantWinConfigDto};

/// Configuration for instant win
#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct InstantWinConfig {
  pub id: Option<EntityId>,
  pub name: Option<String>,
  pub crypto: Option<Crypto>,
  pub ticket_price: Option<E8S>,
  pub total_ticket_count: Option<u32>,
  pub show_remaining_win_tickets: Option<bool>,
  // If the duration of the current batch is 0 or None, it means that the current batch will not end automatically due to the exhaustion of the duration.
  pub duration: Option<Nanoseconds>,
  // If the current batch is finished, will the next batch be automatically started
  pub auto_start_next: Option<bool>,
  pub mode: Option<CloseMode>,
  pub description: Option<String>,
  pub prizes: Option<Vec<Prize>>,
  pub total_prize_amounts: Option<E8S>,
  pub total_price: Option<E8S>,
  pub payout: Option<String>,
  pub win_rate: Option<String>,
  pub meta: Option<MetaData>,
}

impl InstantWinConfig {
  pub fn new(id_gen: &EntityIdGenerator, add_dto: &AddInstantWinConfigDto) -> Self {
    let id = new_entity_id(id_gen);

    let mut config = Self {
      id: Some(id),
      name: Some(add_dto.name.clone()),
      crypto: Some(add_dto.crypto.clone().parse().unwrap_or(Crypto::ICP)),
      ticket_price: Some(add_dto.ticket_price),
      total_ticket_count: Some(add_dto.total_ticket_count),
      show_remaining_win_tickets: Some(add_dto.show_remaining_win_tickets),
      prizes: Some(add_dto.prizes.clone().iter().map(|prize| Prize::from_vo(prize)).collect()),
      duration: Some(add_dto.duration),
      auto_start_next: Some(add_dto.auto_start_next),
      mode: Some(add_dto.mode.parse().unwrap_or(CloseMode::NoTicket)),
      description: Some(add_dto.description.clone()),
      total_price: None,
      payout: None,
      win_rate: None,
      total_prize_amounts: None,
      meta: Some(MetaData::init_create_scene()),
    };

    config.calc_prices();

    config
  }

  pub fn get_name(&self) -> String {
    self.name.clone().unwrap_or_default()
  }

  pub fn update(&self, update_dto: &UpdateInstantWinConfigDto) -> Self {
    let mut config = Self {
      id: self.id.clone(),
      name: Some(update_dto.name.clone()),
      crypto: Some(update_dto.crypto.clone().parse().unwrap_or(Crypto::ICP)),
      ticket_price: Some(update_dto.ticket_price),
      total_ticket_count: Some(update_dto.total_ticket_count),
      show_remaining_win_tickets: Some(update_dto.show_remaining_win_tickets),
      prizes: Some(update_dto.prizes.clone().iter().map(|prize| Prize::from_vo(prize)).collect()),
      duration: Some(update_dto.duration),
      auto_start_next: Some(update_dto.auto_start_next),
      mode: Some(update_dto.mode.parse().unwrap_or(CloseMode::NoTicket)),
      description: Some(update_dto.description.clone()),
      total_price: None,
      payout: None,
      win_rate: None,
      total_prize_amounts: None,
      meta: Some(self.meta.clone().unwrap_or(MetaData::init_create_scene()).update()),
    };

    config.calc_prices();

    config
  }

  pub fn from_update_dto(update_dto: &UpdateInstantWinConfigDto) -> Self {
    let mut config = Self {
      id: Some(update_dto.id),
      name: Some(update_dto.name.clone()),
      crypto: Some(update_dto.crypto.clone().parse().unwrap_or(Crypto::ICP)),
      ticket_price: Some(update_dto.ticket_price),
      total_ticket_count: Some(update_dto.total_ticket_count),
      show_remaining_win_tickets: Some(update_dto.show_remaining_win_tickets),
      prizes: Some(
        update_dto
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
      duration: Some(update_dto.duration),
      auto_start_next: Some(update_dto.auto_start_next),
      mode: Some(update_dto.mode.clone().parse().unwrap_or(CloseMode::NoTicket)),
      description: Some(update_dto.description.clone()),
      total_price: None,
      payout: None,
      win_rate: None,
      total_prize_amounts: None,
      meta: Some(MetaData::init_create_scene()),
    };

    config.calc_prices();

    config
  }

  pub fn to_vo(&self) -> InstantWinConfigVo {
    let meta = self.meta.clone().unwrap_or(MetaData::init_create_scene());

    InstantWinConfigVo {
      id: self.id.clone().unwrap_or_default(),
      name: self.name.clone().unwrap_or_default(),
      crypto: self.crypto.clone().unwrap_or(Crypto::ICP).to_string(),
      ticket_price: self.ticket_price.clone().unwrap_or_default(),
      total_ticket_count: self.total_ticket_count.clone().unwrap_or_default(),
      show_remaining_win_tickets: self.show_remaining_win_tickets.clone().unwrap(),
      prizes: self.prizes.clone().unwrap().iter().map(|prize| prize.to_vo()).collect(),
      duration: self.duration.clone().unwrap_or_default(),
      auto_start_next: self.auto_start_next.clone().unwrap_or(false),
      mode: self.mode.clone().unwrap_or(CloseMode::NoTicket).to_string(),
      description: self.description.clone().unwrap_or_default(),
      total_prize_amounts: self.total_prize_amounts.clone().unwrap_or_default(),
      total_price: self.total_price.clone().unwrap_or_default(),
      win_rate: self.win_rate.clone().unwrap_or_default(),
      payout: self.payout.clone().unwrap_or_default(),
      created_at: meta.created_at.unwrap_or_default(),
      updated_at: meta.updated_at.unwrap_or_default(),
      created_by: meta.created_by.unwrap_or_default(),
      updated_by: meta.updated_by.unwrap_or_default(),
    }
  }

  fn calc_prices(&mut self) -> &Self {
    let total_price = BigDecimal::from(self.total_ticket_count.unwrap()) * BigDecimal::from(self.ticket_price.unwrap());
    let total_prize_amounts = self
      .prizes
      .clone()
      .unwrap_or_default()
      .iter()
      .map(|prize| {
        BigDecimal::from(prize.ticket_count.unwrap_or_default())
          * e4s_to_multiples(prize.multiples.unwrap_or_default())
          * BigDecimal::from(self.ticket_price.unwrap_or_default())
      })
      .sum::<BigDecimal>();
    let total_prize_ticket_count = self
      .prizes
      .clone()
      .unwrap_or_default()
      .iter()
      .map(|prize| BigDecimal::from(prize.ticket_count.unwrap_or_default()))
      .sum::<BigDecimal>();
    self.total_price = Some(total_price.to_u64().unwrap_or_default());
    self.total_prize_amounts = Some(total_prize_amounts.to_u64().unwrap_or_default());
    self.payout = Some((total_prize_amounts / total_price).to_string());
    self.win_rate = Some((total_prize_ticket_count / BigDecimal::from(self.total_ticket_count.unwrap_or(1))).to_string());

    self
  }
}

/// Award configuration for instant win
#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct Prize {
  pub ticket_count: Option<u32>,
  pub multiples: Option<E4S>,
  pub preset_tickets: Option<Vec<TicketNo>>,
  pub col_span: Option<u32>,
}

impl Prize {
  pub fn to_vo(&self) -> PrizeVo {
    PrizeVo {
      ticket_count: self.ticket_count.unwrap_or_default(),
      multiples: self.multiples.unwrap_or_default(),
      preset_tickets: self.preset_tickets.clone().unwrap_or_default(),
      col_span: self.col_span.unwrap_or(1),
    }
  }

  pub fn from_vo(vo: &PrizeVo) -> Self {
    Self {
      ticket_count: Some(vo.ticket_count),
      multiples: Some(vo.multiples),
      preset_tickets: Some(vo.preset_tickets.clone()),
      col_span: Some(vo.col_span),
    }
  }
}

/// Close mode for instant win
#[derive(EnumString, Display, Debug, Clone, Serialize, Deserialize, CandidType)]
pub enum CloseMode {
  #[strum(serialize = "0")]
  NoTicket,
  #[strum(serialize = "1")]
  NoWinTicket,
}

impl Storable for InstantWinConfig {
  fn to_bytes(&self) -> Cow<[u8]> {
    Cow::Owned(Encode!(self).unwrap())
  }

  fn from_bytes(bytes: Cow<[u8]>) -> Self {
    Decode!(bytes.as_ref(), Self).unwrap()
  }

  const BOUND: Bound = Bound::Unbounded;
}

impl Storable for Prize {
  fn to_bytes(&self) -> Cow<[u8]> {
    Cow::Owned(Encode!(self).unwrap())
  }

  fn from_bytes(bytes: Cow<[u8]>) -> Self {
    Decode!(bytes.as_ref(), Self).unwrap()
  }

  const BOUND: Bound = Bound::Unbounded;
}
