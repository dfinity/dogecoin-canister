use crate::constants::doge::DIFFICULTY_ADJUSTMENT_INTERVAL_DOGECOIN;
use crate::header::{is_timestamp_valid, HeaderStore, HeaderValidator, ValidateHeaderError};
use crate::BlockHeight;
use bitcoin::dogecoin::Network as DogecoinNetwork;
use bitcoin::{block::Header, CompactTarget, Target};
use std::time::Duration;

pub struct DogecoinHeaderValidator {
    network: DogecoinNetwork,
}

impl DogecoinHeaderValidator {
    pub fn new(network: DogecoinNetwork) -> Self {
        Self { network }
    }

    pub fn mainnet() -> Self {
        Self::new(DogecoinNetwork::Dogecoin)
    }

    pub fn testnet() -> Self {
        Self::new(DogecoinNetwork::Testnet)
    }

    pub fn regtest() -> Self {
        Self::new(DogecoinNetwork::Regtest)
    }
}

impl HeaderValidator for DogecoinHeaderValidator {
    type Network = DogecoinNetwork;

    fn network(&self) -> &Self::Network {
        &self.network
    }

    /// Returns the maximum difficulty target depending on the network
    fn max_target(&self) -> Target {
        match self.network() {
            Self::Network::Dogecoin => Target::MAX_ATTAINABLE_MAINNET_DOGE,
            Self::Network::Testnet => Target::MAX_ATTAINABLE_TESTNET_DOGE,
            Self::Network::Regtest => Target::MAX_ATTAINABLE_REGTEST_DOGE,
            &other => unreachable!("Unsupported network: {:?}", other),
        }
    }

    /// Returns false iff PoW difficulty level of blocks can be
    /// readjusted in the network after a fixed time interval.
    fn no_pow_retargeting(&self) -> bool {
        match self.network() {
            Self::Network::Dogecoin | Self::Network::Testnet => false,
            Self::Network::Regtest => true,
            &other => unreachable!("Unsupported network: {:?}", other),
        }
    }

    /// Returns the PoW limit bits depending on the network
    fn pow_limit_bits(&self) -> CompactTarget {
        let bits = match self.network() {
            Self::Network::Dogecoin => 0x1e0fffff, // In Dogecoin this is higher than the Genesis compact target (0x1e0ffff0)
            Self::Network::Testnet => 0x1e0fffff,
            Self::Network::Regtest => 0x207fffff,
            &other => unreachable!("Unsupported network: {:?}", other),
        };
        CompactTarget::from_consensus(bits)
    }

    fn pow_target_spacing(&self) -> Duration {
        Duration::from_secs(self.network().params().bitcoin_params.pow_target_spacing)
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

        is_timestamp_valid(store, header, current_time)?;

        let header_target = header.target();
        if header_target > self.max_target() {
            return Err(ValidateHeaderError::TargetDifficultyAboveMax);
        }

        if header.validate_pow_with_scrypt(header_target).is_err() {
            return Err(ValidateHeaderError::InvalidPoWForHeaderTarget);
        }

        let target = self.get_next_target(store, &prev_header, prev_height, header.time);

        if let Err(err) = header.validate_pow_with_scrypt(target) {
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
            DogecoinNetwork::Testnet | DogecoinNetwork::Regtest => {
                if (prev_height + 1) % DIFFICULTY_ADJUSTMENT_INTERVAL_DOGECOIN != 0 {
                    if timestamp
                        > prev_header.time + (self.pow_target_spacing() * 2).as_secs() as u32
                    {
                        // If no block has been found in `pow_target_spacing * 2` minutes, then use
                        // the maximum difficulty target
                        self.max_target()
                    } else {
                        // If the block has been found within `pow_target_spacing * 2` minutes, then
                        // use the previous difficulty target that is not equal to the maximum
                        // difficulty target
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
            DogecoinNetwork::Dogecoin => {
                Target::from_compact(self.compute_next_difficulty(store, prev_header, prev_height))
            }
            &other => unreachable!("Unsupported network: {:?}", other),
        }
    }

    fn find_next_difficulty_in_chain(
        &self,
        store: &impl HeaderStore,
        prev_header: &Header,
        prev_height: BlockHeight,
    ) -> CompactTarget {
        // This is the maximum difficulty target for the network
        let pow_limit_bits = self.pow_limit_bits();
        match self.network() {
            DogecoinNetwork::Testnet | DogecoinNetwork::Regtest => {
                let mut current_header = *prev_header;
                let mut current_height = prev_height;
                let mut current_hash = current_header.block_hash();
                let initial_header_hash = store.get_initial_hash();

                // Keep traversing the blockchain backwards from the recent block to initial
                // header hash.
                loop {
                    // Check if non-limit PoW found or it's time to adjust difficulty.
                    if current_header.bits != pow_limit_bits
                        || current_height % DIFFICULTY_ADJUSTMENT_INTERVAL_DOGECOIN == 0
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
            DogecoinNetwork::Dogecoin => pow_limit_bits,
            &other => unreachable!("Unsupported network: {:?}", other),
        }
    }

    fn compute_next_difficulty(
        &self,
        store: &impl HeaderStore,
        prev_header: &Header,
        prev_height: BlockHeight,
    ) -> CompactTarget {
        // Difficulty is adjusted only once in every interval of 4 hours (240 blocks)
        // If an interval boundary is not reached, then previous difficulty target is
        // returned. Regtest network doesn't adjust PoW difficulty levels. For
        // regtest, simply return the previous difficulty target.

        let height = prev_height + 1;
        if height % DIFFICULTY_ADJUSTMENT_INTERVAL_DOGECOIN != 0 || self.no_pow_retargeting() {
            return prev_header.bits;
        }
        // Computing the `last_adjustment_header`.
        // `last_adjustment_header` is the last header with height multiple of 240 - 1
        // Dogecoin solves the "off-by-one" or time wrap bug in Bitcoin by going back to the full
        // retarget period (hence the - 1).
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
        // IMPORTANT: With the Median Time Past (MTP) rule, a block's timestamp
        // is only required to be greater than the median of the previous 11 blocks.
        // This allows individual block timestamps to decrease relative to their
        // predecessor, which can result in a negative timespan.
        let last_adjustment_time = last_adjustment_header.time;
        let timespan = prev_header.time.saturating_sub(last_adjustment_time) as u64;

        CompactTarget::from_next_work_required_dogecoin(
            prev_header.bits,
            timespan,
            self.network,
            height,
        )
    }
}
