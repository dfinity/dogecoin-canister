# How to Generate `testnet_blocks_200k.txt`

To generate the `testnet_blocks_200k.txt` file from `../canister/test-data/testnet_200k_blocks.dat`, run the
`testnet_200k_blocks` test in the `ic-doge-canister` package with the `save_chain_as_hex` feature enabled:

```shell
cargo test --release -p ic-doge-canister --features save_chain_as_hex testnet_200k_blocks
```
