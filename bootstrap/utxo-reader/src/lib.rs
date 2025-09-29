use ic_doge_interface::Height;
use ic_doge_types::OutPoint;
use ic_doge_canister::types::{Storable, TxOut};
use ic_doge_canister::state::{UTXO_KEY_SIZE, UTXO_VALUE_MAX_SIZE_SMALL, UTXO_VALUE_MAX_SIZE_MEDIUM};
use ic_stable_structures::{
    memory_manager::{MemoryId, MemoryManager}, 
    storable::Blob,
    FileMemory, StableBTreeMap, Storable as StableStorable
};
use sha2::{Digest, Sha256};
use std::{
    fs::File,
    path::Path,
};

// Matches Dogecoin canister memory constants in `canister/src/memory.rs`
const SMALL_UTXOS_MEMORY_ID: MemoryId = MemoryId::new(2);
const MEDIUM_UTXOS_MEMORY_ID: MemoryId = MemoryId::new(3);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Utxo {
    pub outpoint: OutPoint,
    pub txout: TxOut,
    pub height: Height,
}

impl PartialOrd for Utxo {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Utxo {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.height.cmp(&other.height)
            .then(self.outpoint.txid.cmp(&other.outpoint.txid))
            .then(self.outpoint.vout.cmp(&other.outpoint.vout))
    }
}

/// UTXO reader that can extract UTXOs from a canister_state.bin file
pub struct UtxoReader {
    memory_manager: MemoryManager<FileMemory>,
}

impl UtxoReader {
    /// Create a new UtxoReader from a canister state file
    pub fn new<P: AsRef<Path>>(canister_state_path: P) -> Result<Self, std::io::Error> {
        let file = File::open(canister_state_path)?;
        let memory = FileMemory::new(file);
        let memory_manager = MemoryManager::init(memory);
        
        Ok(Self { memory_manager })
    }

    /// Extract all UTXOs from the canister state file
    pub fn extract_utxos(&self) -> Result<Vec<Utxo>, Box<dyn std::error::Error>> {
        let mut utxos = Vec::new();
        
        // Extract small UTXOs from memory region 2
        utxos.extend(self.extract_small_utxos()?);
        
        // Extract medium UTXOs from memory region 3
        utxos.extend(self.extract_medium_utxos()?);
        
        // Note: Large UTXOs are accessed separately via extract_large_utxos_from_state()
        // after the canister state is properly initialized
        
        Ok(utxos)
    }

    /// Extract small UTXOs from stable memory
    fn extract_small_utxos(&self) -> Result<Vec<Utxo>, Box<dyn std::error::Error>> {
        let small_memory = self.memory_manager.get(SMALL_UTXOS_MEMORY_ID);
        let small_utxos_map: StableBTreeMap<Blob<UTXO_KEY_SIZE>, Blob<UTXO_VALUE_MAX_SIZE_SMALL>, _>
            = StableBTreeMap::init(small_memory);
        
        let mut utxos = Vec::new();
        
        for (key_blob, value_blob) in small_utxos_map.iter() {
            let outpoint = StableStorable::from_bytes(std::borrow::Cow::Borrowed(key_blob.as_slice()));
            let (txout, height) = <(TxOut, Height)>::from_bytes(value_blob.as_slice().to_vec());
            
            utxos.push(Utxo {
                outpoint,
                txout,
                height,
            });
        }
        
        Ok(utxos)
    }

    /// Extract medium UTXOs from stable memory
    fn extract_medium_utxos(&self) -> Result<Vec<Utxo>, Box<dyn std::error::Error>> {
        let medium_memory = self.memory_manager.get(MEDIUM_UTXOS_MEMORY_ID);
        let medium_utxos_map: StableBTreeMap<Blob<UTXO_KEY_SIZE>, Blob<UTXO_VALUE_MAX_SIZE_MEDIUM>, _> 
            = StableBTreeMap::init(medium_memory);
        
        let mut utxos = Vec::new();
        
        for (key_blob, value_blob) in medium_utxos_map.iter() {
            let outpoint = StableStorable::from_bytes(std::borrow::Cow::Borrowed(key_blob.as_slice()));
            let (txout, height) = <(TxOut, Height)>::from_bytes(value_blob.as_slice().to_vec());
            
            utxos.push(Utxo {
                outpoint,
                txout,
                height,
            });
        }
        
        Ok(utxos)
    }

    /// Extract large UTXOs from the already-loaded canister state
    pub fn extract_large_utxos_from_state() -> Result<Vec<Utxo>, Box<dyn std::error::Error>> {
        let large_utxos = ic_doge_canister::with_state(|state| {
            state.utxos.utxos.large_utxos.clone()
        });
        
        let large_count = large_utxos.len();
        if large_count > 0 {
            println!("Found {} large UTXOs from canister state", large_count);
        }
        
        let mut utxos = Vec::new();
        for (outpoint, (txout, height)) in large_utxos {
            utxos.push(Utxo {
                outpoint,
                txout,
                height,
            });
        }
        
        Ok(utxos)
    }

    /// Compute SHA256 hash of sorted UTXOs
    pub fn compute_utxo_set_hash(utxos: &[Utxo]) -> String {
        let mut hasher = Sha256::new();
        
        for utxo in utxos {
            hasher.update(&StableStorable::to_bytes(&utxo.outpoint));
            hasher.update(&utxo.txout.value.to_le_bytes());
            hasher.update(&utxo.txout.script_pubkey);
            hasher.update(&utxo.height.to_le_bytes());
        }
        
        hex::encode(hasher.finalize())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_utxo_ordering() {
        use ic_doge_types::Txid;
        
        let txid1 = Txid::from(vec![1u8; 32]);
        let txid2 = Txid::from(vec![2u8; 32]);
        
        let outpoint1 = OutPoint::new(txid1, 0);
        let outpoint2 = OutPoint::new(txid2, 0);
        
        let utxo1 = Utxo {
            outpoint: outpoint1,
            txout: TxOut { value: 100, script_pubkey: vec![1, 2, 3] },
            height: 1000,
        };
        
        let utxo2 = Utxo {
            outpoint: outpoint2,
            txout: TxOut { value: 200, script_pubkey: vec![4, 5, 6] },
            height: 2000,
        };
        
        assert!(utxo1 < utxo2);
        
        let mut utxos = vec![utxo2.clone(), utxo1.clone()];
        utxos.sort();
        assert_eq!(utxos[0], utxo1);
        assert_eq!(utxos[1], utxo2);
    }
    
    #[test]
    fn test_hash_computation() {
        use ic_doge_types::Txid;
        
        let txid = Txid::from(vec![1u8; 32]);
        let outpoint = OutPoint::new(txid, 0);
        
        let utxo = Utxo {
            outpoint,
            txout: TxOut { value: 100, script_pubkey: vec![1, 2, 3] },
            height: 1000,
        };
        
        let utxos = vec![utxo];
        let hash = UtxoReader::compute_utxo_set_hash(&utxos);
        
        // Hash should be deterministic
        let hash2 = UtxoReader::compute_utxo_set_hash(&utxos);
        assert_eq!(hash, hash2);
        
        // Hash should be a valid hex string of correct length (64 chars for SHA256)
        assert_eq!(hash.len(), 64);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }
}
