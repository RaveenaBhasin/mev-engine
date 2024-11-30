use snforge_std::ContractClassTrait;
use snforge_std::DeclareResultTrait;
use snforge_std::{declare};
use starknet::{ContractAddress, contract_address_const};
use crate::IFlashLoan::{IFlashloanReceiverDispatcher, IFlashloanReceiverDispatcherTrait};

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
    flash_loan.start_flashloan(amount);
}
