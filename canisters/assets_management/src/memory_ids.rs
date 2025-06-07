use ic_stable_structures::memory_manager::MemoryId;

/// The Memory ID associated with the event log is between 10 and 19.
pub const LOG_INDEX_MEMORY_ID: MemoryId = MemoryId::new(0);
pub const LOG_DATA_MEMORY_ID: MemoryId = MemoryId::new(1);

/// The Memory ID associated with the assets movement is between 10 and 19.
pub const PROPOSAL_SEQ_MEMORY_ID: MemoryId = MemoryId::new(10);
pub const PROPOSAL_MAP_MEMORY_ID: MemoryId = MemoryId::new(11);

/// The Memory ID associated with the transfer address is between 20 and 29.
pub const TRANSFER_ADDRESS_MEMORY_ID: MemoryId = MemoryId::new(20);
pub const TRANSFER_ADDRESS_SEQ_MEMORY_ID: MemoryId = MemoryId::new(21);
