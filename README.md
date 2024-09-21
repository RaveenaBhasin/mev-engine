MEV-Engine 

MEV-Engine is a library written in Rust which is a simple and modular framework for writing MEV bots or strategies.
MEV searchers face significant challenges when tracking opportunities across different protocols. Each protocol often has its own unique architecture, transaction sequencing, and state synchronization patterns, requiring searchers to constantly adapt their bots to monitor various sources for profitable trades. 
This crate is implemented by keeping modularity in mind to interact with a variety of AMMs for now. We also plan to extend this library for aggregating various lending-borrowing protocols as well.
