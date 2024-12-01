use std::sync::Arc;

use reqwest::{self, Url};
use serde::Deserialize;
use starknet::{
    core::{
        // codec::{Decode, Encode},
        types::{Felt, FunctionCall},
    },
    macros::selector,
    providers::{jsonrpc::HttpTransport, JsonRpcClient},
};

#[derive(Debug, Deserialize)]
pub struct PoolKey {
    token0: String,
    token1: String,
    fee: String,
    tick_spacing: u64,
    extension: String,
}

#[derive(Debug, Deserialize)]
pub struct Route {
    pool_key: PoolKey,
    sqrt_ratio_limit: String,
    skip_ahead: u64,
}

#[derive(Debug, Deserialize)]
pub struct Split {
    amount: String,
    specifiedAmount: String,
    route: Vec<Route>,
}

#[derive(Debug, serde::Deserialize)]
pub struct QuoteResponse {
    total: String,
    splits: Vec<Split>,
}

async fn get_ekubo_quote(
    amount: u64,
    token_from: &str,
    token_to: &str,
    max_splits: u64,
) -> Result<QuoteResponse, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let url = format!(
        "https://mainnet-api.ekubo.org/quote/{}/{}/{}?maxSplits={}",
        amount, token_from, token_to, max_splits
    );

    let response = client
        .get(url)
        .send()
        .await?
        .json::<QuoteResponse>()
        .await?;

    Ok(response)
}

async fn print_quote(amount: u64, token_from: &str, token_to: &str, max_splits: u64) {
    match get_ekubo_quote(amount, token_from, token_to, max_splits).await {
        Ok(quote) => {
            println!("Total: {}", quote.total);
            for (i, split) in quote.splits.iter().enumerate() {
                println!("\nSplit {}:", i + 1);
                println!("  Amount: {}", split.amount);
                println!("  Specified Amount: {}", split.specifiedAmount);
                for (j, route) in split.route.iter().enumerate() {
                    println!("  Route {}:", j + 1);
                    println!("    Pool Key:");
                    println!("      Token0: {}", route.pool_key.token0);
                    println!("      Token1: {}", route.pool_key.token1);
                    println!("      Fee: {}", route.pool_key.fee);
                    println!("      Tick Spacing: {}", route.pool_key.tick_spacing);
                    println!("      Extension: {}", route.pool_key.extension);
                    println!("    Sqrt Ratio Limit: {}", route.sqrt_ratio_limit);
                    println!("    Skip Ahead: {}", route.skip_ahead);
                }
            }
        }
        Err(e) => println!("Error fetching Ekubo quote: {}", e),
    }
}

fn create_rpc_provider(
    rpc_url: &str,
) -> Result<Arc<JsonRpcClient<HttpTransport>>, Box<dyn std::error::Error>> {
    let url = Url::parse(rpc_url)?;
    let provider = JsonRpcClient::new(HttpTransport::new(url));
    Ok(Arc::new(provider))
}

#[derive(Debug)]
pub struct RouteNode {
    pub pool_key: PoolKey,
    pub sqrt_ratio_limit: Felt,
    pub skip_ahead: u64,
}

#[derive(Debug)]
pub struct TokenAmount {
    pub token: Felt,
    pub amount: i128,
}

#[derive(Debug)]
pub struct Swap {
    pub route: Vec<RouteNode>,
    pub token_amount: TokenAmount,
}

fn convert_quote_to_swaps(quote: QuoteResponse) -> Vec<Swap> {
    quote
        .splits
        .into_iter()
        .map(|split| {
            let routes = split
                .route
                .iter()
                .map(|r| {
                    let sqrt_ratio_limit = Felt::from_hex(&r.sqrt_ratio_limit).unwrap_or_default();

                    let pool_key = PoolKey {
                        token0: Felt::from_hex(&r.pool_key.token0)
                            .unwrap_or_default()
                            .to_string(),
                        token1: Felt::from_hex(&r.pool_key.token1)
                            .unwrap_or_default()
                            .to_string(),
                        fee: Felt::from_hex(&r.pool_key.fee)
                            .unwrap_or_default()
                            .to_string(),
                        tick_spacing: r.pool_key.tick_spacing,
                        extension: Felt::from_hex(&r.pool_key.extension.to_string())
                            .unwrap_or_default()
                            .to_string(),
                    };

                    RouteNode {
                        pool_key,
                        sqrt_ratio_limit,
                        skip_ahead: r.skip_ahead,
                    }
                })
                .collect();

            let amount = split.specifiedAmount.parse::<i128>().unwrap_or_default();

            let token = if let Some(first_route) = split.route.first() {
                Felt::from_hex(&first_route.pool_key.token0).unwrap_or_default()
            } else {
                Felt::ZERO
            };
            Swap {
                route: routes,
                token_amount: TokenAmount { token, amount },
            }
        })
        .collect()
}

async fn execute_multihop_swap(
    provider: Arc<JsonRpcClient<HttpTransport>>,
    contract_address: Felt,
    quote: QuoteResponse,
) {
    let swaps = convert_quote_to_swaps(quote);
    // swaps.
}

#[tokio::main]
async fn main() {
    match get_ekubo_quote(10000000000000, "ETH", "USDC", 1).await {
        Ok(quote) => {
            println!("Total: {}", quote.total);
            for (i, split) in quote.splits.iter().enumerate() {
                println!("\nSplit {}:", i + 1);
                println!("  Amount: {}", split.amount);
                println!("  Specified Amount: {}", split.specifiedAmount);
                for (j, route) in split.route.iter().enumerate() {
                    println!("  Route {}:", j + 1);
                    println!("    Pool Key:");
                    println!("      Token0: {}", route.pool_key.token0);
                    println!("      Token1: {}", route.pool_key.token1);
                    println!("      Fee: {}", route.pool_key.fee);
                    println!("      Tick Spacing: {}", route.pool_key.tick_spacing);
                    println!("      Extension: {}", route.pool_key.extension);
                    println!("    Sqrt Ratio Limit: {}", route.sqrt_ratio_limit);
                    println!("    Skip Ahead: {}", route.skip_ahead);
                }
            }
            let swaps = convert_quote_to_swaps(quote);
            println!("swaps {:?}", swaps);
        }
        Err(e) => println!("Error fetching Ekubo quote: {}", e),
    }
}
