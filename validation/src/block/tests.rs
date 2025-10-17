use crate::fixtures::{SimpleHeaderStore, MOCK_CURRENT_TIME};
use crate::ValidateBlockError::InvalidBlockHeader;
use crate::{BlockValidator, ValidateBlockError, ValidateHeaderError};
use bitcoin::consensus::deserialize;
use bitcoin::dogecoin::{Block, Network};
use bitcoin::hashes::Hash;
use bitcoin::BlockHash;
use hex_lit::hex;

// Tests taken from
// https://github.com/rust-bitcoin/rust-bitcoin/blob/674ac57bce47e343d8f7c82e451aed5568766ba0/bitcoin/src/blockdata/block.rs#L537
mod bitcoin_tests {
    use crate::block::validate_block;
    use crate::ValidateBlockError;
    use bitcoin::consensus::deserialize;
    use bitcoin::dogecoin::{Block, Header};
    use bitcoin::hashes::Hash;
    use bitcoin::{
        absolute, dogecoin::Network, transaction, Amount, OutPoint, ScriptBuf, Sequence,
        Transaction, TxIn, TxOut, Txid, Witness,
    };
    use hex_lit::hex;

    #[test]
    fn block_validation_no_transactions() {
        let header = header();
        let transactions = Vec::new(); // Empty transactions

        let block = Block {
            header,
            txdata: transactions,
        };
        match validate_block(&block) {
            Err(ValidateBlockError::NoTransactions) => (),
            other => panic!("Expected NoTransactions error, got: {:?}", other),
        }
    }

    #[test]
    fn block_validation_invalid_coinbase() {
        let header = header();

        // Create a non-coinbase transaction (has a real previous output, not all zeros)
        let non_coinbase_tx = Transaction {
            version: transaction::Version::TWO,
            lock_time: absolute::LockTime::ZERO,
            input: vec![TxIn {
                previous_output: OutPoint {
                    txid: Txid::from_byte_array([1; 32]), // Not all zeros
                    vout: 0,
                },
                script_sig: ScriptBuf::new(),
                sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
                witness: Witness::new(),
            }],
            output: vec![TxOut {
                value: Amount::ONE_BTC,
                script_pubkey: ScriptBuf::new(),
            }],
        };

        let transactions = vec![non_coinbase_tx];
        let block = Block {
            header,
            txdata: transactions,
        };

        match validate_block(&block) {
            Err(ValidateBlockError::InvalidCoinbase) => (),
            other => panic!("Expected InvalidCoinbase error, got: {:?}", other),
        }
    }

    #[test]
    fn block_validation_success_with_coinbase() {
        // Use the genesis block which has a valid coinbase
        let genesis = bitcoin::dogecoin::constants::genesis_block(Network::Dogecoin);

        assert_eq!(
            validate_block(&genesis),
            Ok(()),
            "Genesis block should validate successfully"
        );
    }

    fn header() -> Header {
        let header = hex!("010000004ddccd549d28f385ab457e98d1b11ce80bfea2c5ab93015ade4973e400000000bf4473e53794beae34e64fccc471dace6ae544180816f89591894e0f417a914cd74d6e49ffff001d323b3a7b");
        deserialize(&header).expect("can't deserialize correct block header")
    }
}

#[test]
fn should_validate_header_before_block() {
    const BLOCK_1_HEX: &str = "010000009156352c1818b32e90c9e792efd6a11a82fe7956a630f03bbee236cedae3911a1c525f1049e519256961f407e96e22aef391581de98686524ef500769f777e5fafeda352f0ff0f1e001083540101000000010000000000000000000000000000000000000000000000000000000000000000ffffffff0e04afeda3520102062f503253482fffffffff01004023ef3806000023210338bf57d51a50184cf5ef0dc42ecd519fb19e24574c057620262cc1df94da2ae5ac00000000";
    let network = Network::Dogecoin;
    let validator = BlockValidator::new(
        SimpleHeaderStore::new_with_genesis_dogecoin(network),
        network,
    );
    let mut invalid_block: Block = deserialize(&hex!(BLOCK_1_HEX)).unwrap();

    // Change coinbase to be invalid by having 2 inputs (instead of 1)
    let coinbase = invalid_block.txdata.get_mut(0).unwrap();
    coinbase.input.push(coinbase.input.first().unwrap().clone());
    assert_eq!(
        validator.validate_block(&invalid_block, MOCK_CURRENT_TIME),
        Err(ValidateBlockError::InvalidCoinbase)
    );

    // Invalidate header
    invalid_block.header.prev_blockhash = BlockHash::all_zeros();
    assert_eq!(
        validator.validate_block(&invalid_block, MOCK_CURRENT_TIME),
        Err(InvalidBlockHeader(ValidateHeaderError::PrevHeaderNotFound))
    );
}

#[test]
fn should_validate_first_block_after_genesis() {
    // https://blockchair.com/dogecoin/block/1
    const BLOCK_1_HEX: &str = "010000009156352c1818b32e90c9e792efd6a11a82fe7956a630f03bbee236cedae3911a1c525f1049e519256961f407e96e22aef391581de98686524ef500769f777e5fafeda352f0ff0f1e001083540101000000010000000000000000000000000000000000000000000000000000000000000000ffffffff0e04afeda3520102062f503253482fffffffff01004023ef3806000023210338bf57d51a50184cf5ef0dc42ecd519fb19e24574c057620262cc1df94da2ae5ac00000000";
    let network = Network::Dogecoin;
    let validator = BlockValidator::new(
        SimpleHeaderStore::new_with_genesis_dogecoin(network),
        network,
    );
    let first_block: Block = deserialize(&hex!(BLOCK_1_HEX)).unwrap();
    assert_eq!(
        first_block.block_hash().to_string(),
        "82bc68038f6034c0596b6e313729793a887fded6e92a31fbdf70863f89d9bea2"
    );

    assert_eq!(
        validator.validate_block(&first_block, MOCK_CURRENT_TIME),
        Ok(())
    );
}

#[test]
fn should_prevent_merkle_tree_collision() {
    // https://blockchair.com/dogecoin/block/3106
    const BLOCK_HEADER_3106_HEX: &str = "01000000f714dd9528ed84c5656cb60df6ad3954f24d1e0e954d5e7545c6928f65a922efd3b39a8fce07e12ade8cb3c0d188adb191b425d37f6d4eea7324084fbda243b3ca96a552c326751c00a6c804";
    const BLOCK_3107_HEX: &str = "01000000fb51a6890eb828ae209deba28afa64f1c4c633dcb6703479c0ca5f0f71add40f610ded5e1d531728a527d8f80168eaeefbb6ebd2e3e519ad2a3e1ab82cfbbf830197a552c326751c0069c98d0501000000010000000000000000000000000000000000000000000000000000000000000000ffffffff2602230c062f503253482f040297a55208f800069e020000000d2f7374726174756d506f6f6c2f00000000010052443ba80d00001976a914bbca5040c535345d2cfae8c83150f2a796a13d9088ac0000000001000000016d3a2bdb21acdff4d6fcd0eab3c3519725fac53d0af97c4317eb5d25ebbee584000000006a47304402207b1c5842b2e1af8f5b056fa6304fa83a97f3b27b5df7b5c4dbc7e623507c157002202a07eb3e9b5dfdef2e46f144f12085d011bfbb2a95f5f86e571b237335a3fa1a012102000269a155ebfe927a64fa635d59a0f0cb4fe5aea2a95c67a897cd8d7ebb7527ffffffff02a9fb83c20d0600001976a914e4117b97336a1e544a61ef25283a5ce7227d886088acb7732ee90d0000001976a9146969d1c0c93b047f6e40c950633dd080d29611ab88ac000000000100000001d6aa7674f086849bd1a4737898caf050cc21aac97f267f9e267a4548109d337a000000006a47304402202714ad1f8762a88663c2e1eea18f478e32550abca4909345e992491406b7f19402203a1720e7877c51c58460c84e17369191886eebd5ef973b6fc5718f862761d5220121023284ec8b37e1425980b5e2f01a223cad23d2855dfc4bef9dc23f23b748154d21ffffffff028a7811612b0000001976a9146a747bcc921c0681f7e458251d837d0b29c2437a88acef446b1da40100001976a914cf96c353cdcb3f5ea660e696ca0e7d6ee92a8c5688ac00000000010000000117682ff514de76d488898a06a75650106c9dc522dcf439f845deb6c48b4eaa0f000000006c493046022100db3d635c41d18f3b4ab9259d12528973d65dc7c6bd917e102f18828174d6ea39022100aed803576c3d83f8f6e243588cdb672bc37aea32b9b350478fa94f68f6b34556012103c7ea61975984e7eb05f5f59dfcf00eab32b4c15ca6fe90d6042308bf1cb0ab5effffffff0213fe52717d0100001976a914d413b348f9e0929886992d692653f330727d73e588aceaa0c39e280000001976a9142564011006d37edab0b3c3596b0b28d9b2341dfe88ac0000000001000000015d5f292a7985cdacf6f31e6c3d82ab4d724a11da9f44340e5b968ebfb963c814000000006b483045022076d12b4b7459ed787ddc7a168108245106bcbe9bf4780f6b5197347d640f341c022100e580e77e6c02e4e9744a7d8d1a87ca04356a39b72ea1eab34f4c5f3eac30e28601210323ce145056204da091cca4e2ba69f51dc21b84040c7efd4747d351814a1d4690ffffffff02b66e82d9440000001976a914f29b7d4d4e85bfbadb83383943c7a1a9bc7801e288ac1b862dd2600000001976a914465126e435381aa8e1db75d173cd51832b8f87c888ac00000000";
    let valid_block: Block = deserialize(&hex!(BLOCK_3107_HEX)).unwrap();

    // The Rust implementation is currently subject to
    // [CVE-2012-2459](https://bitcointalk.org/index.php?topic=102395)
    let mut forged_block = valid_block.clone();
    forged_block.txdata.push(forged_block.txdata[4].clone());
    assert!(forged_block.check_merkle_root());

    let store = SimpleHeaderStore::new(deserialize(&hex!(BLOCK_HEADER_3106_HEX)).unwrap(), 3106);
    let validator = BlockValidator::new(store, Network::Dogecoin);

    assert_eq!(
        validator.validate_block(&valid_block, MOCK_CURRENT_TIME),
        Ok(())
    );
    assert_eq!(
        validator.validate_block(&forged_block, MOCK_CURRENT_TIME),
        Err(ValidateBlockError::DuplicateTransactions)
    );
}
