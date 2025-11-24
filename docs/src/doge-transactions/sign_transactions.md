# Signing Transactions

Before a transaction can be sent to the Dogecoin network, each input must be signed. Canisters can sign transactions with threshold ECDSA through the [`sign_with_ecdsa`](https://internetcomputer.org/docs/references/ic-interface-spec#ic-sign_with_ecdsa) system API endpoint.

## Threshold ECDSA

The following snippet shows a simplified example of how to sign a Dogecoin transaction where all UTXOs are owned by `own_address` and `own_address` is a P2PKH address.

```rust
{{#include ../../../examples/basic_dogecoin/src/p2pkh.rs:79:129}}
```

*View the source on GitHub: [p2pkh.rs](https://github.com/dfinity/dogecoin-canister/blob/master/examples/basic_dogecoin/src/p2pkh.rs#L79)*

The signature function `signer: SignFun` passed as argument is shown below. This function makes a call to the `sign_with_ecdsa` system API endpoint to sign the provided transaction sighash.

```rust
{{#include ../../../examples/basic_dogecoin/src/ecdsa.rs:48:69}}
```

*View the source on GitHub: [ecdsa.rs](https://github.com/dfinity/dogecoin-canister/blob/master/examples/basic_dogecoin/src/ecdsa.rs#L51)*

## Resources

- [Learn more about the threshold ECDSA](https://internetcomputer.org/docs/building-apps/network-features/signatures/t-ecdsa).