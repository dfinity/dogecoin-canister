# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [release/2026-02-06] - 2026-02-06

### Changed
- Cherry-pick from dfinity/bitcoin-canister@master (8d51212..38a51b8) ([#72](https://github.com/dfinity/dogecoin-canister/pull/72)):
  - Add network validation (regtest/mainnet) for addresses in get_balance and get_utxos requests
  - Add unified canister_arg to initialize and upgrade Dogecoin canister

## [release/2025-12-10] - 2025-12-10

### Changed
- Use 1,000 transactions in the percentiles calculation ([#60](https://github.com/dfinity/dogecoin-canister/pull/60))
- Cherry-pick from dfinity/bitcoin-canister@master (91b1c67) ([#59](https://github.com/dfinity/dogecoin-canister/pull/59))
- Cherry-pick from dfinity/bitcoin-canister@master (13c6ff2) ([#62](https://github.com/dfinity/dogecoin-canister/pull/62))

### Removed
- Remove testnet ([#61](https://github.com/dfinity/dogecoin-canister/pull/61))

## [release/2025-11-19] - 2025-11-19

### Changed
- Cache unstable blocks in stable memory ([#54](https://github.com/dfinity/dogecoin-canister/pull/54))

## [release/2025-10-24] - 2025-10-24

### Fixed
- Backward-compatible state deserialization ([#44](https://github.com/dfinity/dogecoin-canister/pull/44))

## [pre-release/2025-10-17] - 2025-10-17

Initial pre-release of the Dogecoin canister, adapted from the Bitcoin canister codebase.

### Added
- Add dogecoin header validation ([#2](https://github.com/dfinity/dogecoin-canister/pull/2))
- Add auxpow validation ([#14](https://github.com/dfinity/dogecoin-canister/pull/14))

### Changed
- Adapt crates to use dogecoin header validation instead of bitcoin ([#9](https://github.com/dfinity/dogecoin-canister/pull/9))
- Transition header validation to auxpow validation ([#18](https://github.com/dfinity/dogecoin-canister/pull/18))
- Rename endpoints ([#23](https://github.com/dfinity/dogecoin-canister/pull/23))
- Enable communication with Dogecoin adapter ([#22](https://github.com/dfinity/dogecoin-canister/pull/22))
- Change satoshi to koinu and use nat for get_balance calls ([#27](https://github.com/dfinity/dogecoin-canister/pull/27))
- Ensure get_block_header endpoint returns 80-bytes headers ([#31](https://github.com/dfinity/dogecoin-canister/pull/31))
- Compute pure header only ([#36](https://github.com/dfinity/dogecoin-canister/pull/36))
- Cherry-pick from dfinity/bitcoin-canister@master (6bed9af..292b446) ([#10](https://github.com/dfinity/dogecoin-canister/pull/10))
- Cherry-pick from dfinity/bitcoin-canister@master (defadc1..46e1a4c) ([#32](https://github.com/dfinity/dogecoin-canister/pull/32))
- Fix pipeline with new dogecoin canister ([#3](https://github.com/dfinity/dogecoin-canister/pull/3))

### Fixed
- Use buffered writer during pre-upgrade and set stability threshold to 360 to reduce heap memory pressure ([#41](https://github.com/dfinity/dogecoin-canister/pull/41))
- Modify balance type to u128 ([#33](https://github.com/dfinity/dogecoin-canister/pull/33))
- Increase block range in canister/src/tests.rs ([#20](https://github.com/dfinity/dogecoin-canister/pull/20))


[release/2026-02-06]: https://github.com/dfinity/dogecoin-canister/compare/release/2025-12-10...release/2026-02-06
[release/2025-12-10]: https://github.com/dfinity/dogecoin-canister/compare/release/2025-11-19...release/2025-12-10
[release/2025-11-19]: https://github.com/dfinity/dogecoin-canister/compare/release/2025-10-24...release/2025-11-19
[release/2025-10-24]: https://github.com/dfinity/dogecoin-canister/compare/pre-release/2025-10-17...release/2025-10-24
[pre-release/2025-10-17]: https://github.com/dfinity/dogecoin-canister/compare/baseline-fork...pre-release/2025-10-17

