use crate::{dogecoin_get_utxos, DOGE_CONTEXT};
use ic_cdk::update;
use ic_cdk_bitcoin_canister::{GetUtxosRequest, GetUtxosResponse};

/// Returns the UTXOs of the given Dogecoin address.
#[update]
pub async fn get_utxos(address: String) -> GetUtxosResponse {
    let ctx = DOGE_CONTEXT.with(|ctx| ctx.get());

    dogecoin_get_utxos(&GetUtxosRequest {
        address,
        network: ctx.network.into(),
        filter: None,
    })
    .await
    .unwrap()
}
