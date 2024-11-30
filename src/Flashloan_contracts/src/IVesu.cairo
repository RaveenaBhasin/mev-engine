use starknet::ContractAddress;
#[starknet::interface]
pub trait IVesu<TContractState> {
    fn flash_loan(
        ref self: TContractState,
        receiver: ContractAddress,
        asset: ContractAddress,
        amount: u256,
        is_legacy: bool,
        data: Span<felt252>
    );
}
