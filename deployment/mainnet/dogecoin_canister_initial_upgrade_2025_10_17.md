# Proposal to upgrade the `uploader` canister to the Dogecoin canister

Repository: `https://github.com/dfinity/dogecoin-canister`

Git hash: `454cc3741ef43cfbeae88782ffcf10bbbea00332`

New compressed Wasm hash: `be70d0d15bb32b832ec13dfb3e718d0e856b5f2c5f6bb0cef6333a1fd54daa44`

Upgrade args hash: `a84019ec7750c4692e6f4730af3ccad5e1720a07df4c0edcd58a471162ed94d5`

Target canister: `gordg-fyaaa-aaaan-aaadq-cai`

Previous proposal: https://dashboard.internetcomputer.org/proposal/138938

---

## Motivation

This proposal concludes the launch of the **Dogecoin mainnet canister** (see [initial forum post](https://forum.dfinity.org/t/direct-integration-with-dogecoin/58675/)) by installing the actual Dogecoin canister. It follows from the installation of the `uploader` canister which was used to upload the pre-computed state of the Dogecoin mainnet canister at height **5,906,189**. The Dogecoin canister will then start syncing blocks from height 5,906,189 until the tip.

Note: there is no need to have an additional proposal to enable the API endpoints [as initially described](https://dashboard.internetcomputer.org/proposal/138938). The canister now contains the logic to disable the API if it is not fully synced to the tip (configurable with a parameter `disable_api_if_not_fully_synced`).

## Release Notes

Changes made to the Dogecoin canister since it was forked from the Bitcoin canister:

```
git log --format='%C(auto) %h %s' d4ff6c830996903b9e40d6631b74878c49d5af24.. -- canister
  454cc37 chore(upstream): cherry-pick from dfinity/bitcoin-canister@master (defadc1..46e1a4c) (#32)
  266caa5 fix: use buffered writer during pre-upgrade and set stability threshold to 360 to reduce heap memory pressure (#41)
  da251d5 feat: revert "store auxpow headers into stable memory" (#35)
  064145a feat: ensure `get_block_header` endpoint returns 80-bytes headers (#31)
  52ea627 feat: store auxpow headers into stable memory (#30)
  d21e727 feat: change satoshi to koinu and use nat for get_balance calls (#27)
  d6ebd31 refactor: rename endpoints (#23)
  9fff76d feat: add NetworkAdapter to communicate with Dogecoin adapter (#22)
  f189104 test: increase block range in `canister/src/tests.rs` (#20)
  e3a603c refactor(canister): transition header validation to auxpow validation (#18)
  45dd2c9 feat(validation): add auxpow validation (#14)
  fb7e249 refactor: adapt crates to use dogecoin header validation instead of bitcoin (#9)
  a046700 chore(upstream): cherry-pick from dfinity/bitcoin-canister@master (6bed9af..292b446) (#10)
  5fe5906 feat(ic-doge-validation): add dogecoin header validation (#2)
  d3f3ba6 ci: fix pipeline with new dogecoin canister (#3)
  67fa96e refactor: rename Bitcoin references to Dogecoin
```


## Wasm Verification

Verify that the hash of the gzipped WASM matches the proposed hash.
NOTE: This process is not yet guaranteed to match on Apple Silicon.

```
git fetch
git checkout 454cc3741ef43cfbeae88782ffcf10bbbea00332
./scripts/docker-build ic-doge-canister
sha256sum ic-doge-canister.wasm.gz
```

## Upgrade args

```
git fetch
git checkout 454cc3741ef43cfbeae88782ffcf10bbbea00332
UPGRADE_ARG="(opt record {
    stability_threshold = opt (360 : nat);
    syncing = opt variant { enabled };
    api_access = opt variant { enabled };
    disable_api_if_not_fully_synced = opt variant { enabled };
    burn_cycles = opt variant { enabled };
    lazily_evaluate_fee_percentiles = opt variant { enabled };
})"
didc encode -d candid.did -t '(opt set_config_request)' "$UPGRADE_ARG" | xxd -r -p | sha256sum
```