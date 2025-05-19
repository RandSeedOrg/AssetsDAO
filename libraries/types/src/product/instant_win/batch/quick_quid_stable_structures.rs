use std::{borrow::Cow, collections::BTreeMap};

use candid::{CandidType, Decode, Encode};
use ic_stable_structures::{Storable, storable::Bound};
use serde::{Deserialize, Serialize};

use crate::{
  CellIndex, E8S, EntityId, TimestampNanos, UserId,
  product::{BatchId, E4S, calc_amount_multiple, instant_win::record::transport_structures::InstantWinPlayRecordVo},
};

use super::quick_quid_transport_structures::{
  CardCellPrizeVo, CardDto, QuickQuidBatchExtraVo, QuickQuidExtraConfigDto, QuickQuidExtraRuntimeVo, RuntimeCardVo, RuntimeCellVo,
};

pub type CardCellIndex = u32;

/// The card info of quick quid
#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct RuntimeCard {
  /// Card serial number, unique in each batch, and ordered
  pub ordinal: Option<u32>,
  /// Card background image
  pub background_url: Option<String>,
  /// The start index of the grid in the card
  pub cell_start: Option<usize>,
  /// Grid information in the card
  pub cell_count: Option<usize>,
}

impl RuntimeCard {
  pub fn new(ordinal: u32, background_url: String, cell_start: usize, cell_count: usize) -> Self {
    Self {
      ordinal: Some(ordinal),
      background_url: Some(background_url),
      cell_start: Some(cell_start),
      cell_count: Some(cell_count),
    }
  }

  pub fn to_vo(&self, runtime: &QuickQuidExtraRuntime) -> RuntimeCardVo {
    RuntimeCardVo {
      ordinal: self.get_ordinal(),
      // background_url: self.get_background_url(),
      cells: runtime
        .get_cells()
        .iter()
        .skip(self.get_cell_start())
        .take(self.get_cell_count())
        .map(|(_, card_cell)| card_cell.to_vo())
        .collect(),
    }
  }

  /// According to the user currently requested, the current runtime card is converted to vo
  /// where the card processing will be partially filtered through the caller to avoid the leakage of other people's awards,
  /// unwindowed prizes and bonuses to the front end
  pub fn to_caller_vo(&self, runtime: &QuickQuidExtraRuntime, caller: &UserId) -> RuntimeCardVo {
    RuntimeCardVo {
      ordinal: self.get_ordinal(),
      // background_url: self.get_background_url(),
      cells: runtime
        .get_cells()
        .iter()
        .skip(self.get_cell_start())
        .take(self.get_cell_count())
        .map(|(_, card_cell)| card_cell.to_caller_vo(caller))
        .collect(),
    }
  }

  pub fn get_cell_start(&self) -> usize {
    self.cell_start.unwrap_or_default()
  }

  pub fn get_cell_count(&self) -> usize {
    self.cell_count.unwrap_or_default()
  }

  pub fn get_background_url(&self) -> String {
    self.background_url.clone().unwrap_or_default()
  }

  pub fn get_ordinal(&self) -> u32 {
    self.ordinal.clone().unwrap_or_default()
  }
}

/// The cells info of quick quid
#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct CardCell {
  /// Where the grid is located
  pub index: Option<CellIndex>,
  /// Is the grid locked
  pub locked: Option<bool>,
  /// Users currently locking the grid
  pub locked_by: Option<UserId>,
  /// transaction id that locks the grid
  pub locked_tx: Option<u64>,
  /// After the grid is turned on, the bound lottery information
  pub prize: Option<CardCellPrize>,
  /// Additional information in the grid, currently used to store bonus code, may have multiple additional information, currently only bonus code
  pub extra: Option<Vec<CardCellExtra>>,
}

impl CardCell {
  pub fn new(index: CellIndex) -> Self {
    Self {
      index: Some(index),
      locked: Some(false),
      locked_by: None,
      locked_tx: None,
      prize: None,
      extra: Some(vec![]),
    }
  }

  pub fn new_with_bonus(index: u32, bonus_code: String) -> Self {
    let mut cell = Self::new(index);
    cell.add_extra(CardCellExtra::BonusCode(bonus_code));
    cell
  }

  pub fn to_vo(&self) -> RuntimeCellVo {
    RuntimeCellVo {
      index: self.get_index(),
      prize: self.get_prize().map(|prize| prize.to_vo()),
      bonus_code: {
        match self.get_first_extra() {
          Some(CardCellExtra::BonusCode(code)) => code,
          None => "".to_string(),
        }
      },
    }
  }

  pub fn to_caller_vo(&self, caller: &UserId) -> RuntimeCellVo {
    let prize = self.get_prize().map(|prize| prize.to_caller_vo(caller));

    RuntimeCellVo {
      index: self.get_index(),
      bonus_code: match &prize {
        Some(_) => match self.get_first_extra() {
          Some(CardCellExtra::BonusCode(code)) => code,
          None => "".to_string(),
        },
        None => "".to_string(),
      },
      prize,
    }
  }

  pub fn get_bonus_code(&self) -> String {
    match self.get_first_extra() {
      Some(CardCellExtra::BonusCode(code)) => code,
      None => "".to_string(),
    }
  }

  pub fn get_index(&self) -> u32 {
    self.index.clone().unwrap_or_default()
  }

  pub fn lock(&mut self, user_id: &UserId, tx_id: u64) -> bool {
    if self.is_locked() {
      return false;
    }

    self.locked = Some(true);
    self.locked_by = Some(user_id.clone());
    self.locked_tx = Some(tx_id);
    true
  }

  pub fn unlock(&mut self, user_id: &UserId, tx_id: u64) -> bool {
    // If the grid is not locked and returns directly, there is no need to unlock it
    if !self.is_locked() {
      return false;
    }

    // Unlocking is not allowed if the user locking the grid is not the current user
    if self.get_locked_by() != *user_id {
      return false;
    }

    // If the grid is not locked by the current session, it will not be unlocked
    if self.get_locked_tx() != tx_id {
      return false;
    }

    self.locked = Some(false);
    self.locked_by = None;

    true
  }

  pub fn get_locked_tx(&self) -> u64 {
    self.locked_tx.clone().unwrap_or_default()
  }

  pub fn is_locked(&self) -> bool {
    self.locked.clone().unwrap_or_default()
  }

  pub fn get_locked_by(&self) -> UserId {
    self.locked_by.clone().unwrap_or_default()
  }

  pub fn set_prize(&mut self, prize: CardCellPrize) {
    self.prize = Some(prize);
  }

  pub fn get_prize(&self) -> Option<CardCellPrize> {
    self.prize.clone()
  }

  pub fn add_extra(&mut self, extra: CardCellExtra) {
    if self.extra.is_none() {
      self.extra = Some(vec![]);
    }

    self.extra.as_mut().unwrap().push(extra);
  }

  pub fn get_extras(&self) -> Vec<CardCellExtra> {
    self.extra.clone().unwrap_or_default()
  }

  pub fn get_first_extra(&self) -> Option<CardCellExtra> {
    self.extra.clone().unwrap_or_default().first().cloned()
  }
}

/// The extra information in the grid in Quick quid is currently used to store bonus codes. Enum is used to facilitate expansion
#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub enum CardCellExtra {
  BonusCode(String),
}

/// Grid lottery information
#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct CardCellPrize {
  /// player id
  pub user_id: Option<UserId>,
  /// Win amount
  pub prize_amount: Option<E8S>,
  /// Winning multiple
  pub prize_multiples: Option<E4S>,
  /// Winning time
  pub create_time: Option<TimestampNanos>,
  /// The corresponding play record id
  pub play_record_id: Option<EntityId>,
}

impl CardCellPrize {
  pub fn to_vo(&self) -> CardCellPrizeVo {
    CardCellPrizeVo {
      user_id: self.get_user_id(),
      prize_amount: self.get_prize_amount(),
      prize_multiples: self.get_prize_multiples(),
      create_time: self.get_create_time(),
      play_record_id: self.get_play_record_id(),
    }
  }

  pub fn to_caller_vo(&self, caller: &str) -> CardCellPrizeVo {
    if caller == self.get_user_id() {
      self.to_vo()
    } else {
      CardCellPrizeVo {
        user_id: self.get_user_id(),
        prize_amount: 0,
        prize_multiples: 0,
        create_time: self.get_create_time(),
        play_record_id: self.get_play_record_id(),
      }
    }
  }

  pub fn get_user_id(&self) -> UserId {
    self.user_id.clone().unwrap_or_default()
  }

  pub fn get_prize_amount(&self) -> E8S {
    self.prize_amount.clone().unwrap_or_default()
  }

  pub fn get_prize_multiples(&self) -> E4S {
    self.prize_multiples.clone().unwrap_or_default()
  }

  pub fn get_create_time(&self) -> TimestampNanos {
    self.create_time.clone().unwrap_or_default()
  }

  pub fn get_play_record_id(&self) -> EntityId {
    self.play_record_id.clone().unwrap_or_default()
  }
}

/// The extra configuration information of the Quick quid batch
#[derive(Debug, Clone, Serialize, Deserialize, CandidType, Default)]
pub struct QuickQuidExtraConfig {
  // All bonus code information on the current Batch
  pub bonus_codes: Option<Vec<String>>,
  // Information about all cards in the current batch
  pub cards: Option<Vec<CardConfig>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct CardConfig {
  // Number of grids on the card
  pub cell_count: Option<u16>,
  // Card background image
  pub background_url: Option<String>,
}

impl CardConfig {
  pub fn new(cell_count: u16, background_url: String) -> Self {
    Self {
      cell_count: Some(cell_count),
      background_url: Some(background_url),
    }
  }

  pub fn convert_to_runtime_card(&self, ordinal: u32, cell_start: usize) -> RuntimeCard {
    RuntimeCard::new(ordinal, self.get_background_url(), cell_start, self.get_cell_count() as usize)
  }

  pub fn get_cell_count(&self) -> u16 {
    self.cell_count.clone().unwrap_or_default()
  }

  pub fn get_background_url(&self) -> String {
    self.background_url.clone().unwrap_or_default()
  }

  pub fn to_dto(&self) -> CardDto {
    CardDto {
      cell_count: self.get_cell_count(),
      background_url: self.get_background_url(),
    }
  }
}

impl QuickQuidExtraConfig {
  pub fn new(bonus_codes: Vec<String>, cards: Vec<CardConfig>) -> Self {
    Self {
      bonus_codes: Some(bonus_codes),
      cards: Some(cards),
    }
  }

  pub fn from_dto(config_dto: &QuickQuidExtraConfigDto) -> Self {
    Self {
      bonus_codes: Some(config_dto.bonus_codes.clone()),
      cards: Some(
        config_dto
          .cards
          .iter()
          .map(|card_dto| CardConfig::new(card_dto.cell_count, card_dto.background_url.clone()))
          .collect(),
      ),
    }
  }

  pub fn to_dto(&self) -> QuickQuidExtraConfigDto {
    QuickQuidExtraConfigDto {
      bonus_codes: self.get_bonus_codes(),
      cards: self.get_cards().iter().map(|card| card.to_dto()).collect(),
    }
  }

  pub fn get_cards(&self) -> Vec<CardConfig> {
    self.cards.clone().unwrap_or_default()
  }

  pub fn get_bonus_codes(&self) -> Vec<String> {
    self.bonus_codes.clone().unwrap_or_default()
  }

  pub fn get_card_backgrounds(&self) -> Vec<String> {
    self
      .cards
      .clone()
      .unwrap_or_default()
      .iter()
      .map(|card| card.background_url.clone().unwrap_or_default())
      .collect()
  }
}

#[derive(Debug, Clone, Serialize, Deserialize, CandidType, Default)]
pub struct QuickQuidExtraRuntime {
  pub cards: Option<Vec<RuntimeCard>>,
  pub cells: Option<BTreeMap<CardCellIndex, CardCell>>,
}

impl QuickQuidExtraRuntime {
  pub fn new() -> Self {
    Self {
      cards: Some(vec![]),
      cells: Some(BTreeMap::new()),
    }
  }

  pub fn to_vo(&self) -> QuickQuidExtraRuntimeVo {
    QuickQuidExtraRuntimeVo {
      cards: self.get_cards().iter().map(|card| card.to_vo(self)).collect(),
    }
  }

  pub fn set_cards(&mut self, cards: Vec<RuntimeCard>) {
    self.cards = Some(cards);
  }

  pub fn get_cells(&self) -> BTreeMap<CardCellIndex, CardCell> {
    self.cells.clone().unwrap_or_default()
  }

  pub fn add_card(&mut self, card: RuntimeCard) {
    if self.cards.is_none() {
      self.cards = Some(vec![]);
    }

    self.cards.as_mut().unwrap().push(card);
  }

  pub fn get_cards(&self) -> Vec<RuntimeCard> {
    self.cards.clone().unwrap_or_default()
  }

  pub fn add_cell(&mut self, cell: CardCell) {
    let index = cell.get_index();
    self.cells.as_mut().unwrap().insert(index, cell);
  }

  /// Lock a batch of grids and return the index of the grid that was successfully locked.
  pub fn lock_cells(&mut self, user_id: &UserId, cell_indexes: Vec<CellIndex>, tx: u64) -> Vec<CellIndex> {
    let cells = self.cells.as_mut().unwrap();
    cell_indexes
      .iter()
      .filter(|cell_index| {
        if let Some(cell) = cells.get_mut(cell_index) {
          return cell.lock(user_id, tx);
        }
        return false;
      })
      .map(|cell_index| *cell_index)
      .collect()
  }

  /// Unlock a batch of grids and return to the index of the grid that was successfully unlocked.
  pub fn unlock_cells(&mut self, user_id: &UserId, cell_indexes: Vec<CellIndex>, tx: u64) -> Vec<CellIndex> {
    let cells = self.cells.as_mut().unwrap();
    cell_indexes
      .iter()
      .filter(|cell_index| {
        if let Some(cell) = cells.get_mut(cell_index) {
          cell.unlock(user_id, tx)
        } else {
          false
        }
      })
      .map(|cell_index| *cell_index)
      .collect()
  }

  pub fn bind_cells_and_tickets(
    &mut self,
    user_id: &UserId,
    tx: u64,
    cell_indexes: Vec<CellIndex>,
    play_records: Vec<InstantWinPlayRecordVo>,
    ticket_price: E8S,
  ) -> Vec<CardCell> {
    let cells = self.cells.as_mut().unwrap();
    let mut bind_cells = vec![];
    for (cell_index, play_record) in cell_indexes.iter().zip(play_records.iter()) {
      if let Some(cell) = cells.get_mut(cell_index) {
        // If the grid is not locked, binding is not allowed
        if !cell.is_locked() {
          ic_cdk::println!("Cell [cell_index={}] is not locked", cell_index);
          continue;
        }

        // If the user locking the grid is not the current user, binding is not allowed
        if cell.get_locked_by() != *user_id {
          ic_cdk::println!("Cell [cell_index={}] is not locked by the user [user_id={}]", cell_index, user_id);
          continue;
        }

        // If the grid is not locked by the current session, it will not be bound
        if cell.get_locked_tx() != tx {
          ic_cdk::println!("Cell [cell_index={}] is not locked by the transaction [tx={}]", cell_index, tx);
          continue;
        }

        cell.set_prize(CardCellPrize {
          user_id: Some(play_record.user_id.clone()),
          prize_amount: Some(calc_amount_multiple(ticket_price, play_record.prize_multiple)),
          prize_multiples: Some(play_record.prize_multiple),
          create_time: Some(play_record.create_time),
          play_record_id: Some(play_record.id.clone()),
        });

        bind_cells.push(cell.clone());
      }
    }
    bind_cells
  }
}

/// The extra information of the Quick quid batch
#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct QuickQuidBatchExtra {
  pub batch_id: Option<BatchId>,
  /// Additional information configuration
  pub config: Option<QuickQuidExtraConfig>,
  /// Additional information at runtime will change as the game runs
  pub runtime: Option<QuickQuidExtraRuntime>,
}

impl Storable for QuickQuidBatchExtra {
  fn to_bytes(&self) -> Cow<[u8]> {
    Cow::Owned(Encode!(self).unwrap())
  }

  fn from_bytes(bytes: Cow<[u8]>) -> Self {
    Decode!(bytes.as_ref(), Self).unwrap()
  }

  const BOUND: Bound = Bound::Unbounded;
}

impl QuickQuidBatchExtra {
  pub fn new(batch_id: BatchId, config: QuickQuidExtraConfig) -> Self {
    Self {
      batch_id: Some(batch_id),
      config: Some(config),
      runtime: None,
    }
  }

  pub fn to_vo(&self) -> QuickQuidBatchExtraVo {
    QuickQuidBatchExtraVo {
      batch_id: self.get_batch_id(),
      // Additional information configuration
      config: self.get_config().to_dto(),
      // Additional information at runtime will change as the game runs
      runtime: self.get_runtime().to_vo(),
    }
  }

  pub fn get_config(&self) -> QuickQuidExtraConfig {
    self.config.clone().unwrap_or_default()
  }

  pub fn update_runtime(&mut self, runtime: QuickQuidExtraRuntime) {
    self.runtime = Some(runtime);
  }

  pub fn get_batch_id(&self) -> BatchId {
    self.batch_id.clone().unwrap_or_default()
  }

  pub fn get_runtime(&self) -> QuickQuidExtraRuntime {
    self.runtime.clone().unwrap_or_default()
  }

  pub fn get_mut_runtime(&mut self) -> &mut QuickQuidExtraRuntime {
    self.runtime.as_mut().unwrap()
  }
}
