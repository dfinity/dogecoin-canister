use crate::DOGE_CONTEXT;
use ic_cdk::{
    bitcoin_canister::{bitcoin_get_utxos, GetUtxosRequest, GetUtxosResponse},
    update,
};

/// Returns the UTXOs of the given Dogecoin address.
#[update]
pub async fn get_utxos(address: String) -> GetUtxosResponse {
    let ctx = DOGE_CONTEXT.with(|ctx| ctx.get());

    dogecoin_get_utxos(&GetUtxosRequest {
        address,
        network: ctx.network,
        filter: None,
    })
    .await
    .unwrap()
}
