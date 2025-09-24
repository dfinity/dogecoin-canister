# UTXO Dump Tool

A Rust tool for extracting UTXO (Unspent Transaction Output) data from Bitcoin and Dogecoin chainstate LevelDB databases and exporting to CSV format.

## Chainstate Database Structure

The tool reads and decodes the following key-value entries:
- Obfuscation Key: `0x0e + "obfuscate_key"` → XOR key for UTXO value obfuscation
- UTXO (modern format): `0x43 + txid(32 bytes) + vout(varint)` → UTXO value (modern)
- UTXO (legacy format): `0x63 + txid(32 bytes)` → UTXO value (legacy)

Modern format has been in use since Bitcoin Core 0.15.0. Legacy format is still used in Dogecoin Core 1.14.9 and earlier.

## Usage

```bash
cargo run --bin utxo-dump -- \
  --db /path/to/chainstate \
  --output chainstate_utxos.csv \
  --blockchain bitcoin
```

## Output Format

The tool exports CSV with configurable fields. By default, it includes the following fields: `height`, `txid`, `vout`, `amount`, `type`, `address`, `script`, `coinbase`, and `nsize`.