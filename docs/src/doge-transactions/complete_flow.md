# Complete flow

The following snippet shows the full process, from generating a transaction to submitting it to the Dogecoin network:
```rust
{{#include ../../../examples/basic_dogecoin/src/service/send_from_p2pkh_address.rs}}
```

*View the source on GitHub: [send_from_p2pkh_address.rs](https://github.com/dfinity/dogecoin-canister/blob/master/examples/basic_dogecoin/src/service/send_from_p2pkh_address.rs)*

```rust
{{#include ../../../examples/basic_dogecoin/src/lib.rs:216:222}}
```

*View the source on GitHub: [lib.rs](https://github.com/dfinity/dogecoin-canister/blob/master/examples/basic_dogecoin/src/lib.rs#L216)*

To submit transactions to the Dogecoin network, the Dogecoin API exposes the `dogecoin_send_transaction` method.

```rust
{{#include ../../../examples/basic_dogecoin/src/lib.rs:155:167}}
```

*View the source on GitHub: [lib.rs](https://github.com/dfinity/dogecoin-canister/blob/c947b5c7be61c1b860a8a4cdf5fdd6a5054c61b3/examples/basic_dogecoin/src/lib.rs#L155)*

