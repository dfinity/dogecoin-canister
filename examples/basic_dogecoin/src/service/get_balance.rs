use crate::{dogecoin_get_balance, Amount, DOGE_CONTEXT};
use ic_cdk::{
    bitcoin_canister::{GetBalanceRequest},
    update,
};

/// Returns the balance of the given Dogecoin address.
#[update]
pub async fn get_balance(address: String) -> Amount {
    let ctx = DOGE_CONTEXT.with(|ctx| ctx.get());

    dogecoin_get_balance(&GetBalanceRequest {
        address,
        network: ctx.network.into(),
        min_confirmations: None,
    })
    .await
    .unwrap()
}
