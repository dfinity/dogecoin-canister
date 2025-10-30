# Basic Dogecoin

This example demonstrates how to deploy a smart contract on the Internet Computer that can receive and send dogecoin with support for P2PKH address type.

## Table of contents

* [Architecture](#architecture)
* [Deploying from ICP Ninja](#deploying-from-icp-ninja)
* [Building and deploying the smart contract locally](#building-and-deploying-the-smart-contract-locally)
  * [1. Prerequisites](#1-prerequisites)
  * [2. Clone the examples repo](#2-clone-the-examples-repo)
  * [3. Start the ICP execution environment](#3-start-the-icp-execution-environment)
  * [4. Start Dogecoin regtest](#4-start-dogecoin-regtest)
  * [5. Deploy the smart contract](#4-deploy-the-smart-contract)
* [Generating Dogecoin addresses](#generating-dogecoin-addresses)
* [Receiving dogecoin](#receiving-dogecoin)
* [Prerequisites](#prerequisites)
* [Checking balance](#checking-balance)
* [Sending dogecoin](#sending-dogecoin)
* [Retrieving block headers](#retrieving-block-headers)
* [Notes on implementation](#notes-on-implementation)
* [Security considerations and best practices](#security-considerations-and-best-practices)

## Architecture

This example integrates with the Internet Computer's built-in:

* [ECDSA API](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-ecdsa_public_key)
* [Dogecoin API](https://github.com/dfinity/dogecoin-canister/blob/master/INTERFACE_SPECIFICATION.md)

For background on the ICP<>BTC integration, refer to the [Learn Hub](https://learn.internetcomputer.org/hc/en-us/articles/34211154520084-Bitcoin-Integration).


## Deploying from ICP Ninja

This example can be deployed directly to the Internet Computer using ICP Ninja, where it connects to Dogecoin **testnet**. Note: Canisters deployed via ICP Ninja remain live for 50 minutes after signing in with your Internet Identity.

[![](https://icp.ninja/assets/open.svg)](https://icp.ninja/editor?g=https://github.com/dfinity/examples/tree/master/rust/basic_bitcoin)

## Building and deploying the smart contract locally

### 1. Prerequisites

* [x] [Rust toolchain](https://www.rust-lang.org/tools/install)
* [x] [Internet Computer SDK](https://internetcomputer.org/docs/building-apps/getting-started/install)
* [x] [Local Dogecoin testnet (regtest)](https://internetcomputer.org/docs/build-on-btc/btc-dev-env#create-a-local-bitcoin-testnet-regtest-with-bitcoind)
* [x] On macOS, an `llvm` version that supports the `wasm32-unknown-unknown` target is required. The Rust `bitcoin` library relies on the `secp256k1-sys` crate, which requires `llvm` to build. The default `llvm` version provided by XCode does not meet this requirement. Install the [Homebrew version](https://formulae.brew.sh/formula/llvm) using `brew install llvm`.


### 2. Clone the examples repo

```bash
git clone https://github.com/dfinity/examples
cd examples/rust/basic_dogecoin
```

### 3. Start the ICP execution environment


Open a terminal window (terminal 1) and run the following:
```bash
dfx start --enable-bitcoin --bitcoin-node 127.0.0.1:18444
```
This starts a local canister execution environment with Dogecoin support enabled.

### 4. Start Dogecoin regtest

Open another terminal window (terminal 2) and run the following to start the local Dogecoin regtest network:

```bash
dogecoind -conf=$(pwd)/dogecoin.conf -datadir=$(pwd)/dogecoin_data --port=18444
```

### 5. Deploy the smart contract

Open a third terminal (terminal 3) and run the following to deploy the smart contract:

```bash
dfx deploy basic_dogecoin --argument '(variant { regtest })'
```

What this does:

- `dfx deploy` tells the command line interface to `deploy` the smart contract.
- `--argument '(variant { regtest })'` passes the argument `regtest` to initialize the smart contract, telling it to connect to the local Dogecoin regtest network.

Your smart contract is live and ready to use! You can interact with it using either the command line or the Candid UI (the link you see in the terminal).
## Generating Dogecoin addresses

The example demonstrates how to generate and use P2PKH addresses using ECDSA and `sign_with_ecdsa`:

```bash
dfx canister call basic_dogecoin get_p2pkh_address
```

## Receiving dogecoin

Use the `dogecoin-cli` to mine a Dogecoin block and send the block reward in the form of local testnet dogecoin to one of the smart contract addresses.
```bash
dogecoin-cli -conf=$(pwd)/dogecoin.conf generatetoaddress 1 <dogecoin_address>
```

## Checking balance

Check the balance of any Dogecoin address:
```bash
dfx canister call basic_dogecoin get_balance '("<dogecoin_address>")'
```

This uses `dogecoin_get_balance` and works for any supported address type. The balance requires at least one confirmation to be reflected.
## Sending dogecoin

You can send dogecoin using the `send_from_p2pkh_address` endpoint.

The endpoint internally:

1. Estimates fees
2. Looks up spendable UTXOs
3. Builds a transaction to the target address
4. Signs using ECDSA
5. Broadcasts the transaction using `dogecoin_send_transaction`

Example:

```bash
dfx canister call basic_dogecoin send_from_p2pkh_address '(record {
  destination_address = "bcrt1qg8qknn6f3txqg97gt8ca0ctya0vw7ep6d02qmt";
  amount_in_satoshi = 4321;
})'
```

> [!IMPORTANT]
> Newly mined dogecoin, like those you created with the above `dogecoin-cli` command, cannot be spent until 100 additional blocks have been added to the chain. To make your dogecoin spendable, create 100 additional blocks. Choose one of the smart contract addresses as receiver of the block reward or use any valid Dogecoin dummy address.
>
> ```bash
> dogecoin-cli -conf=$(pwd)/dogecoin.conf generatetoaddress 100 <dogecoin_address>
> ```

The function returns the transaction ID. When interacting with the contract deployed on IC mainnet, you can track testnet transactions on [mempool.space](https://mempool.space/testnet4/).

## Retrieving block headers

You can query historical block headers:

```bash
dfx canister call basic_dogecoin get_block_headers '(10: nat32, null)'
# or a range:
dfx canister call basic_dogecoin get_block_headers '(10: nat32, opt (11: nat32))'
```

This calls `dogecoin_get_block_headers`, which is useful for blockchain validation or light client logic.

## Notes on implementation

This example implements several important patterns for Dogecoin integration:

- **Derivation paths**: Keys are derived using structured derivation paths according to BIP-32, ensuring reproducible key generation.
- **Key caching**: Optimization is used to avoid repeated calls to `get_ecdsa_public_key`.
- **Manual transaction construction**: Transactions are assembled and signed manually, ensuring maximum flexibility in construction and fee estimation.
- **Cost optimization**: When testing on mainnet, the [chain-key testing canister](https://github.com/dfinity/chainkey-testing-canister) can be used to save on costs for calling the threshold signing APIs.

## Security considerations and best practices

This example is provided for educational purposes and is not production-ready. It is important to consider security implications when developing applications that interact with Dogecoin or other cryptocurrencies. The code has **not been audited** and may contain vulnerabilities or security issues.

If you base your application on this example, we recommend you familiarize yourself with and adhere to the [security best practices](https://internetcomputer.org/docs/current/references/security/) for developing on the Internet Computer. This example may not implement all the best practices.

For example, the following aspects are particularly relevant for this app:

- [Certify query responses if they are relevant for security](https://internetcomputer.org/docs/building-apps/security/data-integrity-and-authenticity#using-certified-variables-for-secure-queries), since the app e.g. offers a method to read balances.
- [Use a decentralized governance system like SNS to make a smart contract have a decentralized controller](https://internetcomputer.org/docs/building-apps/security/decentralization), since decentralized control may be essential for smart contracts holding dogecoins on behalf of users.

---

*Last updated: October 2025*
