mod common;
mod ecdsa;
mod p2pkh;
mod service;

use ic_cdk::{bitcoin_canister::Network, init, post_upgrade};
use std::cell::Cell;

/// Runtime configuration shared across all Dogecoin-related operations.
///
/// This struct carries network-specific context:
/// - `network`: The ICP Bitcoin API network enum.
/// - `bitcoin_network`: The corresponding network enum from the `bitcoin` crate, used
///   for address formatting and transaction construction.
/// - `key_name`: The global ECDSA key name used when requesting derived keys or making
///   signatures. Different key names are used locally and when deployed on the IC.
///
/// Note: Both `network` and ` dogecoin_network` are needed because ICP and the
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
            network: Network::Testnet,
            dogecoin_network: dogecoin::Network::Testnet,
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
    pub amount_in_satoshi: u64,
}
