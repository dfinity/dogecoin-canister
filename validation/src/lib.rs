mod header;
mod block;

#[cfg(test)]
mod fixtures;


pub use crate::block::{BlockValidator, ValidateBlockError};
pub use crate::header::{HeaderStore, HeaderValidator, ValidateHeaderError};

#[cfg(feature = "btc")]
pub use crate::header::btc::BitcoinHeaderValidator;

#[cfg(feature = "doge")]
pub use crate::header::{
    doge::DogecoinHeaderValidator, AuxPowHeaderValidator, ValidateAuxPowHeaderError,
};

type BlockHeight = u32;
