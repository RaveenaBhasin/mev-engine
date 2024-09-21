use super::amm::pool::AMM;
use std::{
    fs::read_to_string,
    panic::resume_unwind,
    path::Path,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use serde::{Deserialize, Serialize};

use starknet::{
    core::types::{Felt, StarknetError},
    providers::Provider,
};
use tokio::task::JoinHandle;

use crate::{
    amm::{
        factory::{AutomatedMarketMakerFactory, Factory},
        jediswap::{self, factory::JediswapFactory},
        tenKSwap::{self, factory::TenKFactory},
    },
    errors::{AMMError, CheckpointError},
};

#[derive(Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    pub timestamp: usize,
    pub block_number: u64,
    pub factories: Vec<Factory>,
    pub amms: Vec<AMM>,
}

impl Checkpoint {
    pub fn new(
        timestamp: usize,
        block_number: u64,
        factories: Vec<Factory>,
        amms: Vec<AMM>,
    ) -> Checkpoint {
        Checkpoint {
            timestamp,
            block_number,
            factories,
            amms,
        }
    }
}

// Get all pairs from last synced block and sync reserve values for each Dex in the `dexes` vec.
pub async fn sync_amms_from_checkpoint<P, A>(
    path_to_checkpoint: A,
    step: u64,
    provider: Arc<P>,
) -> Result<(Vec<Factory>, Vec<AMM>), AMMError>
where
    P: Provider + Send + Sync + 'static,
    A: AsRef<Path>,
{
    let current_block = provider.block_number().await.unwrap();

    let checkpoint: Checkpoint =
        serde_json::from_str(read_to_string(&path_to_checkpoint)?.as_str())?;

    // Sort all of the pools from the checkpoint into uniswap_v2_pools and uniswap_v3_pools pools so we can sync them concurrently
    let (jediswap_pools, tenkswap_pools) = sort_amms(checkpoint.amms);

    let mut aggregated_amms = vec![];
    let mut handles = vec![];

    // Sync all uniswap v2 pools from checkpoint
    if !jediswap_pools.is_empty() {
        handles.push(
            batch_sync_amms_from_checkpoint(jediswap_pools, Some(current_block), provider.clone())
                .await,
        );
    }

    // Sync all uniswap v2 pools from checkpoint
    if !tenkswap_pools.is_empty() {
        handles.push(
            batch_sync_amms_from_checkpoint(tenkswap_pools, Some(current_block), provider.clone())
                .await,
        );
    }

    // Sync all pools from the since synced block
    handles.extend(
        get_new_amms_from_range(
            checkpoint.factories.clone(),
            checkpoint.block_number,
            current_block,
            step,
            provider.clone(),
        )
        .await,
    );

    for handle in handles {
        match handle.await {
            Ok(sync_result) => aggregated_amms.extend(sync_result?),
            Err(err) => {
                {
                    if err.is_panic() {
                        // Resume the panic on the main task
                        resume_unwind(err.into_panic());
                    }
                }
            }
        }
    }

    //update the sync checkpoint
    save_checkpoint(
        checkpoint.factories.clone(),
        &aggregated_amms,
        current_block,
        path_to_checkpoint,
    )
    .unwrap();

    Ok((checkpoint.factories, aggregated_amms))
}

pub async fn get_new_amms_from_range<P>(
    factories: Vec<Factory>,
    from_block: u64,
    to_block: u64,
    step: u64,
    provider: Arc<P>,
) -> Vec<JoinHandle<Result<Vec<AMM>, AMMError>>>
where
    P: Provider + Send + Sync + 'static,
{
    // Create the filter with all the pair created events
    // Aggregate the populated pools from each thread
    let mut handles = vec![];

    for factory in factories.into_iter() {
        let provider = provider.clone();

        // Spawn a new thread to get all pools and sync data for each dex
        handles.push(tokio::spawn(async move {
            let mut amms = factory
                .get_all_pools_from_logs(from_block, to_block, step, provider.clone())
                .await?;

            factory
                .populate_amm_data(&mut amms, Some(to_block), provider.clone())
                .await?;

            // TODO :  Clean empty pools

            Ok::<_, AMMError>(amms)
        }));
    }

    handles
}

pub async fn batch_sync_amms_from_checkpoint<P>(
    mut amms: Vec<AMM>,
    block_number: Option<u64>,
    provider: Arc<P>,
) -> JoinHandle<Result<Vec<AMM>, AMMError>>
where
    P: Provider + Send + Sync + 'static,
{
    let factory = match amms[0] {
        AMM::JediswapPool(_) => Some(Factory::JediswapFactory(JediswapFactory::new(Felt::ZERO))),
        AMM::TenkSwapPool(_) => Some(Factory::TenKFactory(TenKFactory::new(Felt::ZERO))),
    };

    // Spawn a new thread to get all pools and sync data for each dex
    tokio::spawn(async move {
        if let Some(factory) = factory {
            if amms_are_congruent(&amms) {
                // Get all pool data via batched calls
                factory
                    .populate_amm_data(&mut amms, block_number, provider)
                    .await?;

                // TODO : Clean empty pools

                Ok::<_, AMMError>(amms)
            } else {
                Err(AMMError::IncongruentAMMs)
            }
        } else {
            Ok::<_, AMMError>(vec![])
        }
    })
}

pub fn sort_amms(amms: Vec<AMM>) -> (Vec<AMM>, Vec<AMM>) {
    let mut jediswap_pools = vec![];
    let mut tenkswap_pools = vec![];
    for amm in amms {
        match amm {
            AMM::JediswapPool(_) => jediswap_pools.push(amm),
            AMM::TenkSwapPool(_) => tenkswap_pools.push(amm),
        }
    }

    (jediswap_pools, tenkswap_pools)
}

pub fn amms_are_congruent(amms: &[AMM]) -> bool {
    let expected_amm = &amms[0];

    for amm in amms {
        if std::mem::discriminant(expected_amm) != std::mem::discriminant(amm) {
            return false;
        }
    }
    true
}

pub async fn get_new_pools_from_range<P>(
    factories: Vec<Factory>,
    from_block: u64,
    to_block: u64,
    step: u64,
    provider: Arc<P>,
) -> Vec<JoinHandle<Result<Vec<AMM>, AMMError>>>
where
    P: Provider + Send + Sync + 'static,
{
    // Create the filter with all the pair created events
    // Aggregate the populated pools from each thread
    let mut handles = vec![];

    for factory in factories {
        let provider = provider.clone();

        // Spawn a new thread to get all pools and sync data for each dex
        handles.push(tokio::spawn(async move {
            let mut pools = factory
                .get_all_pools_from_logs(from_block, to_block, step, provider.clone())
                .await?;

            factory
                .populate_amm_data(&mut pools, Some(to_block), provider.clone())
                .await?;

            // TODO :Clean empty pools

            Ok::<_, AMMError>(pools)
        }));
    }

    handles
}

pub fn save_checkpoint<P>(
    factories: Vec<Factory>,
    amms: &[AMM],
    latest_block: u64,
    checkpoint_path: P,
) -> Result<(), CheckpointError>
where
    P: AsRef<Path>,
{
    let checkpoint = Checkpoint::new(
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs_f64() as usize,
        latest_block,
        factories,
        amms.to_vec(),
    );

    std::fs::write(checkpoint_path, serde_json::to_string_pretty(&checkpoint)?)?;

    Ok(())
}

// Deconstructs the checkpoint into a Vec<AMM>
pub fn read_checkpoint<P>(checkpoint_path: P) -> Result<(Vec<AMM>, u64), StarknetError>
where
    P: AsRef<Path>,
{
    let checkpoint: Checkpoint =
        serde_json::from_str(read_to_string(checkpoint_path).unwrap().as_str()).unwrap();
    Ok((checkpoint.amms, checkpoint.block_number))
}
