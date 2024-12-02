use dotenv::dotenv;
use num_bigint::BigUint;
use num_traits::Num;
use reqwest::{self, Url};
use serde::{Deserialize, Serialize};
use serde_json;
use starknet::{
    accounts::{Account, ExecutionEncoding, SingleOwnerAccount},
    contract::ContractFactory,
    core::{
        codec::{Decode, Encode},
        types::{Felt, FunctionCall, U256},
    },
    macros::{felt, selector},
    providers::{jsonrpc::HttpTransport, JsonRpcClient, Provider},
    signers::{LocalWallet, SigningKey},
};
use starknet_core::{
    chain_id,
    types::{
        contract::{CompiledClass, SierraClass},
        BlockId, BlockTag,
    },
};
use std::fs::File;
use std::io::BufReader;
use std::sync::Arc;
use tracing::info;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
struct Tokens {
    eth: String,
    usdc: String,
    wbtc: String,
}

//-------------------------------------------------
//-------------------------------------------------
//             Swap cairo Types
//-------------------------------------------------
//-------------------------------------------------
//-------------------------------------------------

#[derive(Debug, Clone, Eq, PartialEq, Encode, Decode)]
pub struct RouteNode {
    pub pool_key: PoolKey,
    pub sqrt_ratio_limit: U256,
    pub skip_ahead: u128,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Encode, Decode)]
pub struct i129 {
    pub mag: u128,
    pub sign: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
pub struct TokenAmount {
    pub token: Felt,
    pub amount: i129,
}

pub type SwapArray = Vec<Swap>;

#[derive(Debug, Clone, Eq, PartialEq, Encode, Decode)]
pub struct Swap {
    pub route: Vec<RouteNode>,
    pub token_amount: TokenAmount,
}

#[derive(Debug, PartialEq, Eq, Deserialize, Clone, Encode, Decode)]
pub struct PoolKey {
    token0: Felt,
    token1: Felt,
    fee: u128,
    tick_spacing: u128,
    extension: Felt,
}

//-------------------------------------------------
//-------------------------------------------------
//             QuoteResponse Types
//-------------------------------------------------
//-------------------------------------------------
//-------------------------------------------------

#[derive(Debug, Deserialize, Clone)]
pub struct PoolKeyResponse {
    token0: String,
    token1: String,
    fee: String,
    tick_spacing: u64,
    extension: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RouteResponse {
    pool_key: PoolKeyResponse,
    sqrt_ratio_limit: String,
    skip_ahead: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SplitResponse {
    amount: String,
    #[serde(rename = "specifiedAmount")]
    specified_amount: String,
    route: Vec<RouteResponse>,
}

#[derive(Debug, serde::Deserialize, Clone)]
pub struct QuoteResponseApi {
    total: String,
    splits: Vec<SplitResponse>,
}

async fn get_ekubo_quote(
    amount: String,
    token_from: &str,
    token_to: &str,
    max_splits: u64,
) -> Result<QuoteResponseApi, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let url = format!(
        "https://mainnet-api.ekubo.org/quote/{}/{}/{}?maxSplits={}",
        amount, token_from, token_to, max_splits
    );

    let response = client
        .get(url)
        .send()
        .await?
        .json::<QuoteResponseApi>()
        .await?;

    print_quote(response.clone());

    Ok(response)
}

fn create_rpc_provider(
    rpc_url: &str,
) -> Result<Arc<JsonRpcClient<HttpTransport>>, Box<dyn std::error::Error>> {
    let url = Url::parse(rpc_url)?;
    let provider = JsonRpcClient::new(HttpTransport::new(url));
    Ok(Arc::new(provider))
}

fn convert_quote_to_swaps(quote: QuoteResponseApi, token: Felt) -> Vec<Swap> {
    quote
        .splits
        .into_iter()
        .map(|split| {
            let routes = split
                .route
                .iter()
                .map(|r| {
                    let sqrt_ratio_limit = &r.sqrt_ratio_limit;

                    // let pool_key = &r.pool_key;
                    let pool_key = PoolKey {
                        token0: Felt::from_hex(&r.pool_key.token0).unwrap(),
                        token1: Felt::from_hex(&r.pool_key.token1).unwrap(),

                        fee: u128::from_str_radix(r.pool_key.fee.trim_start_matches("0x"), 16)
                            .unwrap(),
                        tick_spacing: r.pool_key.tick_spacing.into(),
                        extension: Felt::from_hex(&r.pool_key.extension.to_string()).unwrap(),
                    };

                    RouteNode {
                        pool_key,
                        sqrt_ratio_limit: U256::from(Felt::from_hex(&r.sqrt_ratio_limit).unwrap())
                            + U256::from(1000000000000000000000000000u128),
                        skip_ahead: r.skip_ahead.into(),
                    }
                })
                .collect();

            let amount = split.specified_amount.parse::<u128>().unwrap();

            Swap {
                route: routes,
                token_amount: TokenAmount {
                    token,
                    amount: i129 {
                        mag: amount,
                        sign: false,
                    },
                },
            }
        })
        .collect()
}

async fn deploy_contract<P>(provider: Arc<P>) -> Felt
where
    P: Provider + Send + Sync,
{
    println!("Starting contract deployment");
    let root_path = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let private_key = std::env::var("PRIVATE_KEY").unwrap();
    let address = std::env::var("ADDRESS").unwrap();
    let contract_artifact: SierraClass = serde_json::from_reader(
        std::fs::File::open(format!(
            "{}/src/Flashloan_contracts/target/dev/snforge_sample_EkuboRouter.contract_class.json",
            root_path
        ))
        .unwrap(),
    )
    .unwrap();
    let class_hash = contract_artifact.class_hash().unwrap();
    // println!("Class hash {:?}", class_hash);

    let casm_class: CompiledClass = serde_json::from_reader(
        std::fs::File::open(format!("{}/src/Flashloan_contracts/target/dev/snforge_sample_EkuboRouter.compiled_contract_class.json", root_path)).unwrap())
        .unwrap();
    let compiled_class_hash = casm_class.class_hash().unwrap();

    let signer = LocalWallet::from(SigningKey::from_secret_scalar(
        Felt::from_hex(&private_key).unwrap(),
    ));
    let address = Felt::from_hex(&address).unwrap();
    let mut account = SingleOwnerAccount::new(
        provider,
        signer,
        address,
        chain_id::MAINNET,
        ExecutionEncoding::New,
    );

    account.set_block_id(BlockId::Tag(BlockTag::Pending));

    let account = Arc::new(account);

    let flattened_class = contract_artifact.flatten().unwrap();

    let result = account
        .declare_v2(Arc::new(flattened_class), compiled_class_hash)
        .send()
        .await;

    match result {
        Ok(res) => {}
        Err(e) => {
            println!("");
        }
    }

    let contract_factory = ContractFactory::new(class_hash, account);
    let deployed_res = contract_factory.deploy_v1(
        vec![
            felt!("0x00000005dd3D2F4429AF886cD1a3b08289DBcEa99A294197E9eB43b0e0325b4b"),
            felt!("0x064b48806902a367c8598f4f95c305e8c1a1acba5f082d294a43793113115691"),
        ],
        felt!("11"),
        true,
    );

    let deployed_address = deployed_res.deployed_address();
    println!("Deployed Address {:?}", deployed_address);

    let deployment_txn = deployed_res.send().await;
    match deployment_txn {
        Ok(success) => {
            println!("Txn hash {:?}", success.transaction_hash);
        }
        Err(e) => {
            println!("Could not deploy contract : {:?}", e);
        }
    }

    deployed_address
}

fn print_quote(quote: QuoteResponseApi) {
    println!("Total: {}", quote.total);
    for (i, split) in quote.splits.iter().enumerate() {
        println!("Split {}:", i + 1);
        println!("  Amount: {}", split.amount);
        println!("  Specified Amount: {}", split.specified_amount);
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

#[tokio::main]
async fn main() {
    dotenv().ok();
    let rpc_url = "http://0.0.0.0:5050";
    let provider = create_rpc_provider(rpc_url).unwrap();
    println!("Fetching quote from Ekubo API");
    let eth_usdc_response: QuoteResponseApi =
        get_ekubo_quote("1000000000000000000".to_string(), "ETH", "USDC", 1)
            .await
            .unwrap();

    let usdc_btc_response: QuoteResponseApi =
        get_ekubo_quote(eth_usdc_response.clone().total, "USDC", "WBTC", 1)
            .await
            .unwrap();

    let wbtc_eth_response = get_ekubo_quote(usdc_btc_response.clone().total, "WBTC", "ETH", 1)
        .await
        .unwrap();

    let root_path = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let tokens: Tokens =
        serde_json::from_reader(std::fs::File::open(format!("{}/tokens.json", root_path)).unwrap())
            .unwrap();
    let swaps = vec![
        convert_quote_to_swaps(eth_usdc_response, Felt::from_hex(&tokens.eth).unwrap()),
        convert_quote_to_swaps(usdc_btc_response, Felt::from_hex(&tokens.usdc).unwrap()),
        convert_quote_to_swaps(wbtc_eth_response, Felt::from_hex(&tokens.wbtc).unwrap()),
    ]
    .concat();

    let address = deploy_contract(provider.clone()).await;
    let mut serialized = vec![];
    swaps.encode(&mut serialized).unwrap();

    println!("---Calling multihop swap---");
    let swap_call = provider
        .clone()
        .call(
            FunctionCall {
                contract_address: address,
                entry_point_selector: selector!("multi_multihop_swap"),
                calldata: serialized,
            },
            BlockId::Tag(BlockTag::Latest),
        )
        .await
        .expect("failed to call contract");
    println!("Result {:?}", swap_call);
}
