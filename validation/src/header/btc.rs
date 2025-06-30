use crate::constants::btc::DIFFICULTY_ADJUSTMENT_INTERVAL_BITCOIN;
use crate::header::{is_timestamp_valid, HeaderStore, HeaderValidator, ValidateHeaderError};
use crate::BlockHeight;
use bitcoin::network::Network as BitcoinNetwork;
use bitcoin::{block::Header, CompactTarget, Target};
use std::time::Duration;

pub struct BitcoinHeaderValidator {
    network: BitcoinNetwork,
}

impl BitcoinHeaderValidator {
    pub fn new(network: BitcoinNetwork) -> Self {
        Self { network }
    }

    pub fn mainnet() -> Self {
        Self::new(BitcoinNetwork::Bitcoin)
    }

    pub fn testnet() -> Self {
        Self::new(BitcoinNetwork::Testnet)
    }

    pub fn regtest() -> Self {
        Self::new(BitcoinNetwork::Regtest)
    }
}

impl HeaderValidator for BitcoinHeaderValidator {
    type Network = BitcoinNetwork;

    fn network(&self) -> &Self::Network {
        &self.network
    }

    fn max_target(&self) -> Target {
        match self.network() {
            Self::Network::Bitcoin => Target::MAX_ATTAINABLE_MAINNET,
            Self::Network::Testnet | Self::Network::Testnet4 => Target::MAX_ATTAINABLE_TESTNET,
            Self::Network::Regtest => Target::MAX_ATTAINABLE_REGTEST,
            Self::Network::Signet => Target::MAX_ATTAINABLE_SIGNET,
            &other => unreachable!("Unsupported network: {:?}", other),
        }
    }

    fn no_pow_retargeting(&self) -> bool {
        match self.network() {
            Self::Network::Bitcoin
            | Self::Network::Testnet
            | Self::Network::Testnet4
            | Self::Network::Signet => false,
            Self::Network::Regtest => true,
            &other => unreachable!("Unsupported network: {:?}", other),
        }
    }

    fn pow_limit_bits(&self) -> CompactTarget {
        let bits = match self.network() {
            Self::Network::Bitcoin => 0x1d00ffff,
            Self::Network::Testnet | Self::Network::Testnet4 => 0x1d00ffff,
            Self::Network::Regtest => 0x207fffff,
            Self::Network::Signet => 0x1e0377ae,
            &other => unreachable!("Unsupported network: {:?}", other),
        };
        CompactTarget::from_consensus(bits)
    }

    fn pow_target_spacing(&self) -> Duration {
        Duration::from_secs(self.network().params().pow_target_spacing)
    }

    fn validate_header(
        &self,
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

        if self.network != BitcoinNetwork::Testnet4 {
            // Skip timestamp validation for Testnet4; the first 2 blocks are 2 days apart.
            // https://mempool.space/testnet4/block/00000000da84f2bafbbc53dee25a72ae507ff4914b867c565be350b0da8bf043
            is_timestamp_valid(store, header, current_time)?;
        }

        let header_target = header.target();
        if header_target > self.max_target() {
            return Err(ValidateHeaderError::TargetDifficultyAboveMax);
        }

        if header.validate_pow(header_target).is_err() {
            return Err(ValidateHeaderError::InvalidPoWForHeaderTarget);
        }

        let target = self.get_next_target(store, &prev_header, prev_height, header.time);
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
        store: &impl HeaderStore,
        prev_header: &Header,
        prev_height: BlockHeight,
        timestamp: u32,
    ) -> Target {
        match self.network() {
            BitcoinNetwork::Testnet | BitcoinNetwork::Testnet4 | BitcoinNetwork::Regtest => {
                if (prev_height + 1) % DIFFICULTY_ADJUSTMENT_INTERVAL_BITCOIN != 0 {
                    // This if statements is reached only for Regtest and Testnet networks
                    // Here is the quote from "https://en.bitcoin.it/wiki/Testnet"
                    // "If no block has been found in 20 minutes, the difficulty automatically
                    // resets back to the minimum for a single block, after which it
                    // returns to its previous value."
                    if timestamp
                        > prev_header.time + (self.pow_target_spacing() * 2).as_secs() as u32
                    {
                        // If no block has been found in 20 minutes, then use the maximum difficulty
                        // target
                        self.max_target()
                    } else {
                        // If the block has been found within 20 minutes, then use the previous
                        // difficulty target that is not equal to the maximum difficulty target
                        Target::from_compact(self.find_next_difficulty_in_chain(
                            store,
                            prev_header,
                            prev_height,
                        ))
                    }
                } else {
                    Target::from_compact(self.compute_next_difficulty(
                        store,
                        prev_header,
                        prev_height,
                    ))
                }
            }
            BitcoinNetwork::Bitcoin | BitcoinNetwork::Signet => {
                Target::from_compact(self.compute_next_difficulty(store, prev_header, prev_height))
            }
            &other => unreachable!("Unsupported network: {:?}", other),
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
        store: &impl HeaderStore,
        prev_header: &Header,
        prev_height: BlockHeight,
    ) -> CompactTarget {
        // This is the maximum difficulty target for the network
        let pow_limit_bits = self.pow_limit_bits();
        match self.network() {
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

    fn compute_next_difficulty(
        &self,
        store: &impl HeaderStore,
        prev_header: &Header,
        prev_height: BlockHeight,
    ) -> CompactTarget {
        // Difficulty is adjusted only once in every interval of 2 weeks (2016 blocks)
        // If an interval boundary is not reached, then previous difficulty target is
        // returned Regtest network doesn't adjust PoW difficulty levels. For
        // regtest, simply return the previous difficulty target.

        let height = prev_height + 1;
        if height % DIFFICULTY_ADJUSTMENT_INTERVAL_BITCOIN != 0 || self.no_pow_retargeting() {
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
        let last = match self.network() {
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

        CompactTarget::from_next_work_required(last, timespan, self.network)
    }
}
