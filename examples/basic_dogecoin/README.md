# Basic Dogecoin

This example demonstrates how a canister can receive and send dogecoin on the Internet Computer, using legacy (P2PKH) addresses.

## Architecture

For a deeper understanding of the ICP ↔ Dogecoin integration, see the [Dogecoin integration concepts](https://docs.internetcomputer.org/concepts/chain-fusion/dogecoin). The Dogecoin integration builds on the same machinery as the [Bitcoin integration](https://docs.internetcomputer.org/concepts/chain-fusion/bitcoin).

This example integrates with the Internet Computer's built-in:

- Threshold ECDSA ([`ecdsa_public_key`](https://docs.internetcomputer.org/references/ic-interface-spec/management-canister/#ic-ecdsa_public_key), [`sign_with_ecdsa`](https://docs.internetcomputer.org/references/ic-interface-spec/management-canister/#ic-sign_with_ecdsa)) — derives P2PKH addresses and signs transactions spending from them
- [Dogecoin canister](https://github.com/dfinity/dogecoin-canister/blob/master/INTERFACE_SPECIFICATION.md) — queries balances, UTXOs, fee percentiles, and block data; submits signed transactions to the Dogecoin network

## Deploying from ICP Ninja

This example can be deployed directly to the Internet Computer using ICP Ninja, where it connects to Dogecoin **mainnet**. Note: Canisters deployed via ICP Ninja remain live for 50 minutes after signing in with your Internet Identity.

[![](https://icp.ninja/assets/open.svg)](https://icp.ninja/editor?g=https://github.com/dfinity/dogecoin-canister/tree/master/examples/basic_dogecoin)

## Build and deploy from the command line

### Prerequisites

- [Node.js](https://nodejs.org/) v18+
- [icp-cli](https://cli.internetcomputer.org/): `npm install -g @icp-sdk/icp-cli @icp-sdk/ic-wasm`
- [Rust](https://www.rust-lang.org/tools/install) v1.85+ with `wasm32-unknown-unknown` target: `rustup target add wasm32-unknown-unknown`
- [Docker](https://docs.docker.com/get-docker/) (required to run the custom network launcher image that bundles dogecoind)
- On macOS, a `clang` with WASM support is required to compile the `secp256k1-sys` C library for the `wasm32-unknown-unknown` target. Xcode's bundled clang does not include the WASM backend. Install the [Homebrew LLVM](https://formulae.brew.sh/formula/llvm) and add it to your PATH:
  ```bash
  brew install llvm
  export PATH="$(brew --prefix llvm)/bin:$PATH"
  ```
  Add the `export` line to your shell profile (`~/.zshrc` or `~/.bashrc`) to make it permanent.

### Install

```bash
git clone https://github.com/dfinity/dogecoin-canister
cd dogecoin-canister/examples/basic_dogecoin
```

### Build the network launcher image

The local network bundles dogecoind inside a custom Docker image. Build it once before starting the network:

```bash
./build-image.sh
```

### Deploy locally and test

```bash
icp network start -d
icp deploy --cycles 30t
bash test.sh
icp network stop
```

> If tests fail with an out-of-cycles error, top up the canister and retry:
> ```bash
> icp canister top-up --amount 30t backend
> ```

### Deploy to the IC network

The `ic` environment deploys to IC mainnet connected to Dogecoin mainnet, using `test_key_1`:

```bash
icp deploy -e ic --cycles 30t
```

#### Choosing a threshold ECDSA key

The IC offers different [deployed threshold keys](https://docs.internetcomputer.org/concepts/chain-key-cryptography/#deployed-keys), set via `key_name` in [`backend/src/lib.rs`](backend/src/lib.rs):

- **`test_key_1`** — a test key. It is what the icp-cli network launcher provisions locally, and it is also available on mainnet with **lower signing costs** than the production key. Prefer it for development and testing, where no production funds are at stake. This example uses `test_key_1` for both `regtest` and `mainnet` by default.
- **`key_1`** — the production key. Use it for real deployments that hold real value, accepting the higher per-signature cost.

## Generating Dogecoin addresses

The example demonstrates how to generate and use P2PKH (legacy) addresses, which are the most common address type on Dogecoin. They are derived using ECDSA and signed with `sign_with_ecdsa`.

```bash
icp canister call backend get_p2pkh_address '()'
```

## Funding and sending dogecoin: a complete walkthrough

This walkthrough shows how to fund an address, check its balance, send dogecoin to another address, and confirm the transfer — using the bundled `dogecoind` in regtest mode.

> **Coinbase maturity:** In Dogecoin, newly mined block rewards (coinbase UTXOs) cannot be spent until a number of additional blocks have been mined on top. Mine enough blocks upfront so the first reward is spendable.

### Step 1 — Get the canister's address and the container ID

```bash
CONTAINER=$(docker ps --filter "ancestor=icp-cli-network-launcher-dogecoin" --format "{{.ID}}" | head -1)
ADDR=$(icp canister call backend get_p2pkh_address '()' | grep -o '"[^"]*"' | tr -d '"')
echo "Address: $ADDR"
```

### Step 2 — Mine 101 blocks to fund the address

Mining 101 blocks ensures the first block reward is past the coinbase maturity threshold and spendable.

```bash
docker exec $CONTAINER dogecoin-cli -regtest \
  -rpcuser=ic-doge-integration -rpcpassword=QPQiNaph19FqUsCrBRN0FII7lyM26B51fAMeBQzCb-E= \
  generatetoaddress 101 "$ADDR"
```

### Step 3 — Check the balance

The IC Dogecoin integration syncs new blocks continuously. If the balance shows 0, wait a few seconds and retry.

```bash
icp canister call backend get_balance "(\"$ADDR\")"
```

### Step 4 — Send dogecoin

```bash
DEST="mhXcJVuNA48bZsrKq4t21jx1neSqyceqTM"
icp canister call backend send_from_p2pkh_address "(record {
  destination_address = \"$DEST\";
  amount_in_koinu = 100000000;
})"
# Returns the transaction ID
```

The transaction is now broadcast to `dogecoind`'s mempool. The destination balance will remain 0 until it is confirmed in a block.

### Step 5 — Mine a confirmation block

```bash
docker exec $CONTAINER dogecoin-cli -regtest \
  -rpcuser=ic-doge-integration -rpcpassword=QPQiNaph19FqUsCrBRN0FII7lyM26B51fAMeBQzCb-E= \
  generatetoaddress 1 "$ADDR"
```

### Step 6 — Verify the destination received the funds

```bash
icp canister call backend get_balance "(\"$DEST\")"
```

Each send internally estimates fees, selects UTXOs, builds a transaction, signs it using ECDSA, and broadcasts it via `dogecoin_send_transaction`.

## Querying UTXOs

You can inspect the UTXOs held at any Dogecoin address:

```bash
icp canister call backend get_utxos "(\"$ADDR\")"
```

This returns all unspent outputs at the address — useful for verifying that funds arrived or for debugging balance issues. The response includes each outpoint (txid + vout index), value in koinu, and confirmation height.

## Retrieving block headers

You can query historical block headers:

```bash
icp canister call backend get_block_headers '(10: nat32, null)'
# or a range:
icp canister call backend get_block_headers '(10: nat32, opt (11: nat32))'
```

This calls `dogecoin_get_block_headers`, which is useful for blockchain validation or light client logic.

## Notes on implementation

This example implements several important patterns for Dogecoin integration:

- **Derivation paths**: Keys are derived using structured derivation paths according to BIP-32, ensuring reproducible key generation.
- **Key caching**: Optimization is used to avoid repeated calls to `get_ecdsa_public_key`.
- **Manual transaction construction**: Transactions are assembled and signed manually, ensuring maximum flexibility in construction and fee estimation.

## Security considerations and best practices

This example is provided for educational purposes and is not production-ready. It is important to consider security implications when developing applications that interact with Dogecoin or other cryptocurrencies. The code has **not been audited** and may contain vulnerabilities or security issues.

If you base your application on this example, we recommend you familiarize yourself with and adhere to the [security best practices](https://docs.internetcomputer.org/guides/security/overview) for developing on the Internet Computer. This example may not implement all the best practices.

For example, the following aspects are particularly relevant for this app:

- Certify query responses if they are relevant for security, since the app offers a method to read balances.
- Use a decentralized governance system like SNS to give a canister a decentralized controller, since decentralized control may be essential for canisters holding dogecoins on behalf of users.
