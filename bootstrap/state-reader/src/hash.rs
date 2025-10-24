use crate::Utxo;
use ic_doge_canister::types::{Address, AddressUtxo, BlockHeaderBlob};
use ic_doge_interface::Height;
use ic_doge_types::BlockHash;
use ic_stable_structures::Storable;
use sha2::{Digest, Sha256};

/// Compute SHA256 hash of UTXO set
pub fn compute_utxo_set_hash(utxos: &[Utxo]) -> String {
    let mut hasher = Sha256::new();

    for utxo in utxos {
        hasher.update(Storable::to_bytes(&utxo.outpoint));
        hasher.update(utxo.txout.value.to_le_bytes());
        hasher.update(&utxo.txout.script_pubkey);
        hasher.update(utxo.height.to_le_bytes());
    }

    hex::encode(hasher.finalize())
}

/// Compute SHA256 hash of address UTXOs data
pub fn compute_address_utxos_hash(address_utxos: &[AddressUtxo]) -> String {
    let mut hasher = Sha256::new();

    for addr_utxo in address_utxos {
        hasher.update(addr_utxo.address.to_string().as_bytes());
        hasher.update(addr_utxo.height.to_le_bytes());
        hasher.update(addr_utxo.outpoint.to_bytes());
    }

    hex::encode(hasher.finalize())
}

/// Compute SHA256 hash of address balances data
pub fn compute_address_balances_hash(balances: &[(Address, u128)]) -> String {
    let mut hasher = Sha256::new();

    for (address, balance) in balances {
        hasher.update(address.to_string().as_bytes());
        hasher.update(balance.to_le_bytes());
    }

    hex::encode(hasher.finalize())
}

/// Compute SHA256 hash of block headers data
pub fn compute_block_headers_hash(headers: &[(BlockHash, BlockHeaderBlob)]) -> String {
    let mut hasher = Sha256::new();

    for (hash, header_blob) in headers {
        hasher.update(hash.to_bytes());
        hasher.update(header_blob.as_slice());
    }

    hex::encode(hasher.finalize())
}

/// Compute SHA256 hash of block heights data
pub fn compute_block_heights_hash(heights: &[(Height, BlockHash)]) -> String {
    let mut hasher = Sha256::new();

    for (height, hash) in heights {
        hasher.update(height.to_le_bytes());
        hasher.update(hash.to_bytes());
    }

    hex::encode(hasher.finalize())
}

/// Compute combined hash of individual hashes
pub fn compute_combined_hash(hashes: &[&str]) -> String {
    let mut hasher = Sha256::new();

    for hash in hashes {
        hasher.update(hash.as_bytes());
    }

    hex::encode(hasher.finalize())
}
