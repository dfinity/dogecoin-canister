use crate::{BlockHeight, HeaderStore};
use bitcoin::block::Header;
use bitcoin::BlockHash;
use std::collections::HashMap;
use std::time::Duration;

pub const MOCK_CURRENT_TIME: Duration = Duration::from_secs(2_634_590_600);

#[derive(Clone)]
struct StoredHeader {
    header: Header,
    height: BlockHeight,
}

pub struct SimpleHeaderStore {
    headers: HashMap<BlockHash, StoredHeader>,
    height: BlockHeight,
    tip_hash: BlockHash,
    initial_hash: BlockHash,
}

impl SimpleHeaderStore {
    pub fn new(initial_header: Header, height: BlockHeight) -> Self {
        let initial_hash = initial_header.block_hash();
        let tip_hash = initial_header.block_hash();
        let mut headers = HashMap::new();
        headers.insert(
            initial_hash,
            StoredHeader {
                header: initial_header,
                height,
            },
        );

        Self {
            headers,
            height,
            tip_hash,
            initial_hash,
        }
    }

    #[cfg(feature = "btc")]
    pub fn new_with_genesis_bitcoin(network: bitcoin::Network) -> Self {
        let genesis = bitcoin::constants::genesis_block(network);
        Self::new(genesis.header, 0)
    }

    #[cfg(feature = "doge")]
    pub fn new_with_genesis_dogecoin(network: bitcoin::dogecoin::Network) -> Self {
        let genesis = bitcoin::dogecoin::constants::genesis_block(network);
        Self::new(*genesis.header, 0)
    }

    pub fn add(&mut self, header: Header) {
        let prev = self
            .headers
            .get(&header.prev_blockhash)
            .expect("prev hash missing");
        let stored_header = StoredHeader {
            header,
            height: prev.height + 1,
        };

        self.height = stored_header.height;
        self.headers.insert(header.block_hash(), stored_header);
        self.tip_hash = header.block_hash();
    }
}

impl HeaderStore for SimpleHeaderStore {
    fn get_with_block_hash(&self, hash: &BlockHash) -> Option<Header> {
        self.headers.get(hash).map(|stored| stored.header)
    }

    fn get_with_height(&self, height: u32) -> Option<Header> {
        let blocks_to_traverse = self.height - height;
        let mut header = self.headers.get(&self.tip_hash).unwrap().header;
        for _ in 0..blocks_to_traverse {
            header = self.headers.get(&header.prev_blockhash).unwrap().header;
        }
        Some(header)
    }

    fn height(&self) -> u32 {
        self.height
    }

    fn get_initial_hash(&self) -> BlockHash {
        self.initial_hash
    }
}
