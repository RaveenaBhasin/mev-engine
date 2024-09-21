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
use tracing::instrument;

use crate::{
    amm::{pool::AutomatedMarketMaker, types::Reserves},
    errors::AMMError,
};

use super::get_data::get_pool_info;

// use super::{pool::AutomatedMarketMaker, types::Reserves};

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct TenkSwapPool {
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
impl AutomatedMarketMaker for TenkSwapPool {
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
        _quote_token: Felt,
        amount_in: Felt,
        provider: Arc<P>,
    ) -> Result<Felt, StarknetError>
    where
        P: Provider + Sync + Send,
    {
        if self.token_a == base_token {
            // println!(
            //     "amount out {:?}",
            //     self.get_amount_out(amount_in, self.reserve_a, self.reserve_b)
            // );
            Ok(self.get_amount_out(amount_in, self.reserve_a, self.reserve_b))
        } else {
            // println!(
            //     "amount out {:?}",
            //     self.get_amount_out(amount_in, self.reserve_b, self.reserve_a)
            // );
            Ok(self.get_amount_out(amount_in, self.reserve_b, self.reserve_a))
        }
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

    #[instrument(skip(self, provider), level = "debug")]
    async fn sync<P>(&mut self, provider: Arc<P>) -> Result<(), StarknetError>
    where
        P: Provider + Send + Sync,
    {
        let Reserves {
            reserve_a,
            reserve_b,
        } = self.get_reserves(provider.clone()).await?;
        tracing::info!(?reserve_a, ?reserve_b, address = ?self.address(), "UniswapV2 sync");

        self.reserve_a = reserve_a;
        self.reserve_b = reserve_b;

        Ok(())
    }
}

impl TenkSwapPool {
    pub fn new(
        pool_address: Felt,
        token_a: Felt,
        token_b: Felt,
        token_a_decimals: u8,
        token_b_decimals: u8,
        reserve_a: Felt,
        reserve_b: Felt,
        fee: u32,
    ) -> TenkSwapPool {
        TenkSwapPool {
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

    pub fn get_amount_out(&self, amount_in: Felt, reserve_in: Felt, reserve_out: Felt) -> Felt {
        let amount_in = BigUint::from_bytes_be(&amount_in.to_bytes_be());
        let reserve_in = BigUint::from_bytes_be(&reserve_in.to_bytes_be());
        let reserve_out = BigUint::from_bytes_be(&reserve_out.to_bytes_be());

        print!(
            "Reserves in get_amount_out {:?} {:?}",
            reserve_in, reserve_out
        );

        if amount_in == BigUint::from(0u32)
            || reserve_in == BigUint::from(0u32)
            || reserve_out == BigUint::from(0u32)
        {
            return Felt::ZERO;
        }

        let fee = (BigUint::from(10000u32) - BigUint::from(self.fee / 10)) / BigUint::from(10u32);
        let amount_in_with_fee = &amount_in * &fee;
        let numerator = &amount_in_with_fee * &reserve_out;
        let denominator = &reserve_in * BigUint::from(1000u32) + &amount_in_with_fee;

        let result = &numerator / &denominator;

        Felt::from_bytes_be_slice(&result.to_bytes_be())
    }

    async fn get_reserves<P>(&mut self, provider: Arc<P>) -> Result<Reserves, StarknetError>
    where
        P: Provider + Sync + Send,
    {
        let call = FunctionCall {
            contract_address: self.pool_address,
            entry_point_selector: get_selector_from_name("get_reserves").unwrap(),
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

    pub async fn new_from_address<P>(
        pool_address: Felt,
        fee: u32,
        provider: Arc<P>,
    ) -> Result<Self, AMMError>
    where
        P: Provider + Send + Sync,
    {
        let mut pool = get_pool_info(pool_address, provider).await.unwrap();
        pool.fee = fee;

        Ok(pool)
    }
}
