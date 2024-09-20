use std::sync::Arc;

use starknet::{core::types::StarknetError, providers::Provider};

use crate::{amm::pool::AutomatedMarketMaker, utils::call_contract};

use super::pool::JediswapPool;

pub async fn get_v2_pool_data_batch_request<P>(
    pool: &mut JediswapPool,
    provider: Arc<P>,
) -> Result<(), StarknetError>
where
    P: Provider,
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

    // call_contract(pprovider, pool.address(), method, calldata);

    Ok(())
}
