# Basic Dogecoin

This example demonstrates how to deploy a smart contract on the Internet Computer that can send and receive dogecoins.

## Table of contents

* [Architecture](#architecture)
* [Deploying from ICP Ninja](#deploying-from-icp-ninja)
* [Building and deploying the smart contract locally](#building-and-deploying-the-smart-contract-locally)
  * [1. Prerequisites](#1-prerequisites)
  * [2. Clone the repo](#2-clone-this-repo)
  * [3. Start the ICP execution environment](#3-start-the-icp-execution-environment)
  * [4. Start Dogecoin regtest](#4-start-dogecoin-regtest)
  * [5. Deploy the smart contract](#5-deploy-the-smart-contract)
* [Generating Dogecoin addresses](#generating-dogecoin-addresses)
* [Receiving dogecoin](#receiving-dogecoin)
* [Checking balance](#checking-balance)
* [Sending dogecoin](#sending-dogecoin)
* [Retrieving block headers](#retrieving-block-headers)
* [Notes on implementation](#notes-on-implementation)
* [Security considerations and best practices](#security-considerations-and-best-practices)

## Architecture

This example integrates with the Internet Computer's built-in:

* [ECDSA API](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-ecdsa_public_key)
* [Dogecoin API](https://github.com/dfinity/dogecoin-canister/blob/master/INTERFACE_SPECIFICATION.md)

For background on the Bitcoin integration, which underpins the ICP<>DOGE integration, refer to the [Learn Hub](https://learn.internetcomputer.org/hc/en-us/articles/34211154520084-Bitcoin-Integration).


## Deploying from ICP Ninja

This example can be deployed directly to the Internet Computer using ICP Ninja, where it connects to Dogecoin **mainnet**. Note: Canisters deployed via ICP Ninja remain live for 50 minutes after signing in with your Internet Identity.

[![](https://icp.ninja/assets/open.svg)](https://icp.ninja/editor?g=https://github.com/dfinity/dogecoin-canister/tree/master/examples/basic_dogecoin)

## Building and deploying the smart contract locally

### 1. Prerequisites

* [x] [Rust toolchain](https://www.rust-lang.org/tools/install)
* [x] [Internet Computer SDK](https://internetcomputer.org/docs/building-apps/getting-started/install)
* [x] [Local Dogecoin regtest](https://dfinity.github.io/dogecoin-canister/environment.html#create-a-local-dogecoin-network-regtest-with-dogecoind)
* [x] On macOS, an `llvm` version that supports the `wasm32-unknown-unknown` target is required. The Rust `bitcoin-dogecoin` library relies on the `secp256k1-sys` crate, which requires `llvm` to build. The default `llvm` version provided by XCode does not meet this requirement. Install the [Homebrew version](https://formulae.brew.sh/formula/llvm) using `brew install llvm`.

The IC SDK includes the `dfx` command-line tool, which is used to manage canisters and interact with the Internet Computer network.

Interacting with Dogecoin requires `dfx` version `0.30.1-beta.0` or higher. You can check your installed version by running:

```bash
dfx --version
```

To install and switch to a specific `dfx` version, use:

```bash
dfxvm install <version>
dfxvm default <version>
```

### 2. Clone this repo

```bash
git clone git@github.com:dfinity/dogecoin-canister.git
cd examples/basic_dogecoin
```

### 3. Start the ICP execution environment

Open a terminal window (terminal 1) and run the following:
```bash
dfx start --enable-dogecoin --dogecoin-node 127.0.0.1:18444
```
This starts a local canister execution environment with Dogecoin support enabled.

### 4. Start Dogecoin regtest

Open another terminal window (terminal 2) and run the following to start the local Dogecoin regtest network:

```bash
dogecoind -datadir=$(pwd)/dogecoin_data --port=18444
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

The example demonstrates how to generate and use P2PKH addresses which are the most common type of addresses in Dogecoin.

Use the Candid UI or CLI to generate an address:

```bash
dfx canister call basic_dogecoin get_p2pkh_address
```

## Receiving dogecoin

Use the `dogecoin-cli` to mine a Dogecoin block and send the block reward in the form of local dogecoins to one of the smart contract addresses.
```bash
dogecoin-cli -datadir=$(pwd)/dogecoin_data generatetoaddress 1 <dogecoin_address>
```

## Checking balance

Check the balance of any Dogecoin address:
```bash
dfx canister call basic_dogecoin get_balance '("<dogecoin_address>")'
```

This uses the Dogecoin API endpoint `dogecoin_get_balance` and works for any supported address type. The balance requires at least one confirmation to be reflected.

## Sending dogecoin

You can send dogecoin using the `send_from_p2pkh_address` endpoint. What this does internally:

1. Estimates fees
2. Looks up spendable UTXOs
3. Builds a transaction to the target address
4. Signs the transaction using ECDSA
5. Broadcasts the transaction using the `dogecoin_send_transaction` API endpoint.

Example to send 1 DOGE (100,000,000 koinus) to a target address:

```bash
dfx canister call basic_dogecoin send_from_p2pkh_address '(record { 
  destination_address = "mhXcJVuNA48bZsrKq4t21jx1neSqyceqTM";
  amount_in_koinu = 100000000;
})'
```

> [!IMPORTANT]
> Newly mined dogecoin, like those you created with the above `dogecoin-cli` command, cannot be spent until 60 additional blocks have been added to the chain on regtest. To make your dogecoin spendable, create 60 additional blocks. Choose one of the smart contract addresses as receiver of the block reward or use any valid Dogecoin dummy address.
>
> ```bash
> dogecoin-cli -datadir=$(pwd)/dogecoin_data generatetoaddress 60 <dogecoin_address>
> ```

## Retrieving block headers

You can query historical block headers:

```bash
dfx canister call basic_dogecoin get_block_headers '(10: nat32, null)'
# or a range:
dfx canister call basic_dogecoin get_block_headers '(10: nat32, opt (11: nat32))'
```

This calls the `dogecoin_get_block_headers` API endpoint, which is useful for blockchain validation or light client logic.

## Notes on implementation

This example implements several important patterns for Dogecoin integration:

- **Derivation paths**: Keys are derived using structured derivation paths according to BIP-32, ensuring reproducible key generation.
- **Key caching**: Optimization is used to avoid repeated calls to `get_ecdsa_public_key`.
- **Manual transaction construction**: Transactions are assembled and signed manually, ensuring maximum flexibility in construction and fee estimation.

## Security considerations and best practices

This example is provided for educational purposes and is not production-ready. It is important to consider security implications when developing applications that interact with Dogecoin or other cryptocurrencies. The code has **not been audited** and may contain vulnerabilities or security issues.

If you base your application on this example, we recommend you familiarize yourself with and adhere to the [security best practices](https://internetcomputer.org/docs/current/references/security/) for developing on the Internet Computer. This example may not implement all the best practices.

For example, the following aspects are particularly relevant for this app:

- [Certify query responses if they are relevant for security](https://internetcomputer.org/docs/building-apps/security/data-integrity-and-authenticity#using-certified-variables-for-secure-queries), since the app, for example, offers a method to read balances.
- [Use a decentralized governance system like SNS to make a smart contract have a decentralized controller](https://internetcomputer.org/docs/building-apps/security/decentralization), since decentralized control may be essential for smart contracts holding dogecoins on behalf of users.

---

*Last updated: November 2025*
