# State Reader

A Rust crate for reading, analyzing, and validating all data from a Dogecoin canister `canister_state.bin` stable memory file.

## Memory Structure

The `canister_state.bin` file contains multiple memory regions that this tool parses:

- **Memory ID 0**: **Upgrade Memory** - Complete serialized canister state (CBOR format) containing in particular large UTXOs.
- **Memory ID 1**: **Address UTXOs** - Index mapping addresses to their UTXOs  
- **Memory ID 2**: **Small UTXOs** - UTXOs with script size ≤ 25 bytes
- **Memory ID 3**: **Medium UTXOs** - UTXOs with script size 26-201 bytes  
- **Memory ID 4**: **Address Balances** - Address to balance mapping
- **Memory ID 5**: **Block Headers** - Block hash to header blob mapping
- **Memory ID 6**: **Block Heights** - Block height to hash mapping

## Usage

### As a CLI Tool

```bash
# Full analysis
cargo run --bin state-reader -- --input /path/to/canister_state.bin --stats

# Quiet mode - only output the combined data hash
cargo run --bin state-reader -- --input /path/to/canister_state.bin --quiet
```

### As a Library

```rust
use state_reader::{UtxoReader, hash};

// Create reader from canister state file  
let reader = UtxoReader::new("canister_state.bin")?;

// Extract ALL canister data
let canister_data = reader.extract_state_data();

// Access specific data types
println!("UTXOs: {}", canister_data.utxos.len());
println!("Addresses: {}", canister_data.balances.len());
println!("Block headers: {}", canister_data.block_headers.len());

// Compute hashes for verification
let utxo_hash = hash::compute_utxo_set_hash(&canister_data.utxos);
let balance_hash = hash::compute_address_balances_hash(&canister_data.balances);
```

## Example Output

```
...
Validating data consistency...
  ✓ No UTXOs at height 0
  ✓ All block headers are properly sized
  ✓ No all-zeros block headers found
  ✓ Header and height counts match (1000001 entries)
  ✓ No duplicate block heights found
  ✓ No duplicate block headers found
  ✓ Block height continuity verified (0 to 1000000 = 1000001 blocks)
All data consistency checks passed ✓

═══════════════════════════════════════════════════════ 1: UTXOs ═══════════════════════════════════════════════════════

First 20 UTXO Details:
Index    Txid                                                               Vout  Value (DOGE)         Height       Script Size
------------------------------------------------------------------------------------------------------------------------
1        b1b716a8f9010475898f3b1ab9a7d729c66ea17d13e821c82e207bff8a3e4929   0     971496               38           35
2        f5f0bd761653c0ddbcb7c67e5cbbde7b52604bc44dbb31f24c779fca54705379   0     994210               41           35
3        00aee7d16a2964146878e2c22e5ddcb8ce914cd2dd55f925754bab4369b04ab3   0     875187               44           35
4        05ee49329929e3d14576bca017bc25a69e0ce72c28e5b27a86276689974fc41a   0     553227               45           35
5        60827734a16ebbc3c90fc84b236c97609ada0e0b191959b05727783ee6776b2c   0     944773               88           35
...

  Total UTXOs: 16,809,215
  Total Value: 102,172,625,547.23972 DOGE
  UTXO Height Range: 38 - 1,000,000
  Script Size Range: 23 - 103 bytes (avg: 25.2)
  Script Size Distribution:
    Small (≤25 bytes):     16,715,445 (99.44%)
    Medium (26-201 bytes): 93,770 (0.56%)
    Large (>201 bytes):    0 (0.00%)
  Value Distribution (DOGE):
    Min:     0.00000000
    25th %:  1
    Median:  1
    Mean:    6,078
    75th %:  111
    90th %:  1,002
    95th %:  2,190
    99th %:  18,075
    Max:     1,000,000,000

    Number of UTXOs with 0 amount: 2214


═══════════════════════════════════════════════════ 2: Address UTXOs ═══════════════════════════════════════════════════

  Total Address UTXOs entries: 16,715,445

First 20 Address UTXO Details:
Index    Address                                  Txid                                                               Vout  Height
------------------------------------------------------------------------------------------------------------------------
1        9rSN1FEMvejr1PVEaqriKNpy1T4CNB7phX       1ae7982bfe8447db9c594e14277ce5cb8c383071d4cb2399db22019e9e580297   1     877352
2        9rSPNQLhpYk8PxspwCPRAfb9Cw9VyuHf3S       ad816c79a749c76ba0e430c65ffa8c892a09473ccb53f94c1973357a3df36130   29    547142
3        9rSVq4Ls6kijytgPox3Ayh6qFY372cXAsS       2d03ba4de7484bc06ab1085fa1972231592f1fa6a2246cdd88971fb1f0e44ced   3     532991
4        9rSVq4Ls6kijytgPox3Ayh6qFY372cXAsS       e50046516f9d8d729fc880ae4c53b097c35fa0121e2c8230b8a058cf8c4c47fb   3     533409
5        9rSVq4Ls6kijytgPox3Ayh6qFY372cXAsS       cdd1fac2afd07af6ffb225234b3fea60311016df794df7b00a2c71757d6c92e6   4     533741
...

  Unique addresses: 1,566,878
  UTXOs per address - Min: 1, Max: 50200, Avg: 10.7, Median: 1
  Single-use addresses: 1,147,816 (73.25%)
  Reused addresses: 419,062 (26.75%)

  Top 5 Addresses by UTXO Count:
    1: DHV4WHmWPvniRCHErFsSbAa4d4ntgmg4Qk (50,200 UTXOs)
    2: DENHisQuuenKs72UB8UunSvKz6purZcXbe (50,186 UTXOs)
    3: DCD5JgWGmj7oDhtDa9KjBVYBftkNfXNNYd (50,186 UTXOs)
    4: DMjSMo5krpHzDKtUAFyVKFaS3pa5yQ1Ln5 (50,185 UTXOs)
    5: DFBA9Wyi98UmbPNnZWMGba8RJ1xaXdsx8R (50,185 UTXOs)

  Height range: 1,463 - 1,000,000

  Address Type Distribution:
    P2PKH (D*):    16,492,456 (98.67%)
    P2SH (A*/9*):  222,989 (1.33%)

══════════════════════════════════════════════════ 3: Address Balance ══════════════════════════════════════════════════

  Total Address Balances entries: 1,565,496

  Total Supply: 101,869,568,961 DOGE

  Balance Distribution (Non-zero addresses):
    Min:     0.00000001 DOGE
    Median:  2.12778687 DOGE
    Mean:    65071.75295324 DOGE
    Max:     17850000416.50004578 DOGE
    25th %:  1.00000000 DOGE
    75th %:  183.94258532 DOGE
    90th %:  5252.24189498 DOGE
    95th %:  29779.21517941 DOGE
    99th %:  446063.53972945 DOGE

  Balance Range Distribution:
    Dust (<0.1 DOGE):      313,706 (20.04%)
    Small (0.1-100 DOGE):  803,314 (51.31%)
    Medium (100-10K DOGE): 320,765 (20.49%)
    Large (10K-10M DOGE):  126,697 (8.09%)
    Whale (>10M DOGE):     1,014 (0.06%)

  Number of addresses with zero balance: 0

  Top 10 Addresses by Balance:
    1: D8EyEfuNsfQ3root9R3ac54mMcLmoNBW6q = 17,850,000,416 DOGE (17.5224% of supply)
    2: DDTtqnuZ5kfRT5qh2c7sNtqrJmV3iXYdGG = 5,031,000,833 DOGE (4.9387% of supply)
    3: DDogepartyxxxxxxxxxxxxxxxxxxw1dfzr = 1,854,575,771 DOGE (1.8205% of supply)
    4: DH6Vr6C4H3YxJTCbVFhz59ZAiW2Ye4oVQi = 700,000,006 DOGE (0.6872% of supply)
    5: DLQ74r85SKqNxHS4KUWk3N7roxNdWJfibX = 700,000,000 DOGE (0.6872% of supply)
    6: DR9Lat8P7FiLrPhPHycYzayb6boxzah3Ei = 700,000,000 DOGE (0.6872% of supply)
    7: DBpNLLEj13LWr14wm1YH24nuqAjodrjaLL = 651,455,830 DOGE (0.6395% of supply)
    8: DP5JmfwdZfkwjREioCoTU8RFK3Pz7F8A4W = 638,767,803 DOGE (0.6270% of supply)
    9: D6wpEhYES5wGBjbVq6p97JkV2fWHFp4vqp = 600,000,000 DOGE (0.5890% of supply)
    10: D6defLv3odMN5wmUyMxMzyDKJ3SRYR6Nqr = 543,588,135 DOGE (0.5336% of supply)

  Wealth Concentration:
    Top 1% of addresses hold: 91.42% of total supply
    Top 5% of addresses hold: 98.65% of total supply
    Top 10% of addresses hold: 99.67% of total supply

═══════════════════════════════════════════════════ 4: Block Headers ═══════════════════════════════════════════════════

  Total block headers entries: 1,000,001
  Total block heights entries: 1,000,001

  Block Height Analysis:
    Height range: 0 - 1,000,000 (span: 1,000,001 blocks)

  Block Header Size Analysis:
    Standard Header (80 bytes): 1,000,001 (100.00%)
    AuxPow Header (>80 bytes):  0 (0.00%)

  Last 5 Block Headers Details:
Block Hash                                                       Height
----------------------------------------------------------------------------------------------------
6aae55bea74235f0c80bd066349d4440c31f2d0f27d54265ecd484d8c1d11b47 1,000,000
a2fbadf323bb72fd39016513f46ddd76e390ec80051cb733a53c48c024ed4df1 999,999
6a952d375e16381f747478ea6bcfd6074d45380dec1cb9c1f36280be4b3d9c4a 999,998
e3a08297608a6461e2a83af6ff0d3903ca0d14dcc4f401a7ed6b509b1b158319 999,997
e3dea80e24e459f6fd8c112378c998689e17771e09ce4d4454d70fa373e49736 999,996

Computing data hashes...
  Computing UTXO set hash (16809215 entries)...
  Computing address UTXOs hash (16715445 entries)...
  Computing address balances hash (1565496 entries)...
  Computing block headers hash (1000001 entries)...
  Computing block heights hash (1000001 entries)...
  Computing combined hash...
════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════
                                                  DATA HASHES (SHA256)
════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════

UTXO Set        : c09379e4ac352a4f8bf4c726bd5d8ce9ca0eb02eba6af92b77b53c1b97b6d668
Address UTXOs   : ab58519c6058149888a6ea5dd93882799f236a86133e72c27e5589ef1fa475d0
Address Balance : 86ec1ff693946ea6e4eba018e03edd84b93e4a77341f447092a21235a220516f
Block Headers   : 2bc2c25b3d4d0ab74ed87d26f93c34ddcb8c0c7f455412f2e0f1558f8aadc937
Block Heights   : 4ebf875af8b5bc2ef915b738238868b12501e5a1e393d9f2c5843338dabef50c

Combined hash   : 927f1b6ce3e854d72721cb14ed1eb5d589aef9b9b715add0a9262fba8ccd25ff
```

