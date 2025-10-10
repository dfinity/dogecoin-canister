#[cfg(feature = "btc")]
mod btc;
#[cfg(feature = "doge")]
mod doge;
mod utils;

use crate::header::tests::utils::test_data_file;
use crate::header::timestamp_is_at_most_2h_in_future;
use crate::header::{is_timestamp_valid, HeaderValidator, ONE_HOUR};
#[cfg(feature = "doge")]
use crate::header::{tests::utils::get_auxpow_headers, AuxPowHeaderValidator};
use crate::HeaderStore;
use crate::ValidateHeaderError;
use bitcoin::block::{Header, Version};
#[cfg(feature = "doge")]
use bitcoin::dogecoin::Header as AuxPowHeader;
use bitcoin::{CompactTarget, Target, TxMerkleNode};
use proptest::proptest;
use std::str::FromStr;
use std::time::Duration;
use utils::{get_headers, next_block_header, MOCK_CURRENT_TIME};

fn verify_consecutive_headers<T: HeaderValidator>(validator: &T, header: Header) {
    let result = validator.validate_header(&header, MOCK_CURRENT_TIME);
    assert!(result.is_ok());
}

#[cfg(feature = "doge")]
fn verify_consecutive_headers_auxpow<T: AuxPowHeaderValidator>(validator: T, header: AuxPowHeader) {
    let result = validator.validate_auxpow_header(&header, MOCK_CURRENT_TIME);
    assert!(result.is_ok());
}

fn verify_header_sequence<T: HeaderValidator>(mut validator: T, file: &str) {
    let headers = get_headers(file);
    for (i, header) in headers.iter().enumerate() {
        let result = validator.validate_header(header, MOCK_CURRENT_TIME);
        assert!(
            result.is_ok(),
            "Failed to validate header on line {} for header {}: {:?}",
            i,
            header.block_hash(),
            result
        );
        validator.store_mut().add(*header);
    }
}

#[cfg(feature = "doge")]
fn verify_header_sequence_auxpow<T: AuxPowHeaderValidator>(mut validator: T, file: &str) {
    let headers = get_auxpow_headers(file);
    for (i, header) in headers.iter().enumerate() {
        let result = validator.validate_auxpow_header(header, MOCK_CURRENT_TIME);
        assert!(
            result.is_ok(),
            "Failed to validate header on line {} for header {}: {:?}",
            i,
            header.block_hash(),
            result
        );
        validator.store_mut().add(header.pure_header);
    }
}

fn verify_with_missing_parent<T: HeaderValidator>(validator: &T, header: Header) {
    let result = validator.validate_header(&header, MOCK_CURRENT_TIME);
    assert!(matches!(
        result,
        Err(ValidateHeaderError::PrevHeaderNotFound)
    ));
}

fn verify_with_invalid_pow<T: HeaderValidator>(validator: &T, mut header: Header) {
    header.bits = validator.pow_limit_bits(); // Modify header to invalidate PoW
    let result = validator.validate_header(&header, MOCK_CURRENT_TIME);
    assert!(matches!(
        result,
        Err(ValidateHeaderError::InvalidPoWForHeaderTarget)
            | Err(ValidateHeaderError::InvalidPoWForComputedTarget)
    ));
}

fn verify_with_invalid_pow_with_computed_target<T: HeaderValidator>(
    validator_regtest: &mut T,
    genesis_header: Header,
) {
    let pow_regtest = validator_regtest.pow_limit_bits();
    let h1 = next_block_header(validator_regtest, genesis_header, pow_regtest);
    let h2 = next_block_header(validator_regtest, h1, pow_regtest);
    let h3 = next_block_header(validator_regtest, h2, pow_regtest);
    validator_regtest.store_mut().add(h1);
    validator_regtest.store_mut().add(h2);
    validator_regtest.store_mut().add(h3);
    // In regtest, this will use the previous difficulty target that is not equal to the
    // maximum difficulty target (`pow_regtest`), meaning that of `genesis_header`.
    // See [`crate::header::find_next_difficulty_in_chain`]
    let result = validator_regtest.validate_header(&h3, MOCK_CURRENT_TIME);
    assert!(matches!(
        result,
        Err(ValidateHeaderError::InvalidPoWForComputedTarget)
    ));
}

fn verify_with_excessive_target<T: HeaderValidator>(validator_mainnet: &T, header: &mut Header) {
    header.bits = CompactTarget::from_hex("0x207fffff").unwrap(); // Target exceeds what is allowed on mainnet
    let result = validator_mainnet.validate_header(&header, MOCK_CURRENT_TIME);
    assert!(matches!(
        result,
        Err(ValidateHeaderError::TargetDifficultyAboveMax)
    ));
}

fn verify_difficulty_adjustment<T: HeaderValidator>(
    validator: &mut T,
    headers_path: &str,
    up_to_height: usize,
) {
    use bitcoin::consensus::Decodable;
    use std::io::BufRead;
    let file = std::fs::File::open(test_data_file(headers_path)).unwrap();

    let rdr = std::io::BufReader::new(file);

    println!("Loading headers...");
    let mut headers = vec![];
    for line in rdr.lines() {
        let header = line.unwrap();
        // If this line fails make sure you install git-lfs.
        let decoded = hex::decode(header.trim()).unwrap();
        let header = Header::consensus_decode(&mut &decoded[..]).unwrap();
        headers.push(header);
    }

    println!("Creating header store...");
    for header in headers[1..].iter() {
        validator.store_mut().add(*header);
    }

    println!("Verifying next targets...");
    proptest!(|(i in 0..=up_to_height)| {
        // Compute what the target of the next header should be.
        let expected_next_target =
            validator.get_next_target(&headers[i], i as u32, headers[i + 1].time);

        // Assert that the expected next target matches the next header's target.
        assert_eq!(
            expected_next_target,
            Target::from_compact(headers[i + 1].bits)
        );
    });
}

// This checks the chain of headers of different lengths
// with non-limit PoW in the first block header and PoW limit
// in all the other headers.
// Expect difficulty to be equal to the non-limit PoW.
fn verify_regtest_difficulty_calculation<T: HeaderValidator>(
    validator: &mut T,
    expected_pow: CompactTarget,
) {
    // From 10 we reach Digishield activation in Dogecoin and hence it would fail.
    let chain_length = 9;
    // Arrange.
    let last_header = utils::build_header_chain(validator, chain_length);
    assert_eq!(validator.store().height() + 1, chain_length);
    // Act.
    let target = validator.get_next_target(
        &last_header,
        validator.store().height(),
        last_header.time + validator.pow_target_spacing().as_secs() as u32,
    );
    // Assert.
    assert_eq!(target, Target::from_compact(expected_pow));
}

fn verify_backdated_block_difficulty<T: HeaderValidator>(
    validator: &mut T,
    difficulty_adjustment_interval: u32,
    expected_target: CompactTarget,
) {
    let chain_length = difficulty_adjustment_interval - 1; // To trigger the difficulty adjustment.

    let current_height = validator.store().height();
    let mut last_header = validator.store().get_with_height(current_height).unwrap();
    for _ in 1..chain_length {
        let new_header = Header {
            prev_blockhash: last_header.block_hash(),
            time: last_header.time - 1, // Each new block is 1 second earlier
            ..last_header
        };
        validator.store_mut().add(new_header);
        last_header = new_header;
    }

    // Act.
    let difficulty = validator.compute_next_difficulty(&last_header, chain_length);

    // Assert.
    assert_eq!(difficulty, expected_target);
}

fn verify_timestamp_rules<T: HeaderValidator>(validator: &T, height_start_header: u32) {
    let mut header = Header {
        version: Version::from_consensus(0x20800004),
        prev_blockhash: validator
            .store()
            .get_with_height(height_start_header + 2)
            .unwrap()
            .block_hash(),
        merkle_root: TxMerkleNode::from_str(
            "c120ff2ae1363593a0b92e0d281ec341a0cc989b4ee836dc3405c9f4215242a6",
        )
        .unwrap(),
        time: validator
            .store()
            .get_with_height(height_start_header + 1)
            .unwrap()
            .time
            + 1, // Larger than median time past
        bits: CompactTarget::from_consensus(0x170e0408),
        nonce: 0xb48e8b0a,
    };
    assert!(is_timestamp_valid(validator.store(), &header, MOCK_CURRENT_TIME).is_ok());

    // Mon Apr 16 2012 15:06:40
    header.time = 1334588800;
    assert!(matches!(
        is_timestamp_valid(validator.store(), &header, MOCK_CURRENT_TIME),
        Err(ValidateHeaderError::HeaderIsOld)
    ));

    let result = validator.validate_header(&header, MOCK_CURRENT_TIME);
    assert!(matches!(result, Err(ValidateHeaderError::HeaderIsOld)));

    header.time = (MOCK_CURRENT_TIME - ONE_HOUR).as_secs() as u32;

    assert!(is_timestamp_valid(validator.store(), &header, MOCK_CURRENT_TIME).is_ok());

    header.time = (MOCK_CURRENT_TIME + 2 * ONE_HOUR + Duration::from_secs(10)).as_secs() as u32;
    assert_eq!(
        is_timestamp_valid(validator.store(), &header, MOCK_CURRENT_TIME),
        Err(ValidateHeaderError::HeaderIsTooFarInFuture {
            block_time: header.time as u64,
            max_allowed_time: (MOCK_CURRENT_TIME + 2 * ONE_HOUR).as_secs()
        })
    );

    let result = validator.validate_header(&header, MOCK_CURRENT_TIME);
    assert_eq!(
        result,
        Err(ValidateHeaderError::HeaderIsTooFarInFuture {
            block_time: header.time as u64,
            max_allowed_time: (MOCK_CURRENT_TIME + 2 * ONE_HOUR).as_secs(),
        })
    );
}

#[test]
fn test_timestamp_is_at_most_2h_in_future() {
    // Time is represented as the number of seconds after 01.01.1970 00:00.
    // Hence, if block time is 10 seconds after that time,
    // 'test_timestamp_is_at_most_2h_in_future' should return true.

    assert!(timestamp_is_at_most_2h_in_future(Duration::from_secs(10), MOCK_CURRENT_TIME).is_ok());

    assert!(
        timestamp_is_at_most_2h_in_future(MOCK_CURRENT_TIME - ONE_HOUR, MOCK_CURRENT_TIME).is_ok()
    );

    assert!(timestamp_is_at_most_2h_in_future(MOCK_CURRENT_TIME, MOCK_CURRENT_TIME).is_ok());

    assert!(
        timestamp_is_at_most_2h_in_future(MOCK_CURRENT_TIME + ONE_HOUR, MOCK_CURRENT_TIME).is_ok()
    );

    assert!(timestamp_is_at_most_2h_in_future(
        MOCK_CURRENT_TIME + 2 * ONE_HOUR - Duration::from_secs(5),
        MOCK_CURRENT_TIME
    )
    .is_ok());

    // 'test_timestamp_is_at_most_2h_in_future' should return false
    // because the time is more than 2 hours from the current time.
    assert_eq!(
        timestamp_is_at_most_2h_in_future(
            MOCK_CURRENT_TIME + 2 * ONE_HOUR + Duration::from_secs(10),
            MOCK_CURRENT_TIME
        ),
        Err(ValidateHeaderError::HeaderIsTooFarInFuture {
            block_time: (MOCK_CURRENT_TIME + 2 * ONE_HOUR).as_secs() + 10,
            max_allowed_time: (MOCK_CURRENT_TIME + 2 * ONE_HOUR).as_secs()
        })
    );
}
