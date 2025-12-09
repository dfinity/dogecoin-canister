# Deploy your first dapp locally

Before using the local Dogecoin regtest instance, you will need to:

- [x] [Setup the local developer environment](./environment.md).
- [x] Clone the [Dogecoin canister repository](https://github.com/dfinity/dogecoin-canister.git).

This page demonstrates how to use the local Dogecoin regtest instance using the [`basic_dogecoin` example](https://github.com/dfinity/dogecoin-canister/tree/master/examples/basic_dogecoin) canister written in Rust. This example will serve as your first example canister to interact with the Dogecoin API. It already implements methods for sending and receiving dogecoin.

```admonish note title="Mac OS X users"
If you are using macOS, an `llvm` version that supports the `wasm32-unknown-unknown` target is required. This is because the Rust `bitcoin-dogecoin` library relies on `secp256k1-sys`, which requires `llvm` to build. The default `llvm` version provided by XCode does not meet this requirement. Instead, install the [Homebrew version](https://formulae.brew.sh/formula/llvm), using `brew install llvm`.
```

## Deploying your canister locally

First, Navigate into the `examples/basic_dogecoin` subdirectory of the Dogecoin canister repo:

```bash
cd examples/basic_dogecoin
```

When you set up your [developer environment](./environment.md), if you created the subdirectory for your `dogecoin_data` files in another project's directory, you either need to create them again or copy them into this project's folder.

Start the local Dogecoin regtest network:

```bash
dogecoind -datadir=$(pwd)/dogecoin_data -printtoconsole --port=18444
```

In another terminal, start `dfx` in your local development environment with the Dogecoin API enabled.

```bash
dfx start --clean --enable-dogecoin
````

In a third terminal, deploy the `basic_dogecoin` canister to your local development environment with the `dfx deploy` command and specify the `regtest` network as an init argument for the canister:

```bash
dfx deploy basic_dogecoin --argument '(variant { regtest })'
```

Congratulations! You have successfully deployed your first canister that can interact with Dogecoin.

## Interacting with your canister

You can interact with your deployed canister using the Candid interface link provided when you deployed the canister. You can also use the `dfx canister call` command to call the canister methods from the command line, as explained below.

### Generating a Dogecoin address

The `basic_dogecoin` example implements a function for generating a Dogecoin P2PKH address using the [`ecdsa_public_key`](https://internetcomputer.org/docs/references/ic-interface-spec#ic-ecdsa_public_key) API endpoint.

You can call this function from the command line:
```bash
dfx canister call basic_dogecoin get_p2pkh_address
```

### Receiving DOGE

In order to generate and receive DOGE on your local Dogecoin regtest, you need to manually mine blocks. DOGE is issued as a reward for each new block mined.

```admonish tip title="Mining blocks"
Block rewards are subject to the [Coinbase maturity rule](https://github.com/dogecoin/dogecoin/blob/7237da74b8c356568644cbe4fba19d994704355b/src/chainparams.cpp#L423): newly mined DOGE can only be spent after 60 more blocks have been mined.
```

Use the following command to mine 61 blocks and distribute the block rewards to the Dogecoin address generated previously:

```bash
dogecoin-cli -datadir=$(pwd)/dogecoin_data generatetoaddress 61 <doge-address>
```

After mining blocks, their hash will be returned. In the `dfx` logs, you will see log entries confirming that the Dogecoin canister which exposes the Dogecoin API has ingested the newly mined blocks.

Then, check your DOGE balance:

```bash
dfx canister call basic_dogecoin get_balance '("<doge-address>")'
```

### Sending DOGE

You can send DOGE using the `send_from_p2pkh_address` function of the `basic_dogecoin` canister. For example, to send 1 DOGE (100,000,000 koinus) to the address `mhXcJVuNA48bZsrKq4t21jx1neSqyceqTM`, run the following command:

```bash
dfx canister call basic_dogecoin send_from_p2pkh_address '(record { destination_address = "mhXcJVuNA48bZsrKq4t21jx1neSqyceqTM"; amount_in_koinu = 100000000; })'
```

This command creates a transaction and sends it to your local Dogecoin regtest. The value returned is the hash of your transaction. Now, you need to mine a block so that your transaction is included in the blockchain:

```bash
dogecoin-cli -datadir=$(pwd)/dogecoin_data generate 1
```

To verify that the transaction was successfully mined, you can use the `getblock` command of `dogecoin-cli`, which requires knowing the block hash. You can get the latest block hash using the `getbestblockhash` command:

```bash
dogecoin-cli -datadir=$(pwd)/dogecoin_data getbestblockhash
dogecoin-cli -datadir=$(pwd)/dogecoin_data getblock <hash_obtained_from_getbestblockhash>
```

After executing these commands, you should see your transaction hash in the list of transactions included in the block. The first transaction in the list is the coinbase transaction which contains the block reward.

### Getting block headers

You can retrieve block headers from the Dogecoin API using the `get_block_headers` function of the `basic_dogecoin` canister. For example, to get block headers from height 0 to height 10:

```bash
dfx canister call basic_dogecoin get_block_headers '(0:nat32, opt (10:nat32))'
```

## Troubleshooting

It's often useful to delete the entire local Dogecoin state and start from scratch. To do this:

In the terminal running `dfx`, stop the process using Ctrl+C, then delete the `.dfx` folder in your project directory which contains the local state of `dfx`.

```
rm -rf .dfx
```

In the terminal running `dogecoind`, stop the daemon using Ctrl+C, then delete the `regtest` data folder located inside `dogecoin_data`.

```bash
rm -r dogecoin_data/regtest
```





