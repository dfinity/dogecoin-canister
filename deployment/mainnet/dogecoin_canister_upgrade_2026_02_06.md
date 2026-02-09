# Proposal to upgrade the Dogecoin canister

Repository: `https://github.com/dfinity/dogecoin-canister.git`

Git hash: `63070ba8c769ca242560ff5ab714e54f860d502f`

New compressed Wasm hash: `5d61d0fbccca7bb7e86b9c914a36c2cacbf4c01dcaee9c4fae04b54bfca4802c`

Upgrade args hash: `d55702edd154306091f1bd4c2e2090f2c42383206d0b939b52738b57e7c1a375`

Target canister: `gordg-fyaaa-aaaan-aaadq-cai`

Previous Dogecoin canister proposal: https://dashboard.internetcomputer.org/proposal/139760

---

## Motivation

The main goal of this proposal is to allow the future installation of the Dogecoin watchdog canister by adding its principal to the Dogecoin canister configuration. The Dogecoin watchdog canister will be responsible for monitoring the latest height of the Dogecoin blockchain and temporary disabling the Dogecoin canister API in case of mismatch.

Additionally, this proposal applies the latest changes from the upstream Bitcoin canister, that is: 
1. adding network validation for addresses in `get_balance` and `get_utxos` requests so that an error is returned to the user in case the address is for a different network (e.g. regtest address for the mainnet canister),
2. in the candid.did file, adding unified canister_arg with init and upgrade variants to ease construction of the init and upgrade arguments of the canister.


## Release Notes

```
git log --format='%C(auto) %h %s' a5f89792a002b060e3f0174f1a87da9764f855f8..63070ba8c769ca242560ff5ab714e54f860d502f -- canister
63070ba chore(ic-doge-canister): release/2026-02-06 (#92)
3713d83 ci: release plz (#74)
8f5ff91 chore(upstream): cherry-pick from dfinity/bitcoin-canister@master (8d51212..38a51b8) (#72)
 ```

## Upgrade args

```
git fetch
git checkout 63070ba8c769ca242560ff5ab714e54f860d502f
didc encode -d canister/candid.did -t '(canister_arg)' '(variant { upgrade = opt record { watchdog_canister = opt opt principal "he6b4-hiaaa-aaaan-aaaeq-cai" } })' | xxd -r -p | sha256sum
```

## Wasm Verification

Verify that the hash of the gzipped WASM matches the proposed hash.

```
git fetch
git checkout 63070ba8c769ca242560ff5ab714e54f860d502f
"./scripts/docker-build" "ic-doge-canister"
sha256sum ./ic-doge-canister.wasm.gz
```