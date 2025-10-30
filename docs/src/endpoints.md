# Dogecoin API Endpoints

To be able to reach the Dogecoin network, your smart contract needs to target one of the available endpoints on the Dogecoin canister.

```admonish note title="Dogecoin Canister"
* Mainnet: [gordg-fyaaa-aaaan-aaadq-cai](https://dashboard.internetcomputer.org/canister/gordg-fyaaa-aaaan-aaadq-cai)

```

```admonish question title="Testnet?"
Dogecoin testnet is **not** supported.
```

## Available Endpoints

See the Dogecoin canister [interface specification](https://github.com/dfinity/dogecoin-canister/blob/master/INTERFACE_SPECIFICATION.md) for more details.

### `dogecoin_get_utxos`

Returns UTXOs for a Dogecoin address. UTXOs can be filtered by minimum confirmations (`min_confirmations`, which may be at most 144) or via a `page` reference.

### `dogecoin_get_utxos_query`

Used to query `dogecoin_get_utxos`. Since this is a query call, it returns quickly but results are not trustworthy.

### `dogecoin_get_balance`

Returns the balance of a Dogecoin address in koinus. Takes an optional argument of `min_confirmations`.

### `dogecoin_get_balance_query`

Used to query `dogecoin_get_balance`. Since this is a query call, it returns quickly but results are not trustworthy.

### `dogecoin_get_current_fee_percentiles`

Returns fee percentiles (in millikoinus/byte) from the most recent 10_000 Dogecoin transactions.

### `dogecoin_get_block_headers`

Returns raw block headers for a given range of heights.

### `dogecoin_send_transaction`

Sends a raw Dogecoin transaction to the network after validation.


## Cycles Cost

The costs of API calls in cycles and USD for the Dogecoin Mainnet APIs are presented in the following tables. As a general principle for the Dogecoin API, some API calls must have a minimum number of cycles attached to them, as indicated in the column Minimum cycles to send with call. Cycles not consumed by the call are returned to the caller. Requiring a relatively large minimum number of cycles makes it possible to change the pricing of API calls without breaking existing smart contracts when the Dogecoin subnet grows in terms of its replication factor in the future. The call for submitting a Dogecoin transaction to the Dogecoin network does not require extra cycles to be attached as the charged cost is independent of the replication factor of the subnet.

The cost per API call in USD uses the USD/XDR exchange rate of May 22, 2025 ($1.354820 USD).

### Dogecoin Mainnet

| Transaction                         | Description                                                                                   | Price (Cycles)                          | Price (USD)                          | Minimum cycles to send with call |
|-----------------------------------|-----------------------------------------------------------------------------------------------|---------------------------------------|------------------------------------|---------------------------------|
| Dogecoin UTXO set for an address    | For retrieving the UTXO set for a Dogecoin address (`dogecoin_get_utxos`)                        | 50_000_000 + 1 cycle per Wasm instruction | $0.00006774 + Wasm instruction cost | 10_000_000_000                 |
| Dogecoin fee percentiles            | For obtaining the fee percentiles of the most recent transactions (`dogecoin_get_current_fee_percentiles`) | 10_000_000                            | $0.00001355                       | 100_000_000                    |
| Dogecoin balance for an address     | For retrieving the balance of a given Dogecoin address (`dogecoin_get_balance`)                  | 10_000_000                            | $0.00001355                       | 100_000_000                    |
| Dogecoin transaction submission     | For submitting a Dogecoin transaction to the Dogecoin network, per transaction (`dogecoin_send_transaction`) | 5_000_000_000                        | $0.00677                         | N/A                           |
| Dogecoin transaction payload        | For submitting a Dogecoin transaction to the Dogecoin network, per byte of payload (`dogecoin_send_transaction`) | 20_000_000                           | $0.00002710                      | N/A                           |
| Dogecoin block headers              | For retrieving the block headers in specified range (`dogecoin_get_block_headers`)              | 50_000_000 + 1 cycle per Wasm instruction | $0.00006774 + Wasm instruction cost | 10_000_000_000                 |

```admonish note
Fees for calling the `dogecoin_get_utxos` and `dogecoin_get_block_headers` endpoints depend on the number of Wasm instructions that the Dogecoin canister consumes when processing the requests to ensure fair charging.
```
