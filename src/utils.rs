use color_eyre::{eyre::eyre, Result};
use starknet::{
    accounts::SingleOwnerAccount,
    core::{
        types::{BlockId, BlockTag, Felt, FunctionCall},
        utils::get_selector_from_name,
    },
    providers::{jsonrpc::HttpTransport, JsonRpcClient, Provider},
    signers::LocalWallet,
};
use std::sync::Arc;

pub type LocalWalletSignerMiddleware =
    Arc<SingleOwnerAccount<Arc<JsonRpcClient<HttpTransport>>, LocalWallet>>;

pub async fn call_contract<P>(
    provider: Arc<P>,
    address: Felt,
    method: &str,
    calldata: Vec<Felt>,
) -> Result<Vec<Felt>>
where
    P: Provider + Sync + Send,
{
    let entry_point_selector = get_selector_from_name(method)
        .map_err(|e| eyre!("Invalid selector for {}: {}", method, e))?;
    let function_call = FunctionCall {
        contract_address: address,
        entry_point_selector,
        calldata,
    };
    provider
        .call(function_call, BlockId::Tag(BlockTag::Latest))
        .await
        .map_err(|e| eyre!("Provider error: {}", e))
}
