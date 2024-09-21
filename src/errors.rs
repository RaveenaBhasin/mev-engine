use starknet::{
    core::types::{Felt, U256},
    providers::ProviderError,
};
use std::time::SystemTimeError;
use thiserror::Error;
use tokio::task::JoinError;

#[derive(Error, Debug)]
pub enum AMMError {
    #[error(transparent)]
    JoinError(#[from] JoinError),
    #[error(transparent)]
    SerdeJsonError(#[from] serde_json::error::Error),
    #[error(transparent)]
    IOError(#[from] std::io::Error),
    #[error("Error when converting from hex to U256")]
    FromHexError,
    #[error("Pair for token_a/token_b does not exist in provided dexes")]
    PairDoesNotExistInDexes(Felt, Felt),
    #[error("Could not initialize new pool from event log")]
    UnrecognizedPoolCreatedEventLog,
    #[error("Error when syncing pool")]
    SyncError(Felt),
    #[error("Error when getting pool data")]
    PoolDataError,
    #[error(transparent)]
    ArithmeticError(#[from] ArithmeticError),
    #[error("No initialized ticks during v3 swap simulation")]
    NoInitializedTicks,
    #[error("No liquidity net found during v3 swap simulation")]
    NoLiquidityNet,
    #[error("Incongruent AMMS supplied to batch request")]
    IncongruentAMMs,
    #[error("Invalid ERC4626 fee")]
    InvalidERC4626Fee,
    #[error(transparent)]
    EventLogError(#[from] EventLogError),
    #[error("Block number not found")]
    BlockNumberNotFound,
    #[error(transparent)]
    SwapSimulationError(#[from] SwapSimulationError),
    #[error(transparent)]
    CheckpointError(#[from] CheckpointError),
    #[error(transparent)]
    ProviderError(#[from] ProviderError),
}

#[derive(Error, Debug)]
pub enum ArithmeticError {
    #[error("Shadow overflow")]
    ShadowOverflow(U256),
    #[error("Rounding Error")]
    RoundingError,
    #[error("Y is zero")]
    YIsZero,
    #[error("Sqrt price overflow")]
    SqrtPriceOverflow,
    #[error("U128 conversion error")]
    U128ConversionError,
    #[error("base token does not exist in pool")]
    BaseTokenDoesNotExist,
    #[error("quote token does not exist in pool")]
    QuoteTokenDoesNotExist,
}

#[derive(Error, Debug)]
pub enum EventLogError {
    #[error("Invalid event signature")]
    InvalidEventSignature,
    #[error("Log Block number not found")]
    LogBlockNumberNotFound,
}

#[derive(Error, Debug)]
pub enum SwapSimulationError {
    #[error("Could not get next tick")]
    InvalidTick,
    #[error("Liquidity underflow")]
    LiquidityUnderflow,
    #[error(transparent)]
    ArithmeticError(#[from] ArithmeticError),
}

#[derive(Error, Debug)]
pub enum CheckpointError {
    #[error(transparent)]
    SystemTimeError(#[from] SystemTimeError),
    #[error(transparent)]
    SerdeJsonError(#[from] serde_json::error::Error)
}
