#[starknet::contract]
pub mod FlashLoanContract {
    use starknet::ContractAddress;
    // use openzeppelin::token::;
    use crate::IFlashLoan::IFlashloanReceiver;
    use starknet::get_caller_address;
    // use crate::{IVesu};
    use crate::IVesu::{IVesuDispatcherTrait, IVesuDispatcher};

    #[storage]
    pub struct Storage {
        pub owner: ContractAddress,
        pub vesu_dispatcher: IVesuDispatcher,
        pub token: ContractAddress,
        pub contract_balance: u256
    }

    #[constructor]
    fn constructor(ref self: ContractState, vesu: ContractAddress) {
        let caller = get_caller_address();
        self.owner.write(caller);
        let vesu_dispatcher = IVesuDispatcher { contract_address: vesu };
        self.vesu_dispatcher.write(vesu_dispatcher);
    }

    #[abi(embed_v0)]
    impl FlashLoanImpl of IFlashloanReceiver<ContractState> {
        fn on_flash_loan(
            ref self: ContractState,
            sender: ContractAddress,
            asset: ContractAddress,
            amount: u256,
            data: Span<felt252>
        ) {
            let vesu = self.vesu_dispatcher.read();
            assert(get_caller_address() == vesu.contract_address, '');
            let token = self.token.read();
            let token = IERC20Dispatcher { contract_address: token };
            let balance = token.balanceOf(get_contract_address());
            assert(balance == amount, 'Successfully received loan amount');
        }

        fn arbitrage(ref self: ContractState, amount: u256) {
            let vesu = self.vesu_dispatcher.read();
            let token = self.token.read();
            // token.approve(vesu.contract_address, amount);
            vesu.flash_loan(get_contract_address(), token, amount, false, array![].span());
        }
    }
}
