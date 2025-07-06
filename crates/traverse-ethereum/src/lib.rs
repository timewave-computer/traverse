//! Ethereum-specific implementation for ZK storage path generation
//!
//! This crate provides Ethereum-specific implementations of the core traits,
//! including Solidity storage layout compilation and Keccak-based key resolution.

mod abi_fetcher;
mod indexer;
mod layout;
mod proof;
mod resolver;

// Re-export the main types for backward compatibility
pub use abi_fetcher::AbiFetcher;
pub use indexer::{IndexerService, MockIndexerService, SemanticValidator, ValidationResult};
pub use layout::EthereumLayoutCompiler;
pub use proof::EthereumProofFetcher;
pub use resolver::EthereumKeyResolver;

// Include RLP encoding/decoding tests
#[cfg(test)]
mod rlp_tests;
