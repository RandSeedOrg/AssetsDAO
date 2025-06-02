/// Memory related to event log ID definition
pub const STAKING_EVENT_LOG: u8 = 0;
pub const STAKING_EVENT_LOG_SEQ: u8 = 1;

/// Memory of the stake pool ID definition
pub const STAKING_POOL: u8 = 10;
pub const STAKING_POOL_SEQ: u8 = 11;

/// Memory of staked account ID definition
pub const STAKING_ACCOUNT: u8 = 20;
pub const STAKING_ACCOUNT_SEQ: u8 = 21;
pub const STAKING_USER_ACCOUNT_INDEX: u8 = 22;
pub const STAKING_POOL_ACCOUNT_INDEX: u8 = 23;
pub const STAKING_UNSTAKE_ON_DAY_ACCOUNT_INDEX: u8 = 24;
// A staked account index was generated that could recover errors
pub const STAKING_RECOVERABLE_ERROR_ACCOUNT_INDEX: u8 = 25;

/// Memory of stake reward ID definition
pub const STAKING_REWARD: u8 = 30;
pub const STAKING_REWARD_SEQ: u8 = 31;
pub const STAKING_ACCOUNT_REWARD_INDEX: u8 = 32;
pub const STAKING_USER_REWARD_INDEX: u8 = 33;
pub const STAKING_POOL_REWARD_INDEX: u8 = 34;
pub const STAKING_USER_ACCOUNT_REWARD_DATE_INDEX: u8 = 35;

/// Memory of subscription notifications ID definition
pub const STAKING_SUBSCRIPTION: u8 = 40;
pub const STAKING_SUBSCRIPTION_SEQ: u8 = 41;

/// Memory of staking pool transaction record ID definition
pub const STAKING_POOL_TRANSACTION_RECORD: u8 = 50;
pub const STAKING_POOL_TRANSACTION_RECORD_TYPE_INDEX: u8 = 51;

/// Memory of NNS staking record ID definition
pub const NNS_STAKING_EXECUTE_RECORD: u8 = 60;
