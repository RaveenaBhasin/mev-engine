use std::{io::Stderr, sync::Arc};

use starknet::{
    core::types::{Felt, StarknetError},
    providers::Provider,
};

use crate::{amm::factory::AutomatedMarketMakerFactory, utils::call_contract};

use super::{factory::JediswapFactory, pool::JediswapPool};

pub async fn get_pool_info<P>(
    pool_address: Felt,
    provider: Arc<P>,
) -> Result<JediswapPool, StarknetError>
where
    P: Provider + Send + Sync,
{
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

    print!(
        " {:?} {:?} {:?} {:?}",
        token_0_address, token_1_address, token0_decimals_parsed, token1_decimals_parsed
    );

    // let call = FunctionCall {
    //     contract_address: self.pool_address,
    //     entry_point_selector: get_selector_from_name("get_reserves").unwrap(),
    //     calldata: vec![],
    // };
    //
    // let result = provider
    //     .call(call, BlockId::Tag(BlockTag::Latest))
    //     .await
    //     .unwrap();
    let reserves_result = call_contract(provider.clone(), pool_address, "get_reserves", vec![])
        .await
        .unwrap();

    let reserve_a = Felt::from_bytes_le(&reserves_result[0].to_bytes_le());
    let reserve_b = Felt::from_bytes_le(&reserves_result[2].to_bytes_le());

    Ok(JediswapPool::new(
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

pub async fn get_all_pools<P>(
    factory: &mut JediswapFactory,
    provider: Arc<P>,
) -> Result<Vec<Felt>, Stderr>
where
    P: Provider + Send + Sync,
{
    let all_pairs = call_contract(provider.clone(), factory.address(), "get_all_pairs", vec![])
        .await
        .unwrap();
    println!("all pair addresses {:?}", all_pairs);
    Ok(all_pairs)
}
