#[cfg(feature = "btc")]
pub mod btc;
#[cfg(feature = "doge")]
pub mod doge;

/// Needed to help test check for the 20 minutes testnet/regtest rule
pub const TEN_MINUTES: u32 = 60 * 10;
