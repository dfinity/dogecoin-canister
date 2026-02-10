# Proposal to install the Dogecoin watchdog canister

Repository: `https://github.com/dfinity/bitcoin-canister.git`

Git hash: `9e3d701a122558f68abbb0cfc5d2099ac61d9e08`

New compressed Wasm hash: `0607f919c650fce2fd5ba60be4af8ff947522f77af9e9164e4b129dbe94a6e62`

Install args hash: `fc2eba0108107479ef7cd9753d41487d99305605b1b3673608c51848ded23885`

Target canister: `he6b4-hiaaa-aaaan-aaaeq-cai`

---

## Motivation

Install the watchdog canister for the Dogecoin canister (Dogecoin mainnet). The watchdog canister monitors the Dogecoin canister’s latest block height by comparing it against heights obtained from multiple sources, and disables the Dogecoin canister API if the reported height deviates beyond an acceptable range (too far behind or ahead).


## Install args

```
git fetch
git checkout 9e3d701a122558f68abbb0cfc5d2099ac61d9e08
didc encode -d watchdog/candid.did -t '(watchdog_arg)' '(variant { init = record { target = variant { dogecoin_mainnet } }})' | xxd -r -p | sha256sum
```

## Wasm Verification

Verify that the hash of the gzipped WASM matches the proposed hash.

```
git fetch
git checkout 9e3d701a122558f68abbb0cfc5d2099ac61d9e08
"./scripts/docker-build" "watchdog"
sha256sum ./watchdog.wasm.gz
```