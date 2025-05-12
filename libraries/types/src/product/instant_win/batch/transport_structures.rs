use candid::CandidType;
use serde::{Deserialize, Serialize};

use crate::{product::instant_win::configs::transport_structures::{InstantWinConfigVo, UpdateInstantWinConfigDto}, EntityId, Nanoseconds, Timestamp, UserId};

use super::quick_quid_transport_structures::QuickQuidBatchExtraVo;


#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct InstantWinBatchVo {
  pub id: EntityId,
  pub product_id: EntityId,
  pub config: InstantWinConfigVo,
  /// 0 Intialized, 1 Running, 2 Paused, 3 Finished
  pub batch_state: String,
  pub quick_quid_extra: Option<QuickQuidBatchExtraVo>,
  pub description: String,
  pub start_time: Timestamp,
  pub pause_time: Timestamp,
  pub accumulated_pause_time: Nanoseconds,
  pub end_time: Timestamp,
  pub remain_duration: i64,
  pub created_at: Timestamp,
  pub created_by: UserId,
  pub updated_at: Timestamp,
  pub updated_by: UserId,
}

#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct AddInstantWinBatchDto {
  pub product_id: EntityId,
  pub config: UpdateInstantWinConfigDto,
  pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct UpdateInstantWinBatchDto {
  pub id: EntityId,
  pub product_id: EntityId,
  pub config: UpdateInstantWinConfigDto,
  pub description: String,
}