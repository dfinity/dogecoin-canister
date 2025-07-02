use crate::constants::btc::test::{
    MAINNET_HEADER_586656, MAINNET_HEADER_705600, MAINNET_HEADER_705601, MAINNET_HEADER_705602,
    TESTNET_HEADER_2132555, TESTNET_HEADER_2132556,
};
use crate::constants::btc::DIFFICULTY_ADJUSTMENT_INTERVAL_BITCOIN;
use crate::header::tests::utils::{bitcoin_genesis_header, btc_files, deserialize_header};
use crate::header::tests::{
    verify_backdated_block_difficulty, verify_consecutive_headers, verify_difficulty_adjustment,
    verify_header_sequence, verify_regtest_difficulty_calculation, verify_timestamp_rules,
    verify_with_excessive_target, verify_with_invalid_pow,
    verify_with_invalid_pow_with_computed_target, verify_with_missing_parent,
};
use crate::header::HeaderValidator;
use crate::BitcoinHeaderValidator;
use bitcoin::network::Network as BitcoinNetwork;
use bitcoin::CompactTarget;

#[test]
fn test_basic_header_validation_mainnet() {
    verify_consecutive_headers(
        BitcoinHeaderValidator::mainnet(),
        MAINNET_HEADER_705600,
        705_600,
        MAINNET_HEADER_705601,
    );
}

#[test]
fn test_basic_header_validation_testnet() {
    verify_consecutive_headers(
        BitcoinHeaderValidator::testnet(),
        TESTNET_HEADER_2132555,
        2_132_555,
        TESTNET_HEADER_2132556,
    );
}

#[test]
fn test_sequential_header_validation_mainnet() {
    verify_header_sequence(
        BitcoinHeaderValidator::mainnet(),
        btc_files::MAINNET_HEADERS_586657_589289_PARSED,
        deserialize_header(MAINNET_HEADER_586656),
        586_656,
    );
} // TODO XC-408: add test for testnet

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
fn test_invalid_pow_mainnet() {
    verify_with_invalid_pow(
        BitcoinHeaderValidator::mainnet(),
        MAINNET_HEADER_705600,
        705_600,
        MAINNET_HEADER_705601,
    );
}

#[test]
fn test_invalid_pow_with_computed_target_regtest() {
    let bitcoin_genesis_header = bitcoin_genesis_header(
        BitcoinNetwork::Bitcoin,
        BitcoinHeaderValidator::mainnet().pow_limit_bits(),
    );
    verify_with_invalid_pow_with_computed_target(
        BitcoinHeaderValidator::regtest(),
        bitcoin_genesis_header,
    );
}

#[test]
fn test_target_exceeds_maximum_mainnet() {
    verify_with_excessive_target(
        BitcoinHeaderValidator::mainnet(),
        BitcoinHeaderValidator::regtest(),
        MAINNET_HEADER_705600,
        705_600,
        MAINNET_HEADER_705601,
    );
}

#[test]
fn test_difficulty_adjustments_mainnet() {
    verify_difficulty_adjustment(
        BitcoinHeaderValidator::mainnet(),
        btc_files::MAINNET_HEADERS_0_782282_RAW,
        700_000,
    );
}

#[test]
fn test_difficulty_adjustments_testnet() {
    verify_difficulty_adjustment(
        BitcoinHeaderValidator::testnet(),
        btc_files::TESTNET_HEADERS_0_2425489_RAW,
        2_400_000,
    );
}

#[test]
fn test_difficulty_regtest() {
    let initial_pow = CompactTarget::from_consensus(7); // Some non-limit PoW, the actual value is not important.
    let genesis_header = bitcoin_genesis_header(BitcoinNetwork::Regtest, initial_pow);
    verify_regtest_difficulty_calculation(
        BitcoinHeaderValidator::regtest(),
        genesis_header,
        initial_pow,
    );
}

#[test]
fn test_backdated_difficulty_adjustment_testnet() {
    let genesis_target = CompactTarget::from_consensus(486604799);
    let genesis_header = bitcoin_genesis_header(BitcoinNetwork::Testnet, genesis_target);
    verify_backdated_block_difficulty(
        BitcoinHeaderValidator::testnet(),
        DIFFICULTY_ADJUSTMENT_INTERVAL_BITCOIN,
        genesis_header,
        CompactTarget::from_consensus(473956288),
    );
}

#[test]
fn test_timestamp_validation_mainnet() {
    verify_timestamp_rules(
        BitcoinHeaderValidator::mainnet(),
        MAINNET_HEADER_705600,
        705_600,
        MAINNET_HEADER_705601,
        MAINNET_HEADER_705602,
    );
}
