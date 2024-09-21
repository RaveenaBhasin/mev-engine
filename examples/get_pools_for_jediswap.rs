use mev_engine::amm::jediswap::pool::JediswapPool;
use mev_engine::amm::pool::AutomatedMarketMaker;
use starknet::core::types::Felt;
use starknet::providers::jsonrpc::HttpTransport;
use starknet::providers::{JsonRpcClient, Url};
use std::sync::Arc;
use tokio;
// use starknet_mev_client::amm::AutomatedMarketMaker;

#[allow(unused)]
fn create_rpc_provider(
    rpc_url: &str,
) -> Result<Arc<JsonRpcClient<HttpTransport>>, Box<dyn std::error::Error>> {
    let url = Url::parse(rpc_url)?;
    let transport = HttpTransport::new(url);
    let provider = JsonRpcClient::new(transport);
    Ok(Arc::new(provider))
}

#[tokio::main]
async fn main() {
    let rpc_url = "https://starknet-mainnet.public.blastapi.io/rpc/v0_7";
    let provider = create_rpc_provider(rpc_url).unwrap();

    let pool = JediswapPool::new_from_address(
        Felt::from_hex("0x7e2a13b40fc1119ec55e0bcf9428eedaa581ab3c924561ad4e955f95da63138")
            .unwrap(),
        0u32,
        provider.clone(),
    )
    .await
    .unwrap();

    pool.simulate_swap(
        Felt::from_hex("0xda114221cb83fa859dbdb4c44beeaa0bb37c7537ad5ae66fe5e0efd20e6eb3").unwrap(),
        Felt::from(100u32),
        provider,
    )
    .await
    .unwrap();
}
