# Developer Environment

To develop Dogecoin applications to be deployed on ICP, your local developer environment will need to include:

- A local Dogecoin regtest node.

- The [Rust toolchain](https://doc.rust-lang.org/book/ch01-01-installation.html) for installing Rust crates and compiling Rust code.

- The [IC SDK](https://internetcomputer.org/docs/building-apps/getting-started/install) for creating, deploying, and managing canisters. You can install it natively on macOS and Linux; however, Windows users will need to set up WSL 2 before installing the IC SDK.

The IC SDK includes the `dfx` command-line tool, which is used to manage canisters and interact with the Internet Computer network.

Interacting with Dogecoin requires `dfx` version `0.30.1-beta.0` or higher. You can check your installed version by running:

```bash
dfx --version
```

To install and switch to a specific `dfx` version, use:

```bash
dfxvm install <version>
dfxvm default <version>
```

## Create a local Dogecoin network (regtest) with `dogecoind`

It is recommended to set up a local Dogecoin [regtest](https://developer.bitcoin.org/examples/testing.html#regtest-mode) network to mine blocks quickly and at will, which facilitates testing various cases without having to rely on the Dogecoin mainnet where blocks are produced every minute on average.

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

```admonish note title="Mac OS X users"
There are currently no released binaries for Mac OS X. You will need to build Dogecoin Core from source. Follow the instructions in the [Dogecoin Core repository](https://github.com/dogecoin/dogecoin/blob/master/doc/build-macos.md).
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

This command assumes that port `18444` on your machine is available. If it isn't, change the specified port accordingly.

## Starting `dfx` with Dogecoin support

To deploy and test projects locally, first start `dfx` in your local development environment with Dogecoin support enabled.

```bash
dfx start --enable-dogecoin
```

`dfx` will run a local instance of the Dogecoin API deployed as a smart contract for your application to interact with. The `--enable-dogecoin` flag uses the default Dogecoin node configuration, `127.0.0.1:18444`. This address and port can be manually configured with the `--dogecoin-node` flag:

```bash
dfx start --enable-dogecoin --dogecoin-node <host_address>:<port>
```

### Configuring the local Dogecoin API

The Dogecoin API can be configured either using the command line option `--dogecoin-node` provided to `dfx` or by modifying the project’s `dfx.json` file.

If both command line options and `dfx.json` configurations are provided, the command line option takes precedence.

The Dogecoin API configuration is specified under the `defaults` section of the `dfx.json` file. Below is an example configuration:

```json
{
  "defaults": {
    "dogecoin": {
      "enabled": true,
      "nodes": ["127.0.0.1:18444"]
    }
  }
}
```

The Dogecoin configuration in `dfx.json` adds fields under `defaults` as shown above. This configuration won't actually have any effect unless `dfx.json` also defines the local IC network, **and** `dfx start` is run within the project directory. An example of a full configuration, including the local IC network definition, can be found in the `dfx.json` file of the [basic_dogecoin example project](https://github.com/dfinity/dogecoin-canister/blob/06fbaecc1b7bcfa91f44fb1fcfabb55ee24c2ba1/examples/basic_dogecoin/dfx.json#L21)


#### Configuration options

- `enabled` (boolean): Determines whether the Dogecoin adapter is enabled. Default: `false`.

- `nodes` (array of strings or null): Lists node addresses to connect to. Most likely, you may want to set this to the default IP and port of `dogecoind`, `127.0.0.1:18444`. Default: `null`.

- `canister_init_arg` (string): Optional initialization arguments for the Dogecoin canister. It sets up initial configuration parameters like stability threshold, network type, block sources, fees, syncing state, API access, and more. Details about these parameters can be found in the [Dogecoin canister interface specification](https://github.com/dfinity/dogecoin-canister/blob/master/INTERFACE_SPECIFICATION.md). By default, the local Dogecoin API mirrors the settings used in the production environment. You can view these settings by searching for the [Dogecoin canister](https://dashboard.internetcomputer.org/canister/gordg-fyaaa-aaaan-aaadq-cai) on the ICP Dashboard and calling its `get_config` endpoint.