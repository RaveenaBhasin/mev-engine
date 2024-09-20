use std::sync::Arc;

use amms::amm::factory;
use starknet::{
    core::types::{Felt, FunctionCall, StarknetError},
    providers::Provider,
};

use crate::{
    amm::{factory::AutomatedMarketMakerFactory, pool::AutomatedMarketMaker},
    utils::call_contract,
};

use super::{factory::JediswapFactory, pool::JediswapPool};

pub async fn get_v2_pool_data_batch_request<P>(
    pool: &mut JediswapPool,
    provider: Arc<P>,
) -> Result<(), StarknetError>
where
    P: Provider + Send + Sync,
{
    // let deployer = IGetUniswapV2PoolDataBatchRequest::deploy_builder(provider, vec![pool.address]);
    // let res = deployer.call_raw().await?;
    //
    // let constructor_return = DynSolType::Array(Box::new(DynSolType::Tuple(vec![
    //     DynSolType::Address,
    //     DynSolType::Uint(8),
    //     DynSolType::Address,
    //     DynSolType::Uint(8),
    //     DynSolType::Uint(112),
    //     DynSolType::Uint(112),
    // ])));
    // let return_data_tokens = constructor_return.abi_decode_sequence(&res)?;
    //
    // if let Some(tokens_arr) = return_data_tokens.as_array() {
    //     for token in tokens_arr {
    //         let pool_data = token
    //             .as_tuple()
    //             .ok_or(AMMError::BatchRequestError(pool.address))?;
    //
    //         *pool = populate_pool_data_from_tokens(pool.to_owned(), pool_data)
    //             .ok_or(AMMError::BatchRequestError(pool.address))?;
    //     }
    // }

    let tokne0_return =
        call_contract(provider.clone(), pool.address(), "token0".into(), vec![]).await;

    Ok(())
}

pub async fn get_all_pools<P>(factory: &mut JediswapFactory, provider: Arc<P>) -> Vec<Felt>
where
    P: Provider + Send + Sync,
{
    let all_pairs = call_contract(
        provider.clone(),
        factory.address(),
        "get_all_pairs".into(),
        vec![],
    )
    .await
    .unwrap();
    all_pairs
}
