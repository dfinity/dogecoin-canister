# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [release/2026-02-05] - 2026-02-05

### Added

- Add NetworkAdapter to communicate with Dogecoin adapter (#22)

- Add auxpow validation (#14)

- Add dogecoin header validation (#2)

- Add default fees for mainnet/testnet networks (#376)

- Add get_successors_request_interval histogram metric (#361)

- Add get_successors request / response metrics for bitcoin canister (#360)

- Add instructions for generating testnet_blocks.txt (#351)

- Add config to eagerly evaluate fee percentiles (#303)

- Add candid interface compatibility check (#300)

- Add a bound on the length of the unstable chain in testnet/regtest (#246)

- Add benchmark for inserting block headers (#242)

- Add panic hook to bitcoin canister (#240)

- Add additional logs to the canister heartbeat (#239)

- Add benchmark for the get_metrics endpoint. (#238)

- Add watchdog_canister to set_config_request api (#211)

- Add watchdog canister to the bitcoin canister config (#193)

- Add test to document behaviour at min_confirmations = 0. (#190)

- Add metric to track if the canister is synced (#167)

- Add tests for bech32 addresses support (#165)

- Add unit tests for stability count confirmations. (#138)

- Add a separate type for `BlockHash` (#97)

- Add bootstrap script to build state offline. (#81)

- Add instruction count histogram for the various endpoints. (#87)

- Add metrics for memory sizes (#82)

- Add ability to update stability threshold. (#83)

- Add ability to charge for cycles. (#79)

- Add endpoint to enable/disable syncing. (#78)

- Add types to `address_utxos` map. (#64)

- Add nice debug output for transaction ids. (#57)

- Add profiling stats to `get_utxos` and `get_balance`. (#53)

- Add new blocks to the e2e syncing test. (#48)

- Add testnet test (#37)


### Changed

- Release plz (#74)

- Cherry-pick from dfinity/bitcoin-canister@master (8d51212..38a51b8) (#72)

- Cherry-pick from dfinity/bitcoin-canister@master (13c6ff2) (#62)

- Use 1,000 transactions in the percentiles calculation (#60)

- Cherry-pick from dfinity/bitcoin-canister@master (91b1c67) (#59)

- Cache unstable blocks in stable memory (#54)

- Cherry-pick from dfinity/bitcoin-canister@master 02af290 (#53)

- Cherry-pick from dfinity/bitcoin-canister@master (af000cc..f202301) (#51)

- Cherry-pick from dfinity/bitcoin-canister@master (b506535) (#50)

- Cherry-pick from dfinity/bitcoin-canister@master (defadc1..46e1a4c) (#32)

- Revert "store auxpow headers into stable memory" (#35)

- Ensure `get_block_header` endpoint returns 80-bytes headers (#31)

- Store auxpow headers into stable memory (#30)

- Change satoshi to koinu and use nat for get_balance calls (#27)

- Rename endpoints (#23)

- Transition header validation to auxpow validation (#18)

- Adapt crates to use dogecoin header validation instead of bitcoin (#9)

- Cherry-pick from dfinity/bitcoin-canister@master (6bed9af..292b446) (#10)

- Rename Bitcoin references to Dogecoin

- Adjust testnet unstable max depth difference (#382)

- Cleanups (#381)

- Update dfx to 0.23 and rust to 1.81 (#372)

- Update get_successors metrics (#364)

- Upgrade bitcoin crate to 0.32.4 for testnet4 support (#349)

- Upgrade stable structures to 0.6.7 (#346)

- Rename usage of BlockHeader to Header for bitcoin crate v.0.32.4 update (#345)

- Upgrade Cargo.lock deps and ic-cdk (#321)

- More enhancements for bootstrapping (#316)

- Reduce memory footprint of pre_upgrade by 50% (#306)

- [EXC-1649] Add network field to BitcoinGetBlockHeadersRequest (#312)

- [EXC-1620] Charge cycles for using get_block_headers endpoint (#309)

- [EXC-1619] Implement `get_block_headers()` endpoint (#298)

- More efficient serialization of unstable blocks (#305)

- Measure memory usage when serializing unstable blocks (#304)

- Make init config fields optional (#302)

- Add stub for Bitcoin headers endpoint (#297)

- Reduce block ingestion instruction limit from 2B to 1B (#294)

- Reduce the max instructions of heartbeats from 4B to 2B (#293)

- Upgrade rust from 1.70 to 1.76 (#281)

- Use the burn cycles API in the Bitcoin canister (#268)

- Improve benchmarking reports (#263)

- Move BlockTree methods inside the BlockTree's impl (#262)

- Add inspect_message endpoint to Bitcoin canister (#253)

- Add Non-replicated Queries in the Bitcoin API (#250)

- [EXC-1379] remove interim code from previous upgrade (#248)

- Skip next block headers if they are already inserted. (#245)

- Move shared types in `ic-btc-types` crate (#244)

- Calculate the main chain height more efficiently (#237)

- Update some crate revisions (#230)

- Unify crate versions by moving them to workspace level (#229)

- Downgrade candid from 0.9.0-beta.3 to 0.8.1 (#224)

- Introduce Txid type (#220)

- Upgrade candid version to 0.9.0-beta.3 (#213)

- Expose bitcoin_canister api_access metric (#205)

- Upgrade stable structures to version 0.5.2 (#176)

- Clean up interface types. (#171)

- [EXC-1375] cache block hash computations to speed up block insertions (#164)

- Move ic_btc_types and ic_btc_test_utils into repo. (#162)

- Add disable_api_if_not_fully_synced config flag (#156)

- Do not respond to requests when not fully synced (#151)

- Upgrade rust to 1.68.0 (#155)

- Use the guard pattern when fetching blocks. (#154)

- Track instruction counts of block insertions (#153)

- Add metric to track block ingestion instruction counts (#150)

- Track cycles received by the canister in a metric (#144)

- Use stability count for counting confirmations. (#139)

- Make the network field in `UnstableBlocks` non-optional (#141)

- Gzip canisters + minor enhancements (#140)

- Use difficulty-based stability threshold for fork resolution (#126)

- Compute block difficulty (#131)

- Block header validation (#129)

- Unify canister build scripts. (#124)

- Add a flag to disable bitcoin apis (#120)

- Revert pretty cycles in error message (#123)

- Prettify cycles error message (#119)

- New fee structure (#115)

- Reduce max UTXOs per response to 1k. (#116)

- Enhancements to the `send_transaction` endpoint. (#109)

- Script to compute main state struct (#105)

- Update byte representation of `(TxOut, Height)`. (#107)

- Implement `send_transaction` endpoint. (#101)

- Upgrade `stable-structures` to 0.3 (#98)

- Implement stable structures' `Storable` trait for `OutPoint`. (#93)

- [EXC-1283] script to compute UTXOs. (#95)

- Move metrics into api module (#86)

- Exc 1232 add metric to track errors when fetching new blocks (#75)

- (EXC-1231) make block ingestion atomic. (#76)

- Minor cleanup to the API of `UtxoSet`. (#73)

- Update utxo ordering test (#74)

- Keep block in `unstable_blocks` until it is fully ingested. (#72)

- Performance enhancement to `get_utxos`. (#71)

- Update candid interface to match interface specification. (#70)

- Introduce e2e test scenario 2 (#68)

- Make the UtxoSet state private. (#62)

- Update get_successors API to match the replica implementation. (#61)

- Store block headers perpetually (EXC-990) (#60)

- Use a cache to serve `get_balance` (#59)

- Make replaying blocks more efficient. (#58)

- Make fee percentiles computation 40x faster (#56)

- Avoid fetching removed utxos. (#55)

- Feat: cache fee percentiles. (#51)

- Instrument get_current_fee_percentiles (#50)

- Make get_utxos more efficient. (#49)

- Cache txids (#47)

- Use ic-wasm to shrink the size of canister's wasm. (#46)

- Introduce a new `Block` type. (#45)

- Profile block ingestion (#44)

- Start profiling the bitcoin canister (#43)

- Cache tx id when replaying blocks. (#41)

- Expose the `get_current_fee_percentiles` endpoint. (#40)

- Use ic-stable-structures crate (#39)

- Remove unused method (#38)

- Move mainchain test into new tests file. (#34)

- Make mainchain test more robust. (#33)

- Delete unneeded test

- Specify the network in `GetSuccessorsRequest`. (#32)

- Use MemoryManager in the stable-structures crate. (#31)

- Add pagination to handle responses > 2MiB. (#29)

- Remove unneeded deps + files (#30)

- Fix subtract overflow bug in computing height

- Refactor time-slicing code (#28)

- Time-slicing within a single transaction. (#27)

- Time-slicing when ingesting stable blocks into UTXO set. (#26)

- Refactor `insert_block` to write stable blocks separately. (#25)

- Expose the get_utxos endpoint (#23)

- Process `GetSuccessorResponse` in a separate heartbeat. (#22)

- Introduce E2E tests + ingest blocks in heartbeat (#21)


### Fixed

- Backward-compatible state deserialization (#44)

- Use buffered writer during pre-upgrade and set stability threshold to 360 to reduce heap memory pressure (#41)

- Increase block range in `canister/src/tests.rs` (#20)

- Fix pipeline with new dogecoin canister (#3)

- Adaptive max depth limit calculation for unstable blocks tree (#385)

- [EXC-1987] Fix encoding of get_block_headers metrics on Bitcoin canister (#383)

- Fix unstable tree block stability check for testnet (#379)

- Fix memory leak (#378)

- Get_successors request sends unique hashes (#363)

- Reduce Bitcoin canister logs by skipping full GetSuccessorsResponse (#359)

- Do not include canbench in production (#317)

- Canister bootstrap scripts (#315)

- [EXC-1634] Add e2e tests for get_block_headers() (#310)

- Return cached fee percentiles if there are no txs in unstable blocks (#299)

- Fix (EXC-1590): make burning cycles configurable (#286)

- Bound length of chain on testnet (#261)

- Deserialize `BlockTree` iteratively (#258)

- Use vbyte for the computation of a transaction fee (#225)

- Drop next block headers above a certain instructions threshold (#247)

- Make api_access metric an enum (#222)

- Next block headers validation (#175)

- Failing tests (#169)

- Broken upgradability test (#168)

- Fix bug in retrieving the caller in the set_config endpoint. (#157)

- Ignore coinbase transactions when computing fee percentiles. (#152)

- Validate that the block is < 2 hours from the current time. (#149)

- Set syncing flag in init method. (#147)

- Replace traps with rejects to correctly charge for cycles (#145)

- Block header validation proptest (#133)

- Broken build (#125)

- Only a canister's controllers can call `set_config`. (#110)

- Fix calculating fee percentiles (#90)

- Improve the fee percentile computation (#88)

- [EXC-1215] gracefully handle invalid responses. (#84)

- Broken build (#85)

- Update endpoint names to match interface spec. (#80)

- Panic when removing an input with zero value. (#69)

- "recursion limit exceeded" error when serializing blocktree. (#63)

- Properly handle time-slicing of multiple transactions. (#35)


### Removed

- Remove testnet (#61)

- Remove `rand` dependency from Bitcoin canister (#348)

- Remove legacy_preupgrade feature (#319)

- Remove dangling todos (#166)

- EXC-1203 refactor: remove store.rs (#52)

- Remove incorrect comment (#36)


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


[release/2025-12-10]: https://github.com/dfinity/dogecoin-canister/compare/release/2025-11-19...release/2025-12-10
[release/2025-11-19]: https://github.com/dfinity/dogecoin-canister/compare/release/2025-10-24...release/2025-11-19
[release/2025-10-24]: https://github.com/dfinity/dogecoin-canister/compare/pre-release/2025-10-17...release/2025-10-24
[pre-release/2025-10-17]: https://github.com/dfinity/dogecoin-canister/compare/baseline-fork...pre-release/2025-10-17

[release/2026-02-05]: https://github.com/dfinity/dogecoin-canister/releases/tag/ic-doge-canister/release/2026-02-05
