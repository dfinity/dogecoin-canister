# Computing the Dogecoin State

Rather than syncing the Dogecoin canister from genesis, which can take several weeks, the state of the canister can be
computed offline much more quickly with the help of `dogecoind`.

## Requirements

* A linux machine
* \>= 16GiB RAM
* \>= 100GB of disk space

## 1. Download Dogecoin Core

Go to `bootstrap` directory:

```shell
cd ./bootstrap
```

Download Dogecoin Core 1.14.9

```shell
curl -L -O https://github.com/dogecoin/dogecoin/releases/download/v1.14.9/dogecoin-1.14.9-x86_64-linux-gnu.tar.gz
```

Unpack the `tar.gz` file

```shell
tar -xvf dogecoin-1.14.9-x86_64-linux-gnu.tar.gz
```

## 2. Setup Environment Variables

```shell
DOGECOIN_DIR=./dogecoin-1.14.9
NETWORK=<mainnet or testnet>
HEIGHT=<height of the state you want to compute>
STABILITY_THRESHOLD=<desired stability threshold>
```

## 3. Download the Dogecoin state

Run `1_download_state.sh`, which downloads the dogecoin state. This can several hours.

```shell
./1_download_state.sh $DOGECOIN_DIR $NETWORK $HEIGHT
```

Once it's done, run the following:

```shell
./check_chaintip.sh $DOGECOIN_DIR $NETWORK
```

Make sure that the output of the above command specifies that you have a chain that has the status "active", and has a
height of at least `$HEIGHT + 10`. For example, if you set the `$HEIGHT` to 10000 in the earlier steps, the height of
the chain should be >= 10010. It should look something like this:

```shell
[
  {
    "height": <height>,
    "hash": "<block hash>",
    "branchlen": 0,
    "status": "active"
  }
]
```

## 4. Compute the Dogecoin Canister's State

```shell
./2_compute_unstable_blocks.sh $DOGECOIN_DIR $NETWORK $HEIGHT
```

```shell
./3_compute_block_headers.sh $DOGECOIN_DIR $NETWORK $HEIGHT
```

```shell
./4_compute_utxo_dump.sh $NETWORK
```

```shell
./5_shuffle_utxo_dump.sh
```

```shell
./6_compute_canister_state.sh $NETWORK $HEIGHT $STABILITY_THRESHOLD
```

(Optional) check output data size:

```shell
$ du -sh ./output/*
13M     ./output/block_headers
1.1G    ./output/canister_state
1.1G    ./output/canister_state.bin
2.4G    ./output/data
2.4G    ./output/data_bk
120K    ./output/unstable_blocks
469M    ./output/utxodump.csv
469M    ./output/utxodump_shuffled.csv
```

Once all these steps are complete, the canister's state will be available in this directory with the name
`canister_state.bin`.

## 5. Compute the State Hashes

A canister's state is uploaded in "chunks" through ingress messages to the `uploader` canister.
The required chunk hashes can be computed as follows:

```shell
cargo run --release --example compute_hashes -- --file ./output/canister_state.bin > chunk_hashes.txt
```

The hashes of each chunk are saved in `./bootstrap/chunk_hashes.txt` and can be used later when building the `uploader`
canister in Docker.

## 6. Build Canisters

```shell
# Go back to root repo directory
$ cd ..

# Specify the path to chunk_hashes.txt for building the uploader canister.
$ ./scripts/docker-build ic-doge-canister uploader --chunk-hashes ./bootstrap/chunk_hashes.txt

# Verify SHA-256 of the canister's WASM.
$ sha256sum *.wasm.gz
c6abf3605cd33d0d640a648ecc1aaf33999032775436481485468a75024f38bc  ic-doge-canister.wasm.gz
2f9a1f7ee91ce2e2c29cc78040197b2687c25ac7fd76a609c79a72c67e3ca1d8  uploader.wasm.gz
```

## 7. (Optional) Setup Testing Subnet & Create Canisters

When installing canister on a testnet first start a farm testnet via `$ ict testnet create`:

```shell
# In a separate terminal and in separate folder clone IC-repo
$ git clone git@github.com:dfinity/ic.git
$ cd ic

# If you are on remote machine make sure to propagate your credentials (otherwise grafana will not start)
$ ssh-add -L

# Start a container to run a testnet inside
$ ./ci/container/container-run.sh

# Before starting the testnet double check `small_bitcoin` testnet settings.
# https://github.com/dfinity/ic/blob/256c598835d637b0b58c5e2117bca011ec417a61/rs/tests/testnets/small_bitcoin.rs#L2
# Setup lifetime big enough for your experiment, provide output directory and log file
$ clear; ict testnet create small_bitcoin --lifetime-mins=10080 --output-dir=./test_tmpdir \
  > output.secret

# Same but with custom grafana dashboards
$ clear; ict testnet create small_bitcoin --lifetime-mins=10080 --output-dir=./test_tmpdir \
  --k8s-branch <repo-branch-name> \
  > output.secret
```

In the `output.secret` file find and save system subnet IPv6 and links to grafana

```shell
      {
        "nodes": [
          {
            ...
            "ipv6": "2602:xx:xx:xx:xx:xx:xx:df47" # <- YOU NEED THIS IPv6 OF SYSTEM NODE
          }
        ],
        ...
        "subnet_type": "system"
      },
  ...
  "grafana": "Grafana at http://grafana.XXX", # <- YOU NEED THIS URL
```

Update the `dfx.json` file (under `deployment/mainnet` or `deployment/testnet`) with the IPv6 address provided above:

```json
    "testnet": {
"providers": [
"http://[2602:xx:xx:xx:xx:xx:xx:df47]:8080" // <- USE IPv6 FROM THE ABOVE
],
"type": "persistent"
}
```

```shell
# Helper constants
NETWORK=mainnet; \
  STABILITY_THRESHOLD=1440; \
  MAINNET_DOGECOIN_CANISTER_ID="gordg-fyaaa-aaaan-aaadq-cai"; \
```

Create corresponding canister

```shell
$ cd deployment/mainnet

$ dfx canister create dogecoin --no-wallet \
    --network testnet \
    --subnet-type system \
    --specified-id $MAINNET_DOGECOIN_CANISTER_ID \
    --provisional-create-canister-effective-canister-id "5v3p4-iyaaa-aaaaa-qaaaa-cai" \
    --with-cycles 1000000000000000000
```

## 8. Install Uploader Canister & Upload Chunks

Prepare install arguments
```shell
# Get canister state size
$ wc -c < ../../bootstrap/output/canister_state.bin
1149304832
```

Calculate required number of pages, page is `64 * 1024` bytes
```txt
ceil(1149304832 / (64 * 1024)) = 17537
```

Calculate args hash
```shell
$ didc encode -t '(nat64)' "(17537)" | xxd -r -p | sha256sum
e299fbe18558a3646ab33e5d28eec04e474339f235cf4f22dd452c98f831a249  -
```

Install uploader canister
```shell
$ dfx canister install \
    --network testnet dogecoin \
    --mode reinstall \
    --wasm ../../uploader.wasm.gz \
    --argument "(17537 : nat64)"  # Use calculated number of pages.
```

Upload chunks
```shell
# USE IPv6 FROM THE ABOVE
$ cargo run --example upload -- \
    --canister-id $MAINNET_DOGECOIN_CANISTER_ID \
    --state ../../bootstrap/output/canister_state.bin \
    --ic-network http://\[2602:xx:xx:xx:xx:xx:xx:df47\]:8080 \
    --fetch-root-key
```

## 9. Upgrade Dogecoin Canister

Prepare upgrade arguments
```shell
# https://internetcomputer.org/docs/references/bitcoin-how-it-works#api-fees-and-pricing
$ CUSTOM_FEES="record { 
  get_utxos_base = 50_000_000 : nat;
  get_utxos_cycles_per_ten_instructions = 10 : nat;
  get_utxos_maximum = 10_000_000_000 : nat;
  get_current_fee_percentiles = 10_000_000 : nat;
  get_current_fee_percentiles_maximum = 100_000_000 : nat;
  get_balance = 10_000_000 : nat;
  get_balance_maximum = 100_000_000 : nat;
  send_transaction_base = 5_000_000_000 : nat;
  send_transaction_per_byte = 20_000_000 : nat;
  get_block_headers_base = 50_000_000 : nat;
  get_block_headers_cycles_per_ten_instructions = 10 : nat;
  get_block_headers_maximum = 10_000_000_000 : nat;
}"

# Prepare the argument for the `post_upgrade` call (make sure to match current prod configuration).
# This config optionally updates settings after canister upgrade.
# Fields not set will keep their current values (which is default after uploader canister).
# Make sure to use correct Candid syntax for `nat` values: https://internetcomputer.org/docs/references/candid-ref#textual-syntax-4
$ POST_UPGRADE_ARG="(opt record {
    network = opt variant { $NETWORK };
    stability_threshold = opt ($STABILITY_THRESHOLD : nat);
    syncing = opt variant { enabled };
    api_access = opt variant { disabled };
    fees = opt $CUSTOM_FEES;
})"
```

```shell
$ didc encode -d ../../canister/candid.did -t '(opt set_config_request)' "$POST_UPGRADE_ARG" | xxd -r -p | sha256sum
6d3bcdfdefaf3dd444a218735277f6d1cba15196d09b9544b7a04dbc3c36642f  -
```

Upgrade Dogecoin canister

```shell
$ dfx canister stop --network testnet dogecoin

$ dfx canister install \
    --network testnet dogecoin \
    --mode upgrade \
    --wasm ../../ic-doge-canister.wasm.gz \
    --argument "$POST_UPGRADE_ARG"

$ dfx canister start --network testnet dogecoin
```
