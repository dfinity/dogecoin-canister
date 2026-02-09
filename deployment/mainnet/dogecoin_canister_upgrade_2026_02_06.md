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

About the upgrade arguments:
* `watchdog_canister`: Set the principal of the watchdog canister to [`he6b4-hiaaa-aaaan-aaaeq-cai`](https://dashboard.internetcomputer.org/canister/he6b4-hiaaa-aaaan-aaaeq-cai), which is a canister controlled by the NNS root ([r7inp-6aaaa-aaaaa-aaabq-cai](https://dashboard.internetcomputer.org/canister/r7inp-6aaaa-aaaaa-aaabq-cai)) on the same subnet ([w4rem](https://dashboard.internetcomputer.org/network/subnets/w4rem-dv5e3-widiz-wbpea-kbttk-mnzfm-tzrc7-svcj3-kbxyb-zamch-hqe)) that will contain the code of the watchdog canister (available in the bitcoin-canister [repository](https://github.com/dfinity/bitcoin-canister/tree/f29c7c21621397ec70ee5018369157850f0e56e0/watchdog)) installed through an upcoming proposal.
## Wasm Verification

Verify that the hash of the gzipped WASM matches the proposed hash.

```
git fetch
git checkout 63070ba8c769ca242560ff5ab714e54f860d502f
"./scripts/docker-build" "ic-doge-canister"
sha256sum ./ic-doge-canister.wasm.gz
```