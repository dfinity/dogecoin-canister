#!/usr/bin/env bash
#
# Script for dumping Dogecoin block headers into a file.
set -euo pipefail

source "./utils.sh"

DOGECOIN_D="$1/bin/dogecoind"
DOGECOIN_CLI="$1/bin/dogecoin-cli"
NETWORK="$2"
STABLE_HEIGHT="$3"

validate_file_exists "$DOGECOIN_D"
validate_file_exists "$DOGECOIN_CLI"
validate_network "$NETWORK"

# Kill all background processes on exit.
trap "kill 0" EXIT

# Create a temporary dogecoin.conf file with the required settings.
CONF_FILE=$(mktemp "dogecoin.conf.XXXXXX")
CONF_FILE_PATH="$DATA_DIR/$CONF_FILE"

generate_config "$NETWORK" "$CONF_FILE_PATH"

# Remove any previously computed block headers file.
rm -f "$BLOCK_HEADERS_FILE"

# Start dogecoind in the background with no network access.
echo "Starting dogecoind for $NETWORK..."
"$DOGECOIN_D" -conf="$CONF_FILE" -datadir="$DATA_DIR" -connect=0 > /dev/null &
DOGECOIND_PID=$!

# Wait for dogecoind to initialize.
echo "Waiting for dogecoind to load..."
until "$DOGECOIN_CLI" -conf="$CONF_FILE" -datadir="$DATA_DIR" getblockcount >/dev/null 2>&1; do
    sleep 5
done

# Function to format seconds as xxh xxm xxs.
format_time() {
    local total_seconds=$1
    local hours=$((total_seconds / 3600))
    local minutes=$(((total_seconds % 3600) / 60))
    local seconds=$((total_seconds % 60))
    printf "%02dh %02dm %02ds" "$hours" "$minutes" "$seconds"
}

# Start timer for ETA calculation.
START_TIME=$(date +%s)

# Retrieve block hashes and headers via dogecoin-cli with progress logging.
echo "Fetching block headers up to height $STABLE_HEIGHT..."
for ((height = 0; height <= STABLE_HEIGHT; height++)); do
    BLOCK_HASH=$("$DOGECOIN_CLI" -conf="$CONF_FILE" -datadir="$DATA_DIR" getblockhash "$height")
    BLOCK_HEADER=$("$DOGECOIN_CLI" -conf="$CONF_FILE" -datadir="$DATA_DIR" getblockheader "$BLOCK_HASH" false)
    PURE_HEADER="${BLOCK_HEADER:0:160}"

    # Append the block hash and header to the file.
    echo "$BLOCK_HASH,$PURE_HEADER" >> "$BLOCK_HEADERS_FILE"

    # Calculate and log progress every 100 blocks.
    if ((height % 100 == 0 || height == STABLE_HEIGHT)); then
        CURRENT_TIME=$(date +%s)
        ELAPSED_TIME=$((CURRENT_TIME - START_TIME))
        PROCESSED_COUNT=$((height + 1))
        TOTAL_COUNT=$((STABLE_HEIGHT + 1))
        PERCENTAGE=$((100 * PROCESSED_COUNT / TOTAL_COUNT))
        REMAINING_TIME=$((ELAPSED_TIME * (TOTAL_COUNT - PROCESSED_COUNT) / PROCESSED_COUNT))
        FORMATTED_ETA=$(format_time "$REMAINING_TIME")

        echo "Processed $PROCESSED_COUNT/$TOTAL_COUNT ($PERCENTAGE%) headers, ETA: $FORMATTED_ETA"
    fi
done

# Compute and display the checksum of the block headers file.
echo "Computing checksum of $BLOCK_HEADERS_FILE..."
sha256sum "$BLOCK_HEADERS_FILE"

# Clean up.
kill "$DOGECOIND_PID"
wait "$DOGECOIND_PID" || true
echo "Done."
