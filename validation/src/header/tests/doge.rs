mod auxpow;

use crate::constants::doge::test::{
    MAINNET_HEADER_DOGE_151556, MAINNET_HEADER_DOGE_151557, MAINNET_HEADER_DOGE_151558,
    MAINNET_HEADER_DOGE_17, MAINNET_HEADER_DOGE_18, MAINNET_HEADER_DOGE_400000,
    MAINNET_HEADER_DOGE_400001, MAINNET_HEADER_DOGE_400002, MAINNET_HEADER_DOGE_521335,
    MAINNET_HEADER_DOGE_521336, TESTNET_HEADER_DOGE_158378, TESTNET_HEADER_DOGE_158379,
    TESTNET_HEADER_DOGE_158380, TESTNET_HEADER_DOGE_293098, TESTNET_HEADER_DOGE_293099,
    TESTNET_HEADER_DOGE_88, TESTNET_HEADER_DOGE_89,
};
use crate::header::doge::ALLOW_DIGISHIELD_MIN_DIFFICULTY_HEIGHT;
use crate::header::tests::utils::{deserialize_header, doge_files, dogecoin_genesis_header};
use crate::header::tests::{
    verify_backdated_block_difficulty, verify_consecutive_headers,
    verify_consecutive_headers_auxpow, verify_difficulty_adjustment, verify_header_sequence,
    verify_header_sequence_auxpow, verify_regtest_difficulty_calculation, verify_timestamp_rules,
    verify_with_excessive_target, verify_with_invalid_pow,
    verify_with_invalid_pow_with_computed_target, verify_with_missing_parent,
};
use crate::{DogecoinHeaderValidator, HeaderValidator};
use bitcoin::dogecoin::constants::genesis_block as dogecoin_genesis_block;
use bitcoin::dogecoin::Network as DogecoinNetwork;
use bitcoin::{CompactTarget, Target};

#[test]
fn test_basic_header_validation_mainnet() {
    verify_consecutive_headers(
        &DogecoinHeaderValidator::mainnet(),
        MAINNET_HEADER_DOGE_17,
        17,
        MAINNET_HEADER_DOGE_18,
    );
}

#[test]
fn test_basic_header_validation_testnet() {
    verify_consecutive_headers(
        &DogecoinHeaderValidator::testnet(),
        TESTNET_HEADER_DOGE_88,
        88,
        TESTNET_HEADER_DOGE_89,
    );
}

#[test]
fn test_basic_header_validation_auxpow_mainnet() {
    verify_consecutive_headers_auxpow(
        DogecoinHeaderValidator::mainnet(),
        MAINNET_HEADER_DOGE_400000,
        400_000,
        MAINNET_HEADER_DOGE_400001,
        MAINNET_HEADER_DOGE_400002,
    );
}

#[test]
fn test_basic_header_validation_auxpow_testnet() {
    verify_consecutive_headers_auxpow(
        DogecoinHeaderValidator::testnet(),
        TESTNET_HEADER_DOGE_158378,
        158_378,
        TESTNET_HEADER_DOGE_158379,
        TESTNET_HEADER_DOGE_158380,
    );
}

#[test]
fn test_sequential_header_validation_mainnet() {
    verify_header_sequence(
        &DogecoinHeaderValidator::mainnet(),
        doge_files::MAINNET_HEADERS_1_15000_PARSED,
        *dogecoin_genesis_block(DogecoinNetwork::Dogecoin).header,
        0,
    );
}

#[test]
fn test_sequential_header_validation_testnet() {
    verify_header_sequence(
        &DogecoinHeaderValidator::testnet(),
        doge_files::TESTNET_HEADERS_1_15000_PARSED,
        *dogecoin_genesis_block(DogecoinNetwork::Testnet).header,
        0,
    );
}

#[test]
fn test_sequential_header_validation_auxpow_mainnet() {
    verify_header_sequence_auxpow(
        DogecoinHeaderValidator::mainnet(),
        doge_files::MAINNET_HEADERS_521337_536336_PARSED,
        deserialize_header(MAINNET_HEADER_DOGE_521335),
        521335,
        deserialize_header(MAINNET_HEADER_DOGE_521336),
    );
}

#[test]
fn test_sequential_header_validation_auxpow_testnet() {
    verify_header_sequence_auxpow(
        DogecoinHeaderValidator::testnet(),
        doge_files::TESTNET_HEADERS_293100_308099_PARSED,
        deserialize_header(TESTNET_HEADER_DOGE_293098),
        293098,
        deserialize_header(TESTNET_HEADER_DOGE_293099),
    );
}

#[test]
fn test_missing_previous_header() {
    verify_with_missing_parent(
        &DogecoinHeaderValidator::mainnet(),
        MAINNET_HEADER_DOGE_151556,
        151_556,
        MAINNET_HEADER_DOGE_151558,
    );
}

#[test]
fn test_invalid_pow_mainnet() {
    verify_with_invalid_pow(
        &DogecoinHeaderValidator::mainnet(),
        MAINNET_HEADER_DOGE_151556,
        151_556,
        MAINNET_HEADER_DOGE_151557,
    );
}

#[test]
fn test_invalid_pow_with_computed_target_regtest() {
    let dogecoin_genesis_header = dogecoin_genesis_header(
        &DogecoinNetwork::Dogecoin,
        CompactTarget::from_consensus(0x000ffff0), // Put a low target
    );
    verify_with_invalid_pow_with_computed_target(
        &DogecoinHeaderValidator::regtest(),
        dogecoin_genesis_header,
    );
}

#[test]
fn test_target_exceeds_maximum_mainnet() {
    verify_with_excessive_target(
        &DogecoinHeaderValidator::mainnet(),
        &DogecoinHeaderValidator::regtest(),
        MAINNET_HEADER_DOGE_151556,
        151_556,
        MAINNET_HEADER_DOGE_151557,
    );
}

#[test]
fn test_difficulty_adjustments_mainnet() {
    verify_difficulty_adjustment(
        &DogecoinHeaderValidator::mainnet(),
        doge_files::MAINNET_HEADERS_0_700000_RAW,
        700_000,
    );
}

#[test]
fn test_difficulty_adjustments_testnet() {
    verify_difficulty_adjustment(
        &DogecoinHeaderValidator::testnet(),
        doge_files::TESTNET_HEADERS_0_2000000_RAW,
        2_000_000,
    );
}

#[test]
fn test_difficulty_regtest() {
    let initial_pow = CompactTarget::from_consensus(0x1d0000ff); // Some non-limit PoW, the actual value is not important.
    let genesis_header = dogecoin_genesis_header(&DogecoinNetwork::Regtest, initial_pow);
    verify_regtest_difficulty_calculation(
        &DogecoinHeaderValidator::regtest(),
        genesis_header,
        initial_pow,
    );
}

#[test]
fn test_backdated_difficulty_adjustment_testnet() {
    let validator = DogecoinHeaderValidator::testnet();
    let genesis_target = CompactTarget::from_consensus(0x1e0ffff0);
    let genesis_header = dogecoin_genesis_header(validator.network(), genesis_target);
    let expected_target = Target::from(genesis_target)
        .min_transition_threshold_dogecoin(validator.network(), 0)
        .to_compact_lossy(); // Target is expected to reach the minimum valid Target threshold allowed in a difficulty adjustment.
    verify_backdated_block_difficulty(
        &validator,
        validator.difficulty_adjustment_interval(0),
        genesis_header,
        expected_target,
    );
}

#[test]
fn test_timestamp_validation_mainnet() {
    verify_timestamp_rules(
        &DogecoinHeaderValidator::mainnet(),
        MAINNET_HEADER_DOGE_151556,
        151_556,
        MAINNET_HEADER_DOGE_151557,
        MAINNET_HEADER_DOGE_151558,
    );
}

#[test]
fn test_digishield_with_min_difficulty_height() {
    let networks = [DogecoinNetwork::Testnet, DogecoinNetwork::Regtest];
    for network in networks.iter() {
        assert!(network
            .params()
            .is_digishield_activated(ALLOW_DIGISHIELD_MIN_DIFFICULTY_HEIGHT));
    }
}
