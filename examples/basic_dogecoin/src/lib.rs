mod common;
mod ecdsa;
mod p2pkh;
mod service;

use ic_cdk::{bitcoin_canister, init, post_upgrade};
use std::cell::Cell;
use bitcoin::dogecoin;
use candid::{CandidType, Deserialize, Principal};
use ic_cdk::bitcoin_canister::{GetBalanceRequest, GetBlockHeadersRequest, GetBlockHeadersResponse, GetCurrentFeePercentilesRequest, GetUtxosRequest, GetUtxosResponse, SendTransactionRequest};
use ic_cdk::call::{Call, CallResult};
use serde::Serialize;

type Amount = candid::Nat;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, CandidType, Deserialize, Serialize)]
pub enum Network {
    Mainnet,
    Testnet,
    Regtest,
}

impl From<Network> for bitcoin_canister::Network {
    fn from(network: Network) -> Self {
        match network {
            Network::Mainnet => bitcoin_canister::Network::Mainnet,
            Network::Testnet => bitcoin_canister::Network::Testnet,
            Network::Regtest => bitcoin_canister::Network::Regtest,
        }
    }
}

/// Unit of Dogecoin transaction fee.
///
/// This is the element in the [`dogecoin_get_fee_percentiles`] response.
pub type MillikoinuPerByte = u64;

pub async fn dogecoin_get_utxos(arg: &GetUtxosRequest) -> CallResult<GetUtxosResponse> {
    let canister_id = get_dogecoin_canister_id(&into_dogecoin_network(arg.network));
    // same cycles cost as for the Bitcoin canister
    let cycles = ic_cdk::bitcoin_canister::cost_get_utxos(arg);
    Ok(Call::bounded_wait(canister_id, "dogecoin_get_utxos")
        .with_arg(arg)
        .with_cycles(cycles)
        .await?
        .candid()?)
}

pub async fn dogecoin_get_fee_percentiles(
    arg: &GetCurrentFeePercentilesRequest,
) -> CallResult<Vec<MillikoinuPerByte>> {
    let canister_id = get_dogecoin_canister_id(&into_dogecoin_network(arg.network));
    // same cycles cost as for the Bitcoin canister
    let cycles = ic_cdk::bitcoin_canister::cost_get_current_fee_percentiles(arg);
    Ok(
        Call::bounded_wait(canister_id, "dogecoin_get_current_fee_percentiles")
            .with_arg(arg)
            .with_cycles(cycles)
            .await?
            .candid()?,
    )
}

/// Sends a Dogecoin transaction to the Dogecoin network.
///
/// **Unbounded-wait call**
///
/// Check the [Dogecoin Canisters Interface Specification](https://github.com/dfinity/dogecoin-canister/blob/master/INTERFACE_SPECIFICATION.md#dogecoin_send_transaction) for more details.
pub async fn dogecoin_send_transaction(arg: &SendTransactionRequest) -> CallResult<()> {
    let canister_id = get_dogecoin_canister_id(&into_dogecoin_network(arg.network));
    // same cycles cost as for the Bitcoin canister
    let cycles = ic_cdk::bitcoin_canister::cost_send_transaction(arg);

    Ok(
        Call::unbounded_wait(canister_id, "dogecoin_send_transaction")
            .with_arg(arg)
            .with_cycles(cycles)
            .await?
            .candid()?,
    )
}

/// Gets the current balance of a Dogecoin address in Koinu.
///
/// **Bounded-wait call**
///
/// Check the [Dogecoin Canisters Interface Specification](https://github.com/dfinity/dogecoin-canister/blob/master/INTERFACE_SPECIFICATION.md#dogecoin_get_balance) for more details.
pub async fn dogecoin_get_balance(arg: &GetBalanceRequest) -> CallResult<Amount> {
    let canister_id = get_dogecoin_canister_id(&into_dogecoin_network(arg.network));
    // same cycles cost as for the Bitcoin canister
    let cycles = bitcoin_canister::cost_get_balance(arg);
    Ok(Call::bounded_wait(canister_id, "dogecoin_get_balance")
        .with_arg(arg)
        .with_cycles(cycles)
        .await?
        .candid()?)
}

/// Gets the block headers in the provided range of block heights.
///
/// **Bounded-wait call**
///
/// Check the [Dogecoin Canisters Interface Specification](https://github.com/dfinity/dogecoin-canister/blob/master/INTERFACE_SPECIFICATION.md#dogecoin_get_block_headers) for more details.
pub async fn dogecoin_get_block_headers(
    arg: &GetBlockHeadersRequest,
) -> CallResult<GetBlockHeadersResponse> {
    let canister_id = get_dogecoin_canister_id(&into_dogecoin_network(arg.network));
    // same cycles cost as for the Bitcoin canister
    let cycles = bitcoin_canister::cost_get_block_headers(arg);
    Ok(Call::bounded_wait(canister_id, "dogecoin_get_block_headers")
        .with_arg(arg)
        .with_cycles(cycles)
        .await?
        .candid()?)
}

/// Gets the canister ID of the Dogecoin canister for the specified network.
pub fn get_dogecoin_canister_id(network: &Network) -> Principal {
    const MAINNET_ID: Principal = Principal::from_slice(&[0_u8, 0, 0, 0, 1, 160, 0, 7, 1, 1]); // "gordg-fyaaa-aaaan-aaadq-cai"
    const TESTNET_ID: Principal = Principal::from_slice(&[0, 0, 0, 0, 1, 160, 0, 8, 1, 1]); // "hd7hi-kqaaa-aaaan-aaaea-cai"
    const REGTEST_ID: Principal = Principal::from_slice(&[0, 0, 0, 0, 1, 160, 0, 8, 1, 1]); // "hd7hi-kqaaa-aaaan-aaaea-cai"

    match network {
        Network::Mainnet => MAINNET_ID,
        Network::Testnet => TESTNET_ID,
        Network::Regtest => REGTEST_ID,
    }
}

fn into_dogecoin_network(network: bitcoin_canister::Network) -> Network {
    match network {
        bitcoin_canister::Network::Mainnet => Network::Mainnet,
        bitcoin_canister::Network::Testnet => Network::Testnet,
        bitcoin_canister::Network::Regtest => Network::Regtest,
    }
}

/// Runtime configuration shared across all Dogecoin-related operations.
///
/// This struct carries network-specific context:
/// - `network`: The ICP Dogecoin API network enum.
/// - `dogecoin_network`: The corresponding network enum from the `dogecoin` crate, used
///   for address formatting and transaction construction.
/// - `key_name`: The global ECDSA key name used when requesting derived keys or making
///   signatures. Different key names are used locally and when deployed on the IC.
///
/// Note: Both `network` and `dogecoin_network` are needed because ICP and the
/// Dogecoin library use distinct network enum types.
#[derive(Clone, Copy)]
pub struct DogecoinContext {
    pub network: Network,
    pub dogecoin_network: dogecoin::Network,
    pub key_name: &'static str,
}

// Global, thread-local instance of the Dogecoin context.
// This is initialized at smart contract init/upgrade time and reused across all API calls.
thread_local! {
    static DOGE_CONTEXT: Cell<DogecoinContext> = const {
        Cell::new(DogecoinContext {
            network: Network::Regtest,
            dogecoin_network: dogecoin::Network::Regtest,
            key_name: "test_key_1",
        })
    };
}

/// Internal shared init logic used both by init and post-upgrade hooks.
fn init_upgrade(network: Network) {
    let key_name = match network {
        Network::Regtest => "dfx_test_key",
        Network::Mainnet | Network::Testnet => "test_key_1",
    };

    let dogecoin_network = match network {
        Network::Mainnet => dogecoin::Network::Dogecoin,
        Network::Testnet => dogecoin::Network::Testnet,
        Network::Regtest => dogecoin::Network::Regtest,
    };

    DOGE_CONTEXT.with(|ctx| {
        ctx.set(DogecoinContext {
            network,
            dogecoin_network,
            key_name,
        })
    });
}

/// Smart contract init hook.
/// Sets up the DogecoinContext based on the given IC Dogecoin network.
#[init]
pub fn init(network: Network) {
    init_upgrade(network);
}

/// Post-upgrade hook.
/// Reinitializes the DogecoinContext with the same logic as `init`.
#[post_upgrade]
fn upgrade(network: Network) {
    init_upgrade(network);
}

/// Input structure for sending Dogecoin.
/// Used in P2PKH transfer endpoint.
#[derive(candid::CandidType, candid::Deserialize)]
pub struct SendRequest {
    pub destination_address: String,
    pub amount_in_koinu: u64,
}
