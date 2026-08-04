#!/usr/bin/env bash

# Configure dfx.json to use pre-built WASM from wasms/ when present (e.g. in CI).
# When wasms/ is not present (local dev), dfx.json is left unchanged and the build step runs.
use_prebuilt_dogecoin_wasm() {
  if [[ -f ../wasms/ic-doge-canister.wasm.gz ]]; then
    sed -i.bak 's|"wasm": "../target/wasm32-unknown-unknown/release/ic-doge-canister.wasm.gz"|"wasm": "../wasms/ic-doge-canister.wasm.gz"|' dfx.json
    sed -i.bak 's|"build": "../scripts/build-canister.sh ic-doge-canister"|"build": "true"|' dfx.json
  fi
}

# Polls the canister's metrics until the gauge `METRIC` has reached at least `HEIGHT`,
# giving up after `ATTEMPTS` polls.
#
# NOTE: the comparison is `>=`, not `==`. The canister only passes *through* the
# intermediate heights on its way to the terminal one, and a once-per-second poll can
# easily miss a height that is only held for a fraction of a second.
wait_until_metric_at_least () {
  METRIC=$1
  HEIGHT=$2
  ATTEMPTS=$3

  DOGECOIN_CANISTER_ID=$(dfx canister id dogecoin)

  while
    METRICS=$(curl "http://$DOGECOIN_CANISTER_ID.raw.localhost:8000/metrics")
    # Metrics lines are `<name> <value> <timestamp>`; take the value of an exact name match.
    VALUE=$(echo "$METRICS" | awk -v name="$METRIC" '$1 == name { print $2; exit }')
    ! [[ "$VALUE" =~ ^[0-9]+$ ]] || (( VALUE < HEIGHT )); do
      # Assignment, not `((ATTEMPTS-=1))`: the latter returns 1 when the result is 0,
      # which under the callers' `set -e` would abort before the message below is printed.
      ATTEMPTS=$((ATTEMPTS - 1))

      if [[ $ATTEMPTS -eq 0 ]]; then
        echo "TIMED OUT waiting for $METRIC >= $HEIGHT (last value: ${VALUE:-none})"
        exit 1
      fi

      sleep 1
  done
}

# Waits until the main chain of the dogecoin canister has reached a certain height.
wait_until_main_chain_height () {
  wait_until_metric_at_least "main_chain_height" "$1" "$2"
}

# Waits until the stable chain of the Dogecoin canister has reached a certain height.
wait_until_stable_height () {
  wait_until_metric_at_least "stable_height" "$1" "$2"
}

# Returns the number of UTXOs found in a response.
num_utxos () {
  UTXOS=$1
  # Count the occurrences of a substring of a UTXO.
  echo "$UTXOS" | grep -o " height = " | wc -l | xargs echo
}
