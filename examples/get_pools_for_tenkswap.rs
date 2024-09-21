use starknet::core::types::Felt;
use starknet::providers::jsonrpc::HttpTransport;
use starknet::providers::{JsonRpcClient, Url};
use starknet_mev_client::amm::pool::AutomatedMarketMaker;
use starknet_mev_client::amm::tenKSwap::pool::TenkSwapPool;
use std::sync::Arc;
use tokio;

fn create_rpc_provider(
    rpc_url: &str,
) -> Result<Arc<JsonRpcClient<HttpTransport>>, Box<dyn std::error::Error>> {
    let url = Url::parse(rpc_url)?;
    let provider = JsonRpcClient::new(HttpTransport::new(url));
    Ok(Arc::new(provider))
}

#[tokio::main]
async fn main() {

    let rpc_url = "https://starknet-mainnet.public.blastapi.io/rpc/v0_7";
    let provider = create_rpc_provider(rpc_url).unwrap();

    let pool = TenkSwapPool::new_from_address(
        Felt::from_hex("0x17e9e62c04b50800d7c59454754fe31a2193c9c3c6c92c093f2ab0faadf8c87")
            .unwrap(),
        300u32,
        provider.clone(),
    )
    .await
    .unwrap();

    pool.simulate_swap(
        Felt::from_hex("0xda114221cb83fa859dbdb4c44beeaa0bb37c7537ad5ae66fe5e0efd20e6eb3").unwrap(),
        Felt::from(100u32),
        provider.clone(),
    )
    .await
    .unwrap();
}
