use mev_engine::amm::jediswap::pool::JediswapPool;
use mev_engine::amm::pool::{AutomatedMarketMaker, AMM};
use mev_engine::amm::tenkswap::pool::TenkSwapPool;
use starknet::core::types::Felt;
use starknet::providers::jsonrpc::HttpTransport;
use starknet::providers::{JsonRpcClient, Provider, Url};
use std::sync::Arc;
use std::time::Duration;
use std::u32;
use tokio;

fn create_rpc_provider(
    rpc_url: &str,
) -> Result<Arc<JsonRpcClient<HttpTransport>>, Box<dyn std::error::Error>> {
    let url = Url::parse(rpc_url)?;
    let provider = JsonRpcClient::new(HttpTransport::new(url));
    Ok(Arc::new(provider))
}

async fn find_arbitrage<P>(pool1: &mut AMM, pool2: &mut AMM, provider: Arc<P>) -> bool
where
    P: Provider + Send + Sync,
{
    pool1.sync(provider.clone()).await.unwrap();
    pool2.sync(provider.clone()).await.unwrap();

    let mut pool1_tokens = pool1.tokens();
    let mut pool2_tokens = pool2.tokens();
    pool1_tokens.sort();
    pool2_tokens.sort();

    assert!(
        pool1_tokens == pool2_tokens,
        "This strategy suppports only common"
    );

    let tokens = pool1_tokens;

    let amount_in = Felt::from(100u32);
    let amount_out = pool1
        .simulate_swap(tokens[0], amount_in, provider.clone())
        .await
        .unwrap();

    let final_amount = pool2
        .simulate_swap(tokens[1], amount_out, provider)
        .await
        .unwrap();

    if final_amount <= amount_in {
        println!("No arbitrage found at, retrying soon...");
        false
    } else {
        println!("Arbitrage opportunity found...");
        true
    }
}

#[derive(Debug)]
struct Summary {
    total_iterations: u32,
    total_opportunities: u32,
}

#[tokio::main]
async fn main() {
    let rpc_url = "https://starknet-mainnet.public.blastapi.io/rpc/v0_7";
    let provider = create_rpc_provider(rpc_url).unwrap();

    let mut tenkswap_pool = AMM::TenkSwapPool(
        TenkSwapPool::new_from_address(
            Felt::from_hex("0x17e9e62c04b50800d7c59454754fe31a2193c9c3c6c92c093f2ab0faadf8c87")
                .unwrap(),
            300u32,
            provider.clone(),
        )
        .await
        .unwrap(),
    );

    let mut jediswap_pool = AMM::JediswapPool(
        JediswapPool::new_from_address(
            Felt::from_hex("0x7e2a13b40fc1119ec55e0bcf9428eedaa581ab3c924561ad4e955f95da63138")
                .unwrap(),
            300u32,
            provider.clone(),
        )
        .await
        .unwrap(),
    );

    let mut summary = Summary {
        total_iterations: 0,
        total_opportunities: 0,
    };
    for _ in 0..3 {
        let found = find_arbitrage(&mut tenkswap_pool, &mut jediswap_pool, provider.clone()).await;
        if found {
            summary.total_opportunities += 1;
        }
        summary.total_iterations += 1;
        tokio::time::sleep(Duration::from_secs(2)).await;
    }

    println!("Summary of the current run: {:?}", summary);

    // pool.simulate_swap(
    //     Felt::from_hex("0xda114221cb83fa859dbdb4c44beeaa0bb37c7537ad5ae66fe5e0efd20e6eb3").unwrap(),
    //     Felt::from(100u32),
    //     provider.clone(),
    // )
    // .await
    // .unwrap();
}
