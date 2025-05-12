use std::{cell::RefCell, str::FromStr};

use ic_stable_structures::{memory_manager::MemoryId, Cell, StableBTreeMap};
use stable_structures::EventLog;
use transport_structures::{EventLogQueryParams, EventTypeCode, StakingEventLogPageRequest, StakingEventLogPageResponse};
use types::{stable_structures::Memory, EntityId};

use crate::{memory_ids::{STAKING_EVENT_LOG, STAKING_EVENT_LOG_SEQ}, MEMORY_MANAGER};

pub mod stable_structures;
pub mod staking_pool_events;
pub mod staking_account_events;
pub mod stake_and_unstake_events;
pub mod stake_reward_events;
pub mod transport_structures;
pub mod transfer_events;

thread_local! {
  /// The stake event log is increased automaticallyIDGenerator
  pub static STAKING_EVENT_LOG_ID: RefCell<Cell<EntityId, Memory>> = RefCell::new(Cell::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(STAKING_EVENT_LOG_SEQ))), 0_u64).unwrap());


  /// Staking event log original data
  pub static STAKING_EVENT_LOG_MAP: RefCell<StableBTreeMap<EntityId, EventLog, Memory>> = RefCell::new(
    StableBTreeMap::init(
      MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(STAKING_EVENT_LOG))),
    )
  );
}

/// Query the stake event log，Only check the last two thousand
#[ic_cdk::query]
fn query_event_logs(request: StakingEventLogPageRequest) -> StakingEventLogPageResponse {
  let StakingEventLogPageRequest { 
    page, 
    page_size, 
    params: EventLogQueryParams {
      event_type,
      start_time,
      end_time,
    },
  } = request;
  let event_type_code = match EventTypeCode::from_str(&event_type) {
    Ok(code) => code,
    Err(_) => EventTypeCode::Undefined,
  };

  let start = (page - 1) * page_size;

  STAKING_EVENT_LOG_MAP.with(|map| {
    let map = map.borrow();
    let filter_records: Vec<EventLog> = map
      .values()
      .into_iter()
      .rev()
      .take(2000)
      .filter(|event_log| event_type_code.is_match(&event_log.get_event_type()))
      .filter(|event_log| {
        if start_time > 0 {
          event_log.get_event_time() >= start_time
        } else {
          true
        }
      })
      .filter(|event_log| {
        if end_time > 0 {
          event_log.get_event_time() <= end_time
        } else {
          true
        }
      })
      .collect();

    let total = filter_records.len() as u32;

    StakingEventLogPageResponse {
      page,
      page_size,
      total,
      records: filter_records
        .iter()
        .skip(start as usize)
        .take(page_size as usize)
        .cloned()
        .collect(),
    }
  })
}