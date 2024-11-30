#[starknet::interface]
pub trait IFlashloanReceiver<TContractState> {
    fn on_flash_loan(
        ref self: TContractState,
        sender: ContractAddress,
        asset: ContractAddress,
        amount: u256,
        data: Span<felt252>
    );
}

