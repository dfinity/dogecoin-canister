# CCoins Deserializer

A Rust implementation for deserializing Dogecoin/Bitcoin CCoins database objects from the chainstate leveldb.

## Overview

This library can parse the compressed binary format used to store UTXO (Unspent Transaction Output) data in Dogecoin and Bitcoin's chainstate database. The format uses sophisticated compression techniques to minimize storage space while maintaining efficient access.

## Database Structure

In the chainstate leveldb:
- **Key**: `('c', txid)` - prefix 'c' + 32-byte transaction ID
- **Value**: Compressed CCoins object containing all outputs for that transaction

## CCoins Serialization Format

The serialized CCoins object contains:

1. **VARINT(version)** - Transaction version number
2. **VARINT(code)** - Header code encoding multiple flags:
    - Bit 0: Coinbase flag (1 = coinbase, 0 = regular tx)
    - Bit 1: vout[0] availability (1 = unspent, 0 = spent)
    - Bit 2: vout[1] availability (1 = unspent, 0 = spent)
    - Higher bits: Number of bitmask bytes for vout[2+]
3. **Spentness bitmask** - Compact representation of spent/unspent status for vout[2] onwards
4. **Compressed outputs** - Only unspent outputs, using amount and script compression
5. **VARINT(height)** - Block height where transaction was included

## Features

### Amount Compression
- Efficiently compresses cryptocurrency amounts by factoring out powers of 10
- Example: 60,000,000,000 satoshis (600 BTC) compresses from 8 bytes to 4 bytes

### Script Compression
- **Pay-to-PubkeyHash (P2PKH)**: 25-byte script → 21 bytes
- **Pay-to-ScriptHash (P2SH)**: 23-byte script → 21 bytes
- **Pay-to-Pubkey**: Full pubkey → 33 bytes compressed
- **Other scripts**: Minimal overhead (1-2 bytes + script length)

### Spentness Tracking
- Compact bitmask tracks which outputs are spent vs unspent
- Only stores bitmask bytes that contain unspent outputs
- vout[0] and vout[1] availability encoded directly in header code

## Usage

```rust
use ccoins_deserializer::CCoinsDeserializer;

// Example data from database (hex encoded)
let hex_data = "0104835800816115944e077fe7c803cfa57f29b36bf87c1d358bb85e";
let data = hex::decode(hex_data).unwrap();

// Deserialize CCoins object
let coins = CCoinsDeserializer::deserialize(&data)?;

println!("Version: {}", coins.version);
println!("Coinbase: {}", coins.coinbase);  
println!("Height: {}", coins.height);

// Iterate through outputs
for (i, output) in coins.outputs.iter().enumerate() {
    match output {
        Some(txout) => {
            println!("vout[{}]: {} satoshis", i, txout.value);
            println!("  Script: {} bytes", txout.script_pubkey.len());
        },
        None => println!("vout[{}]: SPENT", i),
    }
}
```

## Real Examples

### Simple Transaction (1 output)
```
Hex: 0104835800816115944e077fe7c803cfa57f29b36bf87c1d358bb85e

Breakdown:
- 01: version = 1
- 04: code = 4 (vout[1] unspent, 0 bitmask bytes)  
- 8358: compressed amount = 60,000,000,000 satoshis (600 BTC)
- 00: P2PKH script type
- 816115944e077fe7c803cfa57f29b36bf87c1d35: 20-byte address hash
- 8bb85e: height = 203,998
```

### Complex Coinbase (Multiple outputs)
```
Hex: 0109044086ef97d5790061b01caab50f1b8e9c50a5057eb43c2d9563a4ee...

Breakdown:
- 01: version = 1  
- 09: code = 9 (coinbase, vout[0] and vout[1] spent, 2 bitmask bytes follow)
- 044: bitmask indicating vout[4] and vout[16] are unspent
- 86ef97d579...: compressed output at vout[4]
- bbd123...: compressed output at vout[16]  
- Height: 120,891
```

## Running the Code

```bash
# Clone or create the project
cargo new ccoins-deserializer --bin

# Copy the provided files:
# - ccoins_deserializer.rs
# - Cargo.toml  
# - README.md

# Run the example
cargo run

# Run tests
cargo test
```

## Testing

The implementation includes comprehensive tests for:
- VARINT parsing with various values
- Amount compression/decompression roundtrip
- Header code decoding
- Real transaction data examples

## Compatibility

This deserializer is compatible with:
- **Dogecoin** chainstate database format
- **Bitcoin Core** chainstate database format
- Any Bitcoin-based cryptocurrency using the same UTXO storage format

## Technical Notes

### VARINT Format
Uses variable-length encoding where the high bit of each byte indicates continuation:
- Values 0-127: Single byte
- Values 128+: Multiple bytes with continuation bits

### Bitmask Optimization
- Only stores bitmask bytes containing unspent outputs
- Trailing zero bytes are omitted
- Reduces storage for transactions with many spent outputs

### Script Pattern Recognition
Common script patterns are detected and compressed:
- Standard P2PKH, P2SH patterns
- Compressed/uncompressed public key patterns
- Fallback to full script storage for non-standard scripts

## Error Handling

The deserializer handles various error conditions:
- Invalid VARINT encoding
- Truncated data streams
- Invalid script compression formats
- Overflow conditions in amount decompression

## Performance

The implementation prioritizes:
- **Memory efficiency**: Minimal allocations during parsing
- **Correctness**: Strict validation of input data
- **Compatibility**: Exact match with reference C++ implementation

## License

MIT License - See LICENSE file for details.
