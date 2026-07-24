# Developer Environment

To develop Dogecoin applications to be deployed on ICP, your local developer environment will need to include:

- [icp-cli](https://cli.internetcomputer.org/), the command-line tool used to create, deploy, and manage [canisters](https://docs.internetcomputer.org/concepts/canisters) and to run a local development network. Install it with:

  ```bash
  npm install -g @icp-sdk/icp-cli @icp-sdk/ic-wasm
  ```

- The [Rust toolchain](https://doc.rust-lang.org/book/ch01-01-installation.html) with the `wasm32-unknown-unknown` target for compiling canisters:

  ```bash
  rustup target add wasm32-unknown-unknown
  ```

- A local Dogecoin regtest node (`dogecoind`). You can either let icp-cli run one for you inside a Docker container (recommended, see below) or run your own.

```admonish note title="macOS users"
On macOS, a `clang` with WebAssembly support is required to compile the `secp256k1-sys` C library for the `wasm32-unknown-unknown` target. Xcode's bundled `clang` does not include the WebAssembly backend. Install the [Homebrew LLVM](https://formulae.brew.sh/formula/llvm) and add it to your `PATH`:

    brew install llvm
    export PATH="$(brew --prefix llvm)/bin:$PATH"

Add the `export` line to your shell profile (`~/.zshrc` or `~/.bashrc`) to make it permanent.
```

## Running a local Dogecoin regtest network

It is recommended to set up a local Dogecoin [regtest](https://developer.bitcoin.org/examples/testing.html#regtest-mode) network to mine blocks quickly and at will, which facilitates testing various cases without having to rely on the Dogecoin mainnet where blocks are produced every minute on average.

icp-cli runs the ICP Dogecoin integration (a local instance of the Dogecoin API, deployed as a canister for your application to interact with) as part of its managed local network. There are two ways to provide the underlying `dogecoind` regtest node.

### Option 1: Docker image that bundles `dogecoind` (recommended)

This is the approach used by the [`basic_dogecoin` example](https://github.com/dfinity/dogecoin-canister/tree/master/examples/basic_dogecoin). A custom network-launcher Docker image bundles `dogecoind` together with the network launcher, so a single `icp network start` command brings up both the local IC network and a regtest Dogecoin node — you do not need to install or run `dogecoind` yourself. This is especially convenient on macOS, where there are no official `dogecoind` binaries.

The example wires this up in [`icp.yaml`](https://github.com/dfinity/dogecoin-canister/blob/master/examples/basic_dogecoin/icp.yaml) by pointing its `local` network at the custom image:

```yaml
networks:
  - name: local
    mode: managed
    image: icp-cli-network-launcher-dogecoin
```

See [Deploy your first app locally](./deploy.md) for the full workflow (building the image, starting the network, and deploying).

### Option 2: Run your own `dogecoind` and point icp-cli at it

If you prefer to manage the node yourself, run a local `dogecoind` regtest network and connect icp-cli's default (non-Docker) network launcher to it with the `dogecoind-addr` field:

```yaml
networks:
  - name: local
    mode: managed
    dogecoind-addr:
      - "127.0.0.1:18444"
```

`dogecoind-addr` takes the node's P2P address (not its RPC endpoint). For the full set of options, see the [icp-cli configuration reference](https://cli.internetcomputer.org/1.1/reference/configuration/#bitcoin-and-dogecoin-integration).

The steps below set up such a node.

- #### Step 1: Download [Dogecoin Core](https://github.com/dogecoin/dogecoin/releases).

Example for a Linux machine:
```bash
# Download the binary
curl -L -O https://github.com/dogecoin/dogecoin/releases/download/v1.14.9/dogecoin-1.14.9-x86_64-linux-gnu.tar.gz

# Unpack
tar -xvf dogecoin-1.14.9-x86_64-linux-gnu.tar.gz

# Add binaries to the PATH environment variable
export PATH="$(pwd)/dogecoin-1.14.9/bin:$PATH"
```

```admonish note title="macOS users"
There are currently no released binaries for macOS. Either use the Docker image described in Option 1, or build Dogecoin Core from source by following the instructions in the [Dogecoin Core repository](https://github.com/dogecoin/dogecoin/blob/master/doc/build-macos.md).
```

- #### Step 2: Create a subdirectory for Dogecoin data.

This should be created in the project folder root. This allows you to run different local Dogecoin regtest networks for different projects.

```bash
mkdir dogecoin_data
```

- #### Step 3: Create a file called `dogecoin.conf`.

```
cat > dogecoin_data/dogecoin.conf <<EOF
regtest=1
txindex=1
rpcuser=ic-doge-integration
rpcpassword=QPQiNaph19FqUsCrBRN0FII7lyM26B51fAMeBQzCb-E=
rpcauth=ic-doge-integration:cdf2741387f3a12438f69092f0fdad8e\$62081498c98bee09a0dce2b30671123fa561932992ce377585e8e08bb0c11dfa
EOF
```

Explanation of settings:

- `regtest=1`: Enables Dogecoin’s regression test mode for local testing.

- `txindex=1`: Maintains a full transaction index to support lookups by transaction ID.

- `rpcuser=ic-doge-integration`: Sets a default username for JSON-RPC authentication.

- `rpcpassword=QPQ…b-E=`: Sets the password for JSON-RPC authentication.

- `rpcauth=ic-doge-integration:cdf…dfa`: Uses an alternative authentication method for RPC, combining the username and a salted hash.

Find more details about the `dogecoin.conf` settings in the Dogecoin Core Daemon [documentation](https://dogecoin.com/es/dogepedia/how-tos/operating-a-node/#advanced-configuration).

- #### Step 4: Run `dogecoind` to start the Dogecoin client.

```bash
dogecoind -datadir=$(pwd)/dogecoin_data -printtoconsole --port=18444
```

This command assumes that port `18444` on your machine is available. If it isn't, change the specified port accordingly (and update `dogecoind-addr` to match).
