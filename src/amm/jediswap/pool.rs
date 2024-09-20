use core::f64;
use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use starknet::{
    core::types::{BlockId, BlockTag, Felt, FunctionCall, StarknetError},
    providers::Provider,
};

use crate::amm::{pool::AutomatedMarketMaker, types::Reserves};

// use super::{pool::AutomatedMarketMaker, types::Reserves};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct JediswapPool {
    pub pool_address: Felt,
    pub token_a: Felt,
    pub token_b: Felt,
    pub token_a_decimals: u8,
    pub token_b_decimals: u8,
    pub reserve_a: Felt,
    pub reserve_b: Felt,
    pub fee: u32,
}

#[async_trait]
impl AutomatedMarketMaker for JediswapPool {
    fn address(&self) -> Felt {
        self.pool_address
    }

    fn tokens(&self) -> Vec<Felt> {
        vec![self.token_a, self.token_b]
    }

    #[allow(unused)]
    fn calculate_price(&self, base_token: Felt, quote_token: Felt) -> Result<f64, StarknetError> {
        unimplemented!();
    }

    #[allow(unused)]
    async fn simulate_swap<P>(
        &self,
        base_token: Felt,
        quote_token: Felt,
        amount_in: Felt,
        provider: Arc<P>,
    ) -> Result<Felt, StarknetError>
    where
        P: Provider + Sync + Send,
    {
        unimplemented!()
    }

    /// Locally simulates a swap in the AMM.
    /// Mutates the AMM state to the state of the AMM after swapping.
    /// Returns the amount received for `amount_in` of `token_in`.
    #[allow(unused)]
    fn simulate_swap_mut(
        &mut self,
        base_token: Felt,
        quote_token: Felt,
        amount_in: Felt,
    ) -> Result<Felt, StarknetError> {
        unimplemented!()
    }

    async fn get_reserves<P>(&mut self, provider: Arc<P>) -> Result<Reserves, StarknetError>
    where
        P: Provider + Sync + Send,
    {
        let call = FunctionCall {
            contract_address: self.pool_address,
            entry_point_selector: Felt::from_hex(
                "0x3cb0e1486e633fbe3e2fafe8aedf12b70ca1860e7467ddb75a17858cde39312", //selector for get_reserves
            )
            .unwrap(),
            calldata: vec![],
        };

        let result = provider
            .call(call, BlockId::Tag(BlockTag::Latest))
            .await
            .unwrap();

        let reserve_a = Felt::from_bytes_le(&result[0].to_bytes_le());
        let reserve_b = Felt::from_bytes_le(&result[2].to_bytes_le());

        self.reserve_a = reserve_a.clone();
        self.reserve_b = reserve_b.clone();
        // let block_timestamp_last = BigUint::from_bytes_le(&result[2].to_bytes_le());
        Ok(Reserves {
            reserve_a,
            reserve_b,
        })
    }

    async fn populate_data<P>(
        &mut self,
        block_number: Option<u64>,
        middleware: Arc<P>,
    ) -> Result<(), StarknetError>
    where
        P: Provider + Sync + Send,
    {
        // batch_request::get_v2_pool_data_batch_request(self, provider.clone()).await?;
        unimplemented!();
    }
}

impl JediswapPool {
    pub fn new(
        pool_address: Felt,
        token_a: Felt,
        token_b: Felt,
        token_a_decimals: u8,
        token_b_decimals: u8,
        reserve_a: Felt,
        reserve_b: Felt,
        fee: u32,
    ) -> JediswapPool {
        JediswapPool {
            pool_address,
            token_a,
            token_b,
            token_a_decimals,
            token_b_decimals,
            reserve_a,
            reserve_b,
            fee,
        }
    }
}
