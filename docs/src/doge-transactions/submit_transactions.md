# Submitting Transactions

To submit transactions to the Dogecoin network, the Dogecoin API exposes the `dogecoin_send_transaction` method.

The following snippet shows how to send a signed transaction to the Dogecoin network:
```rust
{{#include ../../../examples/basic_dogecoin/src/service/send_from_p2pkh_address.rs:81:91}}
```

View the source on GitHub: [send_from_p2pkh_address.rs](https://github.com/dfinity/dogecoin-canister/blob/master/examples/basic_dogecoin/src/service/send_from_p2pkh_address.rs)