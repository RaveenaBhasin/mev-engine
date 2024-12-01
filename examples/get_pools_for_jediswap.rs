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
        Felt::from_hex("0x04d0390b777b424e43839cd1e744799f3de6c176c7e32c1812a41dbd9c19db6a")
            .unwrap(),
        0u32,
        provider.clone(),
    )
    .await
    .unwrap();

    pool.simulate_swap(
        Felt::from_hex("0x53c91253bc9682c04929ca02ed00b3e423f6710d2ee7e0d5ebb06f3ecf368a8")
            .unwrap(),
        Felt::from(100u32),
        provider,
    )
    .await
    .unwrap();
}
