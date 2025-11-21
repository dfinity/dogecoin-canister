# Complete flow

To submit transactions to the Dogecoin network, the Dogecoin API exposes the `dogecoin_send_transaction` method.

The following snippet shows the complete flow for generating a transaction and submitting it to the Dogecoin network:
```rust
{{#include ../../../examples/basic_dogecoin/src/service/send_from_p2pkh_address.rs}}
```

```rust
{{#include ../../../examples/basic_dogecoin/src/lib.rs:216:222}}
```

```rust
{{#include ../../../examples/basic_dogecoin/src/lib.rs:155:167}}
```

View the source on GitHub: [send_from_p2pkh_address.rs](https://github.com/dfinity/dogecoin-canister/blob/master/examples/basic_dogecoin/src/service/send_from_p2pkh_address.rs)