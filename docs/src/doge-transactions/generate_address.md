# Generating a Dogecoin address

Dogecoin, like Bitcoin, doesn't use accounts; instead, it uses a UTXO model. A UTXO is a Dogecoin transaction output that is unspent.

Each UTXO is associated with a Dogecoin address that is derived from either a public key or a script that defines the conditions under which the UTXO can be spent. A Dogecoin address is often used as a single-use invoice instead of a persistent address to increase privacy.

### Dogecoin P2PKH addresses

Pay-to-public-key-hash (P2PKH) addresses are the most common types of addresses in Dogecoin. On mainnet, they start with the prefix `D`. They encode the hash of an ECDSA public key.

### Dogecoin P2SH addresses

Another type of address is pay-to-script-hash (P2SH) address. It encodes the hash of a Dogecoin script and starts with a `A` or `9` on mainnet. The script can define complex locking conditions such as multisig or timelocks.

## Generating addresses with threshold ECDSA

To generate a Dogecoin address, you need to generate an ECDSA public key. An ECDSA public key can be retrieved using the [`ecdsa_public_key`](https://internetcomputer.org/docs/references/ic-interface-spec#ic-ecdsa_public_key) system API endpoint. The [basic Dogecoin example](https://github.com/dfinity/dogecoin-canister/tree/master/examples/basic_dogecoin) demonstrates how to generate a P2PKH address from a public key.

```rust
{{#include ../../../examples/basic_dogecoin/src/service/get_p2pkh_address.rs:9:26}}
```

```rust
{{#include ../../../examples/basic_dogecoin/src/ecdsa.rs:16:46}}
```

View the source on GitHub: [get_p2pkh_address.rs](https://github.com/dfinity/dogecoin-canister/blob/master/examples/basic_dogecoin/src/service/get_p2pkh_address.rs)

## Resources

[Learn more about Dogecoin P2PKH addresses](https://en.bitcoin.it/wiki/Transaction#Pay-to-PubkeyHash).

[Learn more about the `ecdsa_public_key` API](https://internetcomputer.org/docs/references/ic-interface-spec#ic-ecdsa_public_key).

