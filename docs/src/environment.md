# Developer Environment

To develop Dogecoin applications to be deployed on ICP, your local developer environment will need to include:

- A local Dogecoin regtest node.

- The [Rust toolchain](https://doc.rust-lang.org/book/ch01-01-installation.html) for installing Rust crates and compiling Rust code.

- The [IC SDK](https://internetcomputer.org/docs/building-apps/getting-started/install) for creating, deploying, and managing canisters. You can install it natively on macOS and Linux; however, Windows users will need to set up WSL 2 before installing the IC SDK.

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
dogecoind -datadir=$(pwd)/dogecoin_data -printoconsole --port=18444 
```

This command assumes that port `18444` on your machine is available. If it isn't, change the specified port accordingly.
