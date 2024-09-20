//This is example file: Do cargo run to run this file

use std::sync::Arc;

use starknet::core::types::Felt;
use starknet::providers::jsonrpc::HttpTransport;
use starknet::providers::{JsonRpcClient, Url};
// use starknet_mev_client::amm::AutomatedMarketMaker;
use starknet_mev_client::amm::jediswap::JediswapPool;

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
    let pool = JediswapPool::new(
        Felt::from_hex("0x07e2a13b40fc1119ec55e0bcf9428eedaa581ab3c924561ad4e955f95da63138")
            .unwrap(),
        Felt::from_hex("0xad5d3ec16e143a33da68c00099116ef328a882b65607bec5b2431267934a20").unwrap(),
        Felt::from_hex("0x3610e8e1835afecdd154863369b91f55612defc17933f83f4425533c435a248")
            .unwrap(), // token1
        // Add the correct arguments here
        18u8,
        18u8,
        Felt::ZERO,
        Felt::ZERO,
        30,
    );

    print!("Initialise pool {:?}", pool);

    // let amm = AMM::JediswapPool(pool);
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
