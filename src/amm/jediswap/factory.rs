use core::f64;
use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use starknet::{
    core::types::{BlockId, BlockTag, Felt, FunctionCall, StarknetError},
    providers::Provider,
};

use crate::amm::{factory::AutomatedMarketMakerFactory, pool::AMM, types::Reserves};

// use super::{pool::AutomatedMarketMaker, types::Reserves};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct JediswapFactory {
    pub pool_address: Felt,
}

#[async_trait]
impl AutomatedMarketMakerFactory for JediswapFactory {
    fn address(&self) -> Felt {
        self.pool_address
    }

    async fn fetch_all_pools<P>(&mut self, provider: Arc<P>) -> Vec<AMM>
    where
        P: Provider + Sync + Send,
    {
        unimplemented!()
    }
}

impl JediswapFactory {
    pub fn new(pool_address: Felt) -> JediswapFactory {
        JediswapFactory { pool_address }
    }
}
