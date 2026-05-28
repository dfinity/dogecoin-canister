# Proposal to upgrade the Dogecoin canister

Repository: `https://github.com/dfinity/dogecoin-canister.git`

Git hash: `ca66a0fdceccc56f8208d25c76ba4fc31b4b9cd6`

New compressed Wasm hash: `1921396143c687dee142d3b8bbcc7b17f3684158a6e1d794e4491fdf7058e1ba`

Upgrade args hash: `eaf2fb44952dfc89c3d9c1629eec9fe924722c132f33caef17cf0f5789b449b8`

Target canister: `gordg-fyaaa-aaaan-aaadq-cai`

Previous Dogecoin proposal: https://dashboard.internetcomputer.org/proposal/140852

---

## Motivation

This proposal applies the latest changes from the upstream Bitcoin canister, that is:

1. changes the main chain selection algorithm so that the reported main chain no longer shortens when two competing blocks appear at the same height with equal accumulated difficulty and equal depth (a contested tip). Previously, the algorithm truncated to the fork point in this case, decreasing the reported main chain height by one and causing `dogecoin_get_utxos` and `dogecoin_get_balance` to temporarily exclude transactions from both competing blocks. With this change, ties on `(accumulated_difficulty, depth)` are broken by keeping the first-received child, matching Bitcoin Core's behavior of staying on whichever chain the canister was already following. The tiebreaker only applies while branches are exactly equal — as soon as one branch pulls ahead on either difficulty or depth, it wins outright.

2. improves performance of the `get_blockchain_info` endpoint by caching the per-block net UTXO count delta, avoiding the need to read all transactions in unstable blocks on every call. On the first upgrade, the cached delta is empty and the reported UTXO count is inaccurate; this self-heals as the ~1440 unstable blocks are ingested.

3. bumps the `ic-cdk` dependency to v0.20.0.

In addition, this proposal includes a Dogecoin-specific performance improvement to the `dogecoin_get_utxos` endpoint, which avoids deserializing the whole block when looking up UTXOs in unstable blocks.


## Release Notes

```
git log --format='%C(auto) %h %s' 9fb18ddc9899fdce00c87555e53d1bc19c75bab7..ca66a0fdceccc56f8208d25c76ba4fc31b4b9cd6 -- canister
ca66a0f chore(ic-doge-canister): release/2026-05-27 (#119)
fc816b0 chore(upstream): DEFI-2834: cherry-pick from dfinity/bitcoin-canister@master (707ef54) (#118)
b9f207a chore(upstream): DEFI-2834: cherry-pick from dfinity/bitcoin-canister@master (e354549) (#117)
49458af chore(upstream): cherry-pick from dfinity/bitcoin-canister@master (949c2aa) (#112)
70c22e7 perf: do not deserialize whole block in `get_utxos` call (#110)
fc894e9 perf: add benchmarks for `dogecoin_get_*` endpoints (#109)
131f258 refactor: de-generify `BlockTree<Block>` (#107)
faf239b chore(upstream): cherry-pick from dfinity/bitcoin-canister@master (09d7313) (#108)
9f59b7f refactor: de-generify `GenericState` and `GenericUnstableBlocks` (#106)
 ```

## Upgrade args

```
git fetch
git checkout ca66a0fdceccc56f8208d25c76ba4fc31b4b9cd6
didc encode -d canister/candid.did -t '(canister_arg)' '(variant { upgrade })' | xxd -r -p | sha256sum
```

## Wasm Verification

Verify that the hash of the gzipped WASM matches the proposed hash.

```
git fetch
git checkout ca66a0fdceccc56f8208d25c76ba4fc31b4b9cd6
"./scripts/docker-build" "ic-doge-canister"
sha256sum ./ic-doge-canister.wasm.gz
```