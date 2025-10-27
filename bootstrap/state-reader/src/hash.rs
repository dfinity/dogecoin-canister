use crate::Utxo;
use ic_doge_canister::types::{Address, AddressUtxo, BlockHeaderBlob, TxOut};
use ic_doge_interface::Height;
use ic_doge_types::BlockHash;
use ic_stable_structures::Storable;
use sha2::{Digest, Sha256};

/// Compute SHA256 hash of UTXO set
pub fn compute_utxo_set_hash(utxos: &[Utxo]) -> [u8; 32] {
    let mut hasher = Sha256::new();

    for utxo in utxos {
        let Utxo {
            outpoint,
            txout,
            height,
        } = utxo;
        let TxOut {
            value,
            script_pubkey,
        } = txout;
        hasher.update(Storable::to_bytes(outpoint));
        hasher.update(value.to_le_bytes());
        hasher.update(script_pubkey);
        hasher.update(height.to_le_bytes());
    }

    hasher.finalize().into()
}

/// Compute SHA256 hash of address UTXOs data
pub fn compute_address_utxos_hash(address_utxos: &[AddressUtxo]) -> [u8; 32] {
    let mut hasher = Sha256::new();

    for addr_utxo in address_utxos {
        let AddressUtxo {
            address,
            height,
            outpoint,
        } = addr_utxo;
        hasher.update(address.to_string().as_bytes());
        hasher.update(height.to_le_bytes());
        hasher.update(outpoint.to_bytes());
    }

    hasher.finalize().into()
}

/// Compute SHA256 hash of address balances data
pub fn compute_address_balances_hash(balances: &[(Address, u128)]) -> [u8; 32] {
    let mut hasher = Sha256::new();

    for (address, balance) in balances {
        hasher.update(address.to_string().as_bytes());
        hasher.update(balance.to_le_bytes());
    }

    hasher.finalize().into()
}

/// Compute SHA256 hash of block headers data
pub fn compute_block_headers_hash(headers: &[(BlockHash, BlockHeaderBlob)]) -> [u8; 32] {
    let mut hasher = Sha256::new();

    for (hash, header_blob) in headers {
        hasher.update(hash.to_bytes());
        hasher.update(header_blob.as_slice());
    }

    hasher.finalize().into()
}

/// Compute SHA256 hash of block heights data
pub fn compute_block_heights_hash(heights: &[(Height, BlockHash)]) -> [u8; 32] {
    let mut hasher = Sha256::new();

    for (height, hash) in heights {
        hasher.update(height.to_le_bytes());
        hasher.update(hash.to_bytes());
    }

    hasher.finalize().into()
}

/// Compute combined hash of individual hashes
pub fn compute_combined_hash(hashes: &[[u8; 32]]) -> [u8; 32] {
    let mut hasher = Sha256::new();

    for hash in hashes {
        hasher.update(hash);
    }

    hasher.finalize().into()
}
