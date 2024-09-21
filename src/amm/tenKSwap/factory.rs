use std::{sync::Arc, time::Duration};

use amms::amm::IErc20::new;
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
    amm::{factory::AutomatedMarketMakerFactory, pool::AMM, types::Reserves},
    errors::AMMError,
    utils::call_contract,
};

use super::{get_data::get_all_pools, pool::TenkSwapPool};

// use super::{pool::AutomatedMarketMaker, types::Reserves};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TenKFactory {
    pub factory_address: Felt,
}

#[async_trait]
impl AutomatedMarketMakerFactory for TenKFactory {
    fn address(&self) -> Felt {
        self.factory_address
    }

    // async fn fetch_all_pools<P>(&mut self, provider: Arc<P>) -> Result<Vec<AMM>, AMMError>
    // where
    //     P: Provider + Sync + Send,
    // {
    //     let pool_addresses = get_all_pools(self, provider.clone()).await.unwrap();
    //     let mut all_pools = vec![];
    //     let mut first_val = true;
    //
    //     for pool_address in pool_addresses {
    //         if first_val {
    //             first_val = false;
    //             continue;
    //         }
    //         let pool = get_pool_info(pool_address, provider.clone()).await.unwrap();
    //
    //         tokio::time::sleep(Duration::from_millis(200)).await;
    //         all_pools.push(AMM::JediswapPool(pool));
    //     }
    //     Ok(all_pools)
    // }
    //
    async fn fetch_all_pools<P>(&mut self, provider: Arc<P>) -> Result<Vec<AMM>, AMMError>
    where
        P: Provider + Sync + Send,
    {
        let pools_length = call_contract(
            provider.clone(),
            self.factory_address,
            "allPairsLength",
            vec![],
        )
        .await
        .unwrap()[0];

        let pools_length_parsed =
            u64::from_le_bytes(pools_length.to_bytes_le()[0..8].try_into().unwrap());
        let mut all_pools = vec![];

        for idx in 0..pools_length_parsed {
            let pool = get_all_pools(self.factory_address, idx, provider.clone())
                .await
                .unwrap();
            all_pools.push(AMM::TenkSwapPool(pool));
        }
        Ok(all_pools)
    }
}

impl TenKFactory {
    pub fn new(factory_address: Felt) -> TenKFactory {
        TenKFactory { factory_address }
    }
}
