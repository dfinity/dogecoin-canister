use crate::{dogecoin_get_fee_percentiles, MillikoinuPerByte, DOGE_CONTEXT};
use ic_cdk::{bitcoin_canister::GetCurrentFeePercentilesRequest, update};

/// Returns the 100 fee percentiles measured in millikoinu/byte.
/// Percentiles are computed from the last 10,000 transactions (if available).
#[update]
pub async fn get_current_fee_percentiles() -> Vec<MillikoinuPerByte> {
    let ctx = DOGE_CONTEXT.with(|ctx| ctx.get());

    dogecoin_get_fee_percentiles(&GetCurrentFeePercentilesRequest {
        network: ctx.network.into(),
    })
    .await
    .unwrap()
}
