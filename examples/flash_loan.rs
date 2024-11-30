use starknet::core::types::{BlockId, BlockTag, Felt, FunctionCall};
use starknet::macros::{felt, selector};
use starknet::providers::jsonrpc::HttpTransport;
use starknet::providers::{JsonRpcClient, Provider, Url};
use std::sync::Arc;
use std::time::Duration;
use std::{u32, u64};
use tokio;

fn create_rpc_provider(
    rpc_url: &str,
) -> Result<Arc<JsonRpcClient<HttpTransport>>, Box<dyn std::error::Error>> {
    let url = Url::parse(rpc_url)?;
    let provider = JsonRpcClient::new(HttpTransport::new(url));
    Ok(Arc::new(provider))
}

async fn create_flashloan<P>(provider: Arc<P>)
where
    P: Provider + Send + Sync,
{
    //Token address - 0x049d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7
}

#[tokio::main]
async fn main() {
    let rpc_url = "https://starknet-mainnet.public.blastapi.io/rpc/v0_7";
    let provider = create_rpc_provider(rpc_url).unwrap();

    let token_address = felt!("0x049d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7");
    let vesu_singleton_contract =
        felt!("0x069d0eca40cb01eda7f3d76281ef524cecf8c35f4ca5acc862ff128e7432964b");

    let call_result = provider
        .call(
            FunctionCall {
                contract_address: token_address,
                entry_point_selector: selector!("balanceOf"),
                calldata: vec![vesu_singleton_contract],
            },
            BlockId::Tag(BlockTag::Latest),
        )
        .await
        .expect("failed to call contract");

    let amount: u64 = 1000;
    let flash_loan_call = provider
        .call(
            FunctionCall {
                contract_address: vesu_singleton_contract,
                entry_point_selector: selector!("flash_loan"),
                calldata: vec![],
            },
            BlockId::Tag(BlockTag::Latest),
        )
        .await
        .expect("failed to call contract");
    println!("Result {:?}", call_result);
}
