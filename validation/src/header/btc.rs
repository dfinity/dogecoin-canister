use crate::header::{is_timestamp_valid, HeaderStore, HeaderValidator, ValidateHeaderError};
use crate::BlockHeight;
use bitcoin::network::Network as BitcoinNetwork;
use bitcoin::{block::Header, CompactTarget, Target};
use std::time::Duration;

/// Expected number of blocks for 2 weeks in Bitcoin (2_016).
pub const DIFFICULTY_ADJUSTMENT_INTERVAL: BlockHeight = 6 * 24 * 14;

pub struct BitcoinHeaderValidator<T> {
    store: T,
    network: BitcoinNetwork,
}

impl<T> BitcoinHeaderValidator<T> {
    pub fn new(store: T, network: BitcoinNetwork) -> Self {
        Self { store, network }
    }

    pub fn mainnet(store: T) -> Self {
        Self::new(store, BitcoinNetwork::Bitcoin)
    }

    pub fn testnet(store: T) -> Self {
        Self::new(store, BitcoinNetwork::Testnet)
    }

    pub fn regtest(store: T) -> Self {
        Self::new(store, BitcoinNetwork::Regtest)
    }
}

impl<T: HeaderStore>  HeaderValidator for BitcoinHeaderValidator<T> {
    type Network = BitcoinNetwork;

    fn network(&self) -> &Self::Network {
        &self.network
    }

    fn max_target(&self) -> Target {
        self.network().params().max_attainable_target
    }

    fn no_pow_retargeting(&self) -> bool {
        self.network().params().no_pow_retargeting
    }

    fn pow_limit_bits(&self) -> CompactTarget {
        self.network()
            .params()
            .max_attainable_target
            .to_compact_lossy()
    }

    fn pow_target_spacing(&self) -> Duration {
        Duration::from_secs(self.network().params().pow_target_spacing)
    }

    fn difficulty_adjustment_interval(&self, _height: u32) -> u32 {
        (self.network().params().pow_target_timespan / self.network().params().pow_target_spacing)
            as u32
    }

    fn allow_min_difficulty_blocks(&self, _height: u32) -> bool {
        self.network().params().allow_min_difficulty_blocks
    }

    fn validate_header(
        &self,
        header: &Header,
        current_time: Duration,
    ) -> Result<(), ValidateHeaderError> {
        #[cfg(feature = "canbench-rs")]
        let _p = canbench_rs::bench_scope("validate_header");

        let prev_height = self.store.height();
        let prev_header = match self.store.get_with_block_hash(&header.prev_blockhash) {
            Some(result) => result,
            None => {
                return Err(ValidateHeaderError::PrevHeaderNotFound);
            }
        };

        self.is_timestamp_valid(header, current_time)?;

        let header_target = header.target();
        if header_target > max_target(&self.network) {
            return Err(ValidateHeaderError::TargetDifficultyAboveMax);
        }

        if header.validate_pow(header_target).is_err() {
            return Err(ValidateHeaderError::InvalidPoWForHeaderTarget);
        }

        let target = self.get_next_target(&prev_header, prev_height, header.time);
        if let Err(err) = header.validate_pow(target) {
            match err {
                bitcoin::block::ValidationError::BadProofOfWork => println!("bad proof of work"),
                bitcoin::block::ValidationError::BadTarget => println!("bad target"),
                _ => {}
            };
            return Err(ValidateHeaderError::InvalidPoWForComputedTarget);
        }
        Ok(())
    }

    fn get_next_target(
        &self,
        prev_header: &Header,
        prev_height: BlockHeight,
        timestamp: u32,
    ) -> Target {
        match self.network {
            BitcoinNetwork::Testnet | BitcoinNetwork::Testnet4 | BitcoinNetwork::Regtest => {
                if (prev_height + 1) % DIFFICULTY_ADJUSTMENT_INTERVAL != 0 {
                    // This if statements is reached only for Regtest and Testnet networks
                    // Here is the quote from "https://en.bitcoin.it/wiki/Testnet"
                    // "If no block has been found in 20 minutes, the difficulty automatically
                    // resets back to the minimum for a single block, after which it
                    // returns to its previous value."
                    if timestamp > prev_header.time + TEN_MINUTES * 2 {
                        // If no block has been found in 20 minutes, then use the maximum difficulty
                        // target
                        max_target(&self.network)
                    } else {
                        // If the block has been found within 20 minutes, then use the previous
                        // difficulty target that is not equal to the maximum difficulty target
                        Target::from_compact(
                            self.find_next_difficulty_in_chain(prev_header, prev_height),
                        )
                    }
                } else {
                    Target::from_compact(self.compute_next_difficulty(prev_header, prev_height))
                }
            }
            BitcoinNetwork::Bitcoin | BitcoinNetwork::Signet => {
                Target::from_compact(self.compute_next_difficulty(prev_header, prev_height))
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
        &self,
        prev_header: &Header,
        prev_height: BlockHeight,
    ) -> CompactTarget {
        // This is the maximum difficulty target for the network
        let pow_limit_bits = pow_limit_bits(&self.network);
        match self.network {
            BitcoinNetwork::Testnet | BitcoinNetwork::Testnet4 | BitcoinNetwork::Regtest => {
                let mut current_header = *prev_header;
                let mut current_height = prev_height;
                let mut current_hash = current_header.block_hash();
                let initial_header_hash = self.store.get_initial_hash();

                // Keep traversing the blockchain backwards from the recent block to initial
                // header hash.
                loop {
                    // Check if non-limit PoW found or it's time to adjust difficulty.
                    if current_header.bits != pow_limit_bits
                        || current_height % DIFFICULTY_ADJUSTMENT_INTERVAL == 0
                    {
                        return current_header.bits;
                    }

                    // Stop if we reach the initial header.
                    if current_hash == initial_header_hash {
                        break;
                    }

                    // Traverse to the previous header.
                    let prev_blockhash = current_header.prev_blockhash;
                    current_header = self
                        .store
                        .get_with_block_hash(&prev_blockhash)
                        .expect("previous header should be in the header store");
                    // Update the current height and hash.
                    current_height -= 1;
                    current_hash = prev_blockhash;
                }
                pow_limit_bits
            }
            BitcoinNetwork::Bitcoin | BitcoinNetwork::Signet => pow_limit_bits,
        }
    }

    /// This function returns the difficulty target to be used for the current
    /// header given the previous header
    fn compute_next_difficulty(
        &self,
        prev_header: &Header,
        prev_height: BlockHeight,
    ) -> CompactTarget {
        // Difficulty is adjusted only once in every interval of 2 weeks (2016 blocks)
        // If an interval boundary is not reached, then previous difficulty target is
        // returned Regtest network doesn't adjust PoW difficulty levels. For
        // regtest, simply return the previous difficulty target.

        let height = prev_height + 1;
        if height % DIFFICULTY_ADJUSTMENT_INTERVAL != 0 || no_pow_retargeting(&self.network) {
            return prev_header.bits;
        }
        // Computing the `last_adjustment_header`.
        // `last_adjustment_header` is the last header with height multiple of 2016
        let last_adjustment_height = if height < DIFFICULTY_ADJUSTMENT_INTERVAL {
            0
        } else {
            height - DIFFICULTY_ADJUSTMENT_INTERVAL
        };
        let last_adjustment_header = self
            .store
            .get_with_height(last_adjustment_height)
            .expect("Last adjustment header must exist");

        // Block Storm Fix
        // The mitigation consists of no longer applying the adjustment factor
        // to the last block of the previous difficulty period. Instead,
        // the first block of the difficulty period is used as the base.
        // See https://github.com/bitcoin/bips/blob/master/bip-0094.mediawiki#block-storm-fix
        let last = match self.network {
            Network::Testnet4 => last_adjustment_header.bits,
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

        CompactTarget::from_next_work_required(last, timespan, self.network)
    }
}
