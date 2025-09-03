mod header;

pub use crate::header::{HeaderStore, HeaderValidator, ValidateHeaderError};

#[cfg(feature = "btc")]
pub use crate::header::btc::BitcoinHeaderValidator;

#[cfg(feature = "doge")]
pub use crate::header::doge::DogecoinHeaderValidator;

type BlockHeight = u32;
