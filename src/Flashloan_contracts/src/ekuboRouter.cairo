use ekubo::types::delta::{Delta};
use ekubo::types::i129::{i129, i129Trait};
use ekubo::types::keys::{PoolKey};
use starknet::{ContractAddress};

#[derive(Serde, Copy, Drop)]
pub struct RouteNode {
    pub pool_key: PoolKey,
    pub sqrt_ratio_limit: u256,
    pub skip_ahead: u128,
}

#[derive(Serde, Copy, Drop, Debug)]
pub struct TokenAmount {
    pub token: ContractAddress,
    pub amount: i129,
}

#[derive(Serde, Copy, Drop)]
pub struct Amount {
    pub amount: ContractAddress,
}

#[derive(Serde, Drop)]
pub struct Swap {
    pub route: Array<RouteNode>,
    pub token_amount: TokenAmount,
}

#[starknet::interface]
pub trait IEkuboRouter<TContractState> {
    // Does a single swap against a single node using tokens held by this contract, and receives the
    // output to this contract
    fn swap(ref self: TContractState, node: RouteNode, token_amount: TokenAmount) -> Delta;

    // Does a multihop swap, where the output/input of each hop is passed as input/output of the
    // next swap Note to do exact output swaps, the route must be given in reverse
    fn multihop_swap(
        ref self: TContractState, route: Array<RouteNode>, token_amount: TokenAmount,
    ) -> Array<Delta>;

    // Does multiple multihop swaps
    fn multi_multihop_swap(ref self: TContractState, swaps: Array<Swap>) -> Array<Array<Delta>>;

    fn get_owner(self: @TContractState, amount: Amount) -> ContractAddress;
}

#[starknet::contract]
pub mod EkuboRouter {
    use starknet::storage::{StoragePointerWriteAccess, StoragePointerReadAccess};
    use core::array::{Array, ArrayTrait};
    use core::option::{OptionTrait};
    use ekubo::components::clear::{ClearImpl};
    use ekubo::components::shared_locker::{consume_callback_data, call_core_with_callback};
    use ekubo::interfaces::core::{ICoreDispatcher, ICoreDispatcherTrait, ILocker, SwapParameters};
    use starknet::{ContractAddress};
    use super::{Delta, IEkuboRouter, RouteNode, TokenAmount, Swap};
    use ekubo::types::i129::{i129, i129Trait};
    use core::num::traits::Zero;
    use super::Amount;

    #[abi(embed_v0)]
    impl Clear = ekubo::components::clear::ClearImpl<ContractState>;

    #[storage]
    struct Storage {
        core: ICoreDispatcher,
        owner: ContractAddress,
    }

    #[constructor]
    fn constructor(ref self: ContractState, core: ICoreDispatcher, _owner: ContractAddress) {
        self.core.write(core);
        self.owner.write(_owner);
    }

    #[abi(embed_v0)]
    impl LockerImpl of ILocker<ContractState> {
        fn locked(ref self: ContractState, id: u32, data: Span<felt252>) -> Span<felt252> {
            let core = self.core.read();

            let mut swaps = consume_callback_data::<Array<Swap>>(core, data);
            println!("Number of swaps to process: {:?}", swaps.len());

            let mut total_profit: i129 = Zero::zero();
            let mut token: ContractAddress = Zero::zero();
            let recipient: ContractAddress = self.owner.read();

            while let Option::Some(swap) = swaps.pop_front() {
                let mut route = swap.route;
                let mut token_amount = swap.token_amount;
                println!("Initial token: {:?}", token_amount.token);
                println!("Initial amount: {:?}", token_amount.amount);

                token = swap.token_amount.token;

                let loaned_amount = swap.token_amount;

                while let Option::Some(node) = route.pop_front() {
                    println!("\n--- Processing Route Node ---");
                    let is_token1 = token_amount.token == node.pool_key.token1;
                    println!(
                        "Pool tokens: {:?} / {:?}", node.pool_key.token0, node.pool_key.token1,
                    );
                    println!("Is token1: {:?}", is_token1);
                    println!("Sqrt ratio limit: {:?}", node.sqrt_ratio_limit);
                    println!("Skip ahead: {:?}", node.skip_ahead);

                    let delta = core
                        .swap(
                            node.pool_key,
                            SwapParameters {
                                amount: token_amount.amount,
                                is_token1: is_token1,
                                sqrt_ratio_limit: node.sqrt_ratio_limit,
                                skip_ahead: node.skip_ahead,
                            },
                        );
                    println!(
                        "Swap delta - amount0: {:?}, amount1: {:?}", delta.amount0, delta.amount1,
                    );

                    token_amount =
                        if (is_token1) {
                            TokenAmount { amount: -delta.amount0, token: node.pool_key.token0 }
                        } else {
                            TokenAmount { amount: -delta.amount1, token: node.pool_key.token1 }
                        };

                    println!("Token amount {:?}", token_amount);
                };
                println!("\n=== Swap Complete ===");
                println!("Final token: {:?}", token_amount.token);
                println!("Initial amount: {:?}", loaned_amount.amount);
                println!("Final amount: {:?}", token_amount.amount);

                // assert(token_amount.token == loaned_amount.token, 'the same token');
                total_profit += token_amount.amount - loaned_amount.amount;
            };
            println!("\n=== Final Results ===");
            println!("Total profit: {:?}", total_profit);
            assert(!total_profit.is_negative(), 'unprofitable swap');

            // Withdraw profits
            core.withdraw(token, recipient, total_profit.try_into().unwrap());
            println!("Withdrawal complete");

            let mut serialized: Array<felt252> = array![];
            let mut outputs: Array<Array<Delta>> = ArrayTrait::new();
            Serde::serialize(@outputs, ref serialized);
            println!("=== Locked Function Complete ===");
            serialized.span()
        }
    }


    #[abi(embed_v0)]
    impl EkuboRouterImpl of IEkuboRouter<ContractState> {
        fn swap(ref self: ContractState, node: RouteNode, token_amount: TokenAmount) -> Delta {
            let mut deltas: Array<Delta> = self.multihop_swap(array![node], token_amount);
            deltas.pop_front().unwrap()
        }

        #[inline(always)]
        fn multihop_swap(
            ref self: ContractState, route: Array<RouteNode>, token_amount: TokenAmount,
        ) -> Array<Delta> {
            let mut result = self.multi_multihop_swap(array![Swap { route, token_amount }]);

            result.pop_front().unwrap()
        }

        #[inline(always)]
        fn multi_multihop_swap(ref self: ContractState, swaps: Array<Swap>) -> Array<Array<Delta>> {
            println!("Multiswap");
            call_core_with_callback(self.core.read(), @swaps)
        }

        fn get_owner(self: @ContractState, amount: Amount) -> ContractAddress {
            self.owner.read()
        }
    }
}
