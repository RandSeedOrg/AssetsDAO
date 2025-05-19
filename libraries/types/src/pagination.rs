use candid::CandidType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct PageRequest<T: CandidType> {
  pub page: u32,
  pub page_size: u32,
  pub params: T,
}

impl<T: CandidType> PageRequest<T> {
  pub fn new(page: u32, page_size: u32, params: T) -> Self {
    Self { page, page_size, params }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize, CandidType)]
pub struct PageResponse<T: CandidType> {
  pub page: u32,
  pub page_size: u32,
  pub total: u32,
  pub records: Vec<T>,
}

impl<T: CandidType> PageResponse<T> {
  pub fn new(page: u32, page_size: u32, total: u32, records: Vec<T>) -> Self {
    Self {
      page,
      page_size,
      total,
      records,
    }
  }

  /// Record start position
  pub fn start(&self) -> u32 {
    (self.page - 1) * self.page_size
  }

  /// Record end position
  pub fn end(&self) -> u32 {
    self.start() + self.page_size - 1
  }
}
