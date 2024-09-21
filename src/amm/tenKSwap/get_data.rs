use std::sync::Arc;

use starknet::{core::types::Felt, providers::Provider};

use super::pool::TenkSwapPool;
use crate::errors::AMMError;
use crate::utils::call_contract;

pub async fn get_pool_info<P>(
    pool_address: Felt,
    provider: Arc<P>,
) -> Result<TenkSwapPool, AMMError>
where
    P: Provider + Send + Sync,
{
    let token0 = call_contract(provider.clone(), pool_address, "token0", vec![])
        .await
        .unwrap();
    let token_0_address = token0[0];
    // println!("Token 0 {:?}", token_0_address);

    tracing::info!(?token_0_address, "UniswapV2 sync");

    let token1 = call_contract(provider.clone(), pool_address, "token1", vec![])
        .await
        .unwrap();
    let token_1_address = token1[0];
    // println!("Token 1 {:?}", token_1_address);

    let token0_decimals = call_contract(provider.clone(), token_0_address, "decimals", vec![])
        .await
        .unwrap()[0];
    let token0_decimals_parsed =
        u8::from_le_bytes(token0_decimals.to_bytes_le()[0..1].try_into().unwrap());
    // println!("token 0 decimals {:?}", token0_decimals_parsed);

    let token1_decimals = call_contract(provider.clone(), token_1_address, "decimals", vec![])
        .await
        .unwrap()[0];

    let token1_decimals_parsed =
        u8::from_le_bytes(token1_decimals.to_bytes_le()[0..1].try_into().unwrap());
    // println!("token 1 decimals {:?}", token1_decimals_parsed);

    let reserves_result = call_contract(provider.clone(), pool_address, "getReserves", vec![])
        .await
        .unwrap();
    // println!("Reserve result {:?}", reserves_result);

    let reserve_a = Felt::from_bytes_le(&reserves_result[0].to_bytes_le());
    let reserve_b = Felt::from_bytes_le(&reserves_result[2].to_bytes_le());

    Ok(TenkSwapPool::new(
        pool_address,
        token_0_address,
        token_1_address,
        token0_decimals_parsed,
        token1_decimals_parsed,
        reserve_a,
        reserve_b,
        0u32,
    ))
}
