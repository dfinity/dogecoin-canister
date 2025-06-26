#[cfg(feature = "btc")]
pub mod btc;
#[cfg(feature = "doge")]
pub mod doge;

#[cfg(test)]
pub mod test {
    /// Needed to help test check for the 20 minute testnet/regtest rule
    pub const TEN_MINUTES: u32 = 60 * 10;
}
