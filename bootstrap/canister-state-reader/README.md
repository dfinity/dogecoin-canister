# Canister State Reader

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
# Full analysis with comprehensive statistics
cargo run --bin canister-state-reader -- --input /path/to/canister_state.bin

# Quiet mode - only output the combined hash
cargo run --bin canister-state-reader -- --input /path/to/canister_state.bin --quiet
```

### As a Library

```rust
use canister_state_reader::{UtxoReader, hash};

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
═══════════════════════════════════════ 1: UTXOs ═══════════════════════════════════════

First 20 UTXO Details:
Index    Txid                                                             Vout  Value (DOGE)         Height          Script Size
------------------------------------------------------------------------------------------------------------------------------------
1        a1b2c3d4e5f6789abcdef0123456789abcdef0123456789abcdef012345      0     12                   150000          25
2        b2c3d4e5f6789abcdef0123456789abcdef0123456789abcdef012345        1     5                    150001          66
...

  Total UTXOs: 853,347
  Total Value: 11,227,152,111 DOGE
  UTXO Height Range: 0 - 22,559
  Script Size Range: 25 - 67 bytes (avg: 25.1)
  Script Size Distribution:
    Small (≤25 bytes):     851,227 (99.75%)
    Medium (26-201 bytes): 2,120 (0.25%)
    Large (>201 bytes):    0 (0.00%)
  Value Distribution (DOGE):
    Min:     0.00000000
    25th %:  1
    Median:  100
    Mean:    13,156
    75th %:  1,435
    90th %:  10,498
    95th %:  21,640
    99th %:  138,236
    Max:     84,000,000

═══════════════════════════════════ 2: Address UTXOs ═══════════════════════════════════

  Total Address UTXOs entries: 2,468,134
  
First 20 Address UTXO Details:
Index    Address                                  Txid                                                               Vout  Height
----------------------------------------------------------------------------------------------------------------------------------
1        D596vQLHxTn72k5LaXSMJbgp4sCGeFKQZW       ef421ec2df08e6bedff3e5abfa664d22dd6e7557d50397ffe4feb312717bf662   1     18680
2        D597Q1BJWpheda2QfHNETa1A4Hx52CMBjZ       b059c11569c7c121e94c24a13d960826863defe7c8242cc2e902ee755b02adf7   3     15950
...

  Unique addresses: 160,011
  UTXOs per address - Min: 1, Max: 4064, Avg: 5.3, Median: 1
  Single-use addresses: 103,051 (64.40%)
  Reused addresses: 56,960 (35.60%)

  Top 5 Addresses by UTXO Count:
    1: DEb5f2awGpTzEmMNVrRX45MEn7BH9SheAh (4,064 UTXOs)
    2: D7UnQqDqKpr5V7CjTTcYdktjyTyKbmWvKo (1,177 UTXOs)
    3: DJLfqwBx3Za6QNkxnpt7BhsAMhDEA2aUxo (985 UTXOs)
    4: DCCarszRxCwcH1BEVq5AqKzeDB4ncR1yz3 (716 UTXOs)
    5: D5hCPaqX9f8kNpukcvyjXavtkUGMHJLUBH (687 UTXOs)

  Height range: 1,182 - 2,234,566

  Address Type Distribution:
    P2PKH (D*):     160,011 (100.00%)
    P2SH (A*/9*):    0 (0.00%)

═══════════════════════════════════ 3: Address Balance ═══════════════════════════════════

  Total Address Balances entries: 160,011

  Total Supply: 10,763,921,179 DOGE

  Balance Distribution (Non-zero addresses):
    Min:     0.00000001 DOGE
    Median:  36.34313949 DOGE
    Mean:    67269.88256527 DOGE
    Max:     533110850.00000000 DOGE
    25th %:  1.05421590 DOGE
    75th %:  5199.99990000 DOGE
    90th %:  50050.00000000 DOGE
    95th %:  144972.01240617 DOGE
    99th %:  1000000.00000000 DOGE

  Balance Range Distribution:
    Dust (<0.1 DOGE):     22,184 (13.86%)
    Small (0.1-100 DOGE):   64,402 (40.25%)
    Medium (100-10K DOGE):   39,331 (24.58%)
    Large (10K-10M DOGE):  33,991 (21.24%)
    Whale (>10M DOGE):     103 (0.06%)

  Top 10 Addresses by Balance:
    1: D6fBstwziY72FMb5NZJoccmfQ7RBmDfeju = 533,110,850 DOGE (4.9528% of supply)
    2: DHsqeNgKH5f6Jnch8b9w45ddRBHpGXSBuj = 377,981,247 DOGE (3.5116% of supply)
...

  Wealth Concentration:
    Top 1% of addresses hold: 66.31% of total supply
    Top 5% of addresses hold: 88.81% of total supply
    Top 10% of addresses hold: 95.25% of total supply

═══════════════════════════════════ 4: Block Headers ═══════════════════════════════════

  Total block headers entries: 22,560
  Total block heights entries: 22,560

  Block Height Analysis:
    Height range: 0 - 22,559 (span: 22,560 blocks)

  Block Header Size Analysis:
    Standard Header (80 bytes):  22,560 (100.00%)
    AuxPow Header (>80 bytes): 0 (0.00%)

════════════════════════════════════════════════════════════════════════════════════════
                                      DATA HASHES (SHA256)
════════════════════════════════════════════════════════════════════════════════════════

UTXO Set        : a1b2c3d4e5f6789abcdef0123456789abcdef0123456789abcdef0123456789ab
Address UTXOs   : f9e8d7c6b5a4321fedcba9876543210fedcba9876543210fedcba9876543210
Address Balance : 1234567890abcdef0123456789abcdef0123456789abcdef0123456789abcdef
Block Headers   : fedcba0987654321fedcba0987654321fedcba0987654321fedcba0987654321
Block Heights   : 0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef

Combined hash   : a1b2c3d4e5f6789abcdef0123456789abcdef0123456789abcdef0123456789ab
```

