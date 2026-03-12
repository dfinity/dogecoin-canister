# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.0] - 2026-02-18

### Added

- Cherry-pick from dfinity/bitcoin-canister@master (ff779883..c3c089f) ([#100](https://github.com/dfinity/dogecoin-canister/pull/100))
  - Add `CanisterArg` enum for the canister initialization and upgrade arguments.

- Cherry-pick from dfinity/bitcoin-canister@master (9dcf7e6..d9ab390) ([#97](https://github.com/dfinity/dogecoin-canister/pull/97))
  - Add `BlockchainInfo` struct for the `get_blockchain_info` endpoint return type.

- Cherry-pick from dfinity/bitcoin-canister@master (7f84397) ([#95](https://github.com/dfinity/dogecoin-canister/pull/95))
  - Add a `burn_cycles` field to type `SetConfigRequest`.


## [0.2.0] - 2026-02-06

### Changed

- Cherry-pick from dfinity/bitcoin-canister@master (8d51212..38a51b8) ([#72](https://github.com/dfinity/dogecoin-canister/pull/72))
    - Remove custom `PartialOrd` implementation for type `Utxo`. This is a breaking change in terms of the semantics.
    - Add `AddressForWrongNetwork` variant to `GetBalanceError` and `GetUtxosError` enums for network validation of addresses in the `dogecoin_get_balance` and `dogecoin_get_utxos` endpoints. **Breaking change** for the `ic-doge-interface` crate as a new variant is added to existing enums, but not a breaking change for the canister, which does not return these enums but instead rejects requests with an error message.


## [0.1.0] - 2025-12-18

- Initial release

[0.3.0]: https://github.com/dfinity/dogecoin-canister/compare/ic-doge-interface-0.2.0...ic-doge-interface-0.3.0
[0.2.0]: https://github.com/dfinity/dogecoin-canister/compare/ic-doge-interface-0.1.0...ic-doge-interface-0.2.0
[0.1.0]: https://github.com/dfinity/dogecoin-canister/releases/tag/ic-doge-interface-0.1.0