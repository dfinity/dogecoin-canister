# Proposal to upgrade the Dogecoin canister

Repository: `https://github.com/dfinity/dogecoin-canister`

Git hash: `e7c23733075c48037ac74d974ecdcb56bac9d1d3`

New compressed Wasm hash: `0711bc8316c8f845596fb1a74a7e61e4151df5858e9e0fdde5afe03241cd4254`

Upgrade args hash: `a84019ec7750c4692e6f4730af3ccad5e1720a07df4c0edcd58a471162ed94d5`

Target canister: `gordg-fyaaa-aaaan-aaadq-cai`

Previous proposal: https://dashboard.internetcomputer.org/proposal/138995

---

## Motivation

The [previous proposal](https://dashboard.internetcomputer.org/proposal/138995) to install the Dogecoin canister was accepted but due to a bug related to the deserialization of the canister state, the installation of the WASM failed.

This proposal contains the fix to ensure correct deserialization of the state that was uploaded using the `uploader` canister (see [initial forum post](https://forum.dfinity.org/t/direct-integration-with-dogecoin/58675/)). After the Dogecoin canister starts, it will sync blocks from height 5,906,189 up to the tip, after which its API will become accessible.

## Release Notes

```
git log --format='%C(auto) %h %s' d4ff6c830996903b9e40d6631b74878c49d5af24..e7c23733075c48037ac74d974ecdcb56bac9d1d3 -- canister
 e7c2373 fix: backward-compatible state deserialization (#44)
```


## Wasm Verification

Verify that the hash of the gzipped WASM matches the proposed hash.
NOTE: This process is not yet guaranteed to match on Apple Silicon.

```
git fetch
git checkout e7c23733075c48037ac74d974ecdcb56bac9d1d3
./scripts/docker-build ic-doge-canister
sha256sum ic-doge-canister.wasm.gz
```

## Upgrade args

```
git fetch
git checkout e7c23733075c48037ac74d974ecdcb56bac9d1d3
UPGRADE_ARG="(opt record {
    stability_threshold = opt (360 : nat);
    syncing = opt variant { enabled };
    api_access = opt variant { enabled };
    disable_api_if_not_fully_synced = opt variant { enabled };
    burn_cycles = opt variant { enabled };
    lazily_evaluate_fee_percentiles = opt variant { enabled };
})"
didc encode -d canister/candid.did -t '(opt set_config_request)' "$UPGRADE_ARG" | xxd -r -p | sha256sum
```

* `stability_threshold`: set to 360, which corresponds to 6 hours of blocks produced on the Dogecoin network (on average). This number corresponds to roughly the number of full blocks stored in heap memory. It was chosen to keep the heap memory usage below the available limits. It will be increased in the future once blocks are stored in stable memory.
* `syncing`: set to `enabled` to enable the canister to sync blocks.
* `disable_api_if_not_fully_synced`: set to `enabled` to disable the API if the canister is not fully synced to the tip.
* `api_access`: set to `enabled` to enable the API (once the canister is synced to the tip).
* `burn_cycles`: set to `enabled` to burn received cycles.
* `lazily_evaluate_fee_percentiles`: set to `enabled` to indicate that fee percentiles are only evaluated when fees are requested, rather than updating them automatically whenever a newly received block is processed.