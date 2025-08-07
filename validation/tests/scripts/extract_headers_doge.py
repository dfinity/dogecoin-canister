"""
Dogecoin Block Header Extractor with AuxPow Support

This script extracts block headers from Dogecoin blockchain data files (blk*.dat) and
reconstructs the blockchain chain starting from the genesis block. It can output headers
in either raw hex format or parsed format, and supports auxiliary proof of work (auxpow).

The script supports mainnet, testnet, and regtest networks. It reads binary blockchain data,
identifies blocks using the appropriate network magic bytes, extracts 80-byte headers, and
builds a chain by following previous block hash references.

For blocks with auxpow (auxiliary proof of work), the script automatically detects them
based on the version field and extracts the auxpow data:
- Raw mode: outputs header + auxpow data concatenated as hex
- Parsed mode: outputs header fields plus 7 separate auxpow columns:
  coinbase_tx, parent_hash, coinbase_branch, coinbase_index,
  blockchain_branch, blockchain_index, parent_block_header

Since each blk*.dat file contains a limited number of blocks, you may need to provide
multiple files to cover your desired block range.

Usage:
    python extract_headers_doge.py <blk_file1> [blk_file2 ...] [-o output_file] [--parsed] [--start-block N] [--end-block N] [--network {mainnet,testnet,regtest}]

Example:
    python extract_headers_doge.py blk00000.dat --end-block 5000 -o data/block_headers_mainnet_doge.csv
    python extract_headers_doge.py blk00000.dat --start-block 1 --end-block 5000 -o data/headers_doge_1_5000.csv --parsed

Note: AuxPow (Auxiliary Proof of Work) blocks are automatically detected and handled.
      In raw mode, auxpow data is concatenated to the header hex.
      In parsed mode, auxpow data is split into 7 separate columns:
      coinbase_tx, parent_hash, coinbase_branch, coinbase_index,
      blockchain_branch, blockchain_index, parent_block_header
"""

import struct
import sys
import hashlib
import csv
import argparse

NETWORKS = {
    'mainnet': {
        'magic': b'\xc0\xc0\xc0\xc0',
        'genesis_hash': bytes.fromhex('1a91e3dace36e2be3bf030a65679fe821aa1d6ef92e7c9902eb318182c355691')[::-1]
    },
    'testnet': {
        'magic': b'\xfc\xc1\xb7\xdc',
        'genesis_hash': bytes.fromhex('bb0a78264637406b6360aad926284d544d7049f45189db5664f3c4d07350559e')[::-1]
    },
    'regtest': {
        'magic': b'\xfa\xbf\xb5\xda',
        'genesis_hash': bytes.fromhex('3d2160a3b5dc4a9d62e7e66a295f70313ac808440ef7400d6c0772171ce973a5')[::-1]
    }
}

HEADER_SIZE = 80

def double_sha256(b):
    return hashlib.sha256(hashlib.sha256(b).digest()).digest()

def has_auxpow(version):
    """
    Check if a block version indicates it has auxiliary proof of work.

    Args:
        version (int): Block version as 32-bit integer

    Returns:
        bool: True if block has auxpow, False otherwise
    """
    # Check if auxpow flag (0x100) is set in version
    return (version & 0x100) != 0

def read_varint(data, offset):
    """
    Read a variable-length integer from binary data.

    Args:
        data (bytes): Binary data to read from
        offset (int): Starting offset in data

    Returns:
        tuple: (value, new_offset) where value is the parsed integer
               and new_offset is the position after the varint
    """
    if offset >= len(data):
        return 0, offset

    first_byte = data[offset]

    if first_byte < 0xfd:
        return first_byte, offset + 1
    elif first_byte == 0xfd:
        if offset + 3 > len(data):
            return 0, offset
        return struct.unpack('<H', data[offset + 1:offset + 3])[0], offset + 3
    elif first_byte == 0xfe:
        if offset + 5 > len(data):
            return 0, offset
        return struct.unpack('<I', data[offset + 1:offset + 5])[0], offset + 5
    else:  # first_byte == 0xff
        if offset + 9 > len(data):
            return 0, offset
        return struct.unpack('<Q', data[offset + 1:offset + 9])[0], offset + 9

def extract_auxpow_data(block_data, header_version, parsed_mode=False):
    """
    Extract auxpow data from block if present.

    Args:
        block_data (bytes): Full block data starting from header
        header_version (int): Block version to check for auxpow flag
        parsed_mode (bool): If True, return parsed components; if False, return raw hex

    Returns:
        For parsed_mode=False: tuple (auxpow_data_hex, total_size)
        For parsed_mode=True: tuple (auxpow_components, total_size) where auxpow_components is dict
    """
    if not has_auxpow(header_version):
        if parsed_mode:
            return {"coinbase_tx": "", "parent_hash": "", "coinbase_branch": "",
                   "coinbase_index": "", "blockchain_branch": "", "blockchain_index": "",
                   "parent_block_header": ""}, HEADER_SIZE
        else:
            return "", HEADER_SIZE

    # AuxPow data starts after the 80-byte header
    auxpow_start = HEADER_SIZE
    
    try:
        if len(block_data) < auxpow_start + 1:
            raise ValueError("Block too short for auxpow data")
    
        # Parse the auxpow structure:
        # 1. Coinbase transaction (variable length)
        # 2. Parent hash (32 bytes)
        # 3. Coinbase merkle branch (variable length array)
        # 4. Coinbase merkle index (4 bytes)
        # 5. Blockchain merkle branch (variable length array)
        # 6. Blockchain merkle index (4 bytes)
        # 7. Parent block header (80 bytes)

        offset = auxpow_start
        coinbase_start = offset

        # Read coinbase transaction - starts with version (4 bytes)
        if offset + 4 > len(block_data):
            raise ValueError("Not enough data for coinbase version")

        coinbase_version = struct.unpack('<I', block_data[offset:offset + 4])[0]
        offset += 4

        # Read input count
        input_count, offset = read_varint(block_data, offset)

        # Skip inputs
        for _ in range(input_count):
            # Previous transaction hash (32 bytes) + output index (4 bytes)
            offset += 36
            if offset > len(block_data):
                raise ValueError("Not enough data for input")

            # Script length and script
            script_len, offset = read_varint(block_data, offset)
            offset += script_len
            if offset > len(block_data):
                raise ValueError("Not enough data for input script")

            # Sequence (4 bytes)
            offset += 4
            if offset > len(block_data):
                raise ValueError("Not enough data for sequence")

        # Read output count
        output_count, offset = read_varint(block_data, offset)

        # Skip outputs
        for _ in range(output_count):
            # Value (8 bytes)
            offset += 8
            if offset > len(block_data):
                raise ValueError("Not enough data for output value")

            # Script length and script
            script_len, offset = read_varint(block_data, offset)
            offset += script_len
            if offset > len(block_data):
                raise ValueError("Not enough data for output script")

        # Lock time (4 bytes)
        offset += 4
        if offset > len(block_data):
            raise ValueError("Not enough data for lock time")

        # Extract coinbase transaction
        coinbase_tx = block_data[coinbase_start:offset]

        # Parent hash (32 bytes)
        if offset + 32 > len(block_data):
            raise ValueError("Not enough data for parent hash")
        parent_hash = block_data[offset:offset + 32]
        offset += 32

        # Coinbase merkle branch count and hashes
        coinbase_branch_start = offset
        coinbase_merkle_count, offset = read_varint(block_data, offset)
        offset += coinbase_merkle_count * 32
        if offset > len(block_data):
            raise ValueError("Not enough data for coinbase merkle branch")
        coinbase_branch = block_data[coinbase_branch_start:offset]

        # Coinbase merkle index (4 bytes)
        if offset + 4 > len(block_data):
            raise ValueError("Not enough data for coinbase index")
        coinbase_index = block_data[offset:offset + 4]
        offset += 4

        # Blockchain merkle branch count and hashes
        blockchain_branch_start = offset
        blockchain_merkle_count, offset = read_varint(block_data, offset)
        offset += blockchain_merkle_count * 32
        if offset > len(block_data):
            raise ValueError("Not enough data for blockchain merkle branch")
        blockchain_branch = block_data[blockchain_branch_start:offset]

        # Blockchain merkle index (4 bytes)
        if offset + 4 > len(block_data):
            raise ValueError("Not enough data for blockchain index")
        blockchain_index = block_data[offset:offset + 4]
        offset += 4

        # Parent block header (80 bytes)
        if offset + 80 > len(block_data):
            raise ValueError("Not enough data for parent block header")
        parent_block_header = block_data[offset:offset + 80]
        offset += 80

        if parsed_mode:
            return {
                "coinbase_tx": coinbase_tx.hex(),
                "parent_hash": parent_hash[::-1].hex(),  # Reverse for display (little->big endian)
                "coinbase_branch": coinbase_branch.hex(),
                "coinbase_index": coinbase_index.hex(),
                "blockchain_branch": blockchain_branch.hex(),
                "blockchain_index": blockchain_index.hex(),
                "parent_block_header": parent_block_header.hex()
            }, offset
        else:
            # Extract all auxpow data for raw mode
            auxpow_data = block_data[auxpow_start:offset]
            return auxpow_data.hex(), offset

    except (struct.error, ValueError, IndexError) as e:
        # If we can't parse the auxpow data, raise error to stop script execution
        raise ValueError(f"Failed to parse auxpow data: {str(e)}. "
                        f"Block may be corrupted or have unsupported auxpow format.")

def read_headers_from_blk_files(file_paths, include_parsed_headers, network_config):
    """
    Extract all block headers from multiple Dogecoin blockchain data files (blk*.dat).
    
    Reads multiple binary .dat files, searches for magic bytes to identify blocks,
    extracts the 80-byte headers, and builds a mapping of block hashes to
    their previous block hashes for chain reconstruction.
    
    Args:
        file_paths (list): List of paths to the blk*.dat files
        include_parsed_headers (bool): If True, parse headers into readable format;
                                     if False, keep as raw bytes
        network_config (dict): Network configuration containing magic bytes and genesis hash
    
    Returns:
        tuple: (headers, next_hash)
            - headers: Maps block_hash -> parsed_header_tuple or hex_string
            - next_hash: Maps previous_block_hash -> block_hash for chain building
    """
    headers = {}
    prev_hash = {}
    
    for file_path in file_paths:
        print(f"Reading from {file_path}...")
        file_headers, file_prev_hash = read_headers_from_single_blk(file_path, include_parsed_headers, network_config)
        headers.update(file_headers)
        prev_hash.update(file_prev_hash)
        print(f"  Found {len(file_headers)} headers in {file_path}")
    
    # Convert prev_hash mapping to next_hash mapping
    next_hash = {}
    for curr_hash, prev_block_hash in prev_hash.items():
        next_hash[prev_block_hash] = curr_hash

    return headers, next_hash

def read_headers_from_single_blk(file_path, include_parsed_headers, network_config):
    """
    Extract all block headers from a single Dogecoin blockchain data file (blk*.dat).
    
    Reads the binary .dat file, searches for magic bytes to identify blocks,
    extracts the 80-byte headers, and builds a mapping of block hashes to
    their previous block hashes for chain reconstruction.
    
    Args:
        file_path (str): Path to the blk*.dat file
        include_parsed_headers (bool): If True, parse headers into readable format;
                                     if False, store as hex strings
        network_config (dict): Network configuration containing magic bytes and genesis hash
    
    Returns:
        tuple: (headers, prev_hash)
            - headers: Maps block_hash -> parsed_header_tuple or hex_string
            - prev_hash: Maps block_hash -> previous_block_hash for chain building
    """
    headers = {}
    prev_hash = {}
    magic_bytes = network_config['magic']
    
    with open(file_path, 'rb') as f:
        data = f.read()
    offset = 0

    # Scan through file looking for magic bytes that indicate block starts
    while offset < len(data):
        magic_pos = data.find(magic_bytes, offset)
        if magic_pos == -1:
            break  # No more blocks found
            
        # Ensure we have enough data for the block size field
        if magic_pos + 8 > len(data):
            break

        # Read block size (4 bytes after magic, little-endian)
        block_size = struct.unpack('<I', data[magic_pos + 4:magic_pos + 8])[0]
        block_start = magic_pos + 8
        block_end = block_start + block_size

        # Ensure block doesn't extend beyond file
        if block_end > len(data):
            break

        # Extract the block data
        block = data[block_start:block_end]
        if len(block) < HEADER_SIZE:
            break  # Invalid block - too small for header

        # Extract the 80-byte header (always at start of block)
        header = block[:HEADER_SIZE]
        
        # Parse version to check for auxpow
        version = struct.unpack('<I', header[0:4])[0]

        # Calculate block hash using double SHA-256 (Bitcoin/Dogecoin standard)
        block_hash = double_sha256(header)

        # Extract auxpow data if present
        try:
            if include_parsed_headers:
                auxpow_data, total_header_size = extract_auxpow_data(block, version, parsed_mode=True)
                parsed_header = parse_block_header(header)
                # Add auxpow components as additional fields for parsed headers
                headers[block_hash] = (*parsed_header,
                                     auxpow_data["coinbase_tx"],
                                     auxpow_data["parent_hash"],
                                     auxpow_data["coinbase_branch"],
                                     auxpow_data["coinbase_index"],
                                     auxpow_data["blockchain_branch"],
                                     auxpow_data["blockchain_index"],
                                     auxpow_data["parent_block_header"])
            else:
                # For raw headers, concatenate header with auxpow data
                auxpow_hex, total_header_size = extract_auxpow_data(block, version, parsed_mode=False)
                if auxpow_hex:
                    headers[block_hash] = header.hex() + auxpow_hex
                else:
                    headers[block_hash] = header.hex()
        except ValueError as e:
            # Add context about which block failed
            block_hash_hex = block_hash.hex()
            raise ValueError(f"Error processing block {block_hash_hex} in file {file_path}: {str(e)}")

        # Map block hash to previous block hash extracted from header (bytes 4-36)
        prev_hash[block_hash] = header[4:36]

        # Move to next potential block
        offset = block_end

    return headers, prev_hash

def reconstruct_chain(headers, next_hash, genesis_hash, start_block=0, end_block=None):
    """
    Reconstruct the blockchain chain starting from genesis block.
    
    Args:
        headers (dict): Map of block_hash -> header_data
        next_hash (dict): Map of previous_block_hash -> block_hash
        genesis_hash (bytes): Hash of the genesis block to start from
        start_block (int): Block number to start extraction from (0-indexed)
        end_block (int): Block number to end extraction at (inclusive), None for no limit
    
    Returns:
        list: Ordered list of headers within the specified range
    """
    chain = []
    current_hash = genesis_hash
    block_number = 0

    # Follow the chain by finding blocks that reference the current block as previous
    while True:
        # Get header for current block
        header = headers.get(current_hash)
        if not header:
            if block_number < start_block:
                print(f"Warning: Chain ends at block {block_number}, but start_block is {start_block}")
                print("You may need to provide additional blk*.dat files to cover the desired range.")
            break  # Block not found - end of available chain
            
        # Check if we're within the desired range
        if block_number >= start_block:
            if end_block is None or block_number <= end_block:
                chain.append(header)
            elif block_number > end_block:
                break  # Reached end of desired range
        
        # Find the next block in the chain
        current_hash = next_hash.get(current_hash)
                
        if not current_hash:
            if end_block is not None and block_number < end_block:
                print(f"Warning: Chain ends at block {block_number}, but end_block is {end_block}")
                print("You may need to provide additional blk*.dat files to cover the desired range.")
            break  # No successor found - end of chain

        block_number += 1

    return chain

def parse_block_header(data):
    return (
        data[0:4][::-1].hex(),   # version
        data[4:36][::-1].hex(),  # prev_block
        data[36:68][::-1].hex(), # merkle_root
        data[68:72][::-1].hex(), # timestamp
        data[72:76][::-1].hex(), # bits
        data[76:80][::-1].hex(), # nonce
    )

def write_headers_to_csv(headers, output_file, include_header):
    if include_header:
        with open(output_file, 'w', newline='') as f:
            writer = csv.writer(f)
            writer.writerow(['version', 'prev_block', 'merkle_root', 'timestamp', 'bits', 'nonce',
                           'auxpow_coinbase_tx', 'auxpow_parent_hash', 'auxpow_coinbase_branch', 'auxpow_coinbase_index',
                           'auxpow_blockchain_branch', 'auxpow_blockchain_index', 'auxpow_parent_block_header'])
            for header in headers:
                writer.writerow(header)
    else:
        with open(output_file, 'w', newline='\n') as f:
            for header in headers:
                f.write(header + '\n')

def main():
    parser = argparse.ArgumentParser(description='Extract Dogecoin block headers from blk file(s)')
    parser.add_argument('blk_files', nargs='+', help='Path(s) to the blk*.dat file(s)')
    parser.add_argument('-o', '--output', required=True,
                       help='Output file name (required)')
    parser.add_argument('--parsed', 
                       action='store_true', 
                       help='Output parsed headers in CSV format (default: raw hex format)')
    parser.add_argument('--start-block', type=int, default=0,
                       help='Block number to start extraction from (default: 0)')
    parser.add_argument('--end-block', type=int, default=None,
                       help='Block number to end extraction at (inclusive, default: no limit)')
    parser.add_argument('--network', choices=['mainnet', 'testnet', 'regtest'], default='mainnet',
                       help='Dogecoin network (default: mainnet)')
    
    args = parser.parse_args()
    
    # Get network configuration
    network_config = NETWORKS[args.network]
    genesis_hash = network_config['genesis_hash']

    # Extract headers from the blockchain data files
    print(f"Reading blockchain data from {len(args.blk_files)} file(s) for {args.network}:")
    for f in args.blk_files:
        print(f"  - {f}")
    
    headers, next_hash = read_headers_from_blk_files(args.blk_files, args.parsed, network_config)
    print(f"Total parsed {len(headers)} headers from all files.")

    # Reconstruct the blockchain chain starting from genesis
    chain_headers = reconstruct_chain(headers, next_hash, genesis_hash, args.start_block, args.end_block)
    
    # Display range information
    if args.end_block is not None:
        print(f"Reconstructed chain with {len(chain_headers)} headers (blocks {args.start_block} to {args.end_block}).")
    else:
        print(f"Reconstructed chain with {len(chain_headers)} headers (starting from block {args.start_block}).")

    # Write output in requested format
    write_headers_to_csv(chain_headers, args.output, args.parsed)
    print(f"Written {len(chain_headers)} headers to CSV: {args.output}")

if __name__ == '__main__':
    try:
        main()
    except ValueError as e:
        print(f"Error: {str(e)}", file=sys.stderr)
        sys.exit(1)
    except Exception as e:
        print(f"Unexpected error: {str(e)}", file=sys.stderr)
        sys.exit(1)
