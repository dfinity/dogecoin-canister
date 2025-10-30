use crate::DOGE_CONTEXT;
use ic_cdk::{
    bitcoin_canister::{dogecoin_get_balance, GetBalanceRequest},
    update,
};

/// Returns the balance of the given Dogecoin address.
#[update]
pub async fn get_balance(address: String) -> u64 {
    let ctx = DOGE_CONTEXT.with(|ctx| ctx.get());

    dogecoin_get_balance(&GetBalanceRequest {
        address,
        network: ctx.network,
        min_confirmations: None,
    })
    .await
    .unwrap()
}
