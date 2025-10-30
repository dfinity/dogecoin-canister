# Introduction

```admonish warning title="Work In Progress"
🚧 The developer documentation is under construction. 
```

The *Build on Dogecoin* book is intended for developers to explain how smart contracts on the [Internet Computer](https://internetcomputer.org), often referred as [canisters](https://learn.internetcomputer.org/hc/en-us/articles/34210839162004-Canister-Smart-Contracts), can interact with the [Dogecoin](https://dogecoin.com/) blockchain.

## Background

To interact with the Dogecoin blockchain, your canister will make use of the following:

- **[Dogecoin canister](https://github.com/dfinity/dogecoin-canister)**: Think of it as your decentralized gateway to reach the Dogecoin blockchain. This canister provides an API that can be used by others to query information about the network state, e.g., UTXOs, block information, or the balance of any Dogecoin address; and to send a signed transaction to the network.

- **[Threshold ECDSA](https://internetcomputer.org/docs/building-apps/network-features/signatures/t-ecdsa)**: A canister can have a secret key that is stored in a secure and decentralized manner using chain-key cryptography (several such keys can be computed by key derivation). Messages sent by the canister can be signed using this key, enabling the canister to send signed transactions to Dogecoin.


## Getting Started

First, set up your [development environment](./environment.md). Then, to build smart contracts interacting with the Dogecoin blockchain, you will need to know how to

- [Generate a Dogecoin address](./doge-transactions/generate_address.md). Dogecoin addresses are necessary for your dapp to sign transactions and hold assets like DOGE. An ICP smart contract can have multiple addresses.

- [Create a Dogecoin transaction](./doge-transactions/create_transactions.md). Dogecoin transactions spend unspent transaction outputs (UTXOs) and create new UTXOs. A UTXO is the output of a Dogecoin transaction. It exists until it is used as the input of another transaction.

- [Sign the transaction](./doge-transactions/sign_transactions.md) using one of the supported [threshold signature](https://internetcomputer.org/docs/references/t-sigs-how-it-works) APIs. All inputs of a transaction must be signed before the transaction can be submitted to the Dogecoin network.

- [Submit the transaction](./doge-transactions/submit_transactions.md) by sending a request to the Dogecoin API that specifies the `blob` of the transaction and the target Dogecoin network (mainnet or testnet4).

- [Read information from the Dogecoin network](./read.md), such as transaction details or address balances.

