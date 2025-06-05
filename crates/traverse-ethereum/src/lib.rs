//! Ethereum-specific implementation for ZK storage path generation
//! 
//! This crate provides Ethereum-specific implementations of the core traits,
//! including Solidity storage layout compilation and Keccak-based key resolution.

mod layout;
mod resolver;
mod proof;
mod abi_fetcher;

// Re-export the main types for backward compatibility
pub use layout::EthereumLayoutCompiler;
pub use resolver::EthereumKeyResolver;
pub use proof::EthereumProofFetcher;
pub use abi_fetcher::AbiFetcher;

// Include RLP encoding/decoding tests
#[cfg(test)]
mod rlp_tests; 