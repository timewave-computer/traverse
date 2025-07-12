//! Ethereum-specific implementation for ZK storage path generation
//!
//! This crate provides Ethereum-specific implementations of the core traits,
//! including Solidity storage layout compilation and Keccak-based key resolution.
//!
//! ## Alloy Integration
//!
//! This crate supports selective alloy imports for optimal performance:
//! - `lightweight-alloy`: Selective imports of specific alloy crates
//! - Fallback: Basic functionality without alloy dependencies
//!
//! The lightweight approach imports only these specific alloy crates:
//! - `alloy-primitives`: Core primitive types (Address, B256, U256)
//! - `alloy-sol-types`: ABI encoding/decoding functionality
//! - `alloy-rpc-types-eth`: Essential RPC types for storage proofs
//! - `alloy-provider`: Basic provider functionality
//! - `alloy-transport-http`: HTTP transport layer

mod abi_fetcher;
mod indexer;
mod layout;
mod proof;
mod resolver;

// Lightweight alloy with selective imports
pub mod alloy;

// Re-export the main types for backward compatibility
pub use abi_fetcher::AbiFetcher;
pub use indexer::{IndexerService, MockIndexerService, SemanticValidator, ValidationResult};
pub use layout::EthereumLayoutCompiler;
pub use proof::EthereumProofFetcher;
pub use resolver::EthereumKeyResolver;

// Re-export lightweight alloy types
pub use alloy::{
    LightweightAlloyError, StorageProofResponse, 
    parse_address, parse_b256, alloy_features_available, available_features,
};

// Re-export conditional types
#[cfg(feature = "lightweight-alloy")]
pub use alloy::{LightweightAbi};

#[cfg(feature = "lightweight-alloy")]
pub use alloy::{Address, B256, U256, Bytes, FixedBytes, Uint};

#[cfg(feature = "lightweight-alloy")]
pub use alloy::{sol, SolValue, SolType, SolCall, SolEvent, SolError};

#[cfg(feature = "lightweight-alloy")]
pub use alloy::{
    EIP1186AccountProofResponse, EIP1186StorageProof, Block, Transaction, TransactionReceipt,
};


