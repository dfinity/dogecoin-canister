#!/usr/bin/env bash
set -Eexuo pipefail

SCRIPT_DIR="$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )"
source "${SCRIPT_DIR}/utils.sh"

# Run dfx stop if we run into errors.
trap "dfx stop" EXIT SIGINT

dfx start --background --clean

# Deploy the canister that returns the blocks for scenario 1.
dfx deploy --no-wallet e2e-scenario-1

# Deploy the dogecoin canister, setting the blocks_source to be the source above.
dfx deploy --no-wallet dogecoin --argument "(record {
  stability_threshold = opt 2;
  network = opt variant { regtest };
  blocks_source = opt principal \"$(dfx canister id e2e-scenario-1)\";
})"

# Wait until the ingestion of stable blocks is complete.
wait_until_stable_height 3 60

# Fetch the balance of an address we do not expect to have funds.
BALANCE=$(dfx canister call dogecoin bitcoin_get_balance '(record {
  network = variant { regtest };
  address = "mhXcJVuNA48bZsrKq4t21jx1neSqyceqTM"
})')

if ! [[ $BALANCE = "(0 : nat64)" ]]; then
  echo "FAIL"
  exit 1
fi

BALANCE=$(dfx canister call --query dogecoin bitcoin_get_balance_query '(record {
  network = variant { regtest };
  address = "mhXcJVuNA48bZsrKq4t21jx1neSqyceqTM"
})')

if ! [[ $BALANCE = "(0 : nat64)" ]]; then
  echo "FAIL"
  exit 1
fi

# Fetch the balance of an address we expect to have funds.
BALANCE=$(dfx canister call dogecoin bitcoin_get_balance '(record {
  network = variant { regtest };
  address = "mwoouFKeAiPoLi2oVpiEVYeNZAiE81abto";
  min_confirmations = opt 2;
})')

# Verify that the balance is 50 DOGE.
if ! [[ $BALANCE = "(5_000_000_000 : nat64)" ]]; then
  echo "FAIL"
  exit 1
fi

UTXOS=$(dfx canister call dogecoin bitcoin_get_utxos '(record {
  network = variant { regtest };
  address = "mwoouFKeAiPoLi2oVpiEVYeNZAiE81abto";
})')

# The address has no UTXOs.
if ! [[ $(num_utxos "$UTXOS") = 0 ]]; then
  echo "FAIL"
  exit 1
fi

UTXOS=$(dfx canister call --query dogecoin bitcoin_get_utxos_query '(record {
  network = variant { regtest };
  address = "mwoouFKeAiPoLi2oVpiEVYeNZAiE81abto";
})')

# The address has no UTXOs.
if ! [[ $(num_utxos "$UTXOS") = 0 ]]; then
  echo "FAIL"
  exit 1
fi

# Verify that we are able to fetch the UTXOs of one address.
# We temporarily pause outputting the commands to the terminal as
# this command would print thousands of UTXOs.
set +x
UTXOS=$(dfx canister call --query dogecoin bitcoin_get_utxos_query '(record {
  network = variant { regtest };
  address = "mjCLh7tvtg92WfVgqBbqFd2DoJ86Jr6dFW"
})')

# The address has 10000 UTXOs, but the response is capped to 1000 UTXOs.
if ! [[ $(num_utxos "$UTXOS") = 1000 ]]; then
  echo "FAIL"
  exit 1
fi
set -x

set +x
UTXOS=$(dfx canister call dogecoin bitcoin_get_utxos_query '(record {
  network = variant { regtest };
  address = "mjCLh7tvtg92WfVgqBbqFd2DoJ86Jr6dFW"
})')

# The address has 10000 UTXOs, but the response is capped to 1000 UTXOs.
if ! [[ $(num_utxos "$UTXOS") = 1000 ]]; then
  echo "FAIL"
  exit 1
fi
set -x

# Check that 'bitcoin_get_utxos_query' cannot be called in replicated mode.
set +e
GET_UTXOS_QUERY_REPLICATED_CALL=$(dfx canister call --update dogecoin bitcoin_get_utxos_query '(record {
  network = variant { regtest };
  address = "mjCLh7tvtg92WfVgqBbqFd2DoJ86Jr6dFW";
})' 2>&1)
set -e

if [[ $GET_UTXOS_QUERY_REPLICATED_CALL != *"CanisterReject"* ]]; then
  echo "FAIL"
  exit 1
fi

BALANCE=$(dfx canister call --query dogecoin bitcoin_get_balance_query '(record {
  network = variant { regtest };
  address = "mjCLh7tvtg92WfVgqBbqFd2DoJ86Jr6dFW";
})')

if ! [[ $BALANCE = "(5_000_000_000 : nat64)" ]]; then
  echo "FAIL"
  exit 1
fi

# Check that 'bitcoin_get_balance_query' cannot be called in replicated mode.
set +e
GET_BALANCE_QUERY_REPLICATED_CALL=$(dfx canister call --update dogecoin bitcoin_get_balance_query '(record {
  network = variant { regtest };
  address = "mjCLh7tvtg92WfVgqBbqFd2DoJ86Jr6dFW";
})' 2>&1)
set -e

if [[ $GET_BALANCE_QUERY_REPLICATED_CALL != *"CanisterReject"* ]]; then
  echo "FAIL"
  exit 1
fi

BALANCE=$(dfx canister call dogecoin bitcoin_get_balance '(record {
  network = variant { regtest };
  address = "mjCLh7tvtg92WfVgqBbqFd2DoJ86Jr6dFW";
})')

if ! [[ $BALANCE = "(5_000_000_000 : nat64)" ]]; then
  echo "FAIL"
  exit 1
fi

BALANCE=$(dfx canister call --query dogecoin bitcoin_get_balance_query '(record {
  network = variant { regtest };
  address = "mjCLh7tvtg92WfVgqBbqFd2DoJ86Jr6dFW";
})')

if ! [[ $BALANCE = "(5_000_000_000 : nat64)" ]]; then
  echo "FAIL"
  exit 1
fi

# Request the current fee percentiles. This is only for profiling purposes.
dfx canister call dogecoin bitcoin_get_current_fee_percentiles '(record {
  network = variant { regtest };
})'
dfx canister call dogecoin bitcoin_get_current_fee_percentiles '(record {
  network = variant { regtest };
})'

# Verify that we can fetch the block headers.
ACTUAL_HEADERS=$(dfx canister call dogecoin bitcoin_get_block_headers '(record {
  start_height = 0;
  network = variant { regtest };
})');

# The e2e-scenario-1 canister chains 5 blocks onto the genesis block.
EXPECTED_HEADERS='(
  record {
    tip_height = 5 : nat32;
    block_headers = vec {
      blob "\01\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\69\6a\d2\0e\2d\d4\36\5c\74\59\b4\a4\a5\af\74\3d\5e\92\c6\da\32\29\e6\53\2c\d6\05\f6\53\3f\2a\5b\da\e5\49\4d\ff\ff\7f\20\02\00\00\00";
      blob "\05\00\62\00\a5\73\e9\1c\17\72\07\6c\0d\40\f7\0e\44\08\c8\3a\31\70\5f\29\6a\e6\e7\62\9d\4a\dc\b5\a3\60\21\3d\83\49\03\fe\0e\bd\a1\e3\2a\f3\74\ab\4e\8c\f1\7a\fb\39\03\7d\3a\87\e8\2a\f8\d6\3a\de\82\f2\9a\98\16\e6\49\4d\ff\ff\7f\20\03\00\00\00";
      blob "\05\00\62\00\3a\78\12\4e\93\05\7d\0f\6a\94\cb\51\76\77\40\74\b7\8f\b1\ea\3f\b9\51\be\d1\41\eb\6f\f7\31\88\82\22\be\96\05\81\28\cd\7a\f8\4d\48\32\8e\e7\9e\ae\fd\b8\68\4c\ac\7e\4e\81\a0\b4\7b\14\ce\4f\97\e0\52\e6\49\4d\ff\ff\7f\20\02\00\00\00";
      blob "\05\00\62\00\64\a4\7b\de\08\7a\65\8c\59\db\cc\71\cb\7e\42\cb\22\a5\db\ab\a6\98\30\2b\05\d6\4b\ed\ea\34\fb\0b\94\b9\e5\22\9c\fb\b2\70\17\4b\97\bd\3e\88\db\ce\88\8e\68\78\4d\fa\f8\17\06\f6\75\ff\29\1c\59\cb\8e\e6\49\4d\ff\ff\7f\20\00\00\00\00";
      blob "\05\00\62\00\6c\ad\65\3e\6b\65\ec\25\b0\e8\70\e7\96\ae\cc\c6\52\98\da\f0\10\a2\ab\f6\6e\4f\1d\29\74\f6\0c\75\be\e8\2c\a8\08\96\50\9f\44\f2\91\1d\c8\6c\ab\f8\08\51\f1\52\da\b3\56\3b\30\1f\16\e0\8b\5b\08\b3\ca\e6\49\4d\ff\ff\7f\20\01\00\00\00";
      blob "\05\00\62\00\33\77\86\fd\9c\88\fe\d5\5a\b6\15\a5\92\b0\35\b2\bd\c3\ce\90\07\09\d0\87\24\b8\0a\58\70\52\01\63\09\d2\a0\a2\b0\66\e5\6d\71\dd\29\60\c9\75\ea\30\2e\58\9d\cd\96\f9\6c\54\3f\b9\3d\67\b1\5d\1b\91\06\e7\49\4d\ff\ff\7f\20\01\00\00\00";
    };
  },
)'

if ! [[ $ACTUAL_HEADERS = "$EXPECTED_HEADERS" ]]; then
  echo "FAIL"
  exit 1
fi

echo "SUCCESS"
