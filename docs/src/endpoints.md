# Dogecoin API Endpoints

To be able to reach the Dogecoin network, your canister needs to target one of the available endpoints on the Dogecoin canister.

```admonish example title="Dogecoin Canister"
Dogecoin canister principal ID: [gordg-fyaaa-aaaan-aaadq-cai](https://dashboard.internetcomputer.org/canister/gordg-fyaaa-aaaan-aaadq-cai)
```

```admonish question title="Testnet?"
Dogecoin testnet is **not** supported.
```

## Available Endpoints

### `dogecoin_get_utxos`

Returns the UTXOs associated with a Dogecoin address. UTXOs can be filtered by minimum confirmations (only UTXOs with at least the provided number of confirmations are returned, with some upper bound which varies with the current difficulty target) or via a `page` reference when pagination is used for addresses with many UTXOs.

### `dogecoin_get_utxos_query`

Queries `dogecoin_get_utxos` using a [query call](https://internetcomputer.org/docs/building-apps/interact-with-canisters/query-calls). Since this is a query call, it returns quickly but results are not trustworthy.

### `dogecoin_get_balance`

Returns the balance of a Dogecoin address in koinus (1 DOGE = 100,000,000 koinus). Takes an optional argument `min_confirmations` which can be used to limit the set of considered UTXOs for the calculation of the balance to those with at least the provided number of confirmations.

### `dogecoin_get_balance_query`

Queries `dogecoin_get_balance` using a [query call](https://internetcomputer.org/docs/building-apps/interact-with-canisters/query-calls). Since this is a query call, it returns quickly but results are not trustworthy.

### `dogecoin_get_current_fee_percentiles`

Returns fee percentiles (in millikoinus/byte) from the most recent 1,000 Dogecoin transactions.

### `dogecoin_get_block_headers`

Returns raw block headers for a given range of heights. At most 100 block headers are returned per request.

### `dogecoin_send_transaction`

Sends a raw Dogecoin transaction to the specified network (mainnet or regtest).


```admonish info title="Further references"
See the Dogecoin canister [interface specification](https://github.com/dfinity/dogecoin-canister/blob/master/INTERFACE_SPECIFICATION.md) for more details.
```


## Cycles Cost

The costs of API calls in [cycles](https://internetcomputer.org/docs/building-apps/getting-started/tokens-and-cycles) and USD for the Dogecoin Mainnet APIs are presented in the following table. As a general principle for the Dogecoin API, some API calls must have a minimum number of cycles attached to them, as indicated in the column *Minimum cycles to send with call*. Requiring a relatively large minimum number of cycles makes it possible to change the pricing of API calls without breaking existing canisters when the Dogecoin subnet grows in terms of its replication factor in the future. Cycles not consumed by the call are returned to the caller.

The call for submitting a Dogecoin transaction to the Dogecoin network does not require a minimum number of cycles to send with the call as the charged cost is independent of the replication factor of the subnet.

The cost per API call in USD uses the XDR/USD exchange rate of November 25, 2025 (1 XDR = 1.411492 USD).

| API call                               | Description                                                                | Price (Cycles)                          | Price (USD)                          | Minimum cycles to send with call |
|----------------------------------------|----------------------------------------------------------------------------|---------------------------------------|------------------------------------|---------------------------------|
| `dogecoin_get_utxos`                   | Retrieve the UTXO set for a Dogecoin address                               | 50_000_000 + 1 cycle per Wasm instruction | $0.00007058 + Wasm instruction cost | 10_000_000_000                 |
| `dogecoin_get_current_fee_percentiles` | Obtain the fee percentiles of the most recent transactions                 | 10_000_000                            | $0.00001412                       | 100_000_000                    |
| `dogecoin_get_balance`                 | Retrieve the balance of a given Dogecoin address                           | 10_000_000                            | $0.00001412                      | 100_000_000                    |
| `dogecoin_send_transaction`            | Submit a Dogecoin transaction to the Dogecoin network, per transaction     | 5_000_000_000                        | $0.00706                         | N/A                           |
| `dogecoin_send_transaction`            | Submit a Dogecoin transaction to the Dogecoin network, per byte of payload | 20_000_000                           | $0.00002823                      | N/A                           |
| `dogecoin_get_block_headers`           | Retrieve the block headers in specified range                              | 50_000_000 + 1 cycle per Wasm instruction | $0.00007058 + Wasm instruction cost | 10_000_000_000                 |

```admonish note
Fees for calling the `dogecoin_get_utxos` and `dogecoin_get_block_headers` endpoints depend on the number of Wasm instructions that the Dogecoin canister consumes when processing the requests to ensure fair charging.
```
