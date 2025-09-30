use ic_doge_interface::Height;
use ic_doge_types::{OutPoint, BlockHash};
use ic_doge_canister::types::{Storable, TxOut, Address, AddressUtxo, BlockHeaderBlob};
use ic_doge_canister::state::{UTXO_KEY_SIZE, UTXO_VALUE_MAX_SIZE_SMALL, UTXO_VALUE_MAX_SIZE_MEDIUM};
use ic_stable_structures::{
    memory_manager::{MemoryId, MemoryManager}, 
    storable::Blob,
    FileMemory, StableBTreeMap, Storable as StableStorable
};
use std::{
    fs::File,
    path::Path,
};

pub mod hash;

// Matches Dogecoin canister memory constants in `canister/src/memory.rs`
const ADDRESS_UTXOS_MEMORY_ID: MemoryId = MemoryId::new(1);
const SMALL_UTXOS_MEMORY_ID: MemoryId = MemoryId::new(2);
const MEDIUM_UTXOS_MEMORY_ID: MemoryId = MemoryId::new(3);
const BALANCES_MEMORY_ID: MemoryId = MemoryId::new(4);
const BLOCK_HEADERS_MEMORY_ID: MemoryId = MemoryId::new(5);
const BLOCK_HEIGHTS_MEMORY_ID: MemoryId = MemoryId::new(6);

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

/// Comprehensive data extracted from the canister state stored in stable memory
#[derive(Debug)]
pub struct CanisterData {
    pub address_utxos: Vec<AddressUtxo>,
    pub utxos: Vec<Utxo>,
    pub balances: Vec<(Address, u64)>,
    pub block_headers: Vec<(BlockHash, BlockHeaderBlob)>,
    pub block_heights: Vec<(Height, BlockHash)>,
}

/// UTXO reader that can read stable memory from a file
pub struct UtxoReader {
    memory_manager: MemoryManager<FileMemory>,
}

impl UtxoReader {
    /// Create a new UtxoReader from a stable memory file
    pub fn new<P: AsRef<Path>>(canister_state_path: P) -> Result<Self, std::io::Error> {
        let file = File::open(canister_state_path)?;
        let memory = FileMemory::new(file);
        let memory_manager = MemoryManager::init(memory);
        
        Ok(Self { memory_manager })
    }

    /// Extract all data from stable memory
    pub fn extract_state_data(&self) -> CanisterData {
        CanisterData {
            utxos: self.extract_utxos(),
            address_utxos: self.extract_address_utxos(),
            balances: self.extract_balances(),
            block_headers: self.extract_block_headers(),
            block_heights: self.extract_block_heights(),
        }
    }

    /// Extract all UTXOs from stable memory
    pub fn extract_utxos(&self) -> Vec<Utxo> {
        let mut utxos = Vec::new();

        // Extract small UTXOs from memory region 2
        utxos.extend(self.extract_small_utxos());

        // Extract medium UTXOs from memory region 3
        utxos.extend(self.extract_medium_utxos());

        // Note: Large UTXOs must be accessed separately as they are stored
        // in a separate memory region (upgrades memory region 0)

        utxos
    }

    /// Extract small UTXOs from stable memory
    fn extract_small_utxos(&self) -> Vec<Utxo> {
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
        
        utxos
    }

    /// Extract medium UTXOs from stable memory
    fn extract_medium_utxos(&self) -> Vec<Utxo> {
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
        
        utxos
    }

    /// Extract address to UTXOs map from stable memory
    fn extract_address_utxos(&self) -> Vec<AddressUtxo> {
        let address_utxos_memory = self.memory_manager.get(ADDRESS_UTXOS_MEMORY_ID);
        let address_utxos_map: StableBTreeMap<Blob<{ AddressUtxo::BOUND.max_size() as usize }>, (), _>
            = StableBTreeMap::init(address_utxos_memory);
        
        let mut address_utxos = Vec::new();
        
        for (key_blob, _) in address_utxos_map.iter() {
            let address_utxo = StableStorable::from_bytes(std::borrow::Cow::Borrowed(key_blob.as_slice()));
            address_utxos.push(address_utxo);
        }

        address_utxos
    }

    /// Extract address to balance map from stable memory
    fn extract_balances(&self) -> Vec<(Address, u64)> { // TODO(XC-492): change u64 to u128
        let balances_memory = self.memory_manager.get(BALANCES_MEMORY_ID);
        let balances_map: StableBTreeMap<Address, u64, _> = StableBTreeMap::init(balances_memory);
        
        let mut balances = Vec::new();
        
        for (address, balance) in balances_map.iter() {
            balances.push((address, balance));
        }
        
        balances
    }

    /// Extract block hash to block header map from stable memory
    fn extract_block_headers(&self) -> Vec<(BlockHash, BlockHeaderBlob)> {
        let block_headers_memory = self.memory_manager.get(BLOCK_HEADERS_MEMORY_ID);
        let block_headers_map: StableBTreeMap<BlockHash, BlockHeaderBlob, _> 
            = StableBTreeMap::init(block_headers_memory);
        
        let mut block_headers = Vec::new();
        
        for (block_hash, header_blob) in block_headers_map.iter() {
            block_headers.push((block_hash, header_blob));
        }
        
        block_headers
    }

    /// Extract height to block hash map from stable memory
    fn extract_block_heights(&self) -> Vec<(Height, BlockHash)> {
        let block_heights_memory = self.memory_manager.get(BLOCK_HEIGHTS_MEMORY_ID);
        let block_heights_map: StableBTreeMap<Height, BlockHash, _> 
            = StableBTreeMap::init(block_heights_memory);
        
        let mut block_heights = Vec::new();
        
        for (height, block_hash) in block_heights_map.iter() {
            block_heights.push((height, block_hash));
        }
        
        block_heights
    }

}

