use ic_cdk_macros::{heartbeat, init, inspect_message, post_upgrade, pre_upgrade, query, update};
use ic_doge_canister::types::{HttpRequest, HttpResponse};
use ic_doge_interface::{
    Config, GetBalanceRequest, GetBlockHeadersRequest, GetBlockHeadersResponse,
    GetCurrentFeePercentilesRequest, GetUtxosRequest, GetUtxosResponse, InitConfig,
    MillikoinuPerByte, SendTransactionRequest, SetConfigRequest,
};
use ic_cdk::api::{msg_reject, msg_reply};
use std::marker::PhantomData;

/// Use Nat to represent an arbitrary amount of Koinus because the total amount of DOGE
/// will exceed the bound of u64 by around year 2030.
type Amount = candid::Nat;

#[cfg(target_arch = "wasm32")]
mod printer;

fn hook() {
    #[cfg(target_arch = "wasm32")]
    printer::hook();
}

#[init]
fn init(init_config: InitConfig) {
    hook();
    ic_doge_canister::init(init_config);
}

#[pre_upgrade]
fn pre_upgrade() {
    ic_doge_canister::pre_upgrade();
}

#[post_upgrade]
fn post_upgrade(config_update: Option<SetConfigRequest>) {
    hook();
    ic_doge_canister::post_upgrade(config_update);
}

#[heartbeat]
async fn heartbeat() {
    ic_doge_canister::heartbeat().await
}

// TODO: change to Koinu or Amount?
#[update(manual_reply = true)]
pub fn dogecoin_get_balance(request: GetBalanceRequest) -> PhantomData<Amount> {
    match ic_doge_canister::get_balance(request) {
        Ok(response) => msg_reply(candid::encode_one(response).unwrap()),
        Err(e) => msg_reject(format!("get_balance failed: {:?}", e).as_str()),
    };
    PhantomData
}

#[query(manual_reply = true)]
pub fn dogecoin_get_balance_query(request: GetBalanceRequest) -> PhantomData<Amount> {
    if ic_cdk::api::data_certificate().is_none() {
        msg_reject("get_balance_query cannot be called in replicated mode");
    }
    match ic_doge_canister::get_balance_query(request) {
        Ok(response) => msg_reply(candid::encode_one(response).unwrap()),
        Err(e) => msg_reject(format!("get_balance_query failed: {:?}", e).as_str()),
    }
    PhantomData
}

#[update(manual_reply = true)]
pub fn dogecoin_get_utxos(request: GetUtxosRequest) -> PhantomData<GetUtxosResponse> {
    match ic_doge_canister::get_utxos(request) {
        Ok(response) => msg_reply(candid::encode_one(response).unwrap()),
        Err(e) => msg_reject(format!("get_utxos failed: {:?}", e).as_str()),
    }
    PhantomData
}

#[query(manual_reply = true)]
pub fn dogecoin_get_utxos_query(request: GetUtxosRequest) -> PhantomData<GetUtxosResponse> {
    if ic_cdk::api::data_certificate().is_none() {
        msg_reject("get_utxos_query cannot be called in replicated mode");
    } else {
        match ic_doge_canister::get_utxos_query(request) {
            Ok(response) => msg_reply(candid::encode_one(response).unwrap()),
            Err(e) => msg_reject(format!("get_utxos_query failed: {:?}", e).as_str()),
        }
    }
    PhantomData
}

#[update(manual_reply = true)]
pub fn dogecoin_get_block_headers(
    request: GetBlockHeadersRequest,
) -> PhantomData<GetBlockHeadersResponse> {
    match ic_doge_canister::get_block_headers(request) {
        Ok(response) => msg_reply(candid::encode_one(response).unwrap()),
        Err(e) => msg_reject(format!("get_block_headers failed: {:?}", e).as_str()),
    }
    PhantomData
}

#[update(manual_reply = true)]
async fn dogecoin_send_transaction(request: SendTransactionRequest) -> PhantomData<()> {
    match ic_doge_canister::send_transaction(request).await {
        Ok(_) => msg_reply(candid::encode_one(()).unwrap()),
        Err(e) => msg_reject(format!("send_transaction failed: {:?}", e).as_str()),
    }
    PhantomData
}

#[update]
pub fn dogecoin_get_current_fee_percentiles(
    request: GetCurrentFeePercentilesRequest,
) -> Vec<MillikoinuPerByte> {
    ic_doge_canister::get_current_fee_percentiles(request)
}

#[query]
pub fn get_config() -> Config {
    ic_doge_canister::get_config()
}

#[update]
fn set_config(request: SetConfigRequest) {
    ic_doge_canister::set_config(request)
}

#[query]
pub fn http_request(request: HttpRequest) -> HttpResponse {
    ic_doge_canister::http_request(request)
}

#[inspect_message]
fn inspect_message() {
    // Reject calls to the query endpoints as they are not supported in replicated mode.
    let inspected_method_name = ic_cdk::api::msg_method_name();
    if inspected_method_name.as_str() != "dogecoin_get_balance_query"
        && inspected_method_name.as_str() != "dogecoin_get_utxos_query"
    {
        ic_cdk::api::accept_message();
    }
}

// Expose a method to know if canbench is included in the binary or not.
// This is used in a test to ensure that canbench is _not_ included in the
// production binary.
#[cfg(feature = "canbench-rs")]
#[update]
pub fn has_canbench() -> bool {
    true
}

fn main() {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_candid_interface_compatibility() {
        use candid_parser::utils::{service_compatible, CandidSource};
        use std::path::PathBuf;

        candid::export_service!();
        let rust_interface = __export_service();

        let candid_interface =
            PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap()).join("candid.did");

        service_compatible(
            CandidSource::Text(&rust_interface),
            CandidSource::File(candid_interface.as_path()),
        )
        .expect("The canister implementation is not compatible with the candid.did file");
    }
}
