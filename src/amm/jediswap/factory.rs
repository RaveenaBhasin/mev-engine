use core::f64;
use std::sync::Arc;

use async_trait::async_trait;
use num_bigint::BigUint;
use serde::{Deserialize, Serialize};
use starknet::{
    core::{
        types::{BlockId, BlockTag, Felt, FunctionCall, StarknetError},
        utils::get_selector_from_name,
    },
    providers::Provider,
};

use crate::amm::{factory::AutomatedMarketMakerFactory, pool::AMM, types::Reserves};

use super::pool::JediswapPool;

// use super::{pool::AutomatedMarketMaker, types::Reserves};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct JediswapFactory {
    pub factory_address: Felt,
}

#[async_trait]
impl AutomatedMarketMakerFactory for JediswapFactory {
    fn address(&self) -> Felt {
        self.factory_address
    }

    async fn fetch_all_pools<P>(&mut self, provider: Arc<P>) -> Vec<Felt>
    where
        P: Provider + Sync + Send,
    {
        let call = FunctionCall {
            contract_address: self.factory_address,
            entry_point_selector: Felt::from_hex(
                "0x28ed435a719e8c0f61e6e9ba51b6b65862756bac4a23eee295bab4d097fa57c",
            )
            .unwrap(),
            calldata: vec![],
        };
        let result = provider
            .call(call, BlockId::Tag(BlockTag::Latest))
            .await
            .unwrap();
        result
    }
}

impl JediswapFactory {
    pub fn new(factory_address: Felt) -> JediswapFactory {
        JediswapFactory { factory_address }
    }

    pub async fn get_all_pools_via_batched_request<P>(&mut self, provider: Arc<P>) -> Vec<Felt>
    where
        P: Provider + Sync + Send,
    {
        let all_pairs_selector = get_selector_from_name("get_num_of_pairs").unwrap();
        let call = FunctionCall {
            contract_address: self.factory_address,
            entry_point_selector: all_pairs_selector,
            calldata: vec![],
        };
        let result = provider
            .call(call, BlockId::Tag(BlockTag::Latest))
            .await
            .unwrap();
        // result

        let pairs_length = BigUint::from_bytes_be(&result[0].to_bytes_be());

        let mut pairs = vec![];
        let step = BigUint::from(100u32);
        let mut idx_from = BigUint::from(0u32);
        let mut idx_to = if &step > &pairs_length {
            pairs_length.clone()
        } else {
            step.clone()
        };

        let mut amms = vec![];
        for addr in pairs {
            let amm = JediswapPool {
                pool_address: addr,
                ..Default::default()
            };
            amms.push(AMM::JediswapPool(amm));
        }
        result
    }
}
