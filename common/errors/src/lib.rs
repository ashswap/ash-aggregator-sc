#![no_std]

pub const ERROR_EMPTY_PAYMENTS: &str = "Empty payments";
pub const ERROR_ZERO_AMOUNT: &str = "Invalid zero amount";
pub const ERROR_ZERO_TOKEN_NONCE: &str = "Invalid token nonce";

pub const ERROR_SLIPPAGE_SCREW_YOU: &str = "Slippage screw you";
pub const ERROR_INVALID_AMOUNT_IN: &str = "Invalid amount in";
pub const ERROR_INVALID_TOKEN_IN: &str = "Invalid token in";

pub const ERROR_OUTPUT_LEN_MISMATCH: &str = "Output length mismatch";
pub const ERROR_INVALID_POOL_ADDR: &str = "Invalid pool address";

pub const ERROR_PROTOCOL_NOT_REGISTED: &str = "Protocol hasn't registered";
pub const ERROR_INVALID_FEE_PERCENT: &str = "Invalid fee percent";
pub const ERROR_INVALID_ADDRESS: &str = "Invalid contract";