use starknet::core::types::Felt;

#[derive(Debug)]
pub struct Reserves {
    pub reserve_a: Felt,
    pub reserve_b: Felt,
    // pub block_timestamp_last: BigUint,
}
