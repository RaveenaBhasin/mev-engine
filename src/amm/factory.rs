use std::sync::Arc;

use async_trait::async_trait;
use futures::stream::FuturesUnordered;
use serde::{Deserialize, Serialize};
use starknet::{
    core::types::{BlockId, EventFilter, Felt},
    providers::Provider,
};

use super::{jediswap::factory::JediswapFactory, pool::AMM, tenKSwap::factory::TenKFactory};
use crate::errors::AMMError;

#[async_trait]
pub trait AutomatedMarketMakerFactory {
    /// Returns the address of the AMM.
    fn address(&self) -> Felt;

    async fn fetch_all_pools<P>(&mut self, provider: Arc<P>) -> Result<Vec<AMM>, AMMError>
    where
        P: Provider + Sync + Send;

    fn amm_created_event_signature(&self) -> Vec<Vec<Felt>>;

    /// Populates all AMMs data via batched static calls.
    async fn populate_amm_data<P>(
        &self,
        amms: &mut [AMM],
        block_number: Option<u64>,
        provider: Arc<P>,
    ) -> Result<(), AMMError>
    where
        P: Provider + Send + Sync;
}

macro_rules! factory {
    ($($factory_type:ident),+ $(,)?) => {
        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub enum Factory {
            $($factory_type($factory_type),)+
        }

        #[async_trait]
        impl AutomatedMarketMakerFactory for Factory {
            fn address(&self) -> Felt{
                match self {
                    $(Factory::$factory_type(pool) => pool.address(),)+
                }
            }


            async fn fetch_all_pools<P>(&mut self, provider: Arc<P>) -> Result<Vec<AMM>, AMMError>
            where
            P: Provider + Sync + Send
            {
                match self {
                        $(Factory::$factory_type(pool) => pool.fetch_all_pools(provider).await,)+
                }
            }

            fn amm_created_event_signature(&self) -> Vec<Vec<Felt>> {
                match self {
                    $(Factory::$factory_type(factory) => factory.amm_created_event_signature(),)+
                }
            }


            async fn populate_amm_data<P>(
                &self,
                amms: &mut [AMM],
                block_number: Option<u64>,
                provider: Arc<P>,
            ) -> Result<(), AMMError>
            where
                P: Provider + Send + Sync
            {
                match self {
                    $(Factory::$factory_type(factory) => {
                        factory.populate_amm_data(amms, block_number, provider).await
                    },)+
                }
            }
        }


        impl PartialEq for Factory {
            fn eq(&self, other: &Self) -> bool {
                self.address() == other.address()
            }
        }

        impl Eq for Factory {}
    };
}

factory!(JediswapFactory, TenKFactory);

impl Factory {
    #[allow(unused)]
    pub async fn get_all_pools_from_logs<P>(
        &self,
        mut from_block: u64,
        to_block: u64,
        step: u64,
        provider: Arc<P>,
    ) -> Result<Vec<AMM>, AMMError>
    where
        P: Provider,
    {
        let factory_address = self.address();
        let amm_created_event_signature = self.amm_created_event_signature();
        // let mut futures = FuturesUnordered::new();

        let mut aggregated_amms: Vec<AMM> = vec![];

        while from_block < to_block {
            let provider = provider.clone();
            let mut target_block = from_block + step - 1;
            if target_block > to_block {
                target_block = to_block;
            }

            let filter = EventFilter {
                from_block: Some(BlockId::Number(from_block)),
                to_block: Some(BlockId::Number(to_block)),
                address: Some(factory_address),
                keys: Some(self.amm_created_event_signature()),
            };

            futures.push(async move { provider.get_events(&filter, None, 10).await });

            // from_block += step;
        }

        // while let Some(result) = futures.next().await {
        //     let logs = result.map_err(AMMError::TransportError)?;
        //
        //     for log in logs {
        //         aggregated_amms.push(self.new_empty_amm_from_log(log).unwrap());
        //     }
        // }

        Ok(aggregated_amms)
    }
}
