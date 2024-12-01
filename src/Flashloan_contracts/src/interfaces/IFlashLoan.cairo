use starknet::ContractAddress;

#[derive(Serde, Drop, Copy, Debug, Clone)]
pub enum Dex {
    Ekubo,
    JediSwap,
    TenkSwap,
}

#[derive(Serde, Drop, Copy, Debug)]
pub struct Swap {
    pub dex: Dex,
    pub from_token: ContractAddress,
    pub to_token: ContractAddress,
}

#[starknet::interface]
pub trait IFlashloanReceiver<TContractState> {
    fn on_flash_loan(
        ref self: TContractState,
        sender: ContractAddress,
        asset: ContractAddress,
        amount: u256,
        data: Span<felt252>,
    );

    fn start_flashloan(ref self: TContractState, amount: u256, token_route: Array<Swap>);
}

