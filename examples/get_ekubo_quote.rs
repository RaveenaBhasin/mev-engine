use std::sync::Arc;

use dotenv::dotenv;
use reqwest::{self, Url};
use serde::Deserialize;
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
// #[derive(Debug, Deserialize, Clone)]
// pub struct Route {
//     pool_key: PoolKey,
//     sqrt_ratio_limit: Felt,
//     skip_ahead: u64,
// }
//
// #[derive(Debug, Deserialize, Clone)]
// pub struct Split {
//     amount: Felt,
//     specifiedAmount: Felt,
//     route: Vec<Route>,
// }

#[derive(Debug, PartialEq, Eq, Deserialize, Clone, Encode, Decode)]
pub struct Amount {
    pub amount: Felt,
}

//-------------------------------------------------
//-------------------------------------------------
//             Swap cairo Types
//-------------------------------------------------
//-------------------------------------------------
//-------------------------------------------------

#[derive(Debug, Eq, PartialEq, Encode, Decode)]
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

#[derive(Debug, PartialEq, Eq, Encode, Decode)]
pub struct TokenAmount {
    pub token: Felt,
    pub amount: i129,
}

pub type SwapArray = Vec<Swap>;

#[derive(Debug, Eq, PartialEq, Encode, Decode)]
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
    amount: u64,
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

    println!("get ekubo quote 1 {:?}", response);
    // let response = response?.json::<QuoteResponse>().await?;

    println!("get ekubo quote 2 {:?}", response);

    Ok(response)
}

fn create_rpc_provider(
    rpc_url: &str,
) -> Result<Arc<JsonRpcClient<HttpTransport>>, Box<dyn std::error::Error>> {
    let url = Url::parse(rpc_url)?;
    let provider = JsonRpcClient::new(HttpTransport::new(url));
    Ok(Arc::new(provider))
}

fn convert_quote_to_swaps(quote: QuoteResponseApi) -> Vec<Swap> {
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
                            + U256::from(20454487474956806466920111957593445u128),
                        skip_ahead: r.skip_ahead.into(),
                    }
                })
                .collect();

            // let amount = split.specifiedAmount.parse::<i128>().unwrap_or_default();
            let amount = split.specified_amount;

            let token = if let Some(first_route) = split.route.first() {
                Felt::from_hex(&first_route.pool_key.token0).unwrap()
            } else {
                Felt::ZERO
            };
            Swap {
                route: routes,
                token_amount: TokenAmount {
                    token,
                    amount: i129 {
                        mag: 10000000000000,
                        sign: true,
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
    println!("Class hash {:?}", class_hash);

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
            println!("Not able to declare {:?}", e);
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

    // Felt::from_hex("0x04908e210088083feec6d3dee87ca9e370161b838682b58fde90f53332d8ebd6").unwrap()
}

async fn execute_multihop_swap(
    provider: Arc<JsonRpcClient<HttpTransport>>,
    quote: QuoteResponseApi,
) {
    let address = deploy_contract(provider.clone()).await;

    let swaps = convert_quote_to_swaps(quote);
    println!("Swaps {:?}", swaps);
    let mut serialized = vec![];
    // for swap in &swaps {
    //     let mut swap_serialized = vec![];
    //     swap.encode(&mut swap_serialized).unwrap();
    //     let decoded = Swap::decode(&swap_serialized).unwrap();
    //     assert!(decoded == *swap, "incorrectly serialized");
    //     serialized.push(swap_serialized);
    // }
    swaps.encode(&mut serialized).unwrap();
    let swaps_decoded = SwapArray::decode(&serialized).unwrap();
    assert!(swaps_decoded == swaps, "incorrectly decoded");
    println!("Encoded swaps {:?}", serialized);
    println!("decoded swaps {:?}", swaps_decoded);

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

    let token = "0x064b48806902a367c8598f4f95c305e8c1a1acba5f082d294a43793113115691";
    let amt = Amount {
        amount: Felt::from_hex(token).unwrap(),
        // amount: felt!("0x064b48806902a367c8598f4f95c305e8c1a1acba5f082d294a43793113115691"),
    };

    // println!("Amount {:?}", Felt::from_hex(token).unwrap());
    // println!(
    //     "Amount felt! {:?}",
    //     felt!("0x064b48806902a367c8598f4f95c305e8c1a1acba5f082d294a43793113115691")
    // );

    let mut amount_calldata = vec![];
    amt.encode(&mut amount_calldata).unwrap();
    println!("Amount calldata {:?}", amount_calldata);

    let owner_call = provider
        .clone()
        .call(
            FunctionCall {
                contract_address: address,
                entry_point_selector: selector!("get_owner"),
                calldata: amount_calldata,
            },
            BlockId::Tag(BlockTag::Latest),
        )
        .await
        .expect("failed to call contract");
    print!("{:?} ", owner_call);
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let rpc_url = "http://0.0.0.0:5050";
    let provider = create_rpc_provider(rpc_url).unwrap();
    let get_ekubo_quote = get_ekubo_quote(10000000000, "ETH", "USDC", 10).await;
    match get_ekubo_quote {
        Ok(quote) => {
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

            let encoded_swaps = execute_multihop_swap(provider, quote).await;
            // println!("Encoded swaps {:?}", encoded_swaps);
        }
        Err(e) => println!("Error fetching Ekubo quote: {}", e),
    }
}
