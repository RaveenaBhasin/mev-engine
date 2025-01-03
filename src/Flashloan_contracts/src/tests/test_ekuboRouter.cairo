use snforge_std::{
    ContractClassTrait, test_address, start_cheat_caller_address, stop_cheat_caller_address,
};
use snforge_std::DeclareResultTrait;
use snforge_std::{declare};
use starknet::{contract_address_const};
use crate::ekuboRouter::{IEkuboRouterDispatcher, IEkuboRouterDispatcherTrait};
use crate::ekuboRouter::{RouteNode, TokenAmount};
use ekubo::types::keys::PoolKey;
use ekubo::types::i129::i129;
use ekubo::types::delta::Delta;
use ekubo::interfaces::erc20::{IERC20Dispatcher, IERC20DispatcherTrait};

// https://docs.ekubo.org/integration-guides/reference/contract-addresses
const EKUBO_CORE_ADDRESS: felt252 =
    0x0444a09d96389aa7148f1aada508e30b71299ffe650d9c97fdaae38cb9a23384;
const EKUBO_ROUTER_ADDRESS: felt252 =
    0x0045f933adf0607292468ad1c1dedaa74d5ad166392590e72676a34d01d7b763;

// if we would use the from ekubo library, then we would get an error
// "Entry point selector 0x038ad53218834f943da60c8bafd36c36692dcb35e6d76bdd93202f5c04c0baff not
// found in contract 0x0199741822c2dc722f6f605204f35e56dbc23bceed54818168c4c49e4fb8737e"
// Moreover the types for RouterNode and TokenAmount are different
// This is because the Router interface is outdated and all recent ones have separate methods
// So we can define a necessary interface here manually
// https://voyager.online/contract/0x0199741822c2dc722f6f605204f35e56dbc23bceed54818168c4c49e4fb8737e#readContract
#[starknet::interface]
pub trait IRouter<TContractState> {
    fn multihop_swap(
        ref self: TContractState, route: Array<RouteNode>, token_amount: TokenAmount,
    ) -> Array<Delta>;
}

fn declare_and_deploy() -> IEkuboRouterDispatcher {
    // First declare and deploy a contract
    // (the name of the contract is the contract module name)
    let class = declare("EkuboRouter").unwrap().contract_class();

    // deploy function accepts a snap of an array of contract arguments serialized as felt252
    let (contract_address, _) = class
        .deploy(@array![test_address().into(), EKUBO_CORE_ADDRESS])
        .unwrap();

    // Create a Dispatcher object that will allow interacting with the deployed contract
    IEkuboRouterDispatcher { contract_address }
}

#[should_panic(expected: ('unprofitable swap',))]
#[test]
#[fork("SEPOLIA_FORK")]
fn test_empty_swap() {
    let dispatcher = declare_and_deploy();
    dispatcher.multi_multihop_swap(array![]);
}

#[test]
#[fork("SEPOLIA_FORK")]
fn test_swap() {
    // test data obtained from ekubo api with
    // curl
    // 'https://sepolia-api.ekubo.org/quote/10000000000000000/0x049d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7/0x049d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7'|
    // jq {
    //     "specifiedAmount": "10000000000000000",
    //     "amount": "11678380679671722",
    //     "route": [
    //       {
    //         "pool_key": {
    //           "token0": "0x49d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7",
    //           "token1": "0x7ab0b8855a61f480b4423c46c32fa7c553f0aac3531bbddaa282d86244f7a23",
    //           "fee": "0xccccccccccccccccccccccccccccccc",
    //           "tick_spacing": 354892,
    //           "extension": "0x73ec792c33b52d5f96940c2860d512b3884f2127d25e023eb9d44a678e4b971"
    //         },
    //         "sqrt_ratio_limit": "0x1000003f7f1380b76",
    //         "skip_ahead": "0x0"
    //       },
    //       {
    //         "pool_key": {
    //           "token0": "0x49d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7",
    //           "token1": "0x7ab0b8855a61f480b4423c46c32fa7c553f0aac3531bbddaa282d86244f7a23",
    //           "fee": "0x20c49ba5e353f80000000000000000",
    //           "tick_spacing": 354892,
    //           "extension": "0x73ec792c33b52d5f96940c2860d512b3884f2127d25e023eb9d44a678e4b971"
    //         },
    //         "sqrt_ratio_limit": "0x7ea4d9526482a9577ead999cd4fa76f2ba8dfdca5b3f2f",
    //         "skip_ahead": "0x0"
    //       }
    //     ]
    // }
    let first_node = RouteNode {
        pool_key: PoolKey {
            token0: contract_address_const::<
                0x49d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7,
            >(),
            token1: contract_address_const::<
                0x7ab0b8855a61f480b4423c46c32fa7c553f0aac3531bbddaa282d86244f7a23,
            >(),
            fee: 0xccccccccccccccccccccccccccccccc,
            tick_spacing: 354892,
            // TWAMM Extension
            // https://docs.ekubo.org/integration-guides/reference/contract-addresses
            extension: contract_address_const::<
                0x73ec792c33b52d5f96940c2860d512b3884f2127d25e023eb9d44a678e4b971,
            >(),
        },
        sqrt_ratio_limit: 0x1000003f7f1380b76,
        skip_ahead: 0x0,
    };

    let second_node = RouteNode {
        pool_key: PoolKey {
            token0: contract_address_const::<
                0x49d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7,
            >(),
            token1: contract_address_const::<
                0x7ab0b8855a61f480b4423c46c32fa7c553f0aac3531bbddaa282d86244f7a23,
            >(),
            fee: 0x20c49ba5e353f80000000000000000,
            tick_spacing: 354892,
            // TWAMM Extension
            // https://docs.ekubo.org/integration-guides/reference/contract-addresses
            extension: contract_address_const::<
                0x73ec792c33b52d5f96940c2860d512b3884f2127d25e023eb9d44a678e4b971,
            >(),
        },
        sqrt_ratio_limit: 0x7ea4d9526482a9577ead999cd4fa76f2ba8dfdca5b3f2f,
        skip_ahead: 0x0,
    };

    let token_address = contract_address_const::<
        0x49d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7,
    >();
    let amount: u128 = 0x2386f26fc10000; // 10000000000000000
    let token_amount = TokenAmount {
        token: token_address, amount: i129 { mag: amount, sign: false },
    };

    let token = IERC20Dispatcher { contract_address: token_address };

    // https://sepolia.voyager.online/contract/0x061fa009f87866652b6fcf4d8ea4b87a12f85e8cb682b912b0a79dafdbb7f362
    let rich_account = contract_address_const::<
        0x061fa009f87866652b6fcf4d8ea4b87a12f85e8cb682b912b0a79dafdbb7f362,
    >();
    // Change the caller address to another
    // https://sepolia.voyager.online/token/0x049d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7
    // https://foundry-rs.github.io/starknet-foundry/testing/using-cheatcodes.html#cheating-an-address
    start_cheat_caller_address(token_address, rich_account);
    token.transfer(test_address(), amount.into());
    stop_cheat_caller_address(token_address);
    assert(token.balanceOf(test_address()) >= amount.into(), 'trader has enough funds');

    let router = IRouterDispatcher {
        contract_address: contract_address_const::<EKUBO_ROUTER_ADDRESS>(),
    };
    let result = router.multihop_swap(array![first_node, second_node], token_amount);
    assert_eq!(starknet::get_contract_address(), test_address());
    assert_eq!(*result[0].amount0.mag, amount);
    assert_eq!(*result[1].amount0.mag, 0x2ab89317ae54c0); // 12024890918851776
}

#[test]
#[fork("SEPOLIA_FORK")]
fn test_profitable_arbitrage() {
    let first_node = RouteNode {
        pool_key: PoolKey {
            token0: contract_address_const::<
                0x49d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7,
            >(),
            token1: contract_address_const::<
                0x7ab0b8855a61f480b4423c46c32fa7c553f0aac3531bbddaa282d86244f7a23,
            >(),
            fee: 0xccccccccccccccccccccccccccccccc,
            tick_spacing: 354892,
            // TWAMM Extension
            // https://docs.ekubo.org/integration-guides/reference/contract-addresses
            extension: contract_address_const::<
                0x73ec792c33b52d5f96940c2860d512b3884f2127d25e023eb9d44a678e4b971,
            >(),
        },
        sqrt_ratio_limit: 0x1000003f7f1380b76,
        skip_ahead: 0x0,
    };

    let second_node = RouteNode {
        pool_key: PoolKey {
            token0: contract_address_const::<
                0x49d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7,
            >(),
            token1: contract_address_const::<
                0x7ab0b8855a61f480b4423c46c32fa7c553f0aac3531bbddaa282d86244f7a23,
            >(),
            fee: 0x20c49ba5e353f80000000000000000,
            tick_spacing: 354892,
            // TWAMM Extension
            // https://docs.ekubo.org/integration-guides/reference/contract-addresses
            extension: contract_address_const::<
                0x73ec792c33b52d5f96940c2860d512b3884f2127d25e023eb9d44a678e4b971,
            >(),
        },
        sqrt_ratio_limit: 0x7ea4d9526482a9577ead999cd4fa76f2ba8dfdca5b3f2f,
        skip_ahead: 0x0,
    };

    let token_address = contract_address_const::<
        0x49d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7,
    >();
    let amount: u128 = 0x2386f26fc10000; // 10000000000000000
    let token_amount = TokenAmount {
        token: token_address, amount: i129 { mag: amount, sign: false },
    };

    let token = IERC20Dispatcher { contract_address: token_address };
    // We test a flash loan so the trader shouldn't have enough funds
    let balance_before = token.balanceOf(test_address());
    assert_eq!(balance_before, 0);

    let dispatcher = declare_and_deploy();

    let route = array![first_node, second_node];
    dispatcher.multihop_swap(route, token_amount);
    let balance_after = token.balanceOf(test_address());
    let earned = balance_after - balance_before;
    assert_eq!(earned, (0x2ab89317ae54c0 - amount).into());
}

#[should_panic(expected: ('unauthorized',))]
#[test]
#[fork("SEPOLIA_FORK")]
fn test_access() {
    let dispatcher = declare_and_deploy();
    let other_address = contract_address_const::<
        0x061fa009f87866652b6fcf4d8ea4b87a12f85e8cb682b912b0a79dafdbb7f362,
    >();

    start_cheat_caller_address(dispatcher.contract_address, other_address);
    assert_ne!(other_address, test_address());

    dispatcher.multi_multihop_swap(array![]);
}

#[should_panic(expected: ('unprofitable swap',))]
#[test]
#[fork("SEPOLIA_FORK")]
fn test_unprofitable_arbitrage() {
    // swapping forth and back from the same pool is unprofitable
    let first_node = RouteNode {
        pool_key: PoolKey {
            token0: contract_address_const::<
                0x4718f5a0fc34cc1af16a1cdee98ffb20c31f5cd61d6ab07201858f4287c938d,
            >(),
            token1: contract_address_const::<
                0x49d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7,
            >(),
            fee: 0x20c49ba5e353f80000000000000000,
            tick_spacing: 1000,
            extension: contract_address_const::<0x0>(),
        },
        // ratio_limit = (int("0x12c254430f3344e33e96462b41ae77960", 16) / 2**128) ** 2
        sqrt_ratio_limit: 0x12c254430f3344e33e96462b41ae77960,
        skip_ahead: 0x0,
    };

    let second_node = RouteNode {
        pool_key: PoolKey {
            token0: contract_address_const::<
                0x4718f5a0fc34cc1af16a1cdee98ffb20c31f5cd61d6ab07201858f4287c938d,
            >(),
            token1: contract_address_const::<
                0x49d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7,
            >(),
            fee: 0x20c49ba5e353f80000000000000000,
            tick_spacing: 1000,
            extension: contract_address_const::<0x0>(),
        },
        // sqrt_ratio_limit = (1/ratio_limit)**0.5
        // hex(int(sqrt_ratio_limit * 2**128))
        sqrt_ratio_limit: 0xda58ee1140fdc0000000000000000000,
        skip_ahead: 0x0,
    };

    let token_address = contract_address_const::<
        0x49d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7,
    >();
    let amount: u128 = 0x2386f26fc10000; // 10000000000000000
    let token_amount = TokenAmount {
        token: token_address, amount: i129 { mag: amount, sign: false },
    };

    let token = IERC20Dispatcher { contract_address: token_address };
    let balance_before = token.balanceOf(test_address());
    assert_eq!(balance_before, 0);

    let dispatcher = declare_and_deploy();

    let route = array![first_node, second_node];
    dispatcher.multihop_swap(route, token_amount);
}
