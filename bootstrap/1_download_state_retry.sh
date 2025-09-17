#!/usr/bin/env bash
#
# Script for downloading the Dogecoin state.
set -euo pipefail

source "./utils.sh"

DOGECOIN_D="$1/bin/dogecoind"
NETWORK="$2"

validate_file_exists "$DOGECOIN_D"
validate_network "$NETWORK"

# Create a temporary dogecoin.conf file with the required settings.
CONF_FILE=$(mktemp)
generate_config "$NETWORK" "$CONF_FILE"

$DOGECOIN_D -conf="$CONF_FILE" -datadir="$DATA_DIR"
