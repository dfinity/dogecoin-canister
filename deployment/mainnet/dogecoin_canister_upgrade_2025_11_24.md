# Proposal to upgrade the Dogecoin canister

Repository: `https://github.com/dfinity/dogecoin-canister`

Git hash: `c947b5c7be61c1b860a8a4cdf5fdd6a5054c61b3`

New compressed Wasm hash: `3a0738bf4942c8e93a48e4b1547661fafd486c3ab0e0238c1443f6a12f53ad9d`

Upgrade args hash: `e5bd212e76a439b3120041daeca2872ab6b17f64594ab91c6f5d9e5326721093`

Target canister: `gordg-fyaaa-aaaan-aaadq-cai`

Previous proposal: https://dashboard.internetcomputer.org/proposal/139080

---

## Motivation

This proposal contains a new feature that stores unstable blocks in stable memory, to mitigate the risk of running out of heap memory.
It also raises stability threashold from 360 to 720.


## Release Notes

```
git log --format='%C(auto) %h %s' e7c23733075c48037ac74d974ecdcb56bac9d1d3..c947b5c7be61c1b860a8a4cdf5fdd6a5054c61b3 -- canister
 c947b5c feat: cache unstable blocks in stable memory (#54)
 48e320e chore(upstream): cherry-pick from dfinity/bitcoin-canister@master 02af290 (#53)
 be3e481 chore(upstream): cherry-pick from dfinity/bitcoin-canister@master (af000cc..f202301) (#51)
 023e194 chore(upstream): cherry-pick from dfinity/bitcoin-canister@master (b506535) (#50)
```


## Wasm Verification

Verify that the hash of the gzipped WASM matches the proposed hash.
NOTE: This process is not yet guaranteed to match on Apple Silicon.

```
git fetch
git checkout c947b5c7be61c1b860a8a4cdf5fdd6a5054c61b3
./scripts/docker-build ic-doge-canister
sha256sum ic-doge-canister.wasm.gz
```

## Upgrade args

```
git fetch
git checkout c947b5c7be61c1b860a8a4cdf5fdd6a5054c61b3
UPGRADE_ARG="(opt record {
    stability_threshold = opt (720 : nat);
})"
didc encode -d canister/candid.did -t '(opt set_config_request)' "$UPGRADE_ARG" | xxd -r -p | sha256sum
```

* `stability_threshold`: set to 720, which corresponds to 12 hours of blocks produced on the Dogecoin network (on average). This number was lowed to 360 in the previous upgrade due to concerns of running out of heap memory in extreme situations, which is now addressed by the fix contained in this proposal.
