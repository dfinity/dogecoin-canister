"""
Dogecoin Block Header Extractor

This script extracts block headers from Dogecoin blockchain data files (blk*.dat) and
reconstructs the blockchain chain starting from the genesis block. It can output headers
in either raw hex format or parsed format.

The script supports mainnet, testnet, and regtest networks. It reads binary blockchain data,
identifies blocks using the appropriate network magic bytes, extracts 80-byte headers, and
builds a chain by following previous block hash references.

Since each blk*.dat file contains a limited number of blocks, you may need to provide
multiple files to cover your desired block range.

Usage:
    python extract_headers_doge.py <blk_file1> [blk_file2 ...] [-o output_file] [--parsed] [--start-block N] [--end-block N] [--network {mainnet,testnet,regtest}]

Example:
    python extract_headers_doge.py blk00000.dat --end-block 5000 -o data/block_headers_mainnet_doge.csv
    python extract_headers_doge.py blk00000.dat --start-block 1 --end-block 5000 -o data/headers_doge_1_5000.csv --parsed
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
        
        # Calculate block hash using double SHA-256 (Bitcoin/Dogecoin standard)
        block_hash = double_sha256(header)

        # Store header (parsed tuple or hex string) and previous hash mapping
        if include_parsed_headers:
            headers[block_hash] = parse_block_header(header)
        else:
            headers[block_hash] = header.hex()

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
            writer.writerow(['version', 'prev_block', 'merkle_root', 'timestamp', 'bits', 'nonce'])
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
    main()
