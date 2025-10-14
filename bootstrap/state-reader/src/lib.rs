use ic_doge_canister::state::{
    UTXO_KEY_SIZE, UTXO_VALUE_MAX_SIZE_MEDIUM, UTXO_VALUE_MAX_SIZE_SMALL,
};
use ic_doge_canister::types::{Address, AddressUtxo, BlockHeaderBlob, Storable, TxOut};
use ic_doge_interface::Height;
use ic_doge_types::{BlockHash, OutPoint};
use ic_stable_structures::{
    memory_manager::MemoryManager, storable::Blob, FileMemory, StableBTreeMap,
    Storable as StableStorable,
};
use std::{fs::File, path::Path};

use std::cell::RefCell;

thread_local! {
    static QUIET_FLAG: RefCell<bool> = RefCell::new(false);
}

pub fn set_logging_quiet(quiet: bool) {
    QUIET_FLAG.with(|q| *q.borrow_mut() = quiet);
}

pub fn is_quiet() -> bool {
    QUIET_FLAG.with(|q| *q.borrow())
}

#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => {
        if !$crate::is_quiet() {
            println!($($arg)*);
        }
    };
}

pub mod hash;

/// Memory IDs used by the Dogecoin canister for different memory regions.
/// Match memory constants defined in `canister/src/memory.rs`.
pub mod memory_ids {
    use ic_stable_structures::memory_manager::MemoryId;

    pub const ADDRESS_UTXOS: MemoryId = MemoryId::new(1);
    pub const SMALL_UTXOS: MemoryId = MemoryId::new(2);
    pub const MEDIUM_UTXOS: MemoryId = MemoryId::new(3);
    pub const BALANCES: MemoryId = MemoryId::new(4);
    pub const BLOCK_HEADERS: MemoryId = MemoryId::new(5);
    pub const BLOCK_HEIGHTS: MemoryId = MemoryId::new(6);
}

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
        self.height
            .cmp(&other.height)
            .then(self.outpoint.txid.cmp(&other.outpoint.txid))
            .then(self.outpoint.vout.cmp(&other.outpoint.vout))
    }
}

/// Comprehensive data extracted from the canister state stored in stable memory
#[derive(Debug)]
pub struct CanisterData {
    pub address_utxos: Vec<AddressUtxo>,
    pub utxos: Vec<Utxo>,
    pub balances: Vec<(Address, u128)>,
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

    /// Read data from stable memory
    pub fn read_data(
        &self,
        process_utxos: bool,
        process_balances: bool,
        process_headers: bool,
    ) -> CanisterData {
        log!("Reading canister data from stable memory...");

        let utxos = if process_utxos {
            self.read_utxos()
        } else {
            log!("Skipping UTXOs");
            Vec::new()
        };

        let (address_utxos, balances) = if process_balances {
            (self.read_address_utxos(), self.read_balances())
        } else {
            log!("Skipping address-utxos and balances");
            (Vec::new(), Vec::new())
        };

        let (block_headers, block_heights) = if process_headers {
            (self.read_block_headers(), self.read_block_heights())
        } else {
            log!("Skipping block headers and heights");
            (Vec::new(), Vec::new())
        };

        CanisterData {
            utxos,
            address_utxos,
            balances,
            block_headers,
            block_heights,
        }
    }

    /// Read all UTXOs from stable memory
    pub fn read_utxos(&self) -> Vec<Utxo> {
        log!("Reading UTXOs from stable memory...");
        let mut utxos = Vec::new();

        // Read small UTXOs from memory region 2
        let small_utxos = self.read_small_utxos();
        utxos.extend(small_utxos);

        // Read medium UTXOs from memory region 3
        let medium_utxos = self.extract_medium_utxos();
        utxos.extend(medium_utxos);

        // Note: Large UTXOs must be accessed separately as they are stored
        // in a separate memory region (upgrades memory region 0)

        utxos
    }

    /// Read small UTXOs from stable memory
    fn read_small_utxos(&self) -> Vec<Utxo> {
        log!("  Reading small UTXOs...");
        let small_memory = self.memory_manager.get(memory_ids::SMALL_UTXOS);
        let small_utxos_map: StableBTreeMap<
            Blob<UTXO_KEY_SIZE>,
            Blob<UTXO_VALUE_MAX_SIZE_SMALL>,
            _,
        > = StableBTreeMap::init(small_memory);

        let mut utxos = Vec::new();
        let mut count = 0;

        for (key_blob, value_blob) in small_utxos_map.iter() {
            let outpoint =
                StableStorable::from_bytes(std::borrow::Cow::Borrowed(key_blob.as_slice()));
            let (txout, height) = <(TxOut, Height)>::from_bytes(value_blob.as_slice().to_vec());

            utxos.push(Utxo {
                outpoint,
                txout,
                height,
            });

            count += 1;
            if count % 1_000_000 == 0 {
                log!("    Processed {} small UTXOs...", count);
            }
        }

        utxos
    }

    /// Extract medium UTXOs from stable memory
    fn extract_medium_utxos(&self) -> Vec<Utxo> {
        log!("  Reading medium UTXOs...");
        let medium_memory = self.memory_manager.get(memory_ids::MEDIUM_UTXOS);
        let medium_utxos_map: StableBTreeMap<
            Blob<UTXO_KEY_SIZE>,
            Blob<UTXO_VALUE_MAX_SIZE_MEDIUM>,
            _,
        > = StableBTreeMap::init(medium_memory);

        let mut utxos = Vec::new();
        let mut count = 0;

        for (key_blob, value_blob) in medium_utxos_map.iter() {
            let outpoint =
                StableStorable::from_bytes(std::borrow::Cow::Borrowed(key_blob.as_slice()));
            let (txout, height) = <(TxOut, Height)>::from_bytes(value_blob.as_slice().to_vec());

            utxos.push(Utxo {
                outpoint,
                txout,
                height,
            });

            count += 1;
            if count % 1_000_000 == 0 {
                log!("    Processed {} medium UTXOs...", count);
            }
        }

        utxos
    }

    /// Read address to UTXOs map from stable memory
    fn read_address_utxos(&self) -> Vec<AddressUtxo> {
        log!("Reading address UTXOs from stable memory...");
        let address_utxos_memory = self.memory_manager.get(memory_ids::ADDRESS_UTXOS);
        let address_utxos_map: StableBTreeMap<
            Blob<{ AddressUtxo::BOUND.max_size() as usize }>,
            (),
            _,
        > = StableBTreeMap::init(address_utxos_memory);

        let mut address_utxos = Vec::new();
        let mut count = 0;

        for (key_blob, _) in address_utxos_map.iter() {
            let address_utxo =
                StableStorable::from_bytes(std::borrow::Cow::Borrowed(key_blob.as_slice()));
            address_utxos.push(address_utxo);

            count += 1;
            if count % 1_000_000 == 0 {
                log!("  Processed {} address UTXOs...", count);
            }
        }

        address_utxos
    }

    /// Read address to balance map from stable memory
    fn read_balances(&self) -> Vec<(Address, u128)> {
        log!("Reading address balances from stable memory...");
        let balances_memory = self.memory_manager.get(memory_ids::BALANCES);
        let balances_map: StableBTreeMap<Address, u128, _> = StableBTreeMap::init(balances_memory);

        let mut balances = Vec::new();
        let mut count = 0;

        for (address, balance) in balances_map.iter() {
            balances.push((address, balance));

            count += 1;
            if count % 1_000_000 == 0 {
                log!("  Processed {} address balances...", count);
            }
        }

        balances
    }

    /// Read block hash to block header map from stable memory
    fn read_block_headers(&self) -> Vec<(BlockHash, BlockHeaderBlob)> {
        log!("Reading block headers from stable memory...");
        let block_headers_memory = self.memory_manager.get(memory_ids::BLOCK_HEADERS);
        let block_headers_map: StableBTreeMap<BlockHash, BlockHeaderBlob, _> =
            StableBTreeMap::init(block_headers_memory);

        let mut block_headers = Vec::new();
        let mut count = 0;

        for (block_hash, header_blob) in block_headers_map.iter() {
            block_headers.push((block_hash, header_blob));

            count += 1;
            if count % 1_000_000 == 0 {
                log!("  Processed {} block headers...", count);
            }
        }

        block_headers
    }

    /// Read height to block hash map from stable memory
    fn read_block_heights(&self) -> Vec<(Height, BlockHash)> {
        log!("Reading block heights from stable memory...");
        let block_heights_memory = self.memory_manager.get(memory_ids::BLOCK_HEIGHTS);
        let block_heights_map: StableBTreeMap<Height, BlockHash, _> =
            StableBTreeMap::init(block_heights_memory);

        let mut block_heights = Vec::new();
        let mut count = 0;

        for (height, block_hash) in block_heights_map.iter() {
            block_heights.push((height, block_hash));

            count += 1;
            if count % 1_000_000 == 0 {
                log!("  Processed {} block heights...", count);
            }
        }

        block_heights
    }
}
