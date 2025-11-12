# Basic Dogecoin

This example demonstrates how to deploy a smart contract on the Internet Computer that can send and receive dogecoins.

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

> [!IMPORTANT]
> This feature is not supported yet.


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
