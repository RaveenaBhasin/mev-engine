use dotenv::dotenv;
use starknet::accounts::Account;
use starknet::accounts::{ExecutionEncoding, SingleOwnerAccount};
use starknet::contract::ContractFactory;
use starknet::core::chain_id;
use starknet::core::types::contract::{CompiledClass, SierraClass};
use starknet::core::types::{BlockId, BlockTag, DeployTransactionReceipt, Felt, FunctionCall};
use starknet::macros::{felt, selector};
use starknet::providers::jsonrpc::HttpTransport;
use starknet::providers::{JsonRpcClient, Provider, Url};
use starknet::signers::{LocalWallet, SigningKey};
use std::sync::Arc;

use tokio;

fn create_rpc_provider(
    rpc_url: &str,
) -> Result<Arc<JsonRpcClient<HttpTransport>>, Box<dyn std::error::Error>> {
    let url = Url::parse(rpc_url)?;
    let provider = JsonRpcClient::new(HttpTransport::new(url));
    Ok(Arc::new(provider))
}

async fn _create_flashloan<P>(_provider: Arc<P>)
where
    P: Provider + Send + Sync,
{
    //Token address - 0x049d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7
}

async fn deploy_contract<P>(provider: Arc<P>) -> Felt
where
    P: Provider + Send + Sync,
{
    let root_path = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let private_key = std::env::var("PRIVATE_KEY").unwrap();
    let address = std::env::var("ADDRESS").unwrap();
    let contract_artifact: SierraClass =
        serde_json::from_reader(std::fs::File::open(format!("{}/src/Flashloan_contracts/target/dev/snforge_sample_FlashLoanContract.contract_class.json", root_path)).unwrap())
            .unwrap();
    let class_hash = contract_artifact.class_hash().unwrap();
    println!("Class hash {:?}", class_hash);

    let casm_class: CompiledClass = serde_json::from_reader(
        std::fs::File::open(format!("{}/src/Flashloan_contracts/target/dev/snforge_sample_FlashLoanContract.compiled_contract_class.json", root_path)).unwrap())
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
        .await
        .unwrap();

    println!("Successfully declared the class");

    let contract_factory = ContractFactory::new(result.class_hash, account);
    let deployed_res = contract_factory.deploy_v1(
        vec![
            felt!("0x2545b2e5d519fc230e9cd781046d3a64e092114f07e44771e0d719d148725ef"),
            felt!("0x053c91253bc9682c04929ca02ed00b3e423f6710d2ee7e0d5ebb06f3ecf368a8"),
        ],
        felt!("111"),
        false,
    );

    let deployed_address = deployed_res.deployed_address();
    println!("Deployed Address {:?}", deployed_address);

    let deployment_txn = deployed_res
        .send()
        .await
        .expect("Unable to deploy contract");

    println!("Txn hash {:?}", deployment_txn.transaction_hash);
    println!("Contract deployed success !");

    deployed_address
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let rpc_url = "http://0.0.0.0:5050";
    let provider = create_rpc_provider(rpc_url).unwrap();

    let address = deploy_contract(provider.clone()).await;

    let amount: u64 = 1000;
    let flash_loan_call = provider
        .clone()
        .call(
            FunctionCall {
                contract_address: address,
                entry_point_selector: selector!("start_flashloan"),
                calldata: vec![Felt::from(amount)],
            },
            BlockId::Tag(BlockTag::Latest),
        )
        .await
        .expect("failed to call contract");
    println!("Result {:?}", flash_loan_call);
}
