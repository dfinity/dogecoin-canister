# Proposal to upgrade the Dogecoin canister

Repository: `https://github.com/dfinity/dogecoin-canister.git`

Git hash: `9fb18ddc9899fdce00c87555e53d1bc19c75bab7`

New compressed Wasm hash: `1fe3ce73e9422a0a3e0444ea7b5a615d2b36c4b940d3cbcfe9e3d5c6cdc32494`

Upgrade args hash: `eaf2fb44952dfc89c3d9c1629eec9fe924722c132f33caef17cf0f5789b449b8`

Target canister: `gordg-fyaaa-aaaan-aaadq-cai`

Previous Dogecoin canister proposal: https://dashboard.internetcomputer.org/proposal/140285

---

## Motivation

This proposal applies the latest changes from the upstream Bitcoin canister, that is:

1. adds the `get_blockchain_info` endpoint to the Dogecoin canister. This endpoint allows callers to query the canister for blockchain information, which includes the latest height, the hash, the timestamp and the difficulty of the latest block. This endpoint can be called even if the API of the canister is disabled or if the canister is not in sync with the adapter. Later, the watchdog canister will use this endpoint to check the health of the Dogecoin canister instead of getting the latest height from the /metrics endpoint through HTTPs outcalls.

2. changes the main chain selection rule so that the chain with the greatest accumulated proof-of-work is considered the main chain. The canister's main chain selection used by the `dogecoin_get_balance`, `dogecoin_get_utxos`, `dogecoin_get_current_fee_percentiles`, and `dogecoin_get_block_headers` endpoints previously relied on the longest chain by block count. This does not match Dogecoin's consensus rule, which defines the main chain as the one with the most accumulated proof-of-work. In practice, on the Dogecoin mainnet, difficulty adjustments are gradual and bounded, so the chain with the most work is also typically the longest. For correctness and consistency with Dogecoin, this upgrade adds the greatest accumulated proof-of-work in the main chain selection. Note that this upgrade does not affect when blocks are considered stable as block stability already relies on the accumulated proof-of-work.

3. adds `burn_cycles` field to `SetConfigRequest` in order to be able to set the corresponding configuration flag during upgrades and when calling the `set_config` endpoint.

## Release Notes

```
git log --format='%C(auto) %h %s' 63070ba8c769ca242560ff5ab714e54f860d502f..9fb18ddc9899fdce00c87555e53d1bc19c75bab7 -- canister
9fb18dd chore(ic-doge-canister): release/2026-03-12 (#101)
c2a7e36 chore(upstream): cherry-pick from dfinity/bitcoin-canister@master (ff779883..c3c089f) (#100)
ead2a82 chore(upstream): cherry-pick from dfinity/bitcoin-canister@master (9dcf7e6..d9ab390) (#97)
805a9f9 chore(upstream): cherry-pick from dfinity/bitcoin-canister@master (7f84397) (#95)
 ```

## Upgrade args

```
git fetch
git checkout 9fb18ddc9899fdce00c87555e53d1bc19c75bab7
didc encode -d canister/candid.did -t '(canister_arg)' '(variant { upgrade })' | xxd -r -p | sha256sum
```

## Wasm Verification

Verify that the hash of the gzipped WASM matches the proposed hash.

```
git fetch
git checkout 9fb18ddc9899fdce00c87555e53d1bc19c75bab7
"./scripts/docker-build" "ic-doge-canister"
sha256sum ./ic-doge-canister.wasm.gz
```