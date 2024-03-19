#![no_std]

pub const ERROR_EMPTY_PAYMENTS: &str = "Empty payments";
pub const ERROR_ZERO_AMOUNT: &str = "Invalid amount zero";
pub const ERROR_ZERO_TOKEN_NONCE: &str = "Invalid token nonce";

pub const ERROR_SLIPPAGE_SCREW_YOU: &str = "Slippage too high";
pub const ERROR_INVALID_AMOUNT_IN: &str = "Invalid amount in";
pub const ERROR_INVALID_TOKEN_IN: &str = "Invalid token in";
pub const ERROR_INVALID_TOKEN_OUT: &str = "Invalid token out";
pub const ERROR_INSUFFICIENT_AMOUNT: &str = "Insufficient amount";

pub const ERROR_OUTPUT_LEN_MISMATCH: &str = "Output length mismatch";
pub const ERROR_INVALID_POOL_ADDR: &str = "Invalid pool address";

pub const ERROR_PROTOCOL_NOT_REGISTED: &str = "Protocol unregistered";
pub const ERROR_INVALID_FEE_PERCENT: &str = "Invalid fee percent";
pub const ERROR_INVALID_ADDRESS: &str = "Invalid contract";

pub const ERROR_SAME_TOKEN: &str = "Same token";
pub const ERROR_INVALID_STEPS: &str = "Invalid step";
pub const ERROR_INVALID_FUNCTION_NAME: &str = "Invalid function name";
pub const ERROR_INVALID_FUNCTION_ARGS: &str = "Invalid function args";
