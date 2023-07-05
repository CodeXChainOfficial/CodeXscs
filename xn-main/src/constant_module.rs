multiversx_sc::imports!();
multiversx_sc::derive_imports!();

pub const GRACE_PERIOD: u64 = 21 * 24 * 60 * 60; // 21 days
pub const YEAR_IN_SECONDS: u64 = 365 * 24 * 60 * 60; // 1 year (365 days)
pub const MONTH_IN_SECONDS: u64 = 30 * 24 * 60 * 60; // 1 month (30 days)
pub const DAY_IN_SECONDS: u64 = 24 * 60 * 60; // 1 day
pub const HOUR_IN_SECONDS: u64 = 60 * 60; // 1 hour
pub const MIN_IN_SECONDS: u64 = 60; // 1 min
pub const MIN_LENGTH: usize = 3;
pub const MAX_LENGTH: usize = 256;
pub const NFT_AMOUNT: u32 = 1;
pub const SUB_DOMAIN_COST_USD: u64 = 2_500_000_000u64;
pub const MIGRATION_PERIOD: u64 = 21 * 24 * 60 * 60; // 21 days
pub const WEGLD_ID: &[u8] = b"WEGLD-d7c6bb";
// pub const WEGLD_ID: &[u8] = b"WEGLD-bd4d79";