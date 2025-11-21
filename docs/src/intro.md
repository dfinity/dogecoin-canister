<div align="center">
  <img alt="Interact with Dogecoin from a smart contract on the Internet Computer"
  src="images/Doge-cover.jpg"
  width="100%">
</div>

# Introduction

```admonish warning title="Work In Progress"
🚧 The developer documentation is under construction. 
```

The *Build on Dogecoin* book is intended for developers to explain how smart contracts on the [Internet Computer](https://internetcomputer.org), often referred as [canisters](https://learn.internetcomputer.org/hc/en-us/articles/34210839162004-Canister-Smart-Contracts), can interact with the [Dogecoin](https://dogecoin.com/) blockchain.

## Background

Through a protocol-level integration with the Dogecoin network, canisters deployed on ICP can interact with the Dogecoin network directly without using a bridge or oracle.

To interact with the Dogecoin blockchain, your canister will make use of the following:

- **[Dogecoin canister](https://github.com/dfinity/dogecoin-canister)**: Think of it as your decentralized gateway to reach the Dogecoin blockchain. This canister provides an API that can be used by others to query information about the network state, e.g., UTXOs, block header information, or the balance of any Dogecoin address; and to send signed transactions to the network.

- **[Threshold ECDSA](https://internetcomputer.org/docs/building-apps/network-features/signatures/t-ecdsa)**: Your canister can have a secret key that is stored in a secure and decentralized manner using chain-key cryptography (several such keys can be computed by key derivation). Messages sent by the canister can be signed using this key, enabling your canister to [send signed transactions](./doge-transactions/submit_transactions.md) to the Dogecoin network through the Dogecoin canister.

To submit a Dogecoin transaction from a canister, the following steps are typically performed:

- Request a public key from the threshold ECDSA API
- Derive a Dogecoin address from the public key
- Read UTXOs from the Dogecoin API
- Build the transaction payload
- Sign the transaction using the threshold ECDSA API
- Submit the transaction to the Dogecoin API

The Dogecoin canister relays the request to the Dogecoin network, which receives and processes the request asynchronously.

## Getting Started

First, set up your [development environment](./environment.md). Then, to build canisters interacting with the Dogecoin blockchain, you will need to know how to

- [Generate a Dogecoin address](./doge-transactions/generate_address.md). Dogecoin addresses are necessary for your canister to sign transactions and hold DOGE. A canister can have multiple addresses.

- [Create a Dogecoin transaction](./doge-transactions/create_transactions.md). Dogecoin transactions spend unspent transaction outputs (UTXOs) and create new UTXOs. A UTXO is the output of a Dogecoin transaction. It exists until it is used as the input of another transaction.

- [Sign the transaction](./doge-transactions/sign_transactions.md) using [threshold ECDSA API](https://internetcomputer.org/docs/building-apps/network-features/signatures/t-ecdsa). All inputs of a transaction must be signed before the transaction can be submitted to the Dogecoin network.

- [Submit the transaction](./doge-transactions/complete_flow.md) by sending a request to the Dogecoin API that specifies the blob of the transaction and the target Dogecoin network (mainnet or regtest).

- [Read information from the Dogecoin network](./read.md), such as UTXOs, address balances, or block headers.

## Additional resources

Building Dogecoin applications is not trivial. It’s beneficial to understand core Bitcoin concepts which underpins Dogecoin's, including transactions, UTXOs, the Script language, and hash formats.

- [Mastering Bitcoin: Programming the open blockchain](https://github.com/bitcoinbook/bitcoinbook/blob/develop/BOOK.md)
- [Learn me a Bitcoin](https://learnmeabitcoin.com)
- [Bitcoin wiki](https://en.bitcoin.it/wiki/Main_Page)
