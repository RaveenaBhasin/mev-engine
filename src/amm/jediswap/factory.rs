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

use super::{get_data::get_all_pools, pool::JediswapPool};

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
        // let call = FunctionCall {
        //     contract_address: self.factory_address,
        //     entry_point_selector: Felt::from_hex(
        //         "0x3e415d1aae9ddb9b1ffdb1f3bb6591b593e0a09748f635cdd067a74aba6f671",
        //     )
        //     .unwrap(),
        //     calldata: vec![],
        // };
        // let result = provider
        //     .call(call, BlockId::Tag(BlockTag::Latest))
        //     .await
        //     .unwrap();

        let result = get_all_pools(self, provider).await;
        result
    }
}

impl JediswapFactory {
    pub fn new(factory_address: Felt) -> JediswapFactory {
        JediswapFactory { factory_address }
    }
}
