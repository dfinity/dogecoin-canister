use bitcoin::{consensus::Encodable, dogecoin::Header};
use canbench_rs::{bench, bench_fn, BenchResult};
use ic_doge_canister::types::BlockHeaderBlob;
use ic_doge_canister::{with_state, with_state_mut};
use ic_doge_interface::{InitConfig, Network};
use ic_doge_test_utils::build_regtest_chain;

// Insert 250 block headers with AuxPow information in Regtest.
#[bench(raw)]
fn insert_block_headers_regtest_with_auxpow() -> BenchResult {
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
        true,
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
        Header::consensus_encode(block.auxpow_header(), &mut block_header_blob).unwrap();
        next_block_headers.push(BlockHeaderBlob::from(block_header_blob));
    }

    // Benchmark inserting the block headers.
    let bench_result = bench_fn(|| {
        with_state_mut(|s| {
            ic_doge_canister::state::insert_next_block_headers(s, next_block_headers.as_slice());
        });
    });

    with_state(|s| {
        let max_height = s.unstable_blocks.next_block_headers_max_height().expect(
            "Failed to get next_block_headers_max_height: no new block headers have been inserted.",
        );
        assert_eq!(
            max_height,
            blocks_to_insert + block_headers_to_insert - 1,
            "Expected all headers to be inserted. Max height should be {}, got {}.",
            blocks_to_insert + block_headers_to_insert - 1,
            max_height
        );
    });

    bench_result
}

// Insert the same 250 block headers with AuxPow multiple times in Regtest.
#[bench(raw)]
fn insert_block_headers_multiple_times_regtest_with_auxpow() -> BenchResult {
    let block_headers_to_insert = 250;

    ic_doge_canister::init(InitConfig {
        network: Some(Network::Regtest),
        ..Default::default()
    });

    // Compute the next block headers.
    let chain = build_regtest_chain(block_headers_to_insert, 10, true);

    let mut next_block_headers = vec![];
    for i in 1..block_headers_to_insert {
        let mut block_header_blob = vec![];
        Header::consensus_encode(chain[i as usize].auxpow_header(), &mut block_header_blob)
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

    with_state(|s| {
        let max_height = s.unstable_blocks.next_block_headers_max_height().expect(
            "Failed to get next_block_headers_max_height: no new block headers have been inserted.",
        );
        assert_eq!(
            max_height,
            block_headers_to_insert - 1,
            "Expected all headers to be inserted. Max height should be {}, got {}.",
            block_headers_to_insert - 1,
            max_height
        );
    });

    bench_result
}

fn main() {}
