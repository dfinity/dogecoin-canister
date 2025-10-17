use crate::header::tests::HeaderValidatorExt;
use crate::header::HeaderValidator;
use crate::HeaderStore;
use bitcoin::block::{Header, Version};
use bitcoin::consensus::deserialize;
#[cfg(feature = "doge")]
use bitcoin::dogecoin::{
    auxpow::AuxPow, constants::genesis_block as dogecoin_genesis_block, Header as DogecoinHeader,
    Network as DogecoinNetwork,
};
use bitcoin::hashes::hex::FromHex;
#[cfg(feature = "btc")]
use bitcoin::{
    constants::genesis_block as bitcoin_genesis_block, network::Network as BitcoinNetwork,
};
use bitcoin::{BlockHash, CompactTarget, TxMerkleNode};
use csv::{Reader, StringRecord};
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Duration;

pub const MOCK_CURRENT_TIME: Duration = Duration::from_secs(2_634_590_600);
const TEST_DATA_FOLDER: &str = "tests/data";

#[cfg(feature = "btc")]
pub mod btc_files {
    pub const MAINNET_HEADERS_0_782282_RAW: &str = "btc/headers_0_782282_mainnet_raw.csv";
    pub const TESTNET_HEADERS_0_2425489_RAW: &str = "btc/headers_0_2425489_testnet_raw.csv";
    pub const MAINNET_HEADERS_586657_589289_PARSED: &str =
        "btc/headers_586657_589289_mainnet_parsed.csv";
    pub const TESTNET_HEADERS_1_5000_PARSED: &str = "btc/headers_1_5000_testnet_parsed.csv";
}

#[cfg(feature = "doge")]
pub mod doge_files {
    pub const MAINNET_HEADERS_0_700000_RAW: &str = "doge/headers_0_700000_mainnet_raw.csv";
    pub const TESTNET_HEADERS_0_2000000_RAW: &str = "doge/headers_0_2000000_testnet_raw.csv";
    pub const MAINNET_HEADERS_1_15000_PARSED: &str = "doge/headers_1_15000_mainnet_parsed.csv";
    pub const TESTNET_HEADERS_1_15000_PARSED: &str = "doge/headers_1_15000_testnet_parsed.csv";
    pub const MAINNET_HEADERS_521337_536336_PARSED: &str =
        "doge/headers_521337_536336_mainnet_parsed_with_auxpow.csv"; // Contains 14,955 auxpow blocks out of 15,000
    pub const TESTNET_HEADERS_293100_308099_PARSED: &str =
        "doge/headers_293100_308099_testnet_parsed_with_auxpow.csv"; // Contains 14,746 auxpow blocks out of 15,000
}

pub fn test_data_file(file: &str) -> PathBuf {
    PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap())
        .join(TEST_DATA_FOLDER)
        .join(file)
}

pub fn deserialize_header(encoded_bytes: &str) -> Header {
    let bytes = Vec::from_hex(encoded_bytes).expect("failed to decoded bytes");
    deserialize(bytes.as_slice()).expect("failed to deserialize")
}

#[cfg(feature = "doge")]
pub fn deserialize_auxpow_header(encoded_bytes: &str) -> DogecoinHeader {
    let bytes = Vec::from_hex(encoded_bytes).expect("failed to decoded bytes");
    deserialize(bytes.as_slice()).expect("failed to deserialize")
}

/// Creates a Header from a CSV record with fields: version, prev_blockhash, merkle_root, time, bits, nonce
fn header_from_csv_record(record: &StringRecord) -> Header {
    Header {
        version: Version::from_consensus(i32::from_str_radix(record.get(0).unwrap(), 16).unwrap()),
        prev_blockhash: BlockHash::from_str(record.get(1).unwrap()).unwrap(),
        merkle_root: TxMerkleNode::from_str(record.get(2).unwrap()).unwrap(),
        time: u32::from_str_radix(record.get(3).unwrap(), 16).unwrap(),
        bits: CompactTarget::from_consensus(
            u32::from_str_radix(record.get(4).unwrap(), 16).unwrap(),
        ),
        nonce: u32::from_str_radix(record.get(5).unwrap(), 16).unwrap(),
    }
}

#[cfg(feature = "doge")]
/// Creates an AuxPow from a CSV record with fields: coinbase_tx, parent_hash, coinbase_branch, coinbase_index, blockchain_branch, blockchain_index, parent_block_header
fn auxpow_from_csv_record(record: &StringRecord) -> AuxPow {
    AuxPow {
        coinbase_tx: deserialize(Vec::from_hex(record.get(6).unwrap()).unwrap().as_slice())
            .unwrap(),
        parent_hash: BlockHash::from_str(record.get(7).unwrap()).unwrap(),
        coinbase_branch: deserialize(Vec::from_hex(record.get(8).unwrap()).unwrap().as_slice())
            .unwrap(),
        coinbase_index: i32::from_le_bytes(
            hex::decode(record.get(9).unwrap())
                .unwrap()
                .try_into()
                .unwrap(),
        ),
        blockchain_branch: deserialize(Vec::from_hex(record.get(10).unwrap()).unwrap().as_slice())
            .unwrap(),
        blockchain_index: i32::from_le_bytes(
            hex::decode(record.get(11).unwrap())
                .unwrap()
                .try_into()
                .unwrap(),
        ),
        parent_block_header: deserialize_header(record.get(12).unwrap()),
    }
}

/// This function reads all headers from the specified CSV file and returns them as a `Vec<Header>`.
pub fn get_headers(file: &str) -> Vec<Header> {
    let rdr = Reader::from_path(test_data_file(file));
    assert!(rdr.is_ok(), "Unable to find {file} file");
    let mut rdr = rdr.unwrap();
    let mut headers = vec![];
    for result in rdr.records() {
        let record = result.unwrap();
        let header = header_from_csv_record(&record);
        headers.push(header);
    }
    headers
}

#[cfg(feature = "doge")]
/// This function reads all auxpow headers from the specified CSV file and returns them as a `Vec<Header>`.
pub fn get_auxpow_headers(file: &str) -> Vec<DogecoinHeader> {
    let rdr = Reader::from_path(test_data_file(file));
    assert!(rdr.is_ok(), "Unable to find {file} file");
    let mut rdr = rdr.unwrap();
    let mut headers = vec![];
    for result in rdr.records() {
        let record = result.unwrap();
        let pure_header = header_from_csv_record(&record);
        let aux_pow = pure_header
            .has_auxpow_bit()
            .then(|| auxpow_from_csv_record(&record));
        let header = DogecoinHeader {
            pure_header,
            aux_pow,
        };
        headers.push(header);
    }
    headers
}

#[cfg(feature = "btc")]
pub fn bitcoin_genesis_header(network: BitcoinNetwork, bits: CompactTarget) -> Header {
    Header {
        bits,
        ..bitcoin_genesis_block(network).header
    }
}

#[cfg(feature = "doge")]
pub fn dogecoin_genesis_header(network: &DogecoinNetwork, bits: CompactTarget) -> Header {
    let mut genesis_header = dogecoin_genesis_block(network).header;
    genesis_header.bits = bits;
    genesis_header.pure_header
}

pub fn next_block_header<T: HeaderValidator>(
    validator: &T,
    prev: Header,
    bits: CompactTarget,
) -> Header {
    Header {
        prev_blockhash: prev.block_hash(),
        time: prev.time + validator.pow_target_spacing().as_secs() as u32,
        bits,
        ..prev
    }
}

/// Creates a chain of headers with the given length and
/// proof of work for the first header.
pub fn build_header_chain<T: HeaderValidator + HeaderValidatorExt>(
    validator: &mut T,
    chain_length: u32,
) -> Header {
    let pow_limit = validator.pow_limit_bits();

    let current_height = validator.store().height();
    let mut last_header = validator.store().get_with_height(current_height).unwrap();

    for _ in 1..chain_length {
        let new_header = next_block_header(validator, last_header, pow_limit);
        validator.store_mut().add(new_header);
        last_header = new_header;
    }

    last_header
}
