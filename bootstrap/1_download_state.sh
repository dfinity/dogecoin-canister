#!/usr/bin/env bash
#
# Script for downloading the Dogecoin state up to a specified block height.
set -euo pipefail

source "./utils.sh"

DOGECOIN_D="$1/bin/dogecoind"
DOGECOIN_CLI="$1/bin/dogecoin-cli"
NETWORK="$2"
HEIGHT="$3"
# Blocks from height $((HEIGHT+1)) to $HEIGHT_STOP_SYNC will be considered unstable
HEIGHT_STOP_SYNC=$((HEIGHT + 12))

validate_file_exists "$DOGECOIN_D"
validate_network "$NETWORK"

# Check if the data directory already exists.
if [[ -d "$DATA_DIR" ]]; then
    echo "Error: The '$DATA_DIR' directory already exists. Please remove it or choose another directory."
    exit 1
fi
# Create the data directory (including parent directories if needed).
mkdir -p "$DATA_DIR"

# Generate a temporary dogecoin.conf file with required settings.
CONF_FILE=$(mktemp -u "dogecoin.conf.XXXXXX")
CONF_FILE_PATH="$DATA_DIR/$CONF_FILE"

generate_config "$NETWORK" "$CONF_FILE_PATH"
    # Dogecoin: there is no `stopatheight` option as of v1.14.9.
    # "# Stop running after reaching the given height in the main chain." \
    # "stopatheight=$HEIGHT"

# Log file for monitoring progress.
LOG_FILE=$(mktemp)
echo "Downloading Dogecoin blocks up to height $HEIGHT. Logs can be found in: $LOG_FILE"
echo "This may take several hours. Please wait..."

# Start the Dogecoin daemon.
"$DOGECOIN_D" -conf="$CONF_FILE" -datadir="$DATA_DIR" -printtoconsole > "$LOG_FILE" 2>&1 &
DOGECOIN_PID=$!

# Wait for the RPC interface to become ready
echo "Waiting for dogecoind to load..."
until "$DOGECOIN_CLI" -conf="$CONF_FILE" -datadir="$DATA_DIR" getblockcount >/dev/null 2>&1; do
    sleep 5
done

echo "Starting synchronization..."

last_printed=0
PRINT_EVERY=5000

# Poll until we reach the desired height
while true; do
    COUNT=$("$DOGECOIN_CLI" -conf="$CONF_FILE" -datadir="$DATA_DIR" getblockcount)
    multiple=$(( COUNT / PRINT_EVERY * PRINT_EVERY ))
    if (( multiple > last_printed )); then
        echo "Current block height: $multiple"
        last_printed=$multiple
    fi
    if [[ "$COUNT" -ge "$HEIGHT_STOP_SYNC" ]]; then
        break
    fi
    sleep 1
done

# Invalidate blocks appearing after block at height $HEIGHT_STOP_SYNC
BLOCK_HASH=$("$DOGECOIN_CLI" -conf="$CONF_FILE" -datadir="$DATA_DIR" getblockhash "$((HEIGHT_STOP_SYNC + 1))")
"$DOGECOIN_CLI" -conf="$CONF_FILE" -datadir="$DATA_DIR" invalidateblock "$BLOCK_HASH"

COUNT=$("$DOGECOIN_CLI" -conf="$CONF_FILE" -datadir="$DATA_DIR" getblockcount)
echo "Target height $HEIGHT reached. Stopping node..."

"$DOGECOIN_CLI" -conf="$CONF_FILE" -datadir="$DATA_DIR" stop

# Wait for daemon to exit cleanly
wait $DOGECOIN_PID

# Create a backup of the downloaded data.
echo "Creating a backup of the downloaded state in: $BACKUP_DIR"
cp -r "$DATA_DIR" "$BACKUP_DIR"
echo "Backup complete."
