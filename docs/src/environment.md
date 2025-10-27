# Developer Environment

To develop Dogecoin applications to be deployed on ICP, your local developer environment will need to include:

- Necessary tools and packages for the language you’d like to build your dapp in:

    - The Rust toolchain for installing Rust packages and compiling Rust code.

    - The IC SDK for creating, deploying, and managing smart contracts.

- A local Dogecoin testnet.

- A local instance of the Dogecoin canister.

- An ICP smart contract project.

## Install tooling

### Rust toolchain

Before developing BTC applications in Rust, you will need to install the Rust toolchain, including:

- [The Rust programming language.](https://www.rust-lang.org/tools/install)
- [The `cargo` package manager.](https://doc.rust-lang.org/cargo/getting-started/installation.html)

### IC SDK

The IC SDK includes a CLI tool called `dfx` that is used for creating, managing, and deploying dapps on ICP. You can install it natively on macOS and Linux; however, Windows users will need to set up WSL 2 before installing the IC SDK.

[Learn more about installing the IC SDK](https://internetcomputer.org/docs/building-apps/getting-started/install)


## Create or download an example project

To set up and test your local Dogecoin testnet, you will need a smart contract that implements methods that call the local Dogecoin canister.

Create a new project using `dfx new my_project` or check out the [examples](https://github.com/dfinity/dogecoin-canister/tree/master/examples) that already implements basic methods for calling the Dogecoin canister.

```admonish info
Popular libraries like [Rust's bitcoin crate](https://crates.io/crates/bitcoin) and [Motoko's bitcoin package](https://mops.one/bitcoin) can be used within ICP smart contracts.
```

## Create a local Dogecoin testnet (regtest) with `dogecoind`

It is recommended that developers set up a local Dogecoin testnet on their machine, as it allows them to mine blocks quickly and at will, which facilitates testing various cases without having to rely on the Dogecoin testnet or the Dogecoin mainnet. Alternatively, you can test dapps using the Dogecoin testnet or mainnet through the ICP Dogecoin API. Both workflows are detailed below.

A local Dogecoin testnet deployed on your computer operates in "regression testing mode," or [regtest mode](https://developer.bitcoin.org/examples/testing.html#regtest-mode). Regtest mode is used to instantly create a new, private blockchain with the configuration of a testnet. However, there is one key difference: regtest mode enables the developer to have complete control over the environment, including determining when blocks are created. This allows you to test and iterate faster than relying on the Dogecoin testnet or mainnet.

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

```admonish note
There are currently no released binary for Mac OS X.
```

- #### Step 2: Create a subdirectory for Dogecoin data.

This should be created in the project folder root. This allows you to run different local Dogecoin testing networks for different projects.

```bash
mkdir dogecoin_data
```

- #### Step 3: Create a file called `dogecoin.conf`:

```
cat > dogecoin.conf <<EOF
regtest=1
txindex=1
rpcuser=ic-doge-integration
rpcpassword=QPQiNaph19FqUsCrBRN0FII7lyM26B51fAMeBQzCb-E=
rpcauth=ic-doge-integration:cdf2741387f3a12438f69092f0fdad8e\$62081498c98bee09a0dce2b30671123fa561932992ce377585e8e08bb0c11dfa
EOF
```

Explanation of settings:

- `regtest=1`: Enables Dogecoin’s regression test mode for local blockchain testing.

- `txindex=1`: Maintains a full transaction index to support lookups by transaction ID.

- `rpcuser=ic-btc-integration`: Sets a default username for JSON-RPC authentication.

- `rpcpassword=QPQ…b-E=`: Sets the password for JSON-RPC authentication.

- `rpcauth=ic-btc-integration:cdf…dfa`: Uses an alternative authentication method for RPC, combining the username and a salted hash.

Find more details about `dogecoin.conf` settings in the Dogecoin Core Daemon [documentation](https://dogecoin.com/es/dogepedia/how-tos/operating-a-node/#advanced-configuration).

- #### Step 4: Run `dogecoind` to start the Dogecoin client:

```bash
dogecoind -conf=$(pwd)/dogecoin.conf -datadir=$(pwd)/dogecoin_data --port=18444
```

This command assumes that port `18444` on your machine is available. If it isn't, change the specified port accordingly.

## Starting `dfx` with Dogecoin support

TODO XC-527: add support for Regtest
