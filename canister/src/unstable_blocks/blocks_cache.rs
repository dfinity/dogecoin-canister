use ic_doge_types::{Block, BlockHash};
use ic_stable_structures::StableBTreeMap;
use std::collections::BTreeMap;

pub trait BlocksCache {
    fn insert(&mut self, block_hash: BlockHash, block: Block) -> bool;
    fn remove(&mut self, block_hash: &BlockHash) -> bool;
    fn get(&self, block_hash: &BlockHash) -> Option<Block>;
    fn len(&self) -> u64;
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
    fn len(&self) -> u64 {
        unimplemented!()
    }
}

impl BlocksCache for StableBTreeMap<BlockHash, Vec<u8>, crate::memory::Memory> {
    fn insert(&mut self, block_hash: BlockHash, block: Block) -> bool {
        let mut bytes = Vec::new();
        block.consensus_encode(&mut bytes).unwrap();
        StableBTreeMap::insert(self, block_hash, bytes).is_none()
    }
    fn remove(&mut self, block_hash: &BlockHash) -> bool {
        StableBTreeMap::remove(self, block_hash).is_some()
    }
    fn get(&self, block_hash: &BlockHash) -> Option<Block> {
        use bitcoin::consensus::Decodable;
        let bytes = StableBTreeMap::get(self, block_hash)?;
        let block = bitcoin::dogecoin::Block::consensus_decode(&mut bytes.as_slice()).ok()?;
        Some(Block::new(block))
    }
    fn len(&self) -> u64 {
        StableBTreeMap::len(self)
    }
}

impl BlocksCache for BTreeMap<BlockHash, Block> {
    fn insert(&mut self, block_hash: BlockHash, block: Block) -> bool {
        BTreeMap::insert(self, block_hash, block).is_none()
    }
    fn remove(&mut self, block_hash: &BlockHash) -> bool {
        BTreeMap::remove(self, block_hash).is_some()
    }
    fn get(&self, block_hash: &BlockHash) -> Option<Block> {
        BTreeMap::get(self, block_hash).cloned()
    }
    fn len(&self) -> u64 {
        BTreeMap::len(self) as u64
    }
}
