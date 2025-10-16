use crate::fixtures::SimpleHeaderStore;
use crate::header::btc::DIFFICULTY_ADJUSTMENT_INTERVAL_BITCOIN;
use crate::header::tests::utils::{bitcoin_genesis_header, btc_files, deserialize_header};
use crate::header::tests::{
    verify_backdated_block_difficulty, verify_consecutive_headers, verify_difficulty_adjustment,
    verify_header_sequence, verify_regtest_difficulty_calculation, verify_timestamp_rules,
    verify_with_excessive_target, verify_with_invalid_pow,
    verify_with_invalid_pow_with_computed_target, verify_with_missing_parent,
};
use crate::header::HeaderValidator;
use crate::BitcoinHeaderValidator;
use bitcoin::constants::genesis_block as bitcoin_genesis_block;
use bitcoin::network::Network as BitcoinNetwork;
use bitcoin::CompactTarget;

/// Mainnet 000000000000000000063108ecc1f03f7fd1481eb20f97307d532a612bc97f04
const MAINNET_HEADER_586656: &str = "00008020cff0e07ab39db0f31d4ded81ba2339173155b9c57839110000000000000000007a2d75dce5981ec421a54df706d3d407f66dc9170f1e0d6e48ed1e8a1cad7724e9ed365d083a1f17bc43b10a";
/// Mainnet 0000000000000000000d37dfef7fe1c7bd22c893dbe4a94272c8cf556e40be99
const MAINNET_HEADER_705600: &str = "0400a0205849eed80b320273a73d39933c0360e127d15036a69d020000000000000000006cc2504814505bb6863d960599c1d1f76a4768090ac15b0ad5172f5a5cd918a155d86d6108040e175daab79e";
/// Mainnet 0000000000000000000567617f2101a979d04cff2572a081aa5f29e30800ab75
const MAINNET_HEADER_705601: &str = "04e0002099be406e55cfc87242a9e4db93c822bdc7e17fefdf370d000000000000000000eba036bca22654014f363f3019d0f08b3cdf6b2747ab57eff2e6dc1da266bc0392d96d6108040e176c6624cd";
/// Mainnet 00000000000000000001eea12c0de75000c2546da22f7bf42d805c1d2769b6ef
const MAINNET_HEADER_705602: &str = "0400202075ab0008e3295faa81a07225ff4cd079a901217f616705000000000000000000c027a2615b11b4c75afc9e42d1db135d7124338c1f556f6a14d1257a3bd103a5f4dd6d6108040e1745d26934";

/// Testnet 00000000000000e23bb091a0046e6c73160db0a71aa052c20b10ff7de7554f97
const TESTNET_HEADER_2132555: &str = "004000200e1ff99438666c67c649def743fb82117537c2017bcc6ad617000000000000007fa40cf82bf224909e3174281a57af2eb3a4a2a961d33f50ec0772c1221c9e61ddfdc061ffff001a64526636";
/// Testnet 00000000383cd7fff4692410ccd9bd6201790043bb41b93bacb21e9b85620767
const TESTNET_HEADER_2132556: &str = "00000020974f55e77dff100bc252a01aa7b00d16736c6e04a091b03be200000000000000c44f2d69fc200c4a2211885000b6b67512f42c1bec550f3754e103b6c4046e05a202c161ffff001d09ec1bc4";

#[test]
fn test_basic_header_validation_mainnet() {
    let start_header = deserialize_header(MAINNET_HEADER_705600);
    let store = SimpleHeaderStore::new(start_header, 705_600);
    verify_consecutive_headers(
        &BitcoinHeaderValidator::mainnet(store),
        deserialize_header(MAINNET_HEADER_705601),
    );
}

#[test]
fn test_basic_header_validation_testnet() {
    let start_header = deserialize_header(TESTNET_HEADER_2132555);
    let store = SimpleHeaderStore::new(start_header, 2_132_555);
    verify_consecutive_headers(
        &BitcoinHeaderValidator::testnet(store),
        deserialize_header(TESTNET_HEADER_2132556),
    );
}

#[test]
fn test_sequential_header_validation_mainnet() {
    let start_header = deserialize_header(MAINNET_HEADER_586656);
    let store = SimpleHeaderStore::new(start_header, 586_656);
    verify_header_sequence(
        BitcoinHeaderValidator::mainnet(store),
        btc_files::MAINNET_HEADERS_586657_589289_PARSED,
    );
}

#[test]
fn test_sequential_header_validation_testnet() {
    let start_header = bitcoin_genesis_block(BitcoinNetwork::Testnet).header;
    let store = SimpleHeaderStore::new(start_header, 0);
    verify_header_sequence(
        BitcoinHeaderValidator::testnet(store),
        btc_files::TESTNET_HEADERS_1_5000_PARSED,
    );
}

#[test]
fn test_missing_previous_header() {
    let start_header = deserialize_header(MAINNET_HEADER_705600);
    let store = SimpleHeaderStore::new(start_header, 705_600);
    verify_with_missing_parent(
        &BitcoinHeaderValidator::mainnet(store),
        deserialize_header(MAINNET_HEADER_705602),
    );
}

#[test]
fn test_invalid_pow_mainnet() {
    let start_header = deserialize_header(MAINNET_HEADER_705600);
    let store = SimpleHeaderStore::new(start_header, 705_600);
    verify_with_invalid_pow(
        &BitcoinHeaderValidator::mainnet(store),
        deserialize_header(MAINNET_HEADER_705601),
    );
}

#[test]
fn test_invalid_pow_with_computed_target_regtest() {
    let bitcoin_genesis_header = bitcoin_genesis_header(
        BitcoinNetwork::Regtest,
        CompactTarget::from_consensus(0x1701ddb2), // Put a low target
    );
    let store = SimpleHeaderStore::new(bitcoin_genesis_header, 0);
    let mut header_validator = BitcoinHeaderValidator::regtest(store);
    verify_with_invalid_pow_with_computed_target(&mut header_validator, bitcoin_genesis_header);
}

#[test]
fn test_target_exceeds_maximum_mainnet() {
    let start_header = deserialize_header(MAINNET_HEADER_705600);
    let store = SimpleHeaderStore::new(start_header, 705_600);
    let mut header = deserialize_header(MAINNET_HEADER_705601);
    header.bits = CompactTarget::from_hex("0x207fffff").unwrap(); // Target exceeds what is allowed on mainnet
    verify_with_excessive_target(
        &BitcoinHeaderValidator::mainnet(store),
        &header,
    );
}

#[test]
fn test_difficulty_adjustments_mainnet() {
    let start_header = bitcoin_genesis_block(BitcoinNetwork::Bitcoin).header;
    let store = SimpleHeaderStore::new(start_header, 0);
    verify_difficulty_adjustment(
        &mut BitcoinHeaderValidator::mainnet(store),
        btc_files::MAINNET_HEADERS_0_782282_RAW,
        782_281,
    );
}

#[test]
fn test_difficulty_adjustments_testnet() {
    let start_header = bitcoin_genesis_block(BitcoinNetwork::Testnet).header;
    let store = SimpleHeaderStore::new(start_header, 0);
    verify_difficulty_adjustment(
        &mut BitcoinHeaderValidator::testnet(store),
        btc_files::TESTNET_HEADERS_0_2425489_RAW,
        2_425_488,
    );
}

#[test]
fn test_difficulty_regtest() {
    let initial_pow = CompactTarget::from_consensus(7); // Some non-limit PoW, the actual value is not important.
    let start_header = bitcoin_genesis_header(BitcoinNetwork::Regtest, initial_pow);
    let store = SimpleHeaderStore::new(start_header, 0);
    verify_regtest_difficulty_calculation(&mut BitcoinHeaderValidator::regtest(store), initial_pow);
}

#[test]
fn test_backdated_difficulty_adjustment_testnet() {
    let genesis_target = CompactTarget::from_consensus(486604799);
    let start_header = bitcoin_genesis_header(BitcoinNetwork::Testnet, genesis_target);
    let store = SimpleHeaderStore::new(start_header, 0);
    verify_backdated_block_difficulty(
        &mut BitcoinHeaderValidator::testnet(store),
        DIFFICULTY_ADJUSTMENT_INTERVAL_BITCOIN,
        CompactTarget::from_consensus(473956288),
    );
}

#[test]
fn test_timestamp_validation_mainnet() {
    let start_header = deserialize_header(MAINNET_HEADER_705600);
    let start_header_height = 705_600;
    let mut store = SimpleHeaderStore::new(start_header, start_header_height);
    store.add(deserialize_header(MAINNET_HEADER_705601));
    store.add(deserialize_header(MAINNET_HEADER_705602));
    verify_timestamp_rules(&BitcoinHeaderValidator::mainnet(store), start_header_height);
}
