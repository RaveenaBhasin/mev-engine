# MEV-Engine
A tool to help you find mev transaction on starknet.

## Installation
The library is published at [Crates Repository](https://crates.io/crates/mev-engine). To install run
```bash 
cargo add mev-engine
```

## Run an example
Check the `examples` folder to check out how the Repository can be leveraged complex strategies.
```bash
cargo run --example arbitrage_bot
```

This repository has been written keeping modularity in mind. Say if you want to add an AMM, one has to implement the required traits,
and everything works out of the box.


