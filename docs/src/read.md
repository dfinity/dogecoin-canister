# Reading the Dogecoin State

Canisters can query information about the Dogecoin mainnet programmatically.

## Reading unspent transaction outputs (UTXOs)

To read unspent transaction outputs (UTXOs) associated with an address from the Dogecoin network, make a call to the `dogecoin_get_utxos` Dogecoin API method.

```rust
{{#include ../../examples/basic_dogecoin/backend/src/service/get_utxos.rs}}
```

*View the source on GitHub: [get_utxos.rs](https://github.com/dfinity/dogecoin-canister/blob/master/examples/basic_dogecoin/backend/src/service/get_utxos.rs)*

```rust
{{#include ../../examples/basic_dogecoin/backend/src/lib.rs:129:143}}
```

*View the source on GitHub: [lib.rs](https://github.com/dfinity/dogecoin-canister/blob/master/examples/basic_dogecoin/backend/src/lib.rs#L129)*

## Reading current balance

To read the current balance of a Dogecoin address, make a call to the `dogecoin_get_balance` Dogecoin API method.

```rust
{{#include ../../examples/basic_dogecoin/backend/src/service/get_balance.rs}}
```

*View the source on GitHub: [get_balance.rs](https://github.com/dfinity/dogecoin-canister/blob/master/examples/basic_dogecoin/backend/src/service/get_balance.rs)*

```rust
{{#include ../../examples/basic_dogecoin/backend/src/lib.rs:184:198}}
```

*View the source on GitHub: [lib.rs](https://github.com/dfinity/dogecoin-canister/blob/master/examples/basic_dogecoin/backend/src/lib.rs#L184)*

## Reading the fee percentiles

The transaction fees on the Dogecoin network change dynamically based on the number of pending transactions. In order to get fee percentiles of the last 1,000 transactions, call the `dogecoin_get_current_fee_percentiles` Dogecoin API method.

This endpoint returns 101 numbers that are fees measured in millikoinus (1,000 millikoinus = 1 koinu; 100,000,000 koinus = 1 DOGE) per byte. The ith element of the result corresponds to the ith percentile fee. For example, to get the median fee over the last few blocks, look at the 50th element of the result.

```rust
{{#include ../../examples/basic_dogecoin/backend/src/service/get_current_fee_percentiles.rs}}
```

*View the source on GitHub: [get_current_fee_percentiles.rs](https://github.com/dfinity/dogecoin-canister/blob/master/examples/basic_dogecoin/backend/src/service/get_current_fee_percentiles.rs)*

```rust
{{#include ../../examples/basic_dogecoin/backend/src/lib.rs:145:163}}
```

*View the source on GitHub: [lib.rs](https://github.com/dfinity/dogecoin-canister/blob/master/examples/basic_dogecoin/backend/src/lib.rs#L145)*

## Reading the block headers

To read the block headers within a provided range of start and end heights, make a call to the `dogecoin_get_block_headers` Dogecoin API method. Note that at most 100 block headers are returned per request.

```rust
{{#include ../../examples/basic_dogecoin/backend/src/service/get_block_headers.rs}}
```

*View the source on GitHub: [get_block_headers.rs](https://github.com/dfinity/dogecoin-canister/blob/master/examples/basic_dogecoin/backend/src/service/get_block_headers.rs)*

```rust
{{#include ../../examples/basic_dogecoin/backend/src/lib.rs:200:218}}
```

*View the source on GitHub: [lib.rs](https://github.com/dfinity/dogecoin-canister/blob/master/examples/basic_dogecoin/backend/src/lib.rs#L200)*
