use crate::constants::doge::test::{
    MAINNET_HEADER_DOGE_151556, MAINNET_HEADER_DOGE_151557, MAINNET_HEADER_DOGE_151558,
    MAINNET_HEADER_DOGE_17, MAINNET_HEADER_DOGE_18,
};
use crate::header::tests::{
    verify_consecutive_headers, verify_difficulty_adjustment, verify_header_sequence,
    verify_timestamp_rules, verify_with_excessive_target, verify_with_invalid_pow,
    verify_with_missing_parent,
};
use crate::DogecoinHeaderValidator;
use bitcoin::dogecoin::constants::genesis_block as dogecoin_genesis_block;
use bitcoin::dogecoin::Network as DogecoinNetwork;

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
fn test_sequential_header_validation_mainnet() {
    verify_header_sequence(
        DogecoinHeaderValidator::mainnet(),
        "headers_doge_1_5000.csv",
        dogecoin_genesis_block(DogecoinNetwork::Dogecoin).header,
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
        "tests/data/block_headers_mainnet_doge.csv",
        5_000,
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
