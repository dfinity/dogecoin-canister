#!/usr/bin/env bash
set -Eexuo pipefail

SCRIPT_DIR="$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )"
source "${SCRIPT_DIR}/utils.sh"

# Run dfx stop if we run into errors.
trap "dfx stop" EXIT SIGINT

dfx start --background --clean

# Deploy the dogecoin canister
dfx deploy --no-wallet dogecoin --argument "(record { })"

# Check the canister's metadata section for the Candid interface.
METADATA=$(dfx canister metadata dogecoin candid:service)

# Metadata returned should match the dogecoin canister's .did file.
DIFF_OUTPUT=$(diff "$SCRIPT_DIR/../canister/candid.did" <(echo "$METADATA"))

if [ "$DIFF_OUTPUT" != "" ]; then
  echo "FAIL"
  exit 1
fi

echo "SUCCESS"
