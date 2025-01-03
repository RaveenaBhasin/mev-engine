use snforge_std::ContractClassTrait;
use snforge_std::DeclareResultTrait;
use snforge_std::{declare};
use starknet::{ContractAddress, contract_address_const};
use crate::interfaces::IFlashLoan::{
    IFlashloanReceiverDispatcher, IFlashloanReceiverDispatcherTrait, Swap, Dex,
};

#[test]
#[fork("MAINNET_FORK")]
fn test_flashloan() {
    let class = declare("FlashLoanContract").unwrap().contract_class();
    let vesu: ContractAddress = contract_address_const::<
        0x2545b2e5d519fc230e9cd781046d3a64e092114f07e44771e0d719d148725ef,
    >();
    let token: ContractAddress = contract_address_const::<
        0x053c91253bc9682c04929ca02ed00b3e423f6710d2ee7e0d5ebb06f3ecf368a8,
    >();
    let mut calldata = array![];
    vesu.serialize(ref calldata);
    token.serialize(ref calldata);
    let (addr, _) = class.deploy(@calldata).expect('failed');

    let flash_loan = IFlashloanReceiverDispatcher { contract_address: addr };
    let amount = 1000;

    let swap_route = array![
        Swap {
            dex: Dex::Ekubo,
            from_token: contract_address_const::<0x1>(),
            to_token: contract_address_const::<0x1>(),
        },
    ];
    flash_loan.start_flashloan(amount, swap_route);
}
