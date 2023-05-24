multiversx_sc::imports!();
multiversx_sc::derive_imports!();

pub const GRACE_PERIOD: u64 = 21 * 24 * 60 * 60; // 21 days
pub const YEAR_IN_SECONDS: u64 = 365 * 24 * 60 * 60; // 1 year (365 days)
pub const MIN_LENGTH: usize = 3;
pub const MAX_LENGTH: usize = 256;
pub const NFT_AMOUNT: u32 = 1;