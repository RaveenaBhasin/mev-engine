#[starknet::contract]
pub mod FlashLoanContract {
    use starknet::ContractAddress;
    use openzeppelin_token::erc20::interface::{IERC20Dispatcher, IERC20DispatcherTrait};
    use crate::interfaces::IFlashLoan::{ IFlashloanReceiver, Dex, Swap};
    use starknet::get_caller_address;
    use starknet::get_contract_address;
    // use crate::{IVesu};
    use crate::interfaces::IVesu::{IVesuDispatcherTrait, IVesuDispatcher};
    use starknet::storage::Map;
 
  



    #[storage]
    pub struct Storage {
        pub owner: ContractAddress,
        pub vesu_dispatcher: IVesuDispatcher,
        pub token: ContractAddress,
        pub contract_balance: u256,
        pub routers: Map<Dex, ContractAddress>
    }

    #[constructor]
    fn constructor(ref self: ContractState, vesu: ContractAddress, token: ContractAddress) {
        let caller = get_caller_address();
        self.owner.write(caller);
        let vesu_dispatcher = IVesuDispatcher { contract_address: vesu };
        self.vesu_dispatcher.write(vesu_dispatcher);
        self.token.write(token);
    }

    #[abi(embed_v0)]
    impl FlashLoanImpl of IFlashloanReceiver<ContractState> {
        fn on_flash_loan(
            ref self: ContractState,
            sender: ContractAddress,
            asset: ContractAddress,
            amount: u256,
            data: Span<felt252>,
        ) {
            println!("Reached callback");
            let vesu = self.vesu_dispatcher.read();
            assert(get_caller_address() == vesu.contract_address, '');
            let token = self.token.read();
            let token = IERC20Dispatcher { contract_address: token };
            let balance = token.balance_of(get_contract_address());
            assert(balance == amount, 'received loan amount');
            println!("Completed callback");
        }

        fn start_flashloan(ref self: ContractState, amount: u256, token_route: Swap[]) {
            let vesu = self.vesu_dispatcher.read();
            let token_dispatcher = IERC20Dispatcher { contract_address: self.token.read() };
            token_dispatcher.approve(vesu.contract_address, amount);
            let token = self.token.read();
            let my_address = get_contract_address();
            println!("my address {:?}", my_address);
            vesu.flash_loan(my_address, token, amount, false);
        }
    }
}
