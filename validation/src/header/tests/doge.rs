use crate::constants::doge::test::{
    MAINNET_HEADER_DOGE_151556, MAINNET_HEADER_DOGE_151557, MAINNET_HEADER_DOGE_151558,
    MAINNET_HEADER_DOGE_17, MAINNET_HEADER_DOGE_18, TESTNET_HEADER_DOGE_88, TESTNET_HEADER_DOGE_89,
};
use crate::constants::doge::DIFFICULTY_ADJUSTMENT_INTERVAL_DOGECOIN;
use crate::header::tests::utils::{dogecoin_genesis_header, doge_files};
use crate::header::tests::{
    verify_backdated_block_difficulty, verify_consecutive_headers, verify_difficulty_adjustment,
    verify_header_sequence, verify_regtest_difficulty_calculation, verify_timestamp_rules,
    verify_with_excessive_target, verify_with_invalid_pow,
    verify_with_invalid_pow_with_computed_target, verify_with_missing_parent,
};
use crate::DogecoinHeaderValidator;
use bitcoin::dogecoin::constants::genesis_block as dogecoin_genesis_block;
use bitcoin::dogecoin::Network as DogecoinNetwork;
use bitcoin::{CompactTarget, Target};

#[test]
fn test_basic_header_validation_mainnet() {
    verify_consecutive_headers(
        DogecoinHeaderValidator::mainnet(),
        MAINNET_HEADER_DOGE_17,
        17,
        MAINNET_HEADER_DOGE_18,
    );
}

#[test]
fn test_basic_header_validation_testnet() {
    verify_consecutive_headers(
        DogecoinHeaderValidator::testnet(),
        TESTNET_HEADER_DOGE_88,
        88,
        TESTNET_HEADER_DOGE_89,
    );
}

#[test]
fn test_sequential_header_validation_mainnet() {
    verify_header_sequence(
        DogecoinHeaderValidator::mainnet(),
        doge_files::MAINNET_HEADERS_1_5000_PARSED,
        dogecoin_genesis_block(DogecoinNetwork::Dogecoin).header,
        0,
    );
}

#[test]
fn test_sequential_header_validation_testnet() {
    verify_header_sequence(
        DogecoinHeaderValidator::testnet(),
        doge_files::TESTNET_HEADERS_1_5000_PARSED,
        dogecoin_genesis_block(DogecoinNetwork::Testnet).header,
        0,
    );
}

#[test]
fn test_missing_previous_header() {
    verify_with_missing_parent(
        DogecoinHeaderValidator::mainnet(),
        MAINNET_HEADER_DOGE_151556,
        151_556,
        MAINNET_HEADER_DOGE_151558,
    );
}

#[test]
fn test_invalid_pow_mainnet() {
    verify_with_invalid_pow(
        DogecoinHeaderValidator::mainnet(),
        MAINNET_HEADER_DOGE_151556,
        151_556,
        MAINNET_HEADER_DOGE_151557,
    );
}

#[test]
fn test_invalid_pow_with_computed_target_regtest() {
    let dogecoin_genesis_header = dogecoin_genesis_header(
        DogecoinNetwork::Dogecoin,
        CompactTarget::from_consensus(0x000ffff0), // Put a low target
    );
    verify_with_invalid_pow_with_computed_target(
        DogecoinHeaderValidator::regtest(),
        dogecoin_genesis_header,
    );
}

#[test]
fn test_target_exceeds_maximum_mainnet() {
    verify_with_excessive_target(
        DogecoinHeaderValidator::mainnet(),
        DogecoinHeaderValidator::regtest(),
        MAINNET_HEADER_DOGE_151556,
        151_556,
        MAINNET_HEADER_DOGE_151557,
    );
}

#[test]
fn test_difficulty_adjustments_mainnet() {
    verify_difficulty_adjustment(
        DogecoinHeaderValidator::mainnet(),
        doge_files::MAINNET_HEADERS_0_5000_RAW,
        5_000,
    );
}

#[test]
fn test_difficulty_adjustments_testnet() {
    verify_difficulty_adjustment(
        DogecoinHeaderValidator::testnet(),
        doge_files::TESTNET_HEADERS_0_5000_RAW,
        5_000,
    );
}

#[test]
fn test_difficulty_regtest() {
    let initial_pow = CompactTarget::from_consensus(0x1d0000ff); // Some non-limit PoW, the actual value is not important.
    let genesis_header = dogecoin_genesis_header(DogecoinNetwork::Regtest, initial_pow);
    verify_regtest_difficulty_calculation(
        DogecoinHeaderValidator::regtest(),
        genesis_header,
        initial_pow,
    );
}

#[test]
fn test_backdated_difficulty_adjustment_testnet() {
    let genesis_target = CompactTarget::from_consensus(0x1e0ffff0);
    let genesis_header = dogecoin_genesis_header(DogecoinNetwork::Testnet, genesis_target);
    let expected_target = Target::from(genesis_target)
        .min_transition_threshold_dogecoin(0)
        .to_compact_lossy(); // Target is expected to reach the minimum valid Target threshold allowed in a difficulty adjustment.
    verify_backdated_block_difficulty(
        DogecoinHeaderValidator::testnet(),
        DIFFICULTY_ADJUSTMENT_INTERVAL_DOGECOIN,
        genesis_header,
        expected_target,
    );
}

#[test]
fn test_timestamp_validation_mainnet() {
    verify_timestamp_rules(
        DogecoinHeaderValidator::mainnet(),
        MAINNET_HEADER_DOGE_151556,
        151_556,
        MAINNET_HEADER_DOGE_151557,
        MAINNET_HEADER_DOGE_151558,
    );
}
