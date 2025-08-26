use bitcoin::dogecoin::auxpow::{AuxPow, MERGED_MINING_HEADER};
use bitcoin::dogecoin::constants::genesis_block;
use bitcoin::hashes::Hash;
use bitcoin::{
    absolute::LockTime,
    block::{Header as PureHeader, Version},
    dogecoin::auxpow::VERSION_AUXPOW,
    dogecoin::Address,
    dogecoin::Block as DogecoinBlock,
    dogecoin::Header,
    dogecoin::Network,
    secp256k1::Secp256k1,
    Amount, BlockHash, OutPoint, PublicKey, Script, ScriptBuf, Sequence, Target, Transaction, TxIn,
    TxMerkleNode, TxOut, Witness,
};
use ic_doge_types::Block;
use simple_rng::generate_keypair;
use std::str::FromStr;

mod simple_rng;

const DUMMY_CHAIN_ID: i32 = 42;
pub const DOGECOIN_CHAIN_ID: i32 = 98;
const BASE_VERSION: i32 = 5;
const CHAIN_MERKLE_HEIGHT: usize = 3; // Height of the blockchain Merkle tree used in AuxPow
const CHAIN_MERKLE_NONCE: u32 = 7; // Nonce used to calculate block header indexes into blockchain Merkle tree

/// Generates a random P2PKH address.
pub fn random_p2pkh_address(network: Network) -> Address {
    let secp = Secp256k1::new();
    let (_, pk) = generate_keypair(&secp);

    Address::p2pkh(PublicKey::new(pk), network)
}

/// Generates a random P2SH address.
pub fn random_p2sh_address(network: Network) -> Address {
    let secp = Secp256k1::new();
    let (_, pk) = generate_keypair(&secp);
    let pubkey = PublicKey::new(pk);

    // Create a p2pk script: <pubkey> OP_CHECKSIG
    let script = Script::builder()
        .push_key(&pubkey)
        .push_opcode(bitcoin::opcodes::all::OP_CHECKSIG)
        .into_script();

    Address::p2sh(&script, network).expect("Valid script should create valid P2SH address")
}

/// Mines a block that either matches or doesn't match the difficulty target specified in the header.
pub fn mine_header_to_target(header: &mut PureHeader, should_pass: bool) {
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

pub struct BlockBuilder {
    header: Option<Header>,
    prev_header: Option<PureHeader>,
    transactions: Vec<Transaction>,
    with_auxpow: bool,
}

impl BlockBuilder {
    pub fn new() -> Self {
        Self {
            header: None,
            prev_header: None,
            transactions: vec![],
            with_auxpow: false,
        }
    }

    pub fn with_prev_header(mut self, prev_header: PureHeader) -> Self {
        self.prev_header = Some(prev_header);
        self
    }

    pub fn with_header(mut self, header: Header) -> Self {
        self.header = Some(header);
        self
    }

    pub fn with_transaction(mut self, transaction: Transaction) -> Self {
        self.transactions.push(transaction);
        self
    }

    pub fn with_auxpow(mut self, auxpow: bool) -> Self {
        self.with_auxpow = auxpow;
        self
    }

    pub fn build(self) -> DogecoinBlock {
        let txdata = if self.transactions.is_empty() {
            // Create a default coinbase transaction.
            vec![TransactionBuilder::new().build()]
        } else {
            self.transactions
        };

        if let Some(header) = self.header {
            return DogecoinBlock { header, txdata };
        }

        let merkle_root = bitcoin::merkle_tree::calculate_root(
            txdata
                .iter()
                .map(|tx| *tx.compute_txid().as_raw_hash())
                .clone(),
        )
        .unwrap();
        let merkle_root = TxMerkleNode::from_raw_hash(merkle_root);

        let header_builder = match self.prev_header {
            None => HeaderBuilder::genesis(merkle_root),
            Some(prev_header) => HeaderBuilder::new()
                .with_prev_header(prev_header)
                .with_merkle_root(merkle_root),
        };

        if self.with_auxpow {
            let pure_header = header_builder
                .with_version(BASE_VERSION)
                .with_chain_id(DOGECOIN_CHAIN_ID)
                .with_auxpow_bit(true)
                .build();
            let aux_pow = AuxPowBuilder::new(pure_header.block_hash()).build();
            DogecoinBlock {
                header: Header {
                    pure_header,
                    aux_pow: Some(aux_pow),
                },
                txdata,
            }
        } else {
            DogecoinBlock {
                header: header_builder.build().into(),
                txdata,
            }
        }
    }
}

pub struct HeaderBuilder {
    version: i32,
    prev_header: Option<PureHeader>,
    merkle_root: TxMerkleNode,
    with_valid_pow: bool,
}

impl HeaderBuilder {
    pub fn new() -> Self {
        Self {
            version: 1,
            prev_header: None,
            merkle_root: TxMerkleNode::all_zeros(),
            with_valid_pow: true,
        }
    }

    fn genesis(merkle_root: TxMerkleNode) -> Self {
        Self {
            version: 1,
            prev_header: None,
            merkle_root,
            with_valid_pow: true,
        }
    }

    pub fn with_prev_header(mut self, prev_header: PureHeader) -> Self {
        self.prev_header = Some(prev_header);
        self
    }

    pub fn with_merkle_root(mut self, merkle_root: TxMerkleNode) -> Self {
        self.merkle_root = merkle_root;
        self
    }

    pub fn with_version(mut self, version: i32) -> Self {
        self.version = version;
        self
    }

    pub fn with_chain_id(mut self, chain_id: i32) -> Self {
        self.version |= chain_id << 16;
        self
    }

    pub fn with_auxpow_bit(mut self, auxpow_bit: bool) -> Self {
        if auxpow_bit {
            self.version |= VERSION_AUXPOW;
        } else {
            self.version &= !VERSION_AUXPOW;
        }
        self
    }

    pub fn with_valid_pow(mut self, valid_pow: bool) -> Self {
        self.with_valid_pow = valid_pow;
        self
    }

    pub fn build(self) -> PureHeader {
        let time = match &self.prev_header {
            Some(header) => header.time + 60,
            None => 0,
        };
        let bits = match &self.prev_header {
            Some(header) => header.bits,
            None => Target::MAX_ATTAINABLE_REGTEST.to_compact_lossy(),
        };

        let mut header = PureHeader {
            version: Version::from_consensus(self.version),
            time,
            nonce: 0,
            bits,
            merkle_root: self.merkle_root,
            prev_blockhash: self
                .prev_header
                .map_or(BlockHash::all_zeros(), |h| h.block_hash()),
        };

        mine_header_to_target(&mut header, self.with_valid_pow);

        header
    }
}

pub struct AuxPowBuilder {
    aux_block_hash: BlockHash,
    merkle_height: usize,
    merkle_nonce: u32,
    chain_id: i32,
    parent_chain_id: i32,
    base_version: i32,
    with_valid_pow: bool,
}

impl AuxPowBuilder {
    pub fn new(aux_block_hash: BlockHash) -> Self {
        Self {
            aux_block_hash,
            merkle_height: CHAIN_MERKLE_HEIGHT,
            merkle_nonce: CHAIN_MERKLE_NONCE,
            chain_id: DOGECOIN_CHAIN_ID,
            parent_chain_id: DUMMY_CHAIN_ID,
            base_version: BASE_VERSION,
            with_valid_pow: true,
        }
    }

    pub fn with_valid_pow(mut self, valid_pow: bool) -> Self {
        self.with_valid_pow = valid_pow;
        self
    }

    pub fn build(self) -> AuxPow {
        let expected_index =
            AuxPow::get_expected_index(self.merkle_nonce, self.chain_id, self.merkle_height);

        let blockchain_branch: Vec<TxMerkleNode> = (0..self.merkle_height)
            .map(|i| TxMerkleNode::from_byte_array([i as u8; 32]))
            .collect();

        let blockchain_merkle_root =
            AuxPow::compute_merkle_root(self.aux_block_hash, &blockchain_branch, expected_index);
        let mut blockchain_merkle_root_le = blockchain_merkle_root.to_byte_array();
        blockchain_merkle_root_le.reverse();

        let mut script_data = Vec::new();
        script_data.extend_from_slice(&MERGED_MINING_HEADER);
        script_data.extend_from_slice(&blockchain_merkle_root_le);
        script_data.extend_from_slice(&(1u32 << self.merkle_height).to_le_bytes());
        script_data.extend_from_slice(&self.merkle_nonce.to_le_bytes());

        let coinbase_tx = TransactionBuilder::new()
            .with_coinbase_script(ScriptBuf::from_bytes(script_data))
            .build();

        let mut parent_block_header = HeaderBuilder::new()
            .with_version(self.base_version)
            .with_chain_id(self.parent_chain_id)
            .with_merkle_root(TxMerkleNode::from_byte_array(
                coinbase_tx.compute_txid().to_byte_array(),
            ))
            .build();

        mine_header_to_target(&mut parent_block_header, self.with_valid_pow);

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
}

pub struct TransactionBuilder {
    input: Vec<TxIn>,
    output: Vec<TxOut>,
    lock_time: u32,
}

impl TransactionBuilder {
    pub fn new() -> Self {
        Self {
            input: vec![],
            output: vec![],
            lock_time: 0,
        }
    }

    pub fn coinbase() -> Self {
        Self {
            input: vec![Self::coinbase_input(Script::new().into())],
            output: vec![],
            lock_time: 0,
        }
    }

    fn coinbase_input(script_sig: ScriptBuf) -> TxIn {
        TxIn {
            previous_output: OutPoint::null(),
            script_sig,
            sequence: Sequence(0xffffffff),
            witness: Witness::new(),
        }
    }

    pub fn with_coinbase_script(mut self, script_sig: ScriptBuf) -> Self {
        self.input = vec![Self::coinbase_input(script_sig)];
        self
    }

    pub fn with_input(mut self, previous_output: OutPoint, witness: Option<Witness>) -> Self {
        let witness = witness.map_or(Witness::new(), |w| w);
        let input = TxIn {
            previous_output,
            script_sig: Script::new().into(),
            sequence: Sequence(0xffffffff),
            witness,
        };
        self.input.push(input);
        self
    }

    pub fn with_output(mut self, address: &Address, satoshi: u64) -> Self {
        self.output.push(TxOut {
            value: Amount::from_sat(satoshi),
            script_pubkey: address.script_pubkey(),
        });
        self
    }

    pub fn with_lock_time(mut self, time: u32) -> Self {
        self.lock_time = time;
        self
    }

    pub fn build(self) -> Transaction {
        let input = if self.input.is_empty() {
            // Default to coinbase if no inputs provided.
            vec![Self::coinbase_input(Script::new().into())]
        } else {
            self.input
        };
        let output = if self.output.is_empty() {
            // Use default of 50 DOGE.
            vec![TxOut {
                value: Amount::from_sat(50_0000_0000),
                script_pubkey: random_p2pkh_address(Network::Regtest).script_pubkey(),
            }]
        } else {
            self.output
        };

        Transaction {
            version: bitcoin::transaction::Version(1),
            lock_time: LockTime::from_consensus(self.lock_time),
            input,
            output,
        }
    }
}

impl Default for TransactionBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Builds a random chain with the given number of block and transactions
/// starting with the Regtest genesis block.
pub fn build_regtest_chain(
    num_blocks: u32,
    num_transactions_per_block: u32,
    with_auxpow: bool,
) -> Vec<Block> {
    let dogecoin_network = Network::Regtest;
    let genesis_block = Block::new(genesis_block(dogecoin_network));

    // Use a static address to send outputs to.
    // `random_p2pkh_address` isn't used here as it doesn't work in wasm.
    let address = Address::from_str("mhXcJVuNA48bZsrKq4t21jx1neSqyceqTM")
        .unwrap()
        .assume_checked();
    let mut blocks = vec![genesis_block.clone()];
    let mut prev_block: Block = genesis_block;
    let mut value = 1;

    // Since we start with a genesis block, we need `num_blocks - 1` additional blocks.
    for i in 0..num_blocks - 1 {
        let mut block_builder = BlockBuilder::new().with_prev_header(*prev_block.header());

        if with_auxpow && i >= dogecoin_network.params().auxpow_height {
            block_builder = block_builder.with_auxpow(true);
        }

        let mut transactions = vec![];
        for _ in 0..num_transactions_per_block {
            transactions.push(
                TransactionBuilder::new()
                    .with_output(&address, value)
                    .build(),
            );
            // Vary the value of the transaction to ensure that
            // we get unique outpoints in the blockchain.
            value += 1;
        }

        for transaction in transactions.iter() {
            block_builder = block_builder.with_transaction(transaction.clone());
        }

        let block = Block::new(block_builder.build());
        blocks.push(block.clone());
        prev_block = block;
    }

    blocks
}

#[cfg(test)]
mod test {
    mod transaction_builder {
        use crate::{random_p2pkh_address, TransactionBuilder};
        use bitcoin::{dogecoin::Network, OutPoint};

        #[test]
        fn new_build() {
            let tx = TransactionBuilder::new().build();
            assert!(tx.is_coinbase());
            assert_eq!(tx.input.len(), 1);
            assert_eq!(tx.input[0].previous_output, OutPoint::null());
            assert_eq!(tx.output.len(), 1);
            assert_eq!(tx.output[0].value.to_sat(), 50_0000_0000);
        }

        #[test]
        fn coinbase() {
            let tx = TransactionBuilder::new().build();
            assert!(tx.is_coinbase());
            assert_eq!(tx.input.len(), 1);
            assert_eq!(tx.input[0].previous_output, OutPoint::null());
            assert_eq!(tx.output.len(), 1);
            assert_eq!(tx.output[0].value.to_sat(), 50_0000_0000);
        }

        #[test]
        fn with_output() {
            let address = random_p2pkh_address(Network::Regtest);
            let tx = TransactionBuilder::new()
                .with_output(&address, 1000)
                .build();

            assert!(tx.is_coinbase());
            assert_eq!(tx.input.len(), 1);
            assert_eq!(tx.input[0].previous_output, OutPoint::null());
            assert_eq!(tx.output.len(), 1);
            assert_eq!(tx.output[0].value.to_sat(), 1000);
            assert_eq!(tx.output[0].script_pubkey, address.script_pubkey());
        }

        #[test]
        fn with_output_2() {
            let network = Network::Regtest;
            let address_0 = random_p2pkh_address(network);
            let address_1 = random_p2pkh_address(network);
            let tx = TransactionBuilder::new()
                .with_output(&address_0, 1000)
                .with_output(&address_1, 2000)
                .build();

            assert!(tx.is_coinbase());
            assert_eq!(tx.input.len(), 1);
            assert_eq!(tx.input[0].previous_output, OutPoint::null());
            assert_eq!(tx.output.len(), 2);
            assert_eq!(tx.output[0].value.to_sat(), 1000);
            assert_eq!(tx.output[0].script_pubkey, address_0.script_pubkey());
            assert_eq!(tx.output[1].value.to_sat(), 2000);
            assert_eq!(tx.output[1].script_pubkey, address_1.script_pubkey());
        }

        #[test]
        fn with_input() {
            let network = Network::Regtest;
            let address = random_p2pkh_address(network);
            let coinbase_tx = TransactionBuilder::new()
                .with_output(&address, 1000)
                .build();

            let tx = TransactionBuilder::new()
                .with_input(bitcoin::OutPoint::new(coinbase_tx.compute_txid(), 0), None)
                .build();
            assert!(!tx.is_coinbase());
            assert_eq!(tx.input.len(), 1);
            assert_eq!(
                tx.input[0].previous_output,
                bitcoin::OutPoint::new(coinbase_tx.compute_txid(), 0)
            );
            assert_eq!(tx.output.len(), 1);
            assert_eq!(tx.output[0].value.to_sat(), 50_0000_0000);
        }

        #[test]
        fn with_input_2() {
            let network = Network::Regtest;
            let address = random_p2pkh_address(network);
            let coinbase_tx_0 = TransactionBuilder::new()
                .with_output(&address, 1000)
                .build();
            let coinbase_tx_1 = TransactionBuilder::new()
                .with_output(&address, 2000)
                .build();

            let tx = TransactionBuilder::new()
                .with_input(
                    bitcoin::OutPoint::new(coinbase_tx_0.compute_txid(), 0),
                    None,
                )
                .with_input(
                    bitcoin::OutPoint::new(coinbase_tx_1.compute_txid(), 0),
                    None,
                )
                .build();
            assert!(!tx.is_coinbase());
            assert_eq!(tx.input.len(), 2);
            assert_eq!(
                tx.input[0].previous_output,
                bitcoin::OutPoint::new(coinbase_tx_0.compute_txid(), 0)
            );
            assert_eq!(
                tx.input[1].previous_output,
                bitcoin::OutPoint::new(coinbase_tx_1.compute_txid(), 0)
            );
            assert_eq!(tx.output.len(), 1);
            assert_eq!(tx.output[0].value.to_sat(), 50_0000_0000);
        }
    }
}
