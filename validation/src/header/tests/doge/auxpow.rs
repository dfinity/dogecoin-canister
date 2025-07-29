use crate::header::tests::utils::{build_header_chain, dogecoin_genesis_header, SimpleHeaderStore};
use crate::header::{AuxPowHeaderValidator, ValidateAuxPowHeaderError, ValidateHeaderError};
use crate::DogecoinHeaderValidator;
use bitcoin::block::{Header as PureHeader, Header, Version};
use bitcoin::dogecoin::VERSION_AUXPOW;
use bitcoin::dogecoin::{AuxPow, Header as DogecoinHeader, Network as DogecoinNetwork};
use bitcoin::hashes::Hash;
use bitcoin::{
    BlockHash, CompactTarget, ScriptBuf, Target, Transaction, TxIn, TxMerkleNode, TxOut, Witness,
};
use std::time::{SystemTime, UNIX_EPOCH};

const EASY_TARGET_BITS: u32 = 0x207fffff; // Very easy target for regtest
const PARENT_CHAIN_ID: i32 = 42;
const AUXPOW_CHAIN_ID: i32 = 0x0062; // Dogecoin chain ID
const BASE_VERSION: i32 = 5;
const MERKLE_HEIGHT: usize = 3;
const NONCE: u32 = 7;

fn create_test_auxpow(aux_block_hash: BlockHash, chain_id: i32) -> AuxPow {
    let expected_index = AuxPow::get_expected_index(NONCE, chain_id, MERKLE_HEIGHT);

    // Create blockchain branch
    let blockchain_branch: Vec<TxMerkleNode> = (0..MERKLE_HEIGHT)
        .map(|i| TxMerkleNode::from_byte_array([i as u8; 32]))
        .collect();

    let blockchain_merkle_root =
        AuxPow::compute_merkle_root(aux_block_hash, &blockchain_branch, expected_index);

    // Create coinbase transaction with proper script
    let mut script_data = Vec::new();
    script_data.extend_from_slice(&[0xfa, 0xbe, 0x6d, 0x6d]); // Merged mining header
    script_data.extend_from_slice(&blockchain_merkle_root.to_byte_array());
    script_data.extend_from_slice(&(1u32 << MERKLE_HEIGHT).to_le_bytes()); // Size
    script_data.extend_from_slice(&NONCE.to_le_bytes()); // Nonce

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

    // Create parent block header
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

fn create_header_store_before_auxpow() -> (SimpleHeaderStore, Header) {
    let validator = DogecoinHeaderValidator::regtest();
    let genesis_header = dogecoin_genesis_header(
        DogecoinNetwork::Regtest,
        CompactTarget::from_consensus(EASY_TARGET_BITS),
    ); // TODO: replace with true genesis header
       // Build chain up to height 15 - next block will be at height 16 (before AuxPoW activation at height 20)
    let (store, last_header) = build_header_chain(&validator, genesis_header, 16);
    (store, last_header)
}

fn create_header_store_after_auxpow() -> (SimpleHeaderStore, Header) {
    let validator = DogecoinHeaderValidator::regtest();
    let genesis_header = dogecoin_genesis_header(
        DogecoinNetwork::Regtest,
        CompactTarget::from_consensus(EASY_TARGET_BITS),
    ); // TODO: replace with true genesis header
       // Build chain up to height 25 - next block will be at height 26 (after AuxPoW activation at height 20)
    let (store, last_header) = build_header_chain(&validator, genesis_header, 26);
    (store, last_header)
}

fn helper_auxpow_tests(
    before_auxpow: bool,
    base_version: i32,
    chain_id: i32,
    auxpow_bit_set: bool,
    should_mine_header: bool,
    with_aux_pow: bool,
    should_mine_parent_header: bool,
    time: u32,
) -> (bitcoin::dogecoin::Header, SimpleHeaderStore) {
    let (store, prev_header) = if before_auxpow {
        create_header_store_before_auxpow()
    } else {
        create_header_store_after_auxpow()
    };
    let mut version = base_version | (chain_id << 16);
    if auxpow_bit_set {
        version |= VERSION_AUXPOW;
    }

    let mut header = PureHeader {
        version: Version::from_consensus(version),
        prev_blockhash: prev_header.block_hash(),
        merkle_root: TxMerkleNode::all_zeros(),
        time,
        bits: CompactTarget::from_consensus(EASY_TARGET_BITS),
        nonce: 0,
    };
    mine_header_to_target(&mut header, should_mine_header);

    let mut dogecoin_header = DogecoinHeader::new_from_pure_header(header);

    if with_aux_pow {
        dogecoin_header.aux_pow = Some(create_test_auxpow(header.block_hash(), AUXPOW_CHAIN_ID));

        // Mine the parent block to pass PoW
        let auxpow = dogecoin_header.aux_pow.as_mut().unwrap();
        mine_header_to_target(&mut auxpow.parent_block_header, should_mine_parent_header);
    }

    (dogecoin_header, store)
}

#[test]
fn test_auxpow_pow_block_version_checks() {
    let validator = DogecoinHeaderValidator::regtest();
    let current_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // Test version 1 (legacy) - should pass (before AuxPoW activation)
    let (dogecoin_header, store_legacy) =
        helper_auxpow_tests(true, 1, 0, false, true, false, false, current_time as u32);
    assert!(validator
        .validate_auxpow_header(&store_legacy, &dogecoin_header, current_time)
        .is_ok());

    // Test version 3 (no AuxPoW flag) - should fail for AuxPoW validation (before AuxPoW activation)
    // This should fail because version 3 is not legacy but doesn't have AuxPoW flag
    let (dogecoin_header, store_legacy) =
        helper_auxpow_tests(true, 3, 0, false, true, false, false, current_time as u32);
    assert_eq!(
        validator.validate_auxpow_header(&store_legacy, &dogecoin_header, current_time),
        Err(ValidateAuxPowHeaderError::InvalidChainId)
    );

    // Test version 2 with correct chain ID - should pass (after AuxPoW activation)
    let (dogecoin_header, store_auxpow) = helper_auxpow_tests(
        false,
        2,
        AUXPOW_CHAIN_ID,
        false,
        true,
        false,
        false,
        current_time as u32,
    );
    assert!(validator
        .validate_auxpow_header(&store_auxpow, &dogecoin_header, current_time)
        .is_ok());

    // Test AuxPoW version with correct chain ID - should pass (after AuxPoW activation)
    let (dogecoin_header, store_auxpow) = helper_auxpow_tests(
        false,
        BASE_VERSION,
        AUXPOW_CHAIN_ID,
        true,
        false,
        true,
        true,
        current_time as u32,
    );
    assert!(validator
        .validate_auxpow_header(&store_auxpow, &dogecoin_header, current_time)
        .is_ok());

    // Test AuxPoW version with wrong chain ID - should fail (after AuxPoW activation)
    let (dogecoin_header, store_auxpow) = helper_auxpow_tests(
        false,
        BASE_VERSION,
        AUXPOW_CHAIN_ID + 1,
        false,
        true,
        false,
        false,
        current_time as u32,
    );
    assert_eq!(
        validator.validate_auxpow_header(&store_auxpow, &dogecoin_header, current_time),
        Err(ValidateAuxPowHeaderError::InvalidChainId)
    );
}

#[test]
fn test_auxpow_pow_no_auxpow_case() {
    let validator = DogecoinHeaderValidator::regtest();
    let current_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // Test block with AuxPoW flag but no AuxPoW data - should fail
    let (dogecoin_header, store) = helper_auxpow_tests(
        true,
        BASE_VERSION,
        AUXPOW_CHAIN_ID,
        true,
        true,
        false,
        false,
        current_time as u32,
    );
    assert_eq!(
        validator.validate_auxpow_header(&store, &dogecoin_header, current_time),
        Err(ValidateAuxPowHeaderError::InconsistentAuxPowBitSet)
    );

    // Test block without AuxPoW flag and no AuxPoW data - should pass
    let (dogecoin_header, store) = helper_auxpow_tests(
        true,
        BASE_VERSION,
        AUXPOW_CHAIN_ID,
        false,
        true,
        false,
        false,
        current_time as u32,
    );
    assert!(validator
        .validate_auxpow_header(&store, &dogecoin_header, current_time)
        .is_ok());

    // Test block without AuxPoW flag and no AuxPoW data but bad PoW - should fail
    let (dogecoin_header, store) = helper_auxpow_tests(
        true,
        BASE_VERSION,
        AUXPOW_CHAIN_ID,
        false,
        false,
        false,
        false,
        current_time as u32,
    );
    assert_eq!(
        validator.validate_auxpow_header(&store, &dogecoin_header, current_time),
        Err(ValidateHeaderError::InvalidPoWForComputedTarget.into())
    );
}

#[test]
fn test_auxpow_pow_with_auxpow_case() {
    let validator = DogecoinHeaderValidator::regtest();
    let current_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // Test with parent block having invalid PoW - should fail
    let (dogecoin_header, store) = helper_auxpow_tests(
        false,
        BASE_VERSION,
        AUXPOW_CHAIN_ID,
        true,
        false,
        true,
        false,
        current_time as u32,
    );
    assert_eq!(
        validator.validate_auxpow_header(&store, &dogecoin_header, current_time),
        Err(ValidateAuxPowHeaderError::InvalidParentPoW)
    );

    // Test with parent block having valid PoW - should pass
    let (dogecoin_header, store) = helper_auxpow_tests(
        false,
        BASE_VERSION,
        AUXPOW_CHAIN_ID,
        true,
        false,
        true,
        true,
        current_time as u32,
    );
    assert!(validator
        .validate_auxpow_header(&store, &dogecoin_header, current_time)
        .is_ok());
}

#[test]
fn test_auxpow_pow_version_mismatch() {
    let validator = DogecoinHeaderValidator::regtest();
    let current_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // Test block without AuxPoW flag but with AuxPoW data - should fail
    let (dogecoin_header, store) = helper_auxpow_tests(
        false,
        BASE_VERSION,
        AUXPOW_CHAIN_ID,
        false,
        false,
        true,
        true,
        current_time as u32,
    );
    assert_eq!(
        validator.validate_auxpow_header(&store, &dogecoin_header, current_time),
        Err(ValidateAuxPowHeaderError::InconsistentAuxPowBitSet)
    );
}

#[test]
fn test_auxpow_pow_block_modification_invalidates() {
    let validator = DogecoinHeaderValidator::regtest();
    let current_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let (mut dogecoin_header, store) = helper_auxpow_tests(
        false,
        BASE_VERSION,
        AUXPOW_CHAIN_ID,
        true,
        false,
        true,
        true,
        current_time as u32,
    );

    // Should pass initially
    assert!(validator
        .validate_auxpow_header(&store, &dogecoin_header, current_time)
        .is_ok());

    // Modify the merkle root to invalidate the blockchain merkle root in the AuxPow coinbase tx
    dogecoin_header.pure_header.merkle_root = TxMerkleNode::from_byte_array([0xff; 32]);

    // Should fail after modification because AuxPoW references old block hash
    assert_eq!(
        validator.validate_auxpow_header(&store, &dogecoin_header, current_time),
        Err(ValidateAuxPowHeaderError::InvalidAuxPoW)
    );
}
