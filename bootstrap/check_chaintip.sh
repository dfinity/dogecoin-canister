#!/usr/bin/env bash
set -euo pipefail

source "./utils.sh"

DOGECOIN_D="$1/bin/dogecoind"
DOGECOIN_CLI="$1/bin/dogecoin-cli"
NETWORK="$2"

validate_file_exists "$DOGECOIN_D"
validate_file_exists "$DOGECOIN_CLI"
validate_network "$NETWORK"

# Kill all background processes on exit.
trap "kill 0" EXIT

# Create a temporary dogecoin.conf file with the required settings.
CONF_FILE=$(mktemp -u "dogecoin.conf.XXXXXX")
CONF_FILE_PATH="$DATA_DIR/$CONF_FILE"

generate_config "$NETWORK" "$CONF_FILE_PATH"

# Start dogecoind in the background with no network access.
echo "Starting dogecoind for $NETWORK..."
"$DOGECOIN_D" -conf="$CONF_FILE" -datadir="$DATA_DIR" -connect=0 > /dev/null &
DOGECOIND_PID=$!

# Wait for dogecoind to initialize.
echo "Waiting for dogecoind to load..."
until "$DOGECOIN_CLI" -conf="$CONF_FILE" -datadir="$DATA_DIR" getblockcount >/dev/null 2>&1; do
    sleep 5
done

# Get chain tips.
echo "Fetching chain tips for $NETWORK..."
"$DOGECOIN_CLI" -conf="$CONF_FILE" -datadir="$DATA_DIR" getchaintips

# Clean up.
kill "$DOGECOIND_PID"
wait "$DOGECOIND_PID" || true
echo "Done."
