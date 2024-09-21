use std::{sync::Arc, time::Duration};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use starknet::{
    core::{
        types::{Felt, FunctionCall, StarknetError},
        utils::get_selector_from_name,
    },
    providers::Provider,
};

use crate::{
    amm::{
        factory::AutomatedMarketMakerFactory,
        pool::{AutomatedMarketMaker, AMM},
    },
    errors::AMMError,
};

use super::get_data::{get_all_pools, get_pool_info};

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

    async fn fetch_all_pools<P>(&mut self, provider: Arc<P>) -> Result<Vec<AMM>, AMMError>
    where
        P: Provider + Sync + Send,
    {
        let pool_addresses = get_all_pools(self, provider.clone()).await.unwrap();
        let mut all_pools = vec![];
        let mut first_val = true;

        for pool_address in pool_addresses {
            if first_val {
                first_val = false;
                continue;
            }
            let pool = get_pool_info(pool_address, provider.clone()).await.unwrap();

            tokio::time::sleep(Duration::from_millis(200)).await;
            all_pools.push(AMM::JediswapPool(pool));
        }
        Ok(all_pools)
    }

    async fn populate_amm_data<P>(
        &self,
        amms: &mut [AMM],
        _block_number: Option<u64>,
        middleware: Arc<P>,
    ) -> Result<(), AMMError>
    where
        P: Provider + Sync + Send,
    {
        for amm in amms {
            get_pool_info(amm.address(), middleware.clone())
                .await
                .unwrap();
        }
        Ok(())
    }

    fn amm_created_event_signature(&self) -> Felt {
        Felt::ONE
    }
}

impl JediswapFactory {
    pub fn new(factory_address: Felt) -> JediswapFactory {
        JediswapFactory { factory_address }
    }
}
