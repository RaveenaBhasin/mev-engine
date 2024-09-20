//This is example file: Do cargo run to run this file

use std::sync::Arc;

use starknet::core::types::Felt;
use starknet::providers::jsonrpc::HttpTransport;
use starknet::providers::{JsonRpcClient, Provider, Url};
use starknet_mev_client::amm::AutomatedMarketMaker;
use starknet_mev_client::amm::{jediswap::JediswapPool, AMM};

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
    let pool = JediswapPool::new(
        Felt::from_hex("0x07e2a13b40fc1119ec55e0bcf9428eedaa581ab3c924561ad4e955f95da63138")
            .unwrap(),
    );
    let amm = AMM::JediswapPool(pool);
    println!("Jediswap DAI/ETH pool address: {:?}", amm.address());

    let rpc_url = "https://starknet-mainnet.public.blastapi.io/rpc/v0_7";
    let provider = create_rpc_provider(rpc_url).unwrap();

    let block_number = provider.block_number().await.unwrap();
    println!("Current block number: {}", block_number);

    let reserves = amm.get_reserves(provider).await.unwrap();
    println!("reserves {:?}", reserves);
}
