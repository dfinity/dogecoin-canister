# Generating a Dogecoin address

Dogecoin, like Bitcoin, doesn't use accounts; instead, it uses a UTXO model. A UTXO is an unspent transaction output.

Each UTXO is associated with a Dogecoin address that is derived from either a public key or a script that defines the conditions under which the UTXO can be spent. A Dogecoin address is often used as a single-use invoice instead of a persistent address to increase privacy.

## Dogecoin P2PKH addresses

Pay to public key hash (P2PKH) addresses are the most common types of addresses in Dogecoin. On mainnet, they start with the prefix `D` and are 34 characters long. They encode the hash of an ECDSA public key.

## Dogecoin P2SH addresses

There is also another type of address that starts with a `A` or `9` called Pay to script hash (P2SH) that encodes the hash of a Dogecoin script. The script can define complex locking conditions such as multisig or timelocks.

## Generating addresses with threshold ECDSA

To generate a Bitcoin address that can only be spent by a specific smart contract or a specific caller of a smart contract, you need to derive the address from the smart contract's public key.

```rust
{{#include ../../../examples/basic_dogecoin/src/service/get_p2pkh_address.rs:9:26}}
```

View the source on GitHub: [get_p2pkh_address.rs](https://github.com/dfinity/dogecoin-canister/blob/master/examples/basic_dogecoin/src/service/get_p2pkh_address.rs)

## Resources

[Learn more about Dogecoin addresses using ECDSA](https://en.bitcoin.it/wiki/Transaction#Pay-to-PubkeyHash).

[Learn more about the `ecdsa_public_key` API](/docs/references/ic-interface-spec#ic-ecdsa_public_key).

