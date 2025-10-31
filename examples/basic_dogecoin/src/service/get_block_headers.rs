use crate::{dogecoin_get_block_headers, DOGE_CONTEXT};
use ic_cdk::{
    bitcoin_canister::{
        GetBlockHeadersRequest, GetBlockHeadersResponse,
    },
    update,
};

/// Returns the block headers in the given height range.
#[update]
pub async fn get_block_headers(
    start_height: u32,
    end_height: Option<u32>,
) -> GetBlockHeadersResponse {
    let ctx = DOGE_CONTEXT.with(|ctx| ctx.get());

    dogecoin_get_block_headers(&GetBlockHeadersRequest {
        start_height,
        end_height,
        network: ctx.network.into(),
    })
    .await
    .unwrap()
}
