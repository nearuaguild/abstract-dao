use near_sdk::{Duration, Gas};

pub const ONE_MINUTE_NANOS: Duration = 60_000_000_000;

// 250Tgas is for MPC sign, 5Tgas for basic fn operations and 5Tgas for promise creation
pub const MIN_GAS_FOR_GET_SIGNATURE: Gas = Gas::from_tgas(260);

pub const GAS_FOR_PROMISE: Gas = Gas::from_tgas(5);
