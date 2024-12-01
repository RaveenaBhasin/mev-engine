use dotenv::dotenv;
use num_bigint::BigUint;
use num_traits::Num;
use rust_decimal::prelude::*;
use rust_decimal::Decimal;
use starknet::core::types::{BlockId, BlockTag, Felt, FunctionCall};
use starknet::macros::selector;
use starknet::providers::jsonrpc::HttpTransport;
use starknet::providers::{JsonRpcClient, Provider, Url};
use std::sync::Arc;
use tokio;

fn create_rpc_provider(
    rpc_url: &str,
) -> Result<Arc<JsonRpcClient<HttpTransport>>, Box<dyn std::error::Error>> {
    let url = Url::parse(rpc_url)?;
    let provider = JsonRpcClient::new(HttpTransport::new(url));
    Ok(Arc::new(provider))
}

fn calculate_fee(fee_percentage: f64) -> u128 {
    let fee_decimal = fee_percentage / 100.0;
    let scale: f64 = 2.0f64.powi(128); // 2^128
    (fee_decimal * scale).floor() as u128
}

fn calculate_price_from_sqrt_ratio(
    sqrt_ratio: &str,
    token0_decimals: u32,
    token1_decimals: u32,
) -> Decimal {
    let sqrt_ratio = BigUint::from_str_radix(sqrt_ratio.trim_start_matches("0x"), 16).unwrap();
    let pow_2_128 = BigUint::from(2u32).pow(128u32);
    let sqrt_price = sqrt_ratio.clone().to_f64().unwrap() / pow_2_128.to_f64().unwrap();

    let price = sqrt_price * sqrt_price;

    let decimal_adjustment = 10f64.powi((token1_decimals as i32 - token0_decimals as i32).abs());
    if token1_decimals > token0_decimals {
        let price = Decimal::from_f64(price / decimal_adjustment)
            .expect("Failed to convert price to Decimal");
        price
    } else {
        let price = Decimal::from_f64(price * decimal_adjustment)
            .expect("Failed to convert price to Decimal");
        price
    }
}

async fn get_pool_price<P>(
    provider: Arc<P>,
    contract_address: Felt,
    token0: Felt,
    token1: Felt,
    fee_percentage: f64,
    tick_spacing: u128,
    extension: Felt,
) -> Result<Vec<Felt>, Box<dyn std::error::Error>>
where
    P: Provider + Send + Sync,
{
    let fee = calculate_fee(fee_percentage);
    let call = FunctionCall {
        contract_address,
        entry_point_selector: selector!("get_pool_price"),
        calldata: vec![
            token0,
            token1,
            Felt::from(fee),
            Felt::from(tick_spacing),
            extension,
        ],
    };
    let result = provider.call(call, BlockId::Tag(BlockTag::Latest)).await?;

    Ok(result)
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let rpc_url = "http://0.0.0.0:5050";
    let provider = create_rpc_provider(rpc_url).unwrap();
    let contract_address =
        Felt::from_hex("0x00000005dd3D2F4429AF886cD1a3b08289DBcEa99A294197E9eB43b0e0325b4b")
            .unwrap();
    let token0 =
        Felt::from_hex("0x049d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7")
            .unwrap();
    let token1 =
        Felt::from_hex("0x053c91253bc9682c04929ca02ed00b3e423f6710d2ee7e0d5ebb06f3ecf368a8")
            .unwrap();
    let token0_decimals = 18; //ETH
    let token1_decimals = 6; //USDC

    let pool_configs = vec![
        (0.01, 0.02, "0.01% (tick spacing: 0.02%)"),
        (0.05, 0.1, "0.05% (tick spacing: 0.1%)"),
        (0.3, 0.6, "0.3% (tick spacing: 0.6%)"),
    ];

    for (fee_percentage, tick_spacing_percentage, label) in pool_configs {
        println!("\nChecking {} pool", label);
        println!("Fee in fixed-point: {}", calculate_fee(fee_percentage));

        let tick_spacing = (tick_spacing_percentage * 10000.0) as u128;
        println!("Tick spacing: {}", tick_spacing);

        match get_pool_price(
            provider.clone(),
            contract_address,
            token0,
            token1,
            fee_percentage,
            tick_spacing,
            Felt::ZERO,
        )
        .await
        {
            Ok(pool_price) => {
                if !pool_price.is_empty() {
                    let sqrt_ratio_hex = format!("{:x}", pool_price[0]);
                    println!("Sqrt ratio: {}", sqrt_ratio_hex);

                    let price = calculate_price_from_sqrt_ratio(
                        &sqrt_ratio_hex,
                        token0_decimals,
                        token1_decimals,
                    );

                    println!("ETH/USDC Price ({}): {} USDC/ETH", label, price);
                } else {
                    println!("No price data returned for {} fee tier", label);
                }
            }
            Err(e) => {
                println!("Error fetching {} fee tier price: {}", label, e);
            }
        }
    }
}
