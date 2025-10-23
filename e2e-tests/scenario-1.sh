#!/usr/bin/env bash
set -Eexuo pipefail

SCRIPT_DIR="$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )"
source "${SCRIPT_DIR}/utils.sh"
pushd "$SCRIPT_DIR"

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
BALANCE=$(dfx canister call dogecoin dogecoin_get_balance '(record {
  network = variant { regtest };
  address = "mhXcJVuNA48bZsrKq4t21jx1neSqyceqTM"
})')

if ! [[ $BALANCE = "(0 : nat)" ]]; then
  echo "FAIL"
  exit 1
fi

BALANCE=$(dfx canister call --query dogecoin dogecoin_get_balance_query '(record {
  network = variant { regtest };
  address = "mhXcJVuNA48bZsrKq4t21jx1neSqyceqTM"
})')

if ! [[ $BALANCE = "(0 : nat)" ]]; then
  echo "FAIL"
  exit 1
fi

# Fetch the balance of an address we expect to have funds.
BALANCE=$(dfx canister call dogecoin dogecoin_get_balance '(record {
  network = variant { regtest };
  address = "mwoouFKeAiPoLi2oVpiEVYeNZAiE81abto";
  min_confirmations = opt 2;
})')

# Verify that the balance is 50 DOGE.
if ! [[ $BALANCE = "(5_000_000_000 : nat)" ]]; then
  echo "FAIL"
  exit 1
fi

UTXOS=$(dfx canister call dogecoin dogecoin_get_utxos '(record {
  network = variant { regtest };
  address = "mwoouFKeAiPoLi2oVpiEVYeNZAiE81abto";
})')

# The address has no UTXOs.
if ! [[ $(num_utxos "$UTXOS") = 0 ]]; then
  echo "FAIL"
  exit 1
fi

UTXOS=$(dfx canister call --query dogecoin dogecoin_get_utxos_query '(record {
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
UTXOS=$(dfx canister call --query dogecoin dogecoin_get_utxos_query '(record {
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
UTXOS=$(dfx canister call dogecoin dogecoin_get_utxos_query '(record {
  network = variant { regtest };
  address = "mjCLh7tvtg92WfVgqBbqFd2DoJ86Jr6dFW"
})')

# The address has 10000 UTXOs, but the response is capped to 1000 UTXOs.
if ! [[ $(num_utxos "$UTXOS") = 1000 ]]; then
  echo "FAIL"
  exit 1
fi
set -x

# Check that 'dogecoin_get_utxos_query' cannot be called in replicated mode.
set +e
GET_UTXOS_QUERY_REPLICATED_CALL=$(dfx canister call --update dogecoin dogecoin_get_utxos_query '(record {
  network = variant { regtest };
  address = "mjCLh7tvtg92WfVgqBbqFd2DoJ86Jr6dFW";
})' 2>&1)
set -e

if [[ $GET_UTXOS_QUERY_REPLICATED_CALL != *"Canister rejected the message, error code Some(\"IC0406\")"* ]]; then
  echo "FAIL"
  exit 1
fi

BALANCE=$(dfx canister call --query dogecoin dogecoin_get_balance_query '(record {
  network = variant { regtest };
  address = "mjCLh7tvtg92WfVgqBbqFd2DoJ86Jr6dFW";
})')

if ! [[ $BALANCE = "(5_000_000_000 : nat)" ]]; then
  echo "FAIL"
  exit 1
fi

# Check that 'dogecoin_get_balance_query' cannot be called in replicated mode.
set +e
GET_BALANCE_QUERY_REPLICATED_CALL=$(dfx canister call --update dogecoin dogecoin_get_balance_query '(record {
  network = variant { regtest };
  address = "mjCLh7tvtg92WfVgqBbqFd2DoJ86Jr6dFW";
})' 2>&1)
set -e

if [[ $GET_BALANCE_QUERY_REPLICATED_CALL != *"Canister rejected the message, error code Some(\"IC0406\")"* ]]; then
  echo "FAIL"
  exit 1
fi

BALANCE=$(dfx canister call dogecoin dogecoin_get_balance '(record {
  network = variant { regtest };
  address = "mjCLh7tvtg92WfVgqBbqFd2DoJ86Jr6dFW";
})')

if ! [[ $BALANCE = "(5_000_000_000 : nat)" ]]; then
  echo "FAIL"
  exit 1
fi

BALANCE=$(dfx canister call --query dogecoin dogecoin_get_balance_query '(record {
  network = variant { regtest };
  address = "mjCLh7tvtg92WfVgqBbqFd2DoJ86Jr6dFW";
})')

if ! [[ $BALANCE = "(5_000_000_000 : nat)" ]]; then
  echo "FAIL"
  exit 1
fi

# Request the current fee percentiles. This is only for profiling purposes.
dfx canister call dogecoin dogecoin_get_current_fee_percentiles '(record {
  network = variant { regtest };
})'
dfx canister call dogecoin dogecoin_get_current_fee_percentiles '(record {
  network = variant { regtest };
})'

# Verify that we can fetch the block headers.
ACTUAL_HEADERS=$(dfx canister call dogecoin dogecoin_get_block_headers '(record {
  start_height = 0;
  network = variant { regtest };
})');

# The e2e-scenario-1 canister chains 5 blocks onto the genesis block.
EXPECTED_HEADERS='(
  record {
    tip_height = 5 : nat32;
    block_headers = vec {
      blob "\01\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\69\6a\d2\0e\2d\d4\36\5c\74\59\b4\a4\a5\af\74\3d\5e\92\c6\da\32\29\e6\53\2c\d6\05\f6\53\3f\2a\5b\da\e5\49\4d\ff\ff\7f\20\02\00\00\00";
      blob "\01\00\00\00\a5\73\e9\1c\17\72\07\6c\0d\40\f7\0e\44\08\c8\3a\31\70\5f\29\6a\e6\e7\62\9d\4a\dc\b5\a3\60\21\3d\df\13\8d\75\51\24\3d\59\81\b2\9d\15\be\9c\ec\74\5e\a1\9f\8e\fc\ed\ed\84\a0\32\ec\05\fa\d4\c5\f8\16\e6\49\4d\ff\ff\7f\20\00\00\00\00";
      blob "\01\00\00\00\0b\d4\85\8b\f0\8b\7c\07\1a\be\06\0b\17\4a\1d\40\0a\f6\ce\8b\68\8b\c1\9a\50\ea\88\88\ba\59\80\81\1a\cf\2b\8f\48\00\a5\41\80\2f\22\f6\6d\67\20\b7\90\4c\05\09\c4\8f\2e\fd\e4\60\3b\55\5e\67\8d\3d\52\e6\49\4d\ff\ff\7f\20\02\00\00\00";
      blob "\01\00\00\00\66\64\0c\f0\75\af\0f\73\97\e9\73\08\e6\69\06\65\9d\98\2c\ae\5f\cc\18\8c\8d\ba\49\bf\92\ce\d0\93\91\7e\b7\45\61\c9\d6\1c\3c\7f\3f\a1\46\07\5e\e2\f1\28\87\13\13\57\03\04\ca\87\09\83\d8\35\e9\b0\8e\e6\49\4d\ff\ff\7f\20\00\00\00\00";
      blob "\01\00\00\00\b9\24\26\a4\4c\c9\a6\1a\e4\ab\3b\bd\7e\9f\07\49\8b\be\97\4d\5f\f5\07\5e\6d\6e\af\43\a7\b9\26\f5\ca\20\9b\32\83\00\10\4e\ee\1a\bd\84\18\a8\14\3b\37\4d\1f\27\07\20\e9\90\a6\ed\59\4c\e2\e6\ac\61\ca\e6\49\4d\ff\ff\7f\20\00\00\00\00";
      blob "\01\00\00\00\63\90\2e\e5\5c\1b\f2\88\a1\ca\e2\68\b8\f5\3a\be\5b\e4\ab\76\f2\f0\d3\d1\6f\94\86\97\14\a2\39\ba\14\70\ae\27\8d\a9\68\c7\b1\04\96\2c\9b\6b\29\c3\aa\73\78\83\a5\21\8d\72\bb\06\54\bb\2d\4b\ab\f5\06\e7\49\4d\ff\ff\7f\20\03\00\00\00";
    };
  },
)'

if ! [[ $ACTUAL_HEADERS = "$EXPECTED_HEADERS" ]]; then
  echo "FAIL"
  exit 1
fi

echo "SUCCESS"
