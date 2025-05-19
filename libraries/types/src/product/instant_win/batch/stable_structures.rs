use crate::{
  product::{BatchId, BatchState, ProductId, base::batch::stable_structures::Batch, instant_win::configs::stable_structures::InstantWinConfig},
  stable_structures::{EntityIdGenerator, MetaData, new_entity_id},
};

use super::transport_structures::{AddInstantWinBatchDto, InstantWinBatchVo, UpdateInstantWinBatchDto};

/// InstantWinBatch is a cycle of instant win lottery
pub type InstantWinBatch = Batch<InstantWinConfig>;

impl InstantWinBatch {
  pub fn new(id_gen: &EntityIdGenerator, add_dto: AddInstantWinBatchDto) -> Self {
    let id = new_entity_id(id_gen);

    Self {
      id: Some(id),
      product_id: Some(add_dto.product_id),
      config: Some(InstantWinConfig::from_update_dto(&add_dto.config)),
      batch_state: Some(BatchState::New),
      description: Some(add_dto.description.clone()),
      start_time: None,
      pause_time: None,
      accumulated_pause_time: None,
      end_time: None,
      meta: Some(MetaData::init_create_scene()),
    }
  }

  pub fn get_id(&self) -> BatchId {
    self.id.clone().unwrap_or_default()
  }

  pub fn get_product_id(&self) -> ProductId {
    self.product_id.clone().unwrap_or_default()
  }

  pub fn get_state(&self) -> BatchState {
    self.batch_state.clone().unwrap_or_else(|| ic_cdk::trap("Batch state is required"))
  }

  pub fn get_config(&self) -> InstantWinConfig {
    self.config.clone().unwrap_or_else(|| ic_cdk::trap("Batch config is required"))
  }

  pub fn update(self, update_dto: UpdateInstantWinBatchDto) -> Self {
    Self {
      id: self.id,
      product_id: self.product_id,
      config: Some(InstantWinConfig::from_update_dto(&update_dto.config)),
      // Update the batch, it must reset the state to Initialized
      batch_state: Some(BatchState::New),
      description: Some(update_dto.description.clone()),
      start_time: self.start_time,
      pause_time: self.pause_time,
      accumulated_pause_time: self.accumulated_pause_time,
      end_time: self.end_time,
      meta: Some(self.meta.unwrap_or(MetaData::init_create_scene()).update()),
    }
  }

  pub fn start(self) -> Self {
    Self {
      id: self.id,
      product_id: self.product_id,
      config: self.config,
      batch_state: Some(BatchState::Running),
      description: self.description,
      start_time: Some(ic_cdk::api::time()),
      pause_time: self.pause_time,
      accumulated_pause_time: self.accumulated_pause_time,
      end_time: self.end_time,
      meta: Some(self.meta.unwrap_or(MetaData::init_create_scene()).update()),
    }
  }

  pub fn resume(self) -> Self {
    Self {
      id: self.id,
      product_id: self.product_id,
      config: self.config,
      batch_state: Some(BatchState::Running),
      description: self.description,
      start_time: self.start_time,
      pause_time: None,
      accumulated_pause_time: Some(self.accumulated_pause_time.unwrap_or_default() + (ic_cdk::api::time() - self.pause_time.unwrap_or_default())),
      end_time: self.end_time,
      meta: Some(self.meta.unwrap_or(MetaData::init_create_scene()).update()),
    }
  }

  pub fn pause(self) -> Self {
    Self {
      id: self.id,
      product_id: self.product_id,
      config: self.config,
      batch_state: Some(BatchState::Paused),
      description: self.description,
      start_time: self.start_time,
      pause_time: Some(ic_cdk::api::time()),
      accumulated_pause_time: self.accumulated_pause_time,
      end_time: self.end_time,
      meta: Some(self.meta.unwrap_or(MetaData::init_create_scene()).update()),
    }
  }

  pub fn finish(&mut self) -> Self {
    self.batch_state = Some(BatchState::Finished);
    self.end_time = Some(ic_cdk::api::time());
    self.meta = Some(self.meta.clone().unwrap_or(MetaData::init_create_scene()).update());
    self.clone()
  }

  pub fn expired(&mut self) -> Self {
    self.batch_state = Some(BatchState::Expired);
    self.end_time = Some(ic_cdk::api::time());
    self.meta = Some(self.meta.clone().unwrap_or(MetaData::init_create_scene()).update());
    self.clone()
  }

  pub fn generate(self) -> Self {
    Self {
      id: self.id,
      product_id: self.product_id,
      config: self.config,
      batch_state: Some(BatchState::Initialized),
      description: self.description,
      start_time: self.start_time,
      pause_time: self.pause_time,
      accumulated_pause_time: self.accumulated_pause_time,
      end_time: self.end_time,
      meta: Some(self.meta.unwrap_or(MetaData::init_create_scene()).update()),
    }
  }

  pub fn to_vo(&self) -> InstantWinBatchVo {
    let meta = self.meta.clone().unwrap_or(MetaData::init_create_scene());
    InstantWinBatchVo {
      id: self.get_id(),
      product_id: self.get_product_id(),
      config: self.config.clone().unwrap().to_vo(),
      batch_state: self.batch_state.clone().unwrap_or(BatchState::New).to_string(),
      description: self.description.clone().unwrap_or_default(),
      quick_quid_extra: None,
      start_time: self.start_time.unwrap_or_default(),
      pause_time: self.pause_time.unwrap_or_default(),
      accumulated_pause_time: self.accumulated_pause_time.unwrap_or_default(),
      end_time: self.end_time.unwrap_or_default(),
      remain_duration: 0,
      created_at: meta.created_at.unwrap_or_default(),
      created_by: meta.created_by.unwrap_or_default(),
      updated_at: meta.updated_at.unwrap_or_default(),
      updated_by: meta.updated_by.unwrap_or_default(),
    }
  }

  pub fn set_auto_start_next(&mut self, auto_start_next: bool) {
    self.config.as_mut().unwrap().auto_start_next = Some(auto_start_next);
  }
}
