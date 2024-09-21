use starknet::core::types::Felt;
use starknet::providers::jsonrpc::HttpTransport;
use starknet::providers::{JsonRpcClient, Url};
use starknet_mev_client::amm::factory::AutomatedMarketMakerFactory;
use starknet_mev_client::amm::pool::AutomatedMarketMaker;
use starknet_mev_client::amm::tenKSwap::factory::TenKFactory;
use starknet_mev_client::amm::tenKSwap::get_data;
use starknet_mev_client::amm::tenKSwap::pool::TenkSwapPool;
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
    // let mut factory = TenKFactory::new(
    //     Felt::from_hex("0x1c0a36e26a8f822e0d81f20a5a562b16a8f8a3dfd99801367dd2aea8f1a87a2")
    //         .unwrap(),
    // );
    //
    // print!("Initialise factory {:?}", factory);
    //
    let rpc_url = "https://starknet-mainnet.public.blastapi.io/rpc/v0_7";
    let provider = create_rpc_provider(rpc_url).unwrap();
    // let pools = factory.fetch_all_pools(provider.clone()).await.unwrap();
    // let pool = &pools[1];
    // println!("Fetched pools: {:?}", pool);

    let pool = TenkSwapPool::new_from_address(
        Felt::from_hex("0x17e9e62c04b50800d7c59454754fe31a2193c9c3c6c92c093f2ab0faadf8c87")
            .unwrap(),
        30u32,
        provider.clone(),
    )
    .await
    .unwrap();

    pool.simulate_swap(
        Felt::from_hex("0xda114221cb83fa859dbdb4c44beeaa0bb37c7537ad5ae66fe5e0efd20e6eb3").unwrap(),
        Felt::from_hex("0x49d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7")
            .unwrap(),
        Felt::from(100u32),
        provider.clone(),
    )
    .await
    .unwrap();

    // get_data::get_v2_pool_data_batch_request(pools[0], provider)
    //     .await
    //     .unwrap();
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
