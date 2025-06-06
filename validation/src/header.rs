#[cfg(test)]
mod tests;

use crate::{
    constants::{
        max_target, no_pow_retargeting, pow_limit_bits, DIFFICULTY_ADJUSTMENT_INTERVAL_BITCOIN,
        DIFFICULTY_ADJUSTMENT_INTERVAL_DOGECOIN, TEN_MINUTES,
    },
    BlockHeight,
};
use bitcoin::{
    block::Header, dogecoin::Network as DogecoinNetwork, BlockHash, CompactTarget,
    Network as BitcoinNetwork, Target,
};
use ic_doge_types::BlockchainNetwork;

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

/// Validates a header. If a failure occurs, a
/// [ValidateHeaderError](ValidateHeaderError) will be returned.
pub fn validate_header(
    network: &BlockchainNetwork,
    store: &impl HeaderStore,
    header: &Header,
    current_time: u64,
) -> Result<(), ValidateHeaderError> {
    let prev_height = store.height();
    let prev_header = match store.get_with_block_hash(&header.prev_blockhash) {
        Some(result) => result,
        None => {
            return Err(ValidateHeaderError::PrevHeaderNotFound);
        }
    };

    is_timestamp_valid(store, header, current_time)?;

    let header_target = header.target();
    if header_target > max_target(network) {
        return Err(ValidateHeaderError::TargetDifficultyAboveMax);
    }

    match network {
        BlockchainNetwork::Bitcoin(_) => header.validate_pow(header_target),
        BlockchainNetwork::Dogecoin(_) => header.validate_pow_with_scrypt(header_target),
    }
    .map_err(|_| ValidateHeaderError::InvalidPoWForHeaderTarget)?;

    let target = get_next_target(network, store, &prev_header, prev_height, header.time);

    match network {
        BlockchainNetwork::Bitcoin(_) => header.validate_pow(target),
        BlockchainNetwork::Dogecoin(_) => header.validate_pow_with_scrypt(target),
    }
    .map_err(|err| {
        match err {
            bitcoin::block::ValidationError::BadProofOfWork => println!("bad proof of work"),
            bitcoin::block::ValidationError::BadTarget => println!("bad target"),
            _ => {}
        };
        ValidateHeaderError::InvalidPoWForComputedTarget
    })?;

    Ok(())
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

// Returns the next required target at the given timestamp.
// The target is the number that a block hash must be below for it to be accepted.
fn get_next_target(
    network: &BlockchainNetwork,
    store: &impl HeaderStore,
    prev_header: &Header,
    prev_height: BlockHeight,
    timestamp: u32,
) -> Target {
    match network {
        BlockchainNetwork::Bitcoin(btc_network) => {
            match btc_network {
                BitcoinNetwork::Testnet | BitcoinNetwork::Testnet4 | BitcoinNetwork::Regtest => {
                    if (prev_height + 1) % DIFFICULTY_ADJUSTMENT_INTERVAL_BITCOIN != 0 {
                        // This if statements is reached only for Regtest and Testnet networks
                        // Here is the quote from "https://en.bitcoin.it/wiki/Testnet"
                        // "If no block has been found in 20 minutes, the difficulty automatically
                        // resets back to the minimum for a single block, after which it
                        // returns to its previous value."
                        if timestamp > prev_header.time + TEN_MINUTES * 2 {
                            // If no block has been found in 20 minutes, then use the maximum difficulty
                            // target
                            max_target(&BlockchainNetwork::Bitcoin(*btc_network))
                        } else {
                            // If the block has been found within 20 minutes, then use the previous
                            // difficulty target that is not equal to the maximum difficulty target
                            Target::from_compact(find_next_difficulty_in_chain(
                                btc_network,
                                store,
                                prev_header,
                                prev_height,
                            ))
                        }
                    } else {
                        Target::from_compact(compute_next_difficulty(
                            btc_network,
                            store,
                            prev_header,
                            prev_height,
                        ))
                    }
                }
                BitcoinNetwork::Bitcoin | BitcoinNetwork::Signet => Target::from_compact(
                    compute_next_difficulty(btc_network, store, prev_header, prev_height),
                ),
                &other => unreachable!("Unsupported network: {:?}", other),
            }
        }
        BlockchainNetwork::Dogecoin(doge_network) => {
            match doge_network {
                DogecoinNetwork::Dogecoin => Target::from_compact(
                    compute_next_difficulty_dogecoin(doge_network, store, prev_header, prev_height),
                ),
                &other => unreachable!("Unsupported network: {:?}", other),
            }
        }
    }
}

/// This method is only valid when used for testnet and regtest networks.
/// As per "https://en.bitcoin.it/wiki/Testnet",
/// "If no block has been found in 20 minutes, the difficulty automatically
/// resets back to the minimum for a single block, after which it
/// returns to its previous value." This function is used to compute the
/// difficulty target in case the block has been found within 20
/// minutes.
fn find_next_difficulty_in_chain(
    network: &BitcoinNetwork,
    store: &impl HeaderStore,
    prev_header: &Header,
    prev_height: BlockHeight,
) -> CompactTarget {
    // This is the maximum difficulty target for the network
    let pow_limit_bits = pow_limit_bits(&BlockchainNetwork::Bitcoin(*network));
    match network {
        BitcoinNetwork::Testnet | BitcoinNetwork::Testnet4 | BitcoinNetwork::Regtest => {
            let mut current_header = *prev_header;
            let mut current_height = prev_height;
            let mut current_hash = current_header.block_hash();
            let initial_header_hash = store.get_initial_hash();

            // Keep traversing the blockchain backwards from the recent block to initial
            // header hash.
            loop {
                // Check if non-limit PoW found or it's time to adjust difficulty.
                if current_header.bits != pow_limit_bits
                    || current_height % DIFFICULTY_ADJUSTMENT_INTERVAL_BITCOIN == 0
                {
                    return current_header.bits;
                }

                // Stop if we reach the initial header.
                if current_hash == initial_header_hash {
                    break;
                }

                // Traverse to the previous header.
                let prev_blockhash = current_header.prev_blockhash;
                current_header = store
                    .get_with_block_hash(&prev_blockhash)
                    .expect("previous header should be in the header store");
                // Update the current height and hash.
                current_height -= 1;
                current_hash = prev_blockhash;
            }
            pow_limit_bits
        }
        BitcoinNetwork::Bitcoin | BitcoinNetwork::Signet => pow_limit_bits,
        &other => unreachable!("Unsupported network: {:?}", other),
    }
}

/// This function returns the difficulty target to be used for the current
/// header given the previous header in the Bitcoin network
fn compute_next_difficulty(
    network: &BitcoinNetwork,
    store: &impl HeaderStore,
    prev_header: &Header,
    prev_height: BlockHeight,
) -> CompactTarget {
    // Difficulty is adjusted only once in every interval of 2 weeks (2016 blocks)
    // If an interval boundary is not reached, then previous difficulty target is
    // returned Regtest network doesn't adjust PoW difficulty levels. For
    // regtest, simply return the previous difficulty target.

    let height = prev_height + 1;
    if height % DIFFICULTY_ADJUSTMENT_INTERVAL_BITCOIN != 0
        || no_pow_retargeting(&BlockchainNetwork::Bitcoin(*network))
    {
        return prev_header.bits;
    }
    // Computing the `last_adjustment_header`.
    // `last_adjustment_header` is the last header with height multiple of 2016
    let last_adjustment_height = if height < DIFFICULTY_ADJUSTMENT_INTERVAL_BITCOIN {
        0
    } else {
        height - DIFFICULTY_ADJUSTMENT_INTERVAL_BITCOIN
    };
    let last_adjustment_header = store
        .get_with_height(last_adjustment_height)
        .expect("Last adjustment header must exist");

    // Block Storm Fix
    // The mitigation consists of no longer applying the adjustment factor
    // to the last block of the previous difficulty period. Instead,
    // the first block of the difficulty period is used as the base.
    // See https://github.com/bitcoin/bips/blob/master/bip-0094.mediawiki#block-storm-fix
    let last = match network {
        BitcoinNetwork::Testnet4 => last_adjustment_header.bits,
        _ => prev_header.bits,
    };

    // Computing the time interval between the last adjustment header time and
    // current time. The expected value timespan is 2 weeks assuming
    // the expected block time is 10 mins. But most of the time, the
    // timespan will deviate slightly from 2 weeks. Our goal is to
    // readjust the difficulty target so that the expected time taken for the next
    // 2016 blocks is again 2 weeks.
    // IMPORTANT: The bitcoin protocol allows for a roughly 3-hour window around
    // timestamp (1 hour in the past, 2 hours in the future) meaning that
    // the timespan can be negative on testnet networks.
    let last_adjustment_time = last_adjustment_header.time;
    let timespan = prev_header.time.saturating_sub(last_adjustment_time) as u64;

    CompactTarget::from_next_work_required(last, timespan, *network)
}

/// This function returns the difficulty target to be used for the current
/// header given the previous header in the Dogecoin network
fn compute_next_difficulty_dogecoin(
    network: &DogecoinNetwork,
    store: &impl HeaderStore,
    prev_header: &Header,
    prev_height: BlockHeight,
) -> CompactTarget {
    // Difficulty is adjusted only once in every interval of 4 hours (240 blocks)
    // If an interval boundary is not reached, then previous difficulty target is
    // returned Regtest network doesn't adjust PoW difficulty levels. For
    // regtest, simply return the previous difficulty target.

    let height = prev_height + 1;
    if height % DIFFICULTY_ADJUSTMENT_INTERVAL_DOGECOIN != 0
        || no_pow_retargeting(&BlockchainNetwork::Dogecoin(*network))
    {
        return prev_header.bits;
    }
    // Computing the `last_adjustment_header`.
    // `last_adjustment_header` is the last header with height multiple of 240
    // Dogecoin solves the "off-by-one" or Time Wrap bug in Bitcoin by going back to the full retarget period.
    // See: <https://litecoin.info/docs/history/time-warp-attack>
    let last_adjustment_height = if height <= DIFFICULTY_ADJUSTMENT_INTERVAL_DOGECOIN {
        0
    } else {
        height - DIFFICULTY_ADJUSTMENT_INTERVAL_DOGECOIN - 1
    };
    let last_adjustment_header = store
        .get_with_height(last_adjustment_height)
        .expect("Last adjustment header must exist");

    // Computing the time interval between the last adjustment header time and
    // current time. The expected value timespan is 4 hours assuming
    // the expected block time is 1 min. But most of the time, the
    // timespan will deviate slightly from 4 hours. Our goal is to
    // readjust the difficulty target so that the expected time taken for the next
    // 240 blocks is again 4 hours.
    // IMPORTANT: The dogecoin protocol allows for a roughly 2-hour window around
    // timestamp (6 min in the past, 2 hours in the future) meaning that
    // the timespan can be negative on testnet networks.
    let last_adjustment_time = last_adjustment_header.time;
    let timespan = prev_header.time.saturating_sub(last_adjustment_time) as u64;

    CompactTarget::from_next_work_required_dogecoin(prev_header.bits, timespan, *network)
}
