#[cfg(feature = "btc")]
mod btc;
#[cfg(feature = "doge")]
mod doge;
mod utils;

use crate::constants::test::TEN_MINUTES;
use crate::header::timestamp_is_less_than_2h_in_future;
use crate::header::{is_timestamp_valid, HeaderValidator, ONE_HOUR};
use crate::ValidateHeaderError;
use crate::{BlockHeight, HeaderStore};
use bitcoin::block::{Header, Version};
use bitcoin::{CompactTarget, Target, TxMerkleNode};
use proptest::proptest;
use std::path::PathBuf;
use std::str::FromStr;
use utils::{
    deserialize_header, get_headers, next_block_header, SimpleHeaderStore, MOCK_CURRENT_TIME,
};

fn verify_consecutive_headers<T: HeaderValidator>(
    validator: T,
    header_1: &str,
    height_1: BlockHeight,
    header_2: &str,
) {
    let header_1 = deserialize_header(header_1);
    let header_2 = deserialize_header(header_2);
    let store = SimpleHeaderStore::new(header_1, height_1);
    let result = validator.validate_header(&store, &header_2, MOCK_CURRENT_TIME);
    assert!(result.is_ok());
}

fn verify_header_sequence<T: HeaderValidator>(
    validator: T,
    file: &str,
    header: Header,
    height: BlockHeight,
) {
    let mut store = SimpleHeaderStore::new(header, height);
    let headers = get_headers(file);
    for (i, header) in headers.iter().enumerate() {
        let result = validator.validate_header(&store, header, MOCK_CURRENT_TIME);
        assert!(
            result.is_ok(),
            "Failed to validate header on line {}: {:?}",
            i,
            result
        );
        store.add(*header);
    }
}

fn verify_with_missing_parent<T: HeaderValidator>(
    validator: T,
    header_1: &str,
    height_1: BlockHeight,
    header_2: &str,
) {
    let header_1 = deserialize_header(header_1);
    let header_2 = deserialize_header(header_2);
    let store = SimpleHeaderStore::new(header_1, height_1);
    let result = validator.validate_header(&store, &header_2, MOCK_CURRENT_TIME);
    assert!(matches!(
        result,
        Err(ValidateHeaderError::PrevHeaderNotFound)
    ));
}

fn verify_with_invalid_pow<T: HeaderValidator>(
    validator: T,
    header_1: &str,
    height_1: BlockHeight,
    header_2: &str,
) {
    let header_1 = deserialize_header(header_1);
    let mut header_2 = deserialize_header(header_2);
    header_2.bits = validator.pow_limit_bits(); // Modify header to invalidate PoW
    let store = SimpleHeaderStore::new(header_1, height_1);
    let result = validator.validate_header(&store, &header_2, MOCK_CURRENT_TIME);
    assert!(matches!(
        result,
        Err(ValidateHeaderError::InvalidPoWForHeaderTarget)
    ));
}

fn verify_with_invalid_pow_with_computed_target<T: HeaderValidator>(
    validator_regtest: T,
    genesis_header: Header,
) {
    let pow_regtest = validator_regtest.pow_limit_bits();
    let h0 = genesis_header;
    let h1 = next_block_header(&validator_regtest, h0, pow_regtest);
    let h2 = next_block_header(&validator_regtest, h1, pow_regtest);
    let h3 = next_block_header(&validator_regtest, h2, pow_regtest);
    let mut store = SimpleHeaderStore::new(h0, 0);
    store.add(h1);
    store.add(h2);
    // In regtest, this will use the previous difficulty target that is not equal to the
    // maximum difficulty target (`pow_regtest`), meaning that of `genesis_header`.
    // See [`crate::header::find_next_difficulty_in_chain`]
    let result = validator_regtest.validate_header(&store, &h3, MOCK_CURRENT_TIME);
    assert!(matches!(
        result,
        Err(ValidateHeaderError::InvalidPoWForComputedTarget)
    ));
}

fn verify_with_excessive_target<T: HeaderValidator>(
    validator_mainnet: T,
    validator_regtest: T,
    header_1: &str,
    height_1: BlockHeight,
    header_2: &str,
) {
    let header_1 = deserialize_header(header_1);
    let mut header_2 = deserialize_header(header_2);
    header_2.bits = validator_regtest.pow_limit_bits(); // Target exceeds what is allowed on mainnet
    let store = SimpleHeaderStore::new(header_1, height_1);
    let result = validator_mainnet.validate_header(&store, &header_2, MOCK_CURRENT_TIME);
    assert!(matches!(
        result,
        Err(ValidateHeaderError::TargetDifficultyAboveMax)
    ));
}

fn verify_difficulty_adjustment<T: HeaderValidator>(
    validator: T,
    headers_path: &str,
    up_to_height: usize,
) {
    use bitcoin::consensus::Decodable;
    use std::io::BufRead;
    let file = std::fs::File::open(
        PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap()).join(headers_path),
    )
    .unwrap();

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
    let mut store = SimpleHeaderStore::new(headers[0], 0);
    for header in headers[1..].iter() {
        store.add(*header);
    }

    println!("Verifying next targets...");
    proptest!(|(i in 0..up_to_height)| {
        // Compute what the target of the next header should be.
        let expected_next_target =
            validator.get_next_target(&store, &headers[i], i as u32, headers[i + 1].time);

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
    validator: T,
    genesis_header: Header,
    expected_pow: CompactTarget,
) {
    // Arrange.
    for chain_length in 1..10 {
        let (store, last_header) =
            utils::build_header_chain(&validator, genesis_header, chain_length);
        assert_eq!(store.height() + 1, chain_length);
        // Act.
        let target = validator.get_next_target(
            &store,
            &last_header,
            chain_length - 1,
            last_header.time + validator.pow_target_spacing(),
        );
        // Assert.
        assert_eq!(target, Target::from_compact(expected_pow));
    }
}

fn verify_backdated_block_difficulty<T: HeaderValidator>(
    validator: T,
    difficulty_adjustment_interval: u32,
    genesis_header: Header,
    expected_target: CompactTarget,
) {
    let chain_length = difficulty_adjustment_interval - 1; // To trigger the difficulty adjustment.

    // Initialize the header store
    let mut store = SimpleHeaderStore::new(genesis_header, 0);
    let mut last_header = genesis_header;
    for _ in 1..chain_length {
        let new_header = Header {
            prev_blockhash: last_header.block_hash(),
            time: last_header.time - 1, // Each new block is 1 second earlier
            ..last_header
        };
        store.add(new_header);
        last_header = new_header;
    }

    // Act.
    let difficulty = validator.compute_next_difficulty(&store, &last_header, chain_length);

    // Assert.
    assert_eq!(difficulty, expected_target);
}

fn verify_timestamp_rules<T: HeaderValidator>(
    validator: T,
    header_1: &str,
    height_1: u32,
    header_2: &str,
    header_3: &str,
) {
    let header_1 = deserialize_header(header_1);
    let header_2 = deserialize_header(header_2);
    let header_3 = deserialize_header(header_3);
    let mut store = SimpleHeaderStore::new(header_1, height_1);
    store.add(header_2);
    store.add(header_3);

    let mut header = Header {
        version: Version::from_consensus(0x20800004),
        prev_blockhash: header_3.block_hash(),
        merkle_root: TxMerkleNode::from_str(
            "c120ff2ae1363593a0b92e0d281ec341a0cc989b4ee836dc3405c9f4215242a6",
        )
        .unwrap(),
        time: header_3.time + TEN_MINUTES,
        bits: CompactTarget::from_consensus(0x170e0408),
        nonce: 0xb48e8b0a,
    };
    assert!(is_timestamp_valid(&store, &header, MOCK_CURRENT_TIME).is_ok());

    // Mon Apr 16 2012 15:06:40
    header.time = 1334588800;
    assert!(matches!(
        is_timestamp_valid(&store, &header, MOCK_CURRENT_TIME),
        Err(ValidateHeaderError::HeaderIsOld)
    ));

    let result = validator.validate_header(&store, &header, MOCK_CURRENT_TIME);
    assert!(matches!(result, Err(ValidateHeaderError::HeaderIsOld)));

    header.time = (MOCK_CURRENT_TIME - ONE_HOUR) as u32;

    assert!(is_timestamp_valid(&store, &header, MOCK_CURRENT_TIME).is_ok());

    header.time = (MOCK_CURRENT_TIME + 2 * ONE_HOUR + 10) as u32;
    assert_eq!(
        is_timestamp_valid(&store, &header, MOCK_CURRENT_TIME),
        Err(ValidateHeaderError::HeaderIsTooFarInFuture {
            block_time: header.time as u64,
            max_allowed_time: MOCK_CURRENT_TIME + 2 * ONE_HOUR
        })
    );

    let result = validator.validate_header(&store, &header, MOCK_CURRENT_TIME);
    assert_eq!(
        result,
        Err(ValidateHeaderError::HeaderIsTooFarInFuture {
            block_time: header.time as u64,
            max_allowed_time: MOCK_CURRENT_TIME + 2 * ONE_HOUR,
        })
    );
}

#[test]
fn test_timestamp_is_less_than_2h_in_future() {
    // Time is represented as the number of seconds after 01.01.1970 00:00.
    // Hence, if block time is 10 seconds after that time,
    // 'timestamp_is_less_than_2h_in_future' should return true.

    assert!(timestamp_is_less_than_2h_in_future(10, MOCK_CURRENT_TIME).is_ok());

    assert!(
        timestamp_is_less_than_2h_in_future(MOCK_CURRENT_TIME - ONE_HOUR, MOCK_CURRENT_TIME)
            .is_ok()
    );

    assert!(timestamp_is_less_than_2h_in_future(MOCK_CURRENT_TIME, MOCK_CURRENT_TIME).is_ok());

    assert!(
        timestamp_is_less_than_2h_in_future(MOCK_CURRENT_TIME + ONE_HOUR, MOCK_CURRENT_TIME)
            .is_ok()
    );

    assert!(timestamp_is_less_than_2h_in_future(
        MOCK_CURRENT_TIME + 2 * ONE_HOUR - 5,
        MOCK_CURRENT_TIME
    )
    .is_ok());

    // 'timestamp_is_less_than_2h_in_future' should return false
    // because the time is more than 2 hours from the current time.
    assert_eq!(
        timestamp_is_less_than_2h_in_future(
            MOCK_CURRENT_TIME + 2 * ONE_HOUR + 10,
            MOCK_CURRENT_TIME
        ),
        Err(ValidateHeaderError::HeaderIsTooFarInFuture {
            block_time: MOCK_CURRENT_TIME + 2 * ONE_HOUR + 10,
            max_allowed_time: MOCK_CURRENT_TIME + 2 * ONE_HOUR
        })
    );
}
