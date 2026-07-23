#!/usr/bin/env bash
set -e

IMAGE_NAME=icp-cli-network-launcher-dogecoin
# Find the running container built from our custom image
DOGECOIN_CONTAINER=$(docker ps --filter "ancestor=${IMAGE_NAME}" --format "{{.ID}}" | head -1)

# dogecoin-cli invocation inside the container (regtest, with RPC credentials)
doge_cli() {
  docker exec "$DOGECOIN_CONTAINER" dogecoin-cli -regtest \
    -rpcuser=ic-doge-integration -rpcpassword=QPQiNaph19FqUsCrBRN0FII7lyM26B51fAMeBQzCb-E= "$@"
}

# Fail with a message.
fail() {
  echo "FAIL: $1"
  exit 1
}

# Wait for the canister to be ready to serve calls; the HTTP gateway can lag
# briefly right after deploy, making the very first call fail transiently.
echo "=== Waiting for canister to be ready ==="
ready=false
for _ in $(seq 1 30); do
  if icp canister call backend get_p2pkh_address '()' >/dev/null 2>&1; then
    ready=true
    break
  fi
  sleep 1
done
[ "$ready" = true ] || fail "canister not ready within 30s"

echo "=== Test 1: get_p2pkh_address returns a valid Dogecoin address ==="
result=$(icp canister call backend get_p2pkh_address '()')
echo "$result"
echo "$result" | grep -q '"' || fail "no address returned"
echo "PASS"

echo "=== Test 2: get_current_fee_percentiles returns a vec ==="
result=$(icp canister call backend get_current_fee_percentiles '()')
echo "$result"
echo "$result" | grep -q 'vec' || fail "no vec returned"
echo "PASS"

echo "=== Mining 101 blocks to fund test address ==="
[ -n "$DOGECOIN_CONTAINER" ] || fail "network launcher container not running — run 'icp network start -d' first"
addr=$(icp canister call backend get_p2pkh_address '()' | grep -o '"[^"]*"' | tr -d '"')
doge_cli generatetoaddress 101 "$addr" > /dev/null
echo "mined 101 blocks to $addr"

echo "=== Waiting for IC to sync Dogecoin blocks ==="
sync_addr=$(icp canister call backend get_p2pkh_address '()' | grep -o '"[^"]*"' | tr -d '"')
# Poll get_utxos until it succeeds AND reports a non-zero tip_height.
# Using get_utxos (not get_balance) because it reflects the definitive sync state:
# get_balance can return stale non-zero values from previous runs while the dogecoin
# integration canister is still syncing new blocks and rejecting all fresh calls.
synced=false
for _ in $(seq 1 60); do
  if result=$(icp canister call backend get_utxos "(\"$sync_addr\")" 2>/dev/null) &&
     echo "$result" | grep -qE 'tip_height = [1-9][0-9]*'; then
    synced=true
    break
  fi
  sleep 1
done
[ "$synced" = true ] || fail "IC did not sync within 60s"
echo "IC synced"

echo "=== Test 3: get_balance returns non-zero after mining ==="
addr=$(icp canister call backend get_p2pkh_address '()' | grep -o '"[^"]*"' | tr -d '"')
result=$(icp canister call backend get_balance "(\"$addr\")")
echo "$result"
echo "$result" | grep -qE '^\([1-9]' || fail "balance is zero"
echo "PASS"

echo "=== Test 4: get_utxos returns synced chain state after mining ==="
addr=$(icp canister call backend get_p2pkh_address '()' | grep -o '"[^"]*"' | tr -d '"')
result=$(icp canister call backend get_utxos "(\"$addr\")")
echo "$result"
echo "$result" | grep -qE 'tip_height = [1-9][0-9]*' || fail "no synced tip_height"
echo "PASS"

echo "=== Test 5: get_block_headers returns headers ==="
result=$(icp canister call backend get_block_headers '(0: nat32, null)')
echo "$result"
echo "$result" | grep -q 'tip_height' || fail "no block headers returned"
echo "PASS"
