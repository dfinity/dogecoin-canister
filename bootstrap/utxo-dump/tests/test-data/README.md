# Test Data for utxo-dump Integration Tests

This directory contains chainstate database files used for integration testing of the utxo-dump tool.

## Required Files

Place your chainstate databases as compressed tar.gz files:

- **`chainstate-btc-mainnet-250k.tar.gz`** - Bitcoin mainnet chainstate (~250k blocks) **[PRESENT]**
- **`chainstate.tar.gz`** - Dogecoin chainstate database (optional)
- **`chainstate-bitcoin.tar.gz`** - Bitcoin chainstate database (alternative)

## Creating Test Data

### From Dogecoin Node

1. Stop your Dogecoin node: `dogecoin-cli stop`
2. Navigate to your data directory (usually `~/.dogecoin/`)
3. Create compressed archive:
   ```bash
   tar -czf chainstate.tar.gz chainstate/
   ```
4. Copy to this directory:
   ```bash
   cp chainstate.tar.gz /path/to/utxo-dump/tests/test-data/
   ```

### From Bitcoin Node

1. Stop your Bitcoin node: `bitcoin-cli stop`  
2. Navigate to your data directory (usually `~/.bitcoin/`)
3. Create compressed archive:
   ```bash
   tar -czf chainstate-bitcoin.tar.gz chainstate/
   ```
4. Copy to this directory:
   ```bash
   cp chainstate-bitcoin.tar.gz /path/to/utxo-dump/tests/test-data/
   ```

## Recommendations for Test Data

### Size Considerations
- **Small test data**: Use a regtest or testnet chainstate (~1-100 MB)
- **Large test data**: Use mainnet chainstate (can be several GB)

### For Consistent Testing
- Use a specific block height snapshot
- Document the block height and hash in your test
- Keep test data size reasonable for CI/CD

### Example: Create Small Test Data

```bash
# Start dogecoin in regtest mode
dogecoind -regtest -daemon

# Generate some blocks with transactions
dogecoin-cli -regtest generatetoaddress 101 $(dogecoin-cli -regtest getnewaddress)

# Create some transactions
dogecoin-cli -regtest sendtoaddress $(dogecoin-cli -regtest getnewaddress) 1.0

# Generate more blocks to confirm
dogecoin-cli -regtest generatetoaddress 10 $(dogecoin-cli -regtest getnewaddress)

# Stop and archive
dogecoin-cli -regtest stop
tar -czf chainstate.tar.gz -C ~/.dogecoin/regtest chainstate/
```

## File Structure

The tar.gz files should contain:
```
chainstate/
├── CURRENT
├── LOCK  
├── LOG
├── MANIFEST-000001
├── 000003.ldb
├── 000004.ldb
└── ... (other LevelDB files)
```

## Verification

To verify your test data works:

```bash
# Run integration tests
cargo test --test integration_tests

# Run with output to see details
cargo test test_utxo_dump_output_hash -- --nocapture
```

## Notes

- Keep test data files small enough for version control if desired
- Consider using Git LFS for larger test data files
- Test data should contain at least one UTXO entry for meaningful testing
- The obfuscation key must be present in the chainstate for utxo-dump to work
