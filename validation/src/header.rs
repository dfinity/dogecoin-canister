#[cfg(feature = "btc")]
pub mod btc;
#[cfg(feature = "doge")]
pub mod doge;
#[cfg(test)]
mod tests;

use crate::BlockHeight;
use bitcoin::{block::Header, BlockHash, CompactTarget, Target};

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
}

const ONE_HOUR: u64 = 3_600;

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
}

fn timestamp_is_less_than_2h_in_future(
    block_time: u64,
    current_time: u64,
) -> Result<(), ValidateHeaderError> {
    let max_allowed_time = current_time + 2 * ONE_HOUR;

    if block_time > max_allowed_time {
        return Err(ValidateHeaderError::HeaderIsTooFarInFuture {
            block_time,
            max_allowed_time,
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
    current_time: u64,
) -> Result<(), ValidateHeaderError> {
    timestamp_is_less_than_2h_in_future(header.time as u64, current_time)?;
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
    /// Returns the maximum difficulty target depending on the network
    fn max_target(&self) -> Target;

    /// Returns false iff PoW difficulty level of blocks can be
    /// readjusted in the network after a fixed time interval.
    fn no_pow_retargeting(&self) -> bool;

    /// Returns the PoW limit bits depending on the network
    fn pow_limit_bits(&self) -> CompactTarget;

    /// Validates a header. If a failure occurs, a
    /// [ValidateHeaderError](ValidateHeaderError) will be returned.
    fn validate_header(
        &self,
        store: &impl HeaderStore,
        header: &Header,
        current_time: u64,
    ) -> Result<(), ValidateHeaderError>;

    /// Returns the next required target at the given timestamp.
    /// The target is the number that a block hash must be below for it to be accepted.
    fn get_next_target(
        &self,
        store: &impl HeaderStore,
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
        store: &impl HeaderStore,
        prev_header: &Header,
        prev_height: BlockHeight,
    ) -> CompactTarget;

    /// This function returns the difficulty target to be used for the current
    /// header given the previous header in the Bitcoin network
    fn compute_next_difficulty(
        &self,
        store: &impl HeaderStore,
        prev_header: &Header,
        prev_height: BlockHeight,
    ) -> CompactTarget;
}
