use crate::amm::pool::AutomatedMarketMaker;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use starknet::{core::types::Felt, providers::Provider};
use std::sync::Arc;

use crate::{
    amm::{factory::AutomatedMarketMakerFactory, pool::AMM},
    errors::AMMError,
    utils::call_contract,
};

use super::get_data::get_pool_info;

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
        // println!("total no. of pools {:?}", pools_length_parsed);
        let mut all_pools = vec![];

        for idx in 0..pools_length_parsed {
            let pool_address = call_contract(
                provider.clone(),
                self.address(),
                "allPairs",
                vec![Felt::from(idx)],
            )
            .await
            .unwrap()[0];
            // println!("pool address {:?}", pool_address);

            let pool = get_pool_info(pool_address, provider.clone()).await.unwrap();
            all_pools.push(AMM::TenkSwapPool(pool));
        }
        Ok(all_pools)
    }

    fn amm_created_event_signature(&self) -> Vec<Vec<Felt>> {
        vec![vec![Felt::ONE]]
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
}

impl TenKFactory {
    pub fn new(factory_address: Felt) -> TenKFactory {
        TenKFactory { factory_address }
    }
}
