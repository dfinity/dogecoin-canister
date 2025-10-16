use ic_doge_interface::Network;
use ic_doge_types::{Block, BlockHash};
use ic_stable_structures::StableBTreeMap;
use std::collections::BTreeMap;

pub trait BlocksCache {
    fn insert(&mut self, block_hash: BlockHash, block: Block) -> bool;
    fn remove(&mut self, block_hash: &BlockHash) -> bool;
    fn get(&self, block_hash: &BlockHash) -> Option<Block>;
    fn is_empty(&self) -> bool;
    fn len(&self) -> u64;
    fn network(&self) -> Network;
}

/// Dummy implementation that panics!
impl BlocksCache for () {
    fn insert(&mut self, _block_hash: BlockHash, _block: Block) -> bool {
        unimplemented!()
    }
    fn remove(&mut self, _block_hash: &BlockHash) -> bool {
        unimplemented!()
    }
    fn get(&self, _block_hash: &BlockHash) -> Option<Block> {
        unimplemented!()
    }
    fn is_empty(&self) -> bool {
        unimplemented!()
    }
    fn len(&self) -> u64 {
        unimplemented!()
    }
    fn network(&self) -> Network {
        unimplemented!()
    }
}

pub struct StableBTreeCache {
    pub network: Network,
    map: StableBTreeMap<BlockHash, Vec<u8>, crate::memory::Memory>,
}

impl StableBTreeCache {
    pub fn new(network: Network, memory: crate::memory::Memory) -> Self {
        Self {
            network,
            map: StableBTreeMap::init(memory),
        }
    }
}

impl BlocksCache for StableBTreeCache {
    fn insert(&mut self, block_hash: BlockHash, block: Block) -> bool {
        let mut bytes = Vec::new();
        block.consensus_encode(&mut bytes).unwrap();
        self.map.insert(block_hash, bytes).is_none()
    }
    fn remove(&mut self, block_hash: &BlockHash) -> bool {
        self.map.remove(block_hash).is_some()
    }
    fn get(&self, block_hash: &BlockHash) -> Option<Block> {
        use bitcoin::consensus::Decodable;
        let bytes = self.map.get(block_hash)?;
        let block = bitcoin::dogecoin::Block::consensus_decode(&mut bytes.as_slice()).ok()?;
        Some(Block::new(block))
    }
    fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
    fn len(&self) -> u64 {
        self.map.len()
    }
    fn network(&self) -> Network {
        self.network
    }
}

pub struct BTreeCache {
    pub network: Network,
    map: BTreeMap<BlockHash, Block>,
}

impl BTreeCache {
    pub fn new(network: Network) -> Self {
        Self {
            network,
            map: Default::default(),
        }
    }
}

impl BlocksCache for BTreeCache {
    fn insert(&mut self, block_hash: BlockHash, block: Block) -> bool {
        self.map.insert(block_hash, block).is_none()
    }
    fn remove(&mut self, block_hash: &BlockHash) -> bool {
        self.map.remove(block_hash).is_some()
    }
    fn get(&self, block_hash: &BlockHash) -> Option<Block> {
        self.map.get(block_hash).cloned()
    }
    fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
    fn len(&self) -> u64 {
        self.map.len() as u64
    }
    fn network(&self) -> Network {
        self.network
    }
}
