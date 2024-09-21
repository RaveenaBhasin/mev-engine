use std::{io::Stderr, sync::Arc};

use starknet::{
    core::types::{Felt, StarknetError},
    providers::Provider,
};

use super::{factory::TenKFactory, pool::TenkSwapPool};
use crate::utils::call_contract;
use crate::{amm::pool::AMM, errors::AMMError};

pub async fn get_all_pools<P>(
    factory_address: Felt,
    idx: u64,
    provider: Arc<P>,
) -> Result<TenkSwapPool, AMMError>
where
    P: Provider + Send + Sync,
{
    let pool_address = call_contract(
        provider.clone(),
        factory_address,
        "allPairs",
        vec![Felt::from(idx)],
    )
    .await
    .unwrap()[0];

    let token0 = call_contract(provider.clone(), pool_address, "token0", vec![])
        .await
        .unwrap();
    let token_0_address = token0[0];

    let token1 = call_contract(provider.clone(), pool_address, "token1", vec![])
        .await
        .unwrap();
    let token_1_address = token1[0];

    let token0_decimals = call_contract(provider.clone(), token_0_address, "decimals", vec![])
        .await
        .unwrap()[0];
    let token0_decimals_parsed =
        u8::from_le_bytes(token0_decimals.to_bytes_le()[0..1].try_into().unwrap());

    let token1_decimals = call_contract(provider.clone(), token_1_address, "decimals", vec![])
        .await
        .unwrap()[0];

    let token1_decimals_parsed =
        u8::from_le_bytes(token1_decimals.to_bytes_le()[0..1].try_into().unwrap());

    let reserves_result = call_contract(provider.clone(), pool_address, "get_reserves", vec![])
        .await
        .unwrap();

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
