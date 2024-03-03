// unify all error content for whole project
// sys related
pub const E000_ALREADY_INIT: &str = "E000: already initialized";
pub const E001_PROMISE_RESULT_COUNT_INVALID: &str = "E001: promise result count invalid";
pub const E002_NOT_ALLOWED: &str = "E002: not allowed for the caller";
pub const E003_NOT_INIT: &str = "E003: not initialized";
pub const E004_INVALID_GUARDIAN: &str = "E004: invalid guardian";
pub const E005_INVALID_TOKEN: &str = "E005: invalid token";
pub const ERR6_CONTRACT_PAUSED: &str = "E006: contract paused";

// buyback
pub const ERR100_WRONG_MSG_FORMAT: &str = "E100: illegal msg in ft_transfer_call";
pub const ERR101_BUYBACK_IN_PROGRESS: &str = "E101: the current round of buyback has not concluded yet";
pub const ERR102_CROSS_CONTRACT_FAILED: &str ="E102: cross contract call failed";