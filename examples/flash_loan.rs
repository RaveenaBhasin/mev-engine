use starknet::accounts::{ExecutionEncoding, SingleOwnerAccount};
use starknet::contract::ContractFactory;
use starknet::core::chain_id;
use starknet::core::types::contract::legacy::LegacyContractClass;
use starknet::core::types::contract::SierraClass;
use starknet::core::types::{BlockId, BlockTag, Felt, FunctionCall};
use starknet::macros::{felt, selector};
use starknet::providers::jsonrpc::HttpTransport;
use starknet::providers::{JsonRpcClient, Provider, Url};
use starknet::signers::{LocalWallet, SigningKey};
use std::sync::Arc;
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

async fn deploy_contract<P>(provider: Arc<P>)
where
    P: Provider + Send + Sync,
{
    let contract_artifact: SierraClass =
        serde_json::from_reader(std::fs::File::open("/Users/raveena/projects/mev-engine/src/Flashloan_contracts/target/dev/snforge_sample_FlashLoanContract.contract_class.json").unwrap())
            .unwrap();
    let class_hash = contract_artifact.class_hash().unwrap();
    println!("Class hash {:?}", class_hash);

    let signer = LocalWallet::from(SigningKey::from_secret_scalar(
        Felt::from_hex("0x33003003001800009900180300d206308b0070db00121318d17b5e6262150b").unwrap(),
    ));
    let address =
        Felt::from_hex("0x4c0f884b8e5b4f00d97a3aad26b2e5de0c0c76a555060c837da2e287403c01d")
            .unwrap();
    let mut account = SingleOwnerAccount::new(
        provider,
        signer,
        address,
        chain_id::SEPOLIA,
        ExecutionEncoding::New,
    );

    // `SingleOwnerAccount` defaults to checking nonce and estimating fees against the latest
    // block. Optionally change the target block to pending with the following line:
    account.set_block_id(BlockId::Tag(BlockTag::Pending));

    // Wrapping in `Arc` is meaningless here. It's just showcasing it could be done as
    // `Arc<Account>` implements `Account` too.
    let account = Arc::new(account);

    // let contract_factory = ContractFactory::new(class_hash, account);
    // contract_factory
    //     .deploy_v1(
    //         vec![
    //             felt!("0x2545b2e5d519fc230e9cd781046d3a64e092114f07e44771e0d719d148725ef"),
    //             felt!("0x053c91253bc9682c04929ca02ed00b3e423f6710d2ee7e0d5ebb06f3ecf368a8"),
    //         ],
    //         felt!("111"),
    //         false,
    //     )
    //     .send()
    //     .await
    //     .expect("Unable to deploy contract");
}

#[tokio::main]
async fn main() {
    let rpc_url = "http://0.0.0.0:5050";
    let provider = create_rpc_provider(rpc_url).unwrap();

    let token_address = felt!("0x049d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7");
    let vesu_singleton_contract =
        felt!("0x069d0eca40cb01eda7f3d76281ef524cecf8c35f4ca5acc862ff128e7432964b");
    let res = deploy_contract(provider.clone()).await;

    // let call_result = provider
    //     .call(
    //         FunctionCall {
    //             contract_address: token_address,
    //             entry_point_selector: selector!("balanceOf"),
    //             calldata: vec![vesu_singleton_contract],
    //         },
    //         BlockId::Tag(BlockTag::Latest),
    //     )
    //     .await
    //     .expect("failed to call contract");
    //
    // let amount: u64 = 1000;
    // let flash_loan_call = provider
    //     .call(
    //         FunctionCall {
    //             contract_address: vesu_singleton_contract,
    //             entry_point_selector: selector!("flash_loan"),
    //             calldata: vec![],
    //         },
    //         BlockId::Tag(BlockTag::Latest),
    //     )
    //     .await
    //     .expect("failed to call contract");
    // println!("Result {:?}", call_result);
}
