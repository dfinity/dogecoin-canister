#!/bin/sh
# Start dogecoind in regtest mode, then hand off to the IC network launcher.
# dogecoind runs in the background; the launcher becomes PID 1 via exec.

dogecoind \
  -regtest -server \
  -port=18444 \
  -rpcbind=0.0.0.0 -rpcallowip=0.0.0.0/0 \
  -rpcuser=ic-doge-integration -rpcpassword=QPQiNaph19FqUsCrBRN0FII7lyM26B51fAMeBQzCb-E= \
  -txindex=1 -acceptnonstdtxn=1 &

# Wait for dogecoind to accept RPC connections
until dogecoin-cli -regtest \
  -rpcuser=ic-doge-integration -rpcpassword=QPQiNaph19FqUsCrBRN0FII7lyM26B51fAMeBQzCb-E= \
  getblockcount >/dev/null 2>&1; do
  sleep 0.5
done

echo "dogecoind ready on regtest"

# Hand off to the IC network launcher.
# --dogecoind-addr wires the IC Dogecoin subnet to our local dogecoind.
# Port 18444 is the regtest P2P port (used by the launcher for block discovery).
# The regtest RPC port is used by dogecoin-cli inside the container.
exec /app/icp-cli-network-launcher \
  --status-dir=/app/status \
  --config-port 4942 \
  --gateway-port 4943 \
  --bind 0.0.0.0 \
  --dogecoind-addr=127.0.0.1:18444 \
  "$@"
