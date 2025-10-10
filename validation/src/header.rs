#[cfg(feature = "btc")]
pub mod btc;
#[cfg(feature = "doge")]
pub mod doge;
#[cfg(test)]
mod tests;

#[cfg(feature = "doge")]
use bitcoin::dogecoin::Header as AuxPowHeader;

use crate::BlockHeight;
use bitcoin::{block::Header, BlockHash, CompactTarget, Target};
use std::time::Duration;

/// An error thrown when trying to validate a header.
#[derive(Debug, PartialEq)]
pub enum ValidateHeaderError {
    /// Used when the timestamp in the header is lower than
    /// the median of timestamps of past 11 headers.
    HeaderIsOld,
    /// Used when the timestamp in the header is more than 2 hours
    /// from the current time.
    HeaderIsTooFarInFuture {
        block_time: u64,
        max_allowed_time: u64,
    },
    /// Used when the PoW in the header is invalid as per the target mentioned
    /// in the header.
    InvalidPoWForHeaderTarget,
    /// Used when the PoW in the header is invalid as per the target
    /// computed based on the previous headers.
    InvalidPoWForComputedTarget,
    /// Used when the target in the header is greater than the max possible
    /// value.
    TargetDifficultyAboveMax,
    /// Used when the predecessor of the input header is not found in the
    /// HeaderStore.
    PrevHeaderNotFound,
    #[cfg(feature = "doge")]
    /// Used when the AuxPow Header fails validation
    ValidateAuxPowHeader(ValidateAuxPowHeaderError),
}

#[cfg(feature = "doge")]
#[derive(Debug, PartialEq)]
pub enum ValidateAuxPowHeaderError {
    /// Used when version field is obsolete
    VersionObsolete,
    /// Used when legacy blocks are not allowed
    LegacyBlockNotAllowed,
    /// Used when AuxPow blocks are not allowed
    AuxPowBlockNotAllowed,
    /// Used when the chain ID in the header is invalid
    InvalidChainId,
    /// Used when the AuxPow bit in the version field is not set properly
    InconsistentAuxPowBitSet,
    /// Used when the AuxPow proof is incorrect
    InvalidAuxPoW,
    /// Used when the PoW in the parent block is invalid
    InvalidParentPoW,
}

#[cfg(feature = "doge")]
impl From<ValidateAuxPowHeaderError> for ValidateHeaderError {
    fn from(err: ValidateAuxPowHeaderError) -> Self {
        ValidateHeaderError::ValidateAuxPowHeader(err)
    }
}

const ONE_HOUR: Duration = Duration::from_secs(3_600);

pub trait HeaderStore {
    /// Returns the header with the given block hash.
    fn get_with_block_hash(&self, hash: &BlockHash) -> Option<Header>;

    /// Returns the header at the given height.
    fn get_with_height(&self, height: u32) -> Option<Header>;

    /// Returns the height of the tip that the new header will extend.
    fn height(&self) -> u32;

    /// Returns the initial hash the store starts from.
    fn get_initial_hash(&self) -> BlockHash {
        self.get_with_height(0)
            .expect("genesis block header not found")
            .block_hash()
    }

    /// Adds a header to the store.
    fn add(&mut self, header: Header);
}

fn timestamp_is_at_most_2h_in_future(
    block_time: Duration,
    current_time: Duration,
) -> Result<(), ValidateHeaderError> {
    let max_allowed_time = current_time + 2 * ONE_HOUR;

    if block_time > max_allowed_time {
        return Err(ValidateHeaderError::HeaderIsTooFarInFuture {
            block_time: block_time.as_secs(),
            max_allowed_time: max_allowed_time.as_secs(),
        });
    }

    Ok(())
}

/// Validates if a header's timestamp is valid.
/// Bitcoin Protocol Rules wiki https://en.bitcoin.it/wiki/Protocol_rules says,
/// "Reject if timestamp is the median time of the last 11 blocks or before"
/// "Block timestamp must not be more than two hours in the future"
fn is_timestamp_valid(
    store: &impl HeaderStore,
    header: &Header,
    current_time: Duration,
) -> Result<(), ValidateHeaderError> {
    timestamp_is_at_most_2h_in_future(Duration::from_secs(header.time as u64), current_time)?;
    let mut times = vec![];
    let mut current_header: Header = *header;
    let initial_hash = store.get_initial_hash();
    for _ in 0..11 {
        if let Some(prev_header) = store.get_with_block_hash(&current_header.prev_blockhash) {
            times.push(prev_header.time);
            if current_header.prev_blockhash == initial_hash {
                break;
            }
            current_header = prev_header;
        }
    }

    times.sort_unstable();
    let median = times[times.len() / 2];
    if header.time <= median {
        return Err(ValidateHeaderError::HeaderIsOld);
    }

    Ok(())
}

pub trait HeaderValidator {
    type Network;
    type Store: HeaderStore;

    fn network(&self) -> &Self::Network;

    /// Returns a reference to the header store.
    fn store(&self) -> &Self::Store;

    /// Returns a mutable reference to the header store.
    fn store_mut(&mut self) -> &mut Self::Store;

    /// Returns the maximum difficulty target depending on the network
    fn max_target(&self) -> Target;

    /// Returns false iff PoW difficulty level of blocks can be
    /// readjusted in the network after a fixed time interval.
    fn no_pow_retargeting(&self) -> bool;

    /// Returns the PoW limit bits depending on the network
    fn pow_limit_bits(&self) -> CompactTarget;

    /// Returns the target spacing between blocks in seconds.
    fn pow_target_spacing(&self) -> Duration;

    /// Returns the number of blocks between difficulty adjustments at the given height.
    fn difficulty_adjustment_interval(&self, height: u32) -> u32;

    /// Returns `true` if mining a min-difficulty block is allowed after some delay.
    fn allow_min_difficulty_blocks(&self, height: u32) -> bool;

    /// Validates a header. If a failure occurs, a
    /// [ValidateHeaderError](ValidateHeaderError) will be returned.
    fn validate_header(
        &self,
        header: &Header,
        current_time: Duration,
    ) -> Result<(), ValidateHeaderError>;

    /// Returns the next required target at the given timestamp.
    /// The target is the number that a block hash must be below for it to be accepted.
    fn get_next_target(
        &self,
        prev_header: &Header,
        prev_height: BlockHeight,
        timestamp: u32,
    ) -> Target;

    /// This method is only valid when used for testnet and regtest networks.
    /// As per "https://en.bitcoin.it/wiki/Testnet",
    /// "If no block has been found in 20 minutes, the difficulty automatically
    /// resets back to the minimum for a single block, after which it
    /// returns to its previous value." This function is used to compute the
    /// difficulty target in case the block has been found within 20
    /// minutes.
    fn find_next_difficulty_in_chain(
        &self,
        prev_header: &Header,
        prev_height: BlockHeight,
    ) -> CompactTarget;

    /// This function returns the difficulty target to be used for the current
    /// header given the previous header in the Bitcoin network
    fn compute_next_difficulty(
        &self,
        prev_header: &Header,
        prev_height: BlockHeight,
    ) -> CompactTarget;
}

#[cfg(feature = "doge")]
pub trait AuxPowHeaderValidator: HeaderValidator {
    /// Returns `true` if the strict-chain-id rule is enabled.
    fn strict_chain_id(&self) -> bool;
    /// Returns the chain id used in this blockchain for AuxPow mining.
    fn auxpow_chain_id(&self) -> i32;
    /// Returns `true` if mining a legacy block is allowed.
    fn allow_legacy_blocks(&self, height: u32) -> bool;

    /// Validates an AuxPow header. If a failure occurs, a
    /// [ValidateHeaderError](ValidateHeaderError) will be returned.
    fn validate_auxpow_header(
        &self,
        header: &AuxPowHeader,
        current_time: Duration,
    ) -> Result<(), ValidateHeaderError>;
}
