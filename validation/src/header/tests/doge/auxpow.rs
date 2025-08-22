//! Tests for AuxPoW behavior, adapted from the original Dogecoin C++ test suite.
//!
//! These tests are based on:
//! <https://github.com/dogecoin/dogecoin/blob/51cbc1fd5d0d045dda2ad84f53572bbf524c6a8e/src/test/auxpow_tests.cpp#L358>
//!
//! The goal is to verify correctness of merged mining validation logic, particularly around
//! version number and chain ID rules, and child/parent PoW validation.
use crate::header::tests::utils::{build_header_chain, SimpleHeaderStore};
use crate::header::{AuxPowHeaderValidator, ValidateAuxPowHeaderError};
use crate::{DogecoinHeaderValidator, HeaderValidator, ValidateHeaderError};
use bitcoin::block::{Header as PureHeader, Header, Version};
use bitcoin::dogecoin::auxpow::VERSION_AUXPOW;
use bitcoin::dogecoin::constants::genesis_block;
use bitcoin::dogecoin::Header as DogecoinHeader;
use bitcoin::hashes::Hash;
use bitcoin::{CompactTarget, TxMerkleNode};
use ic_doge_test_utils::{mine_header_to_target, AuxPowBuilder};
use std::time::{SystemTime, UNIX_EPOCH};

const EASY_TARGET_BITS: u32 = 0x207fffff; // Very easy target
const AUXPOW_CHAIN_ID: i32 = 98; // Dogecoin chain ID
const BASE_VERSION: i32 = 5;

/// Helper to create a header chain before AuxPow activation in regtest
fn create_header_store_before_auxpow_activation_regtest() -> (SimpleHeaderStore, Header) {
    let validator = DogecoinHeaderValidator::regtest();
    let genesis_header = genesis_block(validator.network()).header;
    // Build chain up to height 15 - next block will be at height 16 (before AuxPow activation at height 20)
    let (store, last_header) = build_header_chain(&validator, *genesis_header, 16);
    (store, last_header)
}

/// Helper to create a header chain that extends beyond AuxPow activation in regtest
fn create_header_store_after_auxpow_activation_regtest() -> (SimpleHeaderStore, Header) {
    let validator = DogecoinHeaderValidator::regtest();
    let genesis_header = genesis_block(validator.network()).header;
    // Build chain up to height 25 - next block will be at height 26 (after AuxPow activation at height 20)
    let (store, last_header) = build_header_chain(&validator, *genesis_header, 26);
    (store, last_header)
}

/// Helper to create a customizable Dogecoin header
fn create_dogecoin_header(
    prev_header: Header,
    base_version: i32,
    chain_id: i32,
    auxpow_bit_set: bool,
    should_mine_header: bool,
    with_aux_pow: bool,
    should_mine_parent_header: bool,
) -> DogecoinHeader {
    let mut version = base_version | (chain_id << 16);
    if auxpow_bit_set {
        version |= VERSION_AUXPOW;
    }

    let mut header = PureHeader {
        version: Version::from_consensus(version),
        prev_blockhash: prev_header.block_hash(),
        merkle_root: TxMerkleNode::all_zeros(),
        time: 1754405226, // August 2025
        bits: CompactTarget::from_consensus(EASY_TARGET_BITS),
        nonce: 0,
    };
    mine_header_to_target(&mut header, should_mine_header);

    let mut dogecoin_header: DogecoinHeader = header.into();

    if with_aux_pow {
        let mut auxpow = AuxPowBuilder::new(header.block_hash()).build();
        mine_header_to_target(&mut auxpow.parent_block_header, should_mine_parent_header);
        dogecoin_header.aux_pow = Some(auxpow);
    }

    dogecoin_header
}

#[test]
fn test_auxpow_version() {
    let validator = DogecoinHeaderValidator::regtest();
    let current_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let (store_legacy, prev_header_legacy) = create_header_store_before_auxpow_activation_regtest();
    let (store_auxpow, prev_header_auxpow) = create_header_store_after_auxpow_activation_regtest();

    // Version 1 (legacy) - should pass (before AuxPow activation)
    let dogecoin_header =
        create_dogecoin_header(prev_header_legacy, 1, 0, false, true, false, false);
    assert!(validator
        .validate_auxpow_header(&store_legacy, &dogecoin_header, current_time)
        .is_ok());

    // Version 3 (with no chain ID) - should fail (before AuxPow activation)
    // This should fail because version 3 is not legacy but doesn't have chain ID
    let dogecoin_header =
        create_dogecoin_header(prev_header_legacy, 3, 0, false, true, false, false);
    assert_eq!(
        validator.validate_auxpow_header(&store_legacy, &dogecoin_header, current_time),
        Err(ValidateAuxPowHeaderError::InvalidChainId)
    );

    // Version 2 (with no chain ID) - should pass (before AuxPow activation)
    let dogecoin_header =
        create_dogecoin_header(prev_header_legacy, 2, 0, false, true, false, false);
    assert!(validator
        .validate_auxpow_header(&store_legacy, &dogecoin_header, current_time)
        .is_ok());

    // Version 2 (with correct chain ID) - should pass (after AuxPow activation)
    let dogecoin_header = create_dogecoin_header(
        prev_header_auxpow,
        2,
        AUXPOW_CHAIN_ID,
        false,
        true,
        false,
        false,
    );
    assert!(validator
        .validate_auxpow_header(&store_auxpow, &dogecoin_header, current_time)
        .is_ok());

    // AuxPow bit set (with correct chain ID) - should fail (before AuxPow activation)
    let dogecoin_header = create_dogecoin_header(
        prev_header_legacy,
        BASE_VERSION,
        AUXPOW_CHAIN_ID,
        true,
        false,
        true,
        true,
    );
    assert_eq!(
        validator.validate_auxpow_header(&store_legacy, &dogecoin_header, current_time),
        Err(ValidateHeaderError::AuxPowBlockNotAllowed.into())
    );

    // AuxPow bit set (with correct chain ID) - should pass (after AuxPow activation)
    let dogecoin_header = create_dogecoin_header(
        prev_header_auxpow,
        BASE_VERSION,
        AUXPOW_CHAIN_ID,
        true,
        false,
        true,
        true,
    );
    assert!(validator
        .validate_auxpow_header(&store_auxpow, &dogecoin_header, current_time)
        .is_ok());

    // AuxPow bit set (with wrong chain ID) - should fail (after AuxPow activation)
    let dogecoin_header = create_dogecoin_header(
        prev_header_auxpow,
        BASE_VERSION,
        AUXPOW_CHAIN_ID + 1,
        false,
        true,
        false,
        false,
    );
    assert_eq!(
        validator.validate_auxpow_header(&store_auxpow, &dogecoin_header, current_time),
        Err(ValidateAuxPowHeaderError::InvalidChainId)
    );
}

#[test]
fn test_without_auxpow_data() {
    let validator = DogecoinHeaderValidator::regtest();
    let current_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let (legacy_store, prev_header_legacy) = create_header_store_before_auxpow_activation_regtest();

    // AuxPow flag unset and no AuxPow data with correct PoW - should pass
    let dogecoin_header = create_dogecoin_header(
        prev_header_legacy,
        BASE_VERSION,
        AUXPOW_CHAIN_ID,
        false,
        true,
        false,
        false,
    );
    assert!(validator
        .validate_auxpow_header(&legacy_store, &dogecoin_header, current_time)
        .is_ok());

    // AuxPow flag unset and no AuxPow data but bad PoW - should fail
    let dogecoin_header = create_dogecoin_header(
        prev_header_legacy,
        BASE_VERSION,
        AUXPOW_CHAIN_ID,
        false,
        false,
        false,
        false,
    );
    assert_eq!(
        validator.validate_auxpow_header(&legacy_store, &dogecoin_header, current_time),
        Err(ValidateHeaderError::InvalidPoWForComputedTarget.into())
    );

    // AuxPow flag set but no AuxPow data - should fail
    let dogecoin_header = create_dogecoin_header(
        prev_header_legacy,
        BASE_VERSION,
        AUXPOW_CHAIN_ID,
        true,
        true,
        false,
        false,
    );
    assert_eq!(
        validator.validate_auxpow_header(&legacy_store, &dogecoin_header, current_time),
        Err(ValidateAuxPowHeaderError::InconsistentAuxPowBitSet)
    );
}

#[test]
fn test_with_auxpow_data() {
    let validator = DogecoinHeaderValidator::regtest();
    let current_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let (store_auxpow, prev_header_auxpow) = create_header_store_after_auxpow_activation_regtest();

    // Parent block with valid PoW - should pass
    let dogecoin_header = create_dogecoin_header(
        prev_header_auxpow,
        BASE_VERSION,
        AUXPOW_CHAIN_ID,
        true,
        false,
        true,
        true,
    );
    assert!(validator
        .validate_auxpow_header(&store_auxpow, &dogecoin_header, current_time)
        .is_ok());

    // Parent block with invalid PoW - should fail
    let dogecoin_header = create_dogecoin_header(
        prev_header_auxpow,
        BASE_VERSION,
        AUXPOW_CHAIN_ID,
        true,
        false,
        true,
        false,
    );
    assert_eq!(
        validator.validate_auxpow_header(&store_auxpow, &dogecoin_header, current_time),
        Err(ValidateAuxPowHeaderError::InvalidParentPoW)
    );

    // Parent block with invalid PoW and block with valid PoW - should fail
    let dogecoin_header = create_dogecoin_header(
        prev_header_auxpow,
        BASE_VERSION,
        AUXPOW_CHAIN_ID,
        true,
        true,
        true,
        false,
    );
    assert_eq!(
        validator.validate_auxpow_header(&store_auxpow, &dogecoin_header, current_time),
        Err(ValidateAuxPowHeaderError::InvalidParentPoW)
    );

    // AuxPow flag unset but with AuxPow data - should fail
    let dogecoin_header = create_dogecoin_header(
        prev_header_auxpow,
        BASE_VERSION,
        AUXPOW_CHAIN_ID,
        false,
        false,
        true,
        true,
    );
    assert_eq!(
        validator.validate_auxpow_header(&store_auxpow, &dogecoin_header, current_time),
        Err(ValidateAuxPowHeaderError::InconsistentAuxPowBitSet)
    );
}

#[test]
fn test_header_modification_invalidates_auxpow_proof() {
    let validator = DogecoinHeaderValidator::regtest();
    let current_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let (store_auxpow, prev_header_auxpow) = create_header_store_after_auxpow_activation_regtest();

    let mut dogecoin_header = create_dogecoin_header(
        prev_header_auxpow,
        BASE_VERSION,
        AUXPOW_CHAIN_ID,
        true,
        false,
        true,
        true,
    );

    // Should pass initially
    assert!(validator
        .validate_auxpow_header(&store_auxpow, &dogecoin_header, current_time)
        .is_ok());

    // Modify the block header
    dogecoin_header.pure_header.time += 1;

    // Should fail after modification because AuxPow references old block header
    assert_eq!(
        validator.validate_auxpow_header(&store_auxpow, &dogecoin_header, current_time),
        Err(ValidateAuxPowHeaderError::InvalidAuxPoW)
    );
}
