mod utils;

use crate::utils::{build_chain_from, build_chain_from_for_each};
use bitcoin::consensus::Decodable;
use bitcoin::dogecoin::constants::genesis_block;
use bitcoin::{block::Header, consensus::Encodable, dogecoin, dogecoin::Block as DogecoinBlock};
use canbench_rs::{bench, bench_fn, bench_scope, BenchResult};
use ic_cdk::init;
use ic_doge_canister::state::main_chain_height;
use ic_doge_canister::{
    types::{BlockHeaderBlob, Slicing},
    with_state, with_state_mut,
};
use ic_doge_interface::{
    GetBalanceRequest, GetBlockHeadersRequest, GetCurrentFeePercentilesRequest, GetUtxosRequest,
    InitConfig, Network, NetworkInRequest,
};
use ic_doge_test_utils::{build_regtest_chain, BlockBuilder, TransactionBuilder};
use ic_doge_types::Block;
use std::cell::RefCell;
use std::str::FromStr;

const ADDRESS: &str = "mwSSBD3NCriNXNMgd6dr2N2rxX9M9zXqrp";

fn parsed_address() -> dogecoin::Address {
    dogecoin::Address::from_str(ADDRESS)
        .unwrap()
        .assume_checked()
}

thread_local! {
    static MAINNET_BLOCKS: RefCell<Vec<Block>> =  const { RefCell::new(vec![])};
}

// Asserts that all blocks have been inserted so benchmarks are not silently run on a partial chain.
// If block insertion hits the instruction limit, the IC will trap, silently leaving fewer blocks than expected.
// Without this check, the benchmark could still run and report misleadingly low instruction counts.
fn assert_chain_height(expected: usize) {
    with_state(|s| {
        let chain_len = main_chain_height(s) as usize;
        assert_eq!(
            chain_len, expected,
            "Expected all blocks to be inserted. Max height should be {}, got {}.",
            expected, chain_len
        );
    });
}

// Asserts that all block headers have been inserted so benchmarks are not silently run on partial data.
fn assert_next_block_headers_max_height(expected: u32) {
    with_state(|s| {
        let max_height = s.unstable_blocks.next_block_headers_max_height().expect(
            "Failed to get next_block_headers_max_height: no new block headers have been inserted.",
        );
        assert_eq!(
            max_height, expected,
            "Expected all headers to be inserted. Max height should be {}, got {}.",
            expected, max_height
        );
    });
}

#[init]
fn init() {
    // Load the mainnet blocks.
    MAINNET_BLOCKS.with(|blocks| {
        blocks.replace(
            include_str!("../dogecoin_blocks_1_5000.hex")
                .trim()
                .split('\n')
                .map(|block_hex| {
                    let block_bytes = hex::decode(block_hex).unwrap();
                    Block::new(
                        DogecoinBlock::consensus_decode(&mut block_bytes.as_slice()).unwrap(),
                    )
                })
                .collect(),
        );
    });

    // Set mock time to avoid timestamp validation failure due to blocks appearing to be > 2 hours
    // in the future.
    let june_2025 = (55.5 * 365.25 * 24.0 * 60.0 * 60.0) as u64;
    ic_doge_canister::runtime::mock_time::set_mock_time_secs(june_2025);
}

// Insert the first 300 blocks of the Dogecoin mainnet.
#[bench(raw)]
fn insert_300_blocks() -> BenchResult {
    let blocks_to_insert = 300;

    ic_doge_canister::init(InitConfig {
        network: Some(Network::Mainnet),
        stability_threshold: Some(blocks_to_insert as u128),
        ..Default::default()
    });

    let result = bench_fn(|| {
        with_state_mut(|s| {
            for i in 0..blocks_to_insert {
                ic_doge_canister::state::insert_block(
                    s,
                    MAINNET_BLOCKS.with(|b| b.borrow()[i].clone()),
                )
                .unwrap();
            }
        });
    });
    assert_chain_height(blocks_to_insert);
    result
}

// Get the metrics when there are many unstable blocks.
#[bench(raw)]
fn get_metrics() -> BenchResult {
    let blocks_to_insert = 3000;

    ic_doge_canister::init(InitConfig {
        network: Some(Network::Mainnet),
        stability_threshold: Some(blocks_to_insert as u128),
        ..Default::default()
    });

    with_state_mut(|s| {
        for i in 0..blocks_to_insert {
            ic_doge_canister::state::insert_block(
                s,
                MAINNET_BLOCKS.with(|b| b.borrow()[i].clone()),
            )
            .unwrap();
        }
    });

    assert_chain_height(blocks_to_insert);

    bench_fn(|| {
        ic_doge_canister::get_metrics();
    })
}

// Insert 100 block headers into a tree containing 800 blocks.
#[bench(raw)]
fn insert_block_headers() -> BenchResult {
    let blocks_to_insert = 800;
    let block_headers_to_insert = 100;

    ic_doge_canister::init(InitConfig {
        network: Some(Network::Mainnet),
        ..Default::default()
    });

    // Insert the blocks.
    with_state_mut(|s| {
        for i in 0..blocks_to_insert {
            ic_doge_canister::state::insert_block(
                s,
                MAINNET_BLOCKS.with(|b| b.borrow()[i as usize].clone()),
            )
            .unwrap();
        }
    });

    assert_chain_height(blocks_to_insert as usize);

    // Compute the next block headers.
    let next_block_headers = MAINNET_BLOCKS.with(|b| {
        let blocks = b.borrow();
        let mut next_block_headers = vec![];
        for i in blocks_to_insert..blocks_to_insert + block_headers_to_insert {
            let mut block_header_blob = vec![];
            Header::consensus_encode(blocks[i as usize].header(), &mut block_header_blob).unwrap();
            next_block_headers.push(BlockHeaderBlob::from(block_header_blob));
        }

        next_block_headers
    });

    // Benchmark inserting the block headers.
    let bench_result = bench_fn(|| {
        with_state_mut(|s| {
            ic_doge_canister::state::insert_next_block_headers(s, next_block_headers.as_slice());
        });
    });

    assert_next_block_headers_max_height(blocks_to_insert + block_headers_to_insert);

    bench_result
}

// Insert the same block headers multiple times.
#[bench(raw)]
fn insert_block_headers_multiple_times() -> BenchResult {
    let block_headers_to_insert = 900;

    ic_doge_canister::init(InitConfig {
        network: Some(Network::Mainnet),
        ..Default::default()
    });

    // Compute the next block headers.
    let next_block_headers = MAINNET_BLOCKS.with(|b| {
        let blocks = b.borrow();
        let mut next_block_headers = vec![];
        for i in 0..block_headers_to_insert {
            let mut block_header_blob = vec![];
            Header::consensus_encode(blocks[i as usize].header(), &mut block_header_blob).unwrap();
            next_block_headers.push(BlockHeaderBlob::from(block_header_blob));
        }

        next_block_headers
    });

    // Benchmark inserting the block headers.
    let bench_result = bench_fn(|| {
        with_state_mut(|s| {
            for _ in 0..10 {
                ic_doge_canister::state::insert_next_block_headers(
                    s,
                    next_block_headers.as_slice(),
                );
            }
        });
    });

    assert_next_block_headers_max_height(block_headers_to_insert);

    bench_result
}

#[bench(raw)]
fn insert_block_with_10k_transactions() -> BenchResult {
    bench_insert_block(10_000)
}

#[bench(raw)]
fn insert_block_with_1k_transactions() -> BenchResult {
    bench_insert_block(1_000)
}

fn bench_insert_block(num_transactions: u32) -> BenchResult {
    /// Create a chain of 2 blocks after genesis.
    ///
    /// 1st block:
    /// * 1 coinbase transaction with `tx_cardinality` outputs
    ///
    /// 2nd block:
    /// * `tx_cardinality` transactions consuming the previous outputs
    fn mini_chain(tx_cardinality: u32) -> [Block; 2] {
        const ADDRESS_1: &str = "mhXcJVuNA48bZsrKq4t21jx1neSqyceqTM";
        const ADDRESS_2: &str = "mwoouFKeAiPoLi2oVpiEVYeNZAiE81abto";

        let address_1 = dogecoin::Address::from_str(ADDRESS_1)
            .unwrap()
            .assume_checked();
        let address_2 = dogecoin::Address::from_str(ADDRESS_2)
            .unwrap()
            .assume_checked();

        // Transaction 1: A coinbase tx with `tx_cardinality` inputs, each giving 1 Koinu to
        // address 1.
        let mut tx_1 = TransactionBuilder::coinbase();
        for i in 0..tx_cardinality {
            tx_1 = tx_1.with_output(&address_1, 1).with_lock_time(i)
        }
        let tx_1 = tx_1.build();
        let tx_1_id: bitcoin::Txid = tx_1.compute_txid();

        // Transaction 2: Consume all the outputs of transaction 1 *in reverse order* and create
        // similar outputs for address 2.
        let mut tx_2 = TransactionBuilder::new();
        for i in (0..tx_cardinality).rev() {
            tx_2 = tx_2.with_input(
                bitcoin::OutPoint {
                    vout: i,
                    txid: tx_1_id,
                },
                None,
            );
        }
        for i in 0..tx_cardinality {
            tx_2 = tx_2.with_output(&address_2, 1).with_lock_time(i);
        }
        let tx_2 = tx_2.build();

        let genesis = genesis_block(dogecoin::Network::Regtest);
        let block_1 = BlockBuilder::default()
            .with_prev_header(*genesis.header)
            .with_transaction(tx_1)
            .build();
        let block_2 = BlockBuilder::default()
            .with_prev_header(*block_1.header)
            .with_transaction(TransactionBuilder::coinbase().build())
            .with_transaction(tx_2)
            .build();
        [Block::new(block_1), Block::new(block_2)]
    }
    let [block_1, block_2] = mini_chain(num_transactions);

    ic_doge_canister::init(InitConfig {
        network: Some(Network::Regtest),
        ..Default::default()
    });

    with_state_mut(|s| {
        ic_doge_canister::state::insert_block(s, block_1).unwrap();
    });

    bench_fn(|| {
        with_state_mut(|s| {
            ic_doge_canister::state::insert_block(s, block_2).unwrap();
        });
    })
}

#[bench(raw)]
fn pre_upgrade_with_many_unstable_blocks() -> BenchResult {
    let blocks_to_insert: usize = 3000;

    let blocks = build_regtest_chain(blocks_to_insert as u32, 100, false);

    ic_doge_canister::init(InitConfig {
        network: Some(Network::Regtest),
        ..Default::default()
    });

    // Insert the blocks.
    with_state_mut(|s| {
        for block in blocks.into_iter().skip(1) {
            ic_doge_canister::state::insert_block(s, block).unwrap();
        }
    });

    assert_chain_height(blocks_to_insert - 1);

    bench_fn(|| {
        ic_doge_canister::pre_upgrade();
    })
}

// Benchmarks `get_blockchain_info` on a single linear chain (typical mainnet scenario).
#[bench(raw)]
fn get_blockchain_info_single_chain() -> BenchResult {
    let blocks_to_insert: usize = 1000;
    let num_transactions_per_block: usize = 300;
    let num_outputs_per_transaction: usize = 3;

    ic_doge_canister::init(InitConfig {
        network: Some(Network::Regtest),
        stability_threshold: Some(blocks_to_insert as u128),
        ..Default::default()
    });

    let address = parsed_address();
    let genesis = genesis_block(dogecoin::Network::Regtest);
    let mut counter = 1u64;
    let chain = build_chain_from(
        *genesis.header,
        blocks_to_insert,
        num_transactions_per_block,
        num_outputs_per_transaction,
        0,
        &address,
        &mut counter,
    );

    with_state_mut(|s| {
        for block in &chain {
            ic_doge_canister::state::insert_block(s, block.clone()).unwrap();
        }
    });

    assert_chain_height(blocks_to_insert);

    bench_fn(|| {
        ic_doge_canister::get_blockchain_info();
    })
}

// Benchmarks `get_blockchain_info` with a main chain and a few short forks.
#[bench(raw)]
fn get_blockchain_info_with_forks() -> BenchResult {
    let blocks_to_insert: usize = 1000;
    let num_transactions_per_block: usize = 300;
    let num_outputs_per_transaction: usize = 3;

    ic_doge_canister::init(InitConfig {
        network: Some(Network::Regtest),
        stability_threshold: Some(blocks_to_insert as u128),
        ..Default::default()
    });

    let address = parsed_address();
    let genesis = genesis_block(dogecoin::Network::Regtest);
    let mut counter = 1u64;
    let chain = build_chain_from(
        *genesis.header,
        blocks_to_insert,
        num_transactions_per_block,
        num_outputs_per_transaction,
        0,
        &address,
        &mut counter,
    );

    with_state_mut(|s| {
        for block in &chain {
            ic_doge_canister::state::insert_block(s, block.clone()).unwrap();
        }
    });

    // Add 5 forks at various heights, each 10 blocks long.
    for &fork_point in &[200, 400, 500, 600, 700] {
        let fork = build_chain_from(
            *chain[fork_point].header(),
            10,
            num_transactions_per_block,
            num_outputs_per_transaction,
            0,
            &address,
            &mut counter,
        );
        with_state_mut(|s| {
            for block in &fork {
                ic_doge_canister::state::insert_block(s, block.clone()).unwrap();
            }
        });
    }

    assert_chain_height(blocks_to_insert);

    bench_fn(|| {
        ic_doge_canister::get_blockchain_info();
    })
}

// Benchmarks `get_blockchain_info` with many branches of varying lengths (testnet-like scenario).
#[bench(raw)]
fn get_blockchain_info_many_branches() -> BenchResult {
    let blocks_to_insert = 1000;
    let num_transactions_per_block: usize = 300;
    let num_outputs_per_transaction: usize = 3;

    ic_doge_canister::init(InitConfig {
        network: Some(Network::Regtest),
        stability_threshold: Some(blocks_to_insert as u128),
        ..Default::default()
    });

    let address = parsed_address();
    let genesis = genesis_block(dogecoin::Network::Regtest);
    let mut counter = 1u64;
    let chain = build_chain_from(
        *genesis.header,
        blocks_to_insert,
        num_transactions_per_block,
        num_outputs_per_transaction,
        0,
        &address,
        &mut counter,
    );

    with_state_mut(|s| {
        for block in &chain {
            ic_doge_canister::state::insert_block(s, block.clone()).unwrap();
        }
    });

    // Add 49 forks at every 10th block, with varying lengths (5 to 14 blocks).
    for i in 0..(blocks_to_insert / 10) - 1 {
        let fork_point = i * 10;
        let fork_len = 5 + (i % 10);
        let fork = build_chain_from(
            *chain[fork_point].header(),
            fork_len,
            num_transactions_per_block,
            num_outputs_per_transaction,
            0,
            &address,
            &mut counter,
        );
        with_state_mut(|s| {
            for block in &fork {
                ic_doge_canister::state::insert_block(s, block.clone()).unwrap();
            }
        });
    }

    assert_chain_height(blocks_to_insert);

    bench_fn(|| {
        ic_doge_canister::get_blockchain_info();
    })
}

// Insert 250 block headers without AuxPow information in Regtest.
#[bench(raw)]
fn insert_block_headers_regtest_without_auxpow() -> BenchResult {
    let blocks_to_insert = 50;
    let block_headers_to_insert = 250;
    let num_transactions_per_block = 10;

    ic_doge_canister::init(InitConfig {
        network: Some(Network::Regtest),
        stability_threshold: Some(144),
        ..Default::default()
    });

    let chain = build_regtest_chain(
        blocks_to_insert + block_headers_to_insert,
        num_transactions_per_block,
        false,
    );

    // Insert the blocks.
    with_state_mut(|s| {
        for block in chain.iter().take(blocks_to_insert as usize).skip(1) {
            ic_doge_canister::state::insert_block(s, block.clone()).unwrap();
        }
    });

    // Compute the next block headers.
    let mut next_block_headers = vec![];
    for block in chain.iter().skip(blocks_to_insert as usize) {
        let mut block_header_blob = vec![];
        dogecoin::Header::consensus_encode(block.auxpow_header(), &mut block_header_blob).unwrap();
        next_block_headers.push(BlockHeaderBlob::from(block_header_blob));
    }

    // Benchmark inserting the block headers.
    let bench_result = bench_fn(|| {
        with_state_mut(|s| {
            ic_doge_canister::state::insert_next_block_headers(s, next_block_headers.as_slice());
        });
    });

    assert_next_block_headers_max_height(blocks_to_insert + block_headers_to_insert - 1);

    bench_result
}

// Insert the same 250 block headers without AuxPow multiple times in Regtest.
#[bench(raw)]
fn insert_block_headers_multiple_times_regtest_without_auxpow() -> BenchResult {
    let block_headers_to_insert = 250;

    ic_doge_canister::init(InitConfig {
        network: Some(Network::Regtest),
        ..Default::default()
    });

    // Compute the next block headers.
    let chain = build_regtest_chain(block_headers_to_insert, 10, false);

    let mut next_block_headers = vec![];
    for i in 1..block_headers_to_insert {
        let mut block_header_blob = vec![];
        dogecoin::Header::consensus_encode(
            chain[i as usize].auxpow_header(),
            &mut block_header_blob,
        )
        .unwrap();
        next_block_headers.push(BlockHeaderBlob::from(block_header_blob));
    }

    // Benchmark inserting the block headers.
    let bench_result = bench_fn(|| {
        with_state_mut(|s| {
            for _ in 0..10 {
                ic_doge_canister::state::insert_next_block_headers(
                    s,
                    next_block_headers.as_slice(),
                );
            }
        });
    });

    assert_next_block_headers_max_height(block_headers_to_insert - 1);

    bench_result
}

#[bench(raw)]
fn dogecoin_get_balance_baseline() -> BenchResult {
    bench_get_balance(3)
}

#[bench(raw)]
fn dogecoin_get_balance_stress() -> BenchResult {
    bench_get_balance(100)
}

fn bench_get_balance(num_outputs_to_address_per_block: usize) -> BenchResult {
    let blocks_to_insert = 1000;
    let num_transactions_per_block = 300;
    let num_outputs_per_transaction = 3;

    ic_doge_canister::init(InitConfig {
        network: Some(Network::Regtest),
        stability_threshold: Some(blocks_to_insert as u128),
        ..Default::default()
    });

    let address = parsed_address();
    let genesis = genesis_block(dogecoin::Network::Regtest);
    let mut counter = 1u64;
    with_state_mut(|s| {
        build_chain_from_for_each(
            *genesis.header,
            blocks_to_insert,
            num_transactions_per_block,
            num_outputs_per_transaction,
            num_outputs_to_address_per_block,
            &address,
            &mut counter,
            |block| {
                ic_doge_canister::state::insert_block(s, block).unwrap();
            },
        );
    });

    assert_chain_height(blocks_to_insert);

    bench_fn(|| {
        ic_doge_canister::get_balance_query(GetBalanceRequest {
            address: ADDRESS.to_string(),
            network: NetworkInRequest::Regtest,
            min_confirmations: None,
        })
        .unwrap();
    })
}

#[bench(raw)]
fn dogecoin_get_utxos_baseline() -> BenchResult {
    bench_get_utxos(3)
}

#[bench(raw)]
fn dogecoin_get_utxos_stress() -> BenchResult {
    bench_get_utxos(100)
}

fn bench_get_utxos(num_outputs_to_address_per_block: usize) -> BenchResult {
    let blocks_to_insert = 1000;
    let num_transactions_per_block = 300;
    let num_outputs_per_transaction = 3;

    ic_doge_canister::init(InitConfig {
        network: Some(Network::Regtest),
        stability_threshold: Some(blocks_to_insert as u128),
        ..Default::default()
    });

    let address = parsed_address();
    let genesis = genesis_block(dogecoin::Network::Regtest);
    let mut counter = 1u64;
    with_state_mut(|s| {
        build_chain_from_for_each(
            *genesis.header,
            blocks_to_insert,
            num_transactions_per_block,
            num_outputs_per_transaction,
            num_outputs_to_address_per_block,
            &address,
            &mut counter,
            |block| {
                ic_doge_canister::state::insert_block(s, block).unwrap();
            },
        );
    });

    assert_chain_height(blocks_to_insert);

    let mut total_utxos = 0;
    let result = bench_fn(|| {
        total_utxos = 0;
        let mut page = None;
        loop {
            let response = ic_doge_canister::get_utxos_query(GetUtxosRequest {
                address: ADDRESS.to_string(),
                network: NetworkInRequest::Regtest,
                filter: page.map(ic_doge_interface::UtxosFilterInRequest::Page),
            })
            .unwrap();
            total_utxos += response.utxos.len();
            match response.next_page {
                Some(next) => page = Some(next),
                None => break,
            }
        }
    });

    let expected_utxos = blocks_to_insert * num_outputs_to_address_per_block;
    assert_eq!(
        total_utxos, expected_utxos,
        "Expected {} UTXOs for the address, got {}.",
        expected_utxos, total_utxos
    );
    result
}

#[bench(raw)]
fn dogecoin_get_current_fee_percentiles() -> BenchResult {
    let blocks_to_insert = 1000;
    let num_transactions_per_block = 300;
    let num_outputs_per_transaction = 3;

    ic_doge_canister::init(InitConfig {
        network: Some(Network::Regtest),
        stability_threshold: Some(blocks_to_insert as u128),
        ..Default::default()
    });

    let address = parsed_address();
    let genesis = genesis_block(dogecoin::Network::Regtest);
    let mut counter = 1u64;
    with_state_mut(|s| {
        build_chain_from_for_each(
            *genesis.header,
            blocks_to_insert,
            num_transactions_per_block,
            num_outputs_per_transaction,
            0,
            &address,
            &mut counter,
            |block| {
                ic_doge_canister::state::insert_block(s, block).unwrap();
            },
        );
    });

    assert_chain_height(blocks_to_insert);

    bench_fn(|| {
        ic_doge_canister::get_current_fee_percentiles_without_fees(
            GetCurrentFeePercentilesRequest {
                network: NetworkInRequest::Regtest,
            },
        );
    })
}

// Reproduces the synchronous work of a heartbeat on a fully-synced canister over a
// realistic unstable block tree, attributing the cost across its components.
//
// NOTE: the async fetch (the inter-canister `bitcoin_get_successors` call) is excluded,
// since canbench builds to wasm32 where the real call is compiled in and cannot execute.
#[bench(raw)]
fn heartbeat_steady_state_synced() -> BenchResult {
    let blocks_to_insert: usize = 100;
    let num_transactions_per_block: usize = 300;
    let num_outputs_per_transaction: usize = 3;

    ic_doge_canister::init(InitConfig {
        network: Some(Network::Regtest),
        stability_threshold: Some((blocks_to_insert * 2) as u128),
        ..Default::default()
    });

    let address = parsed_address();
    let genesis = genesis_block(dogecoin::Network::Regtest);
    let mut counter = 1u64;
    let chain = build_chain_from(
        *genesis.header,
        blocks_to_insert,
        num_transactions_per_block,
        num_outputs_per_transaction,
        0,
        &address,
        &mut counter,
    );

    with_state_mut(|s| {
        for block in &chain {
            ic_doge_canister::state::insert_block(s, block.clone()).unwrap();
        }
    });

    assert_chain_height(blocks_to_insert);

    bench_fn(|| {
        {
            let _s = bench_scope("collect_metrics_tip_depths");
            with_state(|s| {
                let _ = s.unstable_blocks.tip_depths();
            });
        }

        {
            let _s = bench_scope("ingest_no_op");
            with_state_mut(|s| {
                let slicing = ic_doge_canister::state::ingest_stable_blocks_into_utxoset(s);
                assert!(matches!(slicing, Slicing::Done(false)));
            });
        }

        {
            let _s = bench_scope("build_get_successors_request");
            with_state(|s| {
                let _ = ic_doge_canister::state::get_block_hashes(s);
            });
        }
    })
}

#[bench(raw)]
fn dogecoin_get_block_headers_baseline() -> BenchResult {
    bench_get_block_headers(1440)
}

#[bench(raw)]
fn dogecoin_get_block_headers_stress() -> BenchResult {
    bench_get_block_headers(5000)
}

fn bench_get_block_headers(blocks_to_insert: usize) -> BenchResult {
    ic_doge_canister::init(InitConfig {
        network: Some(Network::Regtest),
        stability_threshold: Some(blocks_to_insert as u128),
        ..Default::default()
    });

    let address = parsed_address();
    let genesis = genesis_block(dogecoin::Network::Regtest);
    let mut counter = 1u64;
    let chain = build_chain_from(
        *genesis.header,
        blocks_to_insert,
        1,
        1,
        0,
        &address,
        &mut counter,
    );

    with_state_mut(|s| {
        for block in &chain {
            ic_doge_canister::state::insert_block(s, block.clone()).unwrap();
        }
    });

    assert_chain_height(blocks_to_insert);

    bench_fn(|| {
        ic_doge_canister::get_block_headers_without_fees(GetBlockHeadersRequest {
            start_height: 0,
            end_height: None,
            network: NetworkInRequest::Regtest,
        })
        .unwrap();
    })
}

fn main() {}
