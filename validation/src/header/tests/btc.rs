use crate::constants::btc::DIFFICULTY_ADJUSTMENT_INTERVAL_BITCOIN;
use crate::constants::btc::test::{
    MAINNET_HEADER_586656, MAINNET_HEADER_705600, MAINNET_HEADER_705601, MAINNET_HEADER_705602,
    TESTNET_HEADER_2132555, TESTNET_HEADER_2132556,
};
use crate::header::tests::utils::bitcoin_genesis_header;
use crate::header::tests::utils::deserialize_header;
use crate::header::tests::{
    verify_backdated_block_difficulty, verify_consecutive_headers, verify_difficulty_adjustment,
    verify_header_sequence, verify_regtest_difficulty_calculation, verify_timestamp_rules,
    verify_with_excessive_target, verify_with_invalid_pow, verify_with_missing_parent,
    verify_with_wrong_computed_target,
};
use crate::header::HeaderValidator;
use crate::BitcoinHeaderValidator;
use bitcoin::network::Network as BitcoinNetwork;
use bitcoin::CompactTarget;

#[test]
fn test_basic_header_validation() {
    verify_consecutive_headers(
        BitcoinHeaderValidator::mainnet(),
        MAINNET_HEADER_705600,
        705_600,
        MAINNET_HEADER_705601,
    );

    verify_consecutive_headers(
        BitcoinHeaderValidator::testnet(),
        TESTNET_HEADER_2132555,
        2_132_555,
        TESTNET_HEADER_2132556,
    );
}

#[test]
fn test_sequential_header_validation() {
    verify_header_sequence(
        BitcoinHeaderValidator::mainnet(),
        "headers.csv",
        deserialize_header(MAINNET_HEADER_586656),
        586_656,
    );
}

#[test]
fn test_missing_previous_header() {
    verify_with_missing_parent(
        BitcoinHeaderValidator::mainnet(),
        MAINNET_HEADER_705600,
        705_600,
        MAINNET_HEADER_705602,
    );
}

#[test]
fn test_invalid_pow() {
    verify_with_invalid_pow(
        BitcoinHeaderValidator::mainnet(),
        MAINNET_HEADER_705600,
        705_600,
        MAINNET_HEADER_705601,
    );
}

#[test]
fn test_invalid_computed_target() {
    let bitcoin_genesis_header = bitcoin_genesis_header(
        BitcoinNetwork::Bitcoin,
        BitcoinHeaderValidator::mainnet().pow_limit_bits(),
    );
    verify_with_wrong_computed_target(BitcoinHeaderValidator::regtest(), bitcoin_genesis_header);
}

#[test]
fn test_target_exceeds_maximum() {
    verify_with_excessive_target(
        BitcoinHeaderValidator::mainnet(),
        BitcoinHeaderValidator::regtest(),
        MAINNET_HEADER_705600,
        705_600,
        MAINNET_HEADER_705601,
    );
}

#[test]
fn test_difficulty_adjustments() {
    verify_difficulty_adjustment(
        BitcoinHeaderValidator::mainnet(),
        "tests/data/block_headers_mainnet.csv",
        700_000,
    );

    verify_difficulty_adjustment(
        BitcoinHeaderValidator::testnet(),
        "tests/data/block_headers_testnet.csv",
        2_400_000,
    );
}

#[test]
fn test_regtest_difficulty() {
    let initial_pow = CompactTarget::from_consensus(7); // Some non-limit PoW, the actual value is not important.

    let genesis_header = bitcoin_genesis_header(BitcoinNetwork::Regtest, initial_pow);
    verify_regtest_difficulty_calculation(
        BitcoinHeaderValidator::regtest(),
        genesis_header,
        initial_pow,
    );
}

#[test]
fn test_backdated_difficulty_adjustment() {
    let genesis_target = 486604799;
    let genesis_difficulty = CompactTarget::from_consensus(genesis_target);
    let genesis_header = bitcoin_genesis_header(BitcoinNetwork::Testnet, genesis_difficulty);
    verify_backdated_block_difficulty(
        BitcoinHeaderValidator::testnet(),
        DIFFICULTY_ADJUSTMENT_INTERVAL_BITCOIN,
        genesis_header,
        473956288,
    );
}

#[test]
fn test_timestamp_validation() {
    verify_timestamp_rules(
        BitcoinHeaderValidator::mainnet(),
        MAINNET_HEADER_705600,
        705_600,
        MAINNET_HEADER_705601,
        MAINNET_HEADER_705602,
    );
}
