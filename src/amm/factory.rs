use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use starknet::{
    core::types::{Felt, StarknetError},
    providers::Provider,
};

use crate::errors::AMMError;

use super::{jediswap::factory::JediswapFactory, pool::AMM};

#[async_trait]
pub trait AutomatedMarketMakerFactory {
    /// Returns the address of the AMM.
    fn address(&self) -> Felt;

    async fn fetch_all_pools<P>(&mut self, provider: Arc<P>) -> Result<Vec<AMM>, AMMError>
    where
        P: Provider + Sync + Send;
}

macro_rules! factory {
    ($($pool_type:ident),+ $(,)?) => {
        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub enum Factory {
            $($pool_type($pool_type),)+
        }

        #[async_trait]
        impl AutomatedMarketMakerFactory for Factory {
            fn address(&self) -> Felt{
                match self {
                    $(Factory::$pool_type(pool) => pool.address(),)+
                }
            }


            async fn fetch_all_pools<P>(&mut self, provider: Arc<P>) -> Result<Vec<AMM>, AMMError>
            where
            P: Provider + Sync + Send
            {
                match self {
                        $(Factory::$pool_type(pool) => pool.fetch_all_pools(provider).await)+
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

factory!(JediswapFactory);
