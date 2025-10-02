#!/usr/bin/env bash
#
# Script for preparing the unstable blocks file and setting the chainstate database
# to the exact height needed.
set -euo pipefail

source "./utils.sh"

DOGECOIN_D="$1/bin/dogecoind"
DOGECOIN_CLI="$1/bin/dogecoin-cli"
NETWORK="$2"
HEIGHT="$3"

validate_file_exists "$DOGECOIN_D"
validate_file_exists "$DOGECOIN_CLI"
validate_network "$NETWORK"

# Kill all background processes on exit.
trap "kill 0" EXIT

# Create a temporary dogecoin.conf file with the required settings.
CONF_FILE=$(mktemp -u "dogecoin.conf.XXXXXX")
CONF_FILE_PATH="$DATA_DIR/$CONF_FILE"

generate_config "$NETWORK" "$CONF_FILE_PATH"

echo "Preparing the unstable blocks..."
# Start dogecoind in the background with no network access.
"$DOGECOIN_D" -conf="$CONF_FILE" -datadir="$DATA_DIR" -connect=0 > /dev/null &
DOGECOIND_PID=$!

# Wait for dogecoind to initialize.
echo "Waiting for dogecoind to load..."
sleep 30

# Fetch block hashes for unstable blocks.
echo "Fetching block hash at height $((HEIGHT + 1))..."
BLOCK_HASH_1=$("$DOGECOIN_CLI" -conf="$CONF_FILE" -datadir="$DATA_DIR" getblockhash $((HEIGHT + 1)))
echo "Hash: $BLOCK_HASH_1"

echo "Fetching block hash at height $((HEIGHT + 2))..."
BLOCK_HASH_2=$("$DOGECOIN_CLI" -conf="$CONF_FILE" -datadir="$DATA_DIR" getblockhash $((HEIGHT + 2)))
echo "Hash: $BLOCK_HASH_2"

# Save the unstable blocks to a file.
"$DOGECOIN_CLI" -conf="$CONF_FILE" -datadir="$DATA_DIR" getblock "$BLOCK_HASH_1" 0 > "$UNSTABLE_BLOCKS_FILE"
"$DOGECOIN_CLI" -conf="$CONF_FILE" -datadir="$DATA_DIR" getblock "$BLOCK_HASH_2" 0 >> "$UNSTABLE_BLOCKS_FILE"
echo "Unstable blocks saved to $UNSTABLE_BLOCKS_FILE."

# Invalidate the unstable blocks.
echo "Invalidating unstable blocks..."
"$DOGECOIN_CLI" -conf="$CONF_FILE" -datadir="$DATA_DIR" invalidateblock "$BLOCK_HASH_1"

# Compute checksum of the unstable blocks file.
echo "Computing checksum of unstable blocks..."
sha256sum "$UNSTABLE_BLOCKS_FILE"
echo "Done."

# Clean up.
kill "$DOGECOIND_PID"
wait "$DOGECOIND_PID" || true
