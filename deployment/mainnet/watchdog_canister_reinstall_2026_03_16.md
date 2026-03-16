# Proposal to reinstall the Dogecoin watchdog canister

Repository: `https://github.com/dfinity/bitcoin-canister.git`

Git hash: `4e1ea8fd4bd2e2fb85350e4b1b3d4cc2410e389b`

New compressed Wasm hash: `541a859230e0bebf4e32573eee07755bc8a9602185a4675be60b87ed74ab1071`

Install args hash: `fc2eba0108107479ef7cd9753d41487d99305605b1b3673608c51848ded23885`

Target canister: `he6b4-hiaaa-aaaan-aaaeq-cai`

Previous Dogecoin watchdog proposal: https://dashboard.internetcomputer.org/proposal/140433

---

## Motivation

Reinstall the watchdog canister monitoring the Dogecoin canister (Dogecoin mainnet network) using the latest release [watchdog/release/2026-03-13](https://github.com/dfinity/bitcoin-canister/releases/tag/watchdog%2Frelease%2F2026-03-13).

This proposal aims to use the new `get_blockchain_info` endpoint of the Dogecoin canister to retrieve its height instead of retrieving it from the `/metrics` HTTP endpoint using HTTPs outcalls which are less reliable than intercanister calls.

This proposal also updates the list of providers used by the watchdog to fetch Dogecoin mainnet latest height:
- Adds the `api_bitcore` provider (https://api.bitcore.io/api/DOGE/mainnet/block?limit=1).
- Adds the `psy_protocol` provider (https://doge-electrs-demo.qed.me/blocks/tip/height).
- Removes the `tokenview` provider (https://doge.tokenview.io/api/chainstat/doge ): behind Cloudflare.

The number of providers is increased from 3 to 4, which increases the resilience of the watchdog canister.

## Release Notes

```
git log --format='%C(auto) %h %s' 9e3d701a122558f68abbb0cfc5d2099ac61d9e08..4e1ea8fd4bd2e2fb85350e4b1b3d4cc2410e389b -- watchdog
4e1ea8f chore(watchdog): watchdog/release/2026-03-13 (#503)
5e68e4a feat(watchdog): use `get_blockchain_info` canister endpoint to retrieve monitored canister height (#484)
efe7c9d feat(watchdog): replace api_bitaps with api_bitcore provider (#500)
e7dcc11 fix(watchdog): rename `HealthStatus` field names `canister_height` and `explorer_height` (#492)
21b1571 feat(watchdog): replace tokenview explorer with psy protocol (#493)
d9ab390 fix(e2e-tests): only use pre-built wasms if present (#488)
9dcf7e6 test: use wasms from reproducible build for various e2e tests (#473)
f29c7c2 chore: update watchdog CHANGELOG.md release/2025-12-03 (#475)
 ```

## Install args

```
git fetch
git checkout 4e1ea8fd4bd2e2fb85350e4b1b3d4cc2410e389b
didc encode -d watchdog/candid.did -t '(watchdog_arg)' '(variant { init = record { target = variant { dogecoin_mainnet } }})' | xxd -r -p | sha256sum
```

## Wasm Verification

Verify that the hash of the gzipped WASM matches the proposed hash.

```
git fetch
git checkout 4e1ea8fd4bd2e2fb85350e4b1b3d4cc2410e389b
"./scripts/docker-build" "watchdog"
sha256sum ./watchdog.wasm.gz
```