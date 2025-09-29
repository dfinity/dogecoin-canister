# UTXO Reader

A Rust crate for reading and analyzing UTXOs from a Dogecoin canister `canister_state.bin` file.

## Overview

This crate provides functionality to:

1. **Read a `canister_state.bin` file** - Parse the stable memory structure used by the Dogecoin canister
2. **Extract all UTXOs** - Read both small and medium UTXOs from their respective memory regions  
3. **Sort UTXOs deterministically** - Sort by outpoint value for consistent ordering
4. **Compute a hash** - Generate a SHA256 hash of the sorted UTXO set for verification

## Memory Structure

The `canister_state.bin` file contains multiple memory regions:

- **Memory ID 1**: Address UTXOs (not used by this tool)
- **Memory ID 2**: Small UTXOs (script size ≤ 25 bytes) 
- **Memory ID 3**: Medium UTXOs (script size > 25 && ≤ 201 bytes)
- **Memory ID 4**: Balances (not used by this tool)

Large UTXOs (script size > 201 bytes) are stored as part of the main canister state (serialized with the rest of the state). The tool automatically reads all UTXO types from the canister state file.

## UTXO Structure

Each UTXO consists of:
- **OutPoint**: Transaction ID (32 bytes) + output index (4 bytes) 
- **TxOut**: Value in satoshis (8 bytes) + script (variable size)
- **Height**: Block height when UTXO was created (4 bytes)

## Usage

### As a CLI Tool

```bash
# Extract and hash ALL UTXOs (small, medium, and large) from canister state file
cargo run --bin utxo-reader -- --input /path/to/canister_state.bin

# Quiet mode - only output the hash
cargo run --bin utxo-reader -- --input /path/to/canister_state.bin --quiet
```

### As a Library

```rust
use utxo_reader::UtxoReader;

// Create reader from canister state file
let reader = UtxoReader::new("canister_state.bin")?;

// Extract all UTXOs
let utxos = reader.extract_utxos()?;

// Compute hash of the UTXO set
let hash = UtxoReader::compute_utxos_hash(&utxos);

println!("Found {} UTXOs with hash: {}", utxos.len(), hash);
```

## Output

The tool provides:

- **UTXO count**: Total number of UTXOs found
- **Hash**: SHA256 hash of the deterministically ordered UTXO set
- **Statistics**: Total value, height range, script size distribution
- **Detailed listing**: Individual UTXO information for the first 20 UTXOs (unless in quiet mode)

### Example Output

```
Reading canister state from: /path/to/canister_state.bin
Extracting UTXOs from stable memory regions...
Found 12345 total UTXOs

First 20 UTXO Details:
Index    Txid                                                             Vout     Value (DOGE)         Height   Script Size
------------------------------------------------------------------------------------------------------------------------------------
1        a1b2c3d4e5f6...                                                  0        0.50000000           100000   25
2        b2c3d4e5f6a7...                                                  1        0.25000000           100001   23
...

UTXO Set Hash (SHA256): a1b2c3d4e5f6...

Statistics:
  Total UTXOs: 12345
  Total Value: 1234567890 satoshis (12.34567890 DOGE)
  Height Range: 100000 - 200000
  Script Size Range: 20 - 25 bytes (avg: 23.1)
```

## Building

```bash
cd bootstrap/utxo-reader
cargo build --release
```

## Testing

```bash
cargo test
```

## Dependencies

- `ic-stable-structures`: For reading stable memory regions
- `ic-doge-types`: For Dogecoin-specific types (OutPoint, etc.)
- `ic-doge-canister`: For reusing existing serialization logic
- `sha2`: For computing SHA256 hashes
- `clap`: For command-line argument parsing
- `hex`: For hexadecimal encoding

## Limitations

- Requires the complete `canister_state.bin` file to be available locally
- Note: Genesis block coinbase outputs are excluded from the UTXO set as they are unspendable by Bitcoin consensus

## UTXO Coverage

- **Small UTXOs (≤25 bytes)**: Read from stable memory region 2
- **Medium UTXOs (26-201 bytes)**: Read from stable memory region 3  
- **Large UTXOs (>201 bytes)**: Read from serialized canister state
