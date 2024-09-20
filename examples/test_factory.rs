use starknet::core::types::Felt;
use starknet::providers::jsonrpc::HttpTransport;
use starknet::providers::{JsonRpcClient, Url};
use starknet_mev_client::amm::factory::AutomatedMarketMakerFactory;
use starknet_mev_client::amm::jediswap::factory::JediswapFactory;
use std::sync::Arc;
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
    let mut factory = JediswapFactory::new(
        Felt::from_hex("0xdad44c139a476c7a17fc8141e6db680e9abc9f56fe249a105094c44382c2fd").unwrap(),
    );

    print!("Initialise factory {:?}", factory);

    let rpc_url = "https://starknet-mainnet.public.blastapi.io/rpc/v0_7";
    let provider = create_rpc_provider(rpc_url).unwrap();
    let pools = factory.fetch_all_pools(provider).await;
    println!("Fetched pools: {:?}", pools);
    // let amm = AMM::JediswapPool`(pool);
    // println!("Jediswap DAI/ETH pool address: {:?}", amm.address());
    //
    // let rpc_url = "https://starknet-mainnet.public.blastapi.io/rpc/v0_7";
    // let provider = create_rpc_provider(rpc_url).unwrap();
    //
    // let block_number = provider.block_number().await.unwrap();
    // println!("Current block number: {}", block_number);
    //
    // let reserves = amm.get_reserves(provider).await.unwrap();
    // println!("reserves {:?}", reserves);
}
