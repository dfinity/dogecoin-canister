# Creating Dogecoin Transactions

Unspent transaction outputs (UTXOs) are used as inputs to build Dogecoin transactions. Every Dogecoin transaction spends one or more UTXOs and in return creates new UTXOs. A UTXO exists until it is used as input for a future transaction. In order to create a Dogecoin transaction, you need to:

1. Get the available UTXOs corresponding to a Dogecoin address controlled by your ICP canister using the `dogecoin_get_utxos` API endpoint.

2. Calculate an appropriate transaction fee using the `dogecoin_get_current_fee_percentiles` API endpoint.

3. Select a subset of the available UTXOs to spend that covers the transaction amount and fee.

4. Create a transaction that spends the selected UTXOs and creates new UTXOs. You will need at least one for the recipient and, in most cases, one to collect the change.

A UTXO has the following structure:

```rust
// Unspent transaction output (UTXO).
pub struct Utxo {
    /// See [Outpoint].
    pub outpoint: Outpoint,
    /// Value in the units of koinu.
    pub value: Koinu,
    /// Height in the blockchain.
    pub height: u32,
}

/// Identifier of [Utxo].
pub struct Outpoint {
    /// Transaction Identifier.
    pub txid: Vec<u8>,
    /// The output index in the transaction.
    pub vout: u32,
}
```

## Get available UTXOs

To get the available UTXOs for a Dogecoin address, use the `dogecoin_get_utxos` API endpoint. The following example demonstrates how to retrieve UTXOs for a given Dogecoin P2PKH address.

```rust
{{#include ../../../examples/basic_dogecoin/src/service/get_utxos.rs:1:19}}
```

```rust
{{#include ../../../examples/basic_dogecoin/src/lib.rs:119:128}}
```

View the source on GitHub: [get_utxo.rs](https://github.com/dfinity/dogecoin-canister/blob/master/examples/basic_dogecoin/src/service/get_utxos.rs)

## Calculate transaction fee per byte

The transaction fee of a Dogecoin transaction is calculated based on the size of the transaction in bytes. An appropriate fee per byte can be determined by looking at the fees of recent transactions on the Dogecoin mainnet. The following snippet shows how to estimate the fee per byte for a transaction using the dogecoin_get_current_fee_percentiles API endpoint and choosing the 50th percentile.

```rust
{{#include ../../../examples/basic_dogecoin/src/service/get_current_fee_percentiles.rs:1:15}}
```

```rust
{{#include ../../../examples/basic_dogecoin/src/lib.rs:135:148}}
```

View the source on GitHub: [get_current_fee_percentiles.rs](https://github.com/dfinity/dogecoin-canister/blob/master/examples/basic_dogecoin/src/service/get_current_fee_percentiles.rs)

## Build the transaction

Now the transaction can be built. Since the fee of a transaction is based on its size, the transaction has to be built iteratively and signed with a mock signer that adds the respective size of the signature. Each selected UTXO is used as an input for the transaction and requires a signature.

The following snippet shows a simplified version of how to build a transaction that will be signed by a P2PKH address:

```rust
{{#include ../../../examples/basic_dogecoin/src/p2pkh.rs:20:70}}
```

View the source on GitHub: [p2pkh.rs](https://github.com/dfinity/dogecoin-canister/blob/master/examples/basic_dogecoin/src/p2pkh.rs)