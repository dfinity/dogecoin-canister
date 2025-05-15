# dogecoin-canister-watchdog

Watchdog service for a dogecoin_canister that compares its latest block height against several dogecoin explorer APIs and decides if dogecoin_canister is healthy or not.

## Commands

```sh
$ dfx stop

$ dfx start --background  --clean

$ dfx deploy watchdog

...
URLs:
  Backend canister via Candid interface:
    watchdog: http://127.0.0.1:4943/?canisterId=ryjl3-tyaaa-aaaaa-aaaba-cai&id=rrkah-fqaaa-aaaaa-aaaaq-cai
```
