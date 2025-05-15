# How to Generate `testnet_blocks.txt`

To generate the `testnet_blocks.txt` file from `./canister/test-data/testnet4_10k_blocks.dat`, run the `testnet_10k_blocks` test in the `ic-doge-canister` package with the `save_chain_as_hex` feature enabled:

```shell
cargo test --release -p ic-doge-canister --features save_chain_as_hex testnet_10k_blocks
```
