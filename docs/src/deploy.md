# Deploy your first app locally

Before using the local Dogecoin regtest instance, you will need to:

- [Set up the local developer environment](./environment.md).
- Clone the [Dogecoin canister repository](https://github.com/dfinity/dogecoin-canister.git).

This page demonstrates how to use the local Dogecoin regtest instance using the [`basic_dogecoin` example](https://github.com/dfinity/dogecoin-canister/tree/master/examples/basic_dogecoin) canister written in Rust. This example will serve as your first example canister to interact with the Dogecoin API. It already implements methods for sending and receiving dogecoin.

This walkthrough uses the recommended Docker-based setup, where icp-cli runs both the local IC network and a bundled `dogecoind` regtest node in a single container. (If you would rather run your own `dogecoind`, see [Option 2 in the developer environment](./environment.md#option-2-run-your-own-dogecoind-and-point-icp-cli-at-it).)

## Deploying your canister locally

First, navigate into the `examples/basic_dogecoin` subdirectory of the Dogecoin canister repo:

```bash
cd examples/basic_dogecoin
```

Build the custom network-launcher image. This bundles `dogecoind` with the network launcher and only needs to be done once (re-run it to pick up new releases):

```bash
./build-image.sh
```

Start the local network in the background. This launches the local IC network together with the regtest Dogecoin node:

```bash
icp network start -d
```

Deploy the `basic_dogecoin` canister. The `local` environment in [`icp.yaml`](https://github.com/dfinity/dogecoin-canister/blob/master/examples/basic_dogecoin/icp.yaml) already initializes it with the `regtest` network argument:

```bash
icp deploy --cycles 30t
```

Congratulations! You have successfully deployed your first canister that can interact with Dogecoin.

```admonish tip title="Out of cycles?"
If a call later fails with an out-of-cycles error, top up the canister and retry:

    icp canister top-up --amount 30t backend
```

## Interacting with your canister

You can interact with your deployed canister using the Candid interface link printed when you deployed the canister. You can also use the `icp canister call` command to call the canister methods from the command line, as explained below.

The mining and inspection commands below talk to the bundled `dogecoind` through the container. Capture the container ID once:

```bash
CONTAINER=$(docker ps --filter "ancestor=icp-cli-network-launcher-dogecoin" --format "{{.ID}}" | head -1)
```

### Generating a Dogecoin address

The `basic_dogecoin` example implements a function for generating a Dogecoin P2PKH address using the [`ecdsa_public_key`](https://docs.internetcomputer.org/references/ic-interface-spec/management-canister/#ic-ecdsa_public_key) API endpoint.

You can call this function from the command line:
```bash
icp canister call backend get_p2pkh_address '()'
```

### Receiving dogecoins

In order to generate and receive dogecoins on your local Dogecoin regtest, you need to manually mine blocks. Dogecoin is issued as a reward for each new block mined.

```admonish tip title="Mining blocks"
Block rewards are subject to the [coinbase maturity rule](https://github.com/dogecoin/dogecoin/blob/7237da74b8c356568644cbe4fba19d994704355b/src/chainparams.cpp#L423): newly mined dogecoins can only be spent after 60 more blocks have been mined.
```

Use the following command to mine 61 blocks and distribute the block rewards to the Dogecoin address generated previously:

```bash
docker exec $CONTAINER dogecoin-cli -regtest \
  -rpcuser=ic-doge-integration -rpcpassword=QPQiNaph19FqUsCrBRN0FII7lyM26B51fAMeBQzCb-E= \
  generatetoaddress 61 <doge-address>
```

The IC Dogecoin integration ingests the newly mined blocks continuously. Then, check your dogecoin balance:

```bash
icp canister call backend get_balance '("<doge-address>")'
```

### Sending dogecoins

You can send dogecoins using the `send_from_p2pkh_address` function of the `basic_dogecoin` canister. For example, to send 1 DOGE (100,000,000 koinus) to the address `mhXcJVuNA48bZsrKq4t21jx1neSqyceqTM`, run the following command:

```bash
icp canister call backend send_from_p2pkh_address '(record { destination_address = "mhXcJVuNA48bZsrKq4t21jx1neSqyceqTM"; amount_in_koinu = 100000000; })'
```

This command creates a transaction and sends it to your local Dogecoin regtest. The value returned is the hash of your transaction. Now, you need to mine a block so that your transaction is included in the blockchain:

```bash
docker exec $CONTAINER dogecoin-cli -regtest \
  -rpcuser=ic-doge-integration -rpcpassword=QPQiNaph19FqUsCrBRN0FII7lyM26B51fAMeBQzCb-E= \
  generatetoaddress 1 <doge-address>
```

To verify that the transaction was successfully mined, you can use the `getblock` command of `dogecoin-cli`, which requires knowing the block hash. You can get the latest block hash using the `getbestblockhash` command:

```bash
docker exec $CONTAINER dogecoin-cli -regtest \
  -rpcuser=ic-doge-integration -rpcpassword=QPQiNaph19FqUsCrBRN0FII7lyM26B51fAMeBQzCb-E= getbestblockhash
docker exec $CONTAINER dogecoin-cli -regtest \
  -rpcuser=ic-doge-integration -rpcpassword=QPQiNaph19FqUsCrBRN0FII7lyM26B51fAMeBQzCb-E= getblock <hash_obtained_from_getbestblockhash>
```

After executing these commands, you should see your transaction hash in the list of transactions included in the block. The first transaction in the list is the coinbase transaction which contains the block reward.

### Getting block headers

You can retrieve block headers from the Dogecoin API using the `get_block_headers` function of the `basic_dogecoin` canister. For example, to get block headers from height 0 to height 10:

```bash
icp canister call backend get_block_headers '(0:nat32, opt (10:nat32))'
```

## Troubleshooting

It's often useful to delete the entire local state and start from scratch. To do this, stop the network (this also stops the bundled `dogecoind`):

```bash
icp network stop
```

Then start it again to get a fresh regtest chain and a clean IC state:

```bash
icp network start -d
```
