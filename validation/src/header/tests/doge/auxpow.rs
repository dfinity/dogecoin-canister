use crate::header::tests::utils::{build_header_chain, SimpleHeaderStore};
use crate::header::{AuxPowHeaderValidator, ValidateAuxPowHeaderError};
use crate::{DogecoinHeaderValidator, HeaderValidator, ValidateHeaderError};
use bitcoin::block::{Header as PureHeader, Header, Version};
use bitcoin::dogecoin::auxpow::VERSION_AUXPOW;
use bitcoin::dogecoin::constants::genesis_block;
use bitcoin::dogecoin::{auxpow::AuxPow, auxpow::MERGED_MINING_HEADER, Header as DogecoinHeader};
use bitcoin::hashes::Hash;
use bitcoin::{
    BlockHash, CompactTarget, ScriptBuf, Target, Transaction, TxIn, TxMerkleNode, TxOut, Witness,
};
use std::time::{SystemTime, UNIX_EPOCH};

const EASY_TARGET_BITS: u32 = 0x207fffff; // Very easy target
const PARENT_CHAIN_ID: i32 = 42; // Random chain ID
const AUXPOW_CHAIN_ID: i32 = 98; // Dogecoin chain ID
const BASE_VERSION: i32 = 5;
const MERKLE_HEIGHT: usize = 3; // Merkle tree height
const MERKLE_NONCE: u32 = 7; // Nonce used to calculate block header indexes into blockchain merkle tree

/// Helper to build a valid AuxPow struct for a given `aux_block_hash` and `chain_id`
fn build_valid_auxpow(aux_block_hash: BlockHash, chain_id: i32) -> AuxPow {
    let expected_index = AuxPow::get_expected_index(MERKLE_NONCE, chain_id, MERKLE_HEIGHT);

    let blockchain_branch: Vec<TxMerkleNode> = (0..MERKLE_HEIGHT)
        .map(|i| TxMerkleNode::from_byte_array([i as u8; 32]))
        .collect();

    let blockchain_merkle_root =
        AuxPow::compute_merkle_root(aux_block_hash, &blockchain_branch, expected_index);
    let mut blockchain_merkle_root_le = blockchain_merkle_root.to_byte_array();
    blockchain_merkle_root_le.reverse();

    let mut script_data = Vec::new();
    script_data.extend_from_slice(&MERGED_MINING_HEADER);
    script_data.extend_from_slice(&blockchain_merkle_root_le);
    script_data.extend_from_slice(&(1u32 << MERKLE_HEIGHT).to_le_bytes());
    script_data.extend_from_slice(&MERKLE_NONCE.to_le_bytes());

    let coinbase_tx = Transaction {
        version: bitcoin::transaction::Version::ONE,
        lock_time: bitcoin::absolute::LockTime::ZERO,
        input: vec![TxIn {
            previous_output: bitcoin::OutPoint::null(),
            script_sig: ScriptBuf::from_bytes(script_data),
            sequence: bitcoin::Sequence::MAX,
            witness: Witness::default(),
        }],
        output: vec![TxOut {
            value: bitcoin::Amount::from_sat(5000000000),
            script_pubkey: ScriptBuf::new(),
        }],
    };

    let parent_block_header = PureHeader {
        version: Version::from_consensus(BASE_VERSION | (PARENT_CHAIN_ID << 16)),
        prev_blockhash: BlockHash::all_zeros(),
        merkle_root: TxMerkleNode::from_byte_array(coinbase_tx.compute_txid().to_byte_array()),
        time: 1000000,
        bits: CompactTarget::from_consensus(EASY_TARGET_BITS),
        nonce: 0,
    };

    AuxPow {
        coinbase_tx,
        parent_hash: BlockHash::all_zeros(),
        coinbase_branch: vec![], // Empty since coinbase is the only tx
        coinbase_index: 0,
        blockchain_branch,
        blockchain_index: expected_index,
        parent_block_header,
    }
}

/// Helper to mine a block that either matches or doesn't match the difficulty target specified in the header.
fn mine_header_to_target(header: &mut PureHeader, should_pass: bool) {
    let target = Target::from_compact(header.bits);
    header.nonce = 0;

    loop {
        let hash = header.block_hash_with_scrypt();
        let hash_target = Target::from_le_bytes(hash.to_byte_array());
        let passes_pow = hash_target <= target;

        if (should_pass && passes_pow) || (!should_pass && !passes_pow) {
            break;
        }

        header.nonce += 1;
        if header.nonce == 0 {
            // Overflow, adjust time and continue
            header.time += 1;
        }
    }
}

/// Helper to create a header chain before AuxPow activation in regtest
fn create_header_store_before_auxpow_activation_regtest() -> (SimpleHeaderStore, Header) {
    let validator = DogecoinHeaderValidator::regtest();
    let genesis_header = genesis_block(&validator.network()).header;
    // Build chain up to height 15 - next block will be at height 16 (before AuxPow activation at height 20)
    let (store, last_header) = build_header_chain(&validator, *genesis_header, 16);
    (store, last_header)
}

/// Helper to create a header chain that extends beyond AuxPow activation in regtest
fn create_header_store_after_auxpow_activation_regtest() -> (SimpleHeaderStore, Header) {
    let validator = DogecoinHeaderValidator::regtest();
    let genesis_header = genesis_block(&validator.network()).header;
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

    let mut dogecoin_header = DogecoinHeader::new_from_pure_header(header);

    if with_aux_pow {
        dogecoin_header.aux_pow = Some(build_valid_auxpow(header.block_hash(), AUXPOW_CHAIN_ID));

        // Mine the parent block to pass PoW
        let auxpow = dogecoin_header.aux_pow.as_mut().unwrap();
        mine_header_to_target(&mut auxpow.parent_block_header, should_mine_parent_header);
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
