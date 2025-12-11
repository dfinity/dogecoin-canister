# Proposal to upgrade the Dogecoin canister

Repository: `https://github.com/dfinity/dogecoin-canister`

Git hash: `a5f89792a002b060e3f0174f1a87da9764f855f8`

New compressed Wasm hash: `2efd5a5b28315a3d4896419df14a4aad8a346174f53cc7727a63b37ec086b3ac`

Upgrade args hash: `ddc650475c15fe881c3e014b70aa908a405c2d4500ceabd9df09db3c2398c820`

Target canister: `gordg-fyaaa-aaaan-aaadq-cai`

Previous proposal: https://dashboard.internetcomputer.org/proposal/139499

---

## Motivation

This proposal contains several upstream cherry-picks to upgrade the `ic-cdk` version used to `v0.19.0` and add a tool to verify the endpoints the canister’s WASM exports against its Candid interface.

Additionally, it changes the number of transactions used in the fee percentiles calculation from 10,000 to 1,000, to make sure the same timespan of transaction is considered in the calculation as the Bitcoin canister (approximately 40 minutes).

This proposal also removes testnet from the interface of the canister as the Dogecoin testnet network is not supported.

Finally, it raises the stability threshold from 720 to 1,440, corresponding to an average of one day worth of blocks, for additional security.

## Release Notes

```
git log --format='%C(auto) %h %s' c947b5c7be61c1b860a8a4cdf5fdd6a5054c61b3..a5f89792a002b060e3f0174f1a87da9764f855f8 -- canister
 6c4fe71 chore(upstream): cherry-pick from dfinity/bitcoin-canister@master (13c6ff2) (#62)
 875421f refactor!: remove testnet (#61)
 0e72fe9 feat: use 1,000 transactions in the percentiles calculation (#60)
 3bb69b2 chore(upstream): cherry-pick from dfinity/bitcoin-canister@master (91b1c67) (#59)
 ```

## Wasm Verification

Verify that the hash of the gzipped WASM matches the proposed hash.
NOTE: This process is not yet guaranteed to match on Apple Silicon.

```
git fetch
git checkout a5f89792a002b060e3f0174f1a87da9764f855f8
./scripts/docker-build ic-doge-canister
sha256sum ic-doge-canister.wasm.gz
```

## Upgrade args

```
git fetch
git checkout a5f89792a002b060e3f0174f1a87da9764f855f8
UPGRADE_ARG="(opt record {
    stability_threshold = opt (1440 : nat);
})"
didc encode -d canister/candid.did -t '(opt set_config_request)' "$UPGRADE_ARG" | xxd -r -p | sha256sum
```

* `stability_threshold`: set to 1,440, which corresponds to 24 hours of blocks produced on the Dogecoin network (on average).
