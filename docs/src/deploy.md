# Deploy your first app locally

Before using the local Dogecoin regtest instance, you will need to:

- [x] [Setup the local developer environment](./environment.md).
- [x] Clone the `@dfinity/dogecoin-canister` repo: `git clone https://github.com/dfinity/dogecoin-canister.git`

This page will demonstrate how to use the local Dogecoin regtest instance using the [`basic_dogecoin` example](https://github.com/dfinity/dogecoin-canister/tree/master/examples/basic_dogecoin) project written in Rust. The `basic_dogecoin` example provides a simple smart contract that implements methods for sending and receiving dogecoin.

```admonish note title="Mac OS X users"
If you are using macOS, an `llvm` version that supports the `wasm32-unknown-unknown` target is required. This is because the Rust `bitcoin-dogecoin` library relies on `secp256k1-sys`, which requires `llvm` to build. The default `llvm` version provided by XCode does not meet this requirement. Instead, install the [Homebrew version](https://formulae.brew.sh/formula/llvm), using `brew install llvm`.
```

Navigate into the `examples/basic_dogecoin` subdirectory of the Dogecoin canister repo:

```bash
cd examples/basic_dogecoin
```

When you set up your [developer environment](./environment.md), if you created the subdirectory for your `dogecoin_data` files in another project's directory, you either need to create them again or copy them into this project's folder.

Start the local Dogecoin regtest network:

```bash
dogecoind -datadir=$(pwd)/dogecoin_data -printoconsole --port=18444
```

Next, deploy the `basic_dogecoin` canister to your local development environment with the `dfx deploy` command and specify the `regtest` network as an init argument for the canister:

```bash
dfx start --clean --enable-dogecoin --background // If dfx is not already running
dfx deploy basic_dogecoin --argument '(variant { regtest })'
```

## Make calls to the local Dogecoin regtest

### Generating a Dogecoin address

The `basic_bitcoin` example implements a function for generating a Dogecoin P2PKH address using the [`ecdsa_public_key`](https://internetcomputer.org/docs/references/ic-interface-spec#ic-ecdsa_public_key) API endpoint.

You can call this function from the command line:
```bash
dfx canister call basic_dogecoin get_p2pkh_address
```

### Receiving DOGE

In order to generate and receive DOGE on your local Dogecoin regtest, you need to manually mine blocks. DOGE is issued as a reward for each new block mined.

```admonish tip title="Mining blocks"
Block rewards are subject to the [Coinbase maturity rule](https://github.com/dogecoin/dogecoin/blob/7237da74b8c356568644cbe4fba19d994704355b/src/chainparams.cpp#L423): newly mined DOGE can only be spent after 60 more blocks have been mined.
```

Use the following command to mine blocks and distribute the block rewards to a specified Dogecoin address:

```bash
dogecoin-cli -datadir=$(pwd)/dogecoin_data generatetoaddress <number-of-blocks> <doge-address>
```

After mining a block, its hash will be returned. In the `dfx` logs, you will see a log entry confirming that `dfx` has ingested the newly mined block. Syncing the first Dogecoin block can take up to 30 seconds. Subsequent blocks sync nearly instantly.

Then, check your DOGE balance:

```bash
dfx canister call basic_dogecoin get_balance '("<doge-address>")'
```

### Sending DOGE

You can send DOGE using the `send_from_p2pkh_address` function of the `basic_bitcoin` canister:

```bash
dfx canister call basic_dogecoin send_from_p2pkh_address '(record { destination_address = "mhXcJVuNA48bZsrKq4t21jx1neSqyceqTM"; amount_in_koinu = 100000000; })'
```

This command creates a transaction and sends it to your local Dogecoin regtest. Now, you need to mine a block so that the transaction you just sent becomes part of the blockchain:

```bash
dogecoin-cli -datadir=$(pwd)/dogecoin_data generate 1
```

## Getting block headers

You can retrieve block headers from your local Dogecoin regtest using the `get_block_headers` function of the `basic_dogecoin` canister. For example, to get the block headers from height 0 to height 10:

```bash
dfx canister call basic_dogecoin get_block_headers '(0:nat32, opt 10:nat32)'
```


## Troubleshooting

### Sending transactions

If you're trying to send a transaction and the transaction isn't being mined, try sending the same transaction using `dogecoin-cli`, as it can reveal helpful errors:

```bash
dogecoin-cli -datadir=$(pwd)/dogecoin_data sendrawtransaction <tx-in-hex>
```

### Resetting the state

It's often useful to delete the entire local Dogecoin state and start from scratch. To do this:

- #### Step 1: Run the following commands in the directory of your `dfx` project to delete the local state of `dfx`.

```
dfx stop
rm -rf .dfx
```

- #### Step 2: In the folder where you're running `dogecoind`, stop the `dogecoind` process if it is running, and then delete the data folder you created.

```bash
dogecoin-cli -datadir=$(pwd)/dogecoin_data stop
rm -r dogecoin_data
```





