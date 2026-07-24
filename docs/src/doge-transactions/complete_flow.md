# Complete flow

The following snippet shows the full process, from generating a transaction to submitting it to the Dogecoin network:
```rust
{{#include ../../../examples/basic_dogecoin/backend/src/service/send_from_p2pkh_address.rs}}
```

*View the source on GitHub: [send_from_p2pkh_address.rs](https://github.com/dfinity/dogecoin-canister/blob/master/examples/basic_dogecoin/backend/src/service/send_from_p2pkh_address.rs)*

```rust
{{#include ../../../examples/basic_dogecoin/backend/src/lib.rs:231:237}}
```

*View the source on GitHub: [lib.rs](https://github.com/dfinity/dogecoin-canister/blob/master/examples/basic_dogecoin/backend/src/lib.rs#L231)*

To submit transactions to the Dogecoin network, the Dogecoin API exposes the `dogecoin_send_transaction` method.

```rust
{{#include ../../../examples/basic_dogecoin/backend/src/lib.rs:170:182}}
```

*View the source on GitHub: [lib.rs](https://github.com/dfinity/dogecoin-canister/blob/master/examples/basic_dogecoin/backend/src/lib.rs#L170)*
