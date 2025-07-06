//! CosmWasm contract analysis and storage layout generation
//!
//! This crate provides functionality to analyze CosmWasm contracts and generate
//! storage layouts for ZK coprocessor integration. It handles CosmWasm-specific
//! storage patterns, message parsing, and storage key generation.
//!
//! # Features
//!
//! - **Contract Analysis**: Parse CosmWasm message schemas and identify storage patterns
//! - **Storage Layout Generation**: Convert CosmWasm contracts to canonical layout format
//! - **Query Resolution**: Generate storage keys for CosmWasm state access
//! - **Proof Integration**: Support for Cosmos/Tendermint storage proofs
//!
//! # Usage
//!
//! ```rust,ignore
//! use traverse_cosmos::{CosmosLayoutCompiler, CosmosKeyResolver};
//! use traverse_core::LayoutCompiler;
//!
//! let compiler = CosmosLayoutCompiler;
//! let layout = compiler.compile_layout(&contract_msg_file)?;
//!
//! let resolver = CosmosKeyResolver;
//! let path = resolver.resolve(&layout, "config.owner")?;
//! ```

#![cfg_attr(not(feature = "std"), no_std)]

pub mod contract;
pub mod layout;
pub mod resolver;

#[cfg(feature = "client")]
pub mod proof;

pub use contract::{ContractAnalysis, CosmWasmContract};
pub use layout::CosmosLayoutCompiler;
pub use resolver::CosmosKeyResolver;

#[cfg(feature = "client")]
pub use proof::{
    cosmos_iavl_spec, verify_iavl_proof, CosmosChainConfig, CosmosProofFetcher, IavlProof,
};

/// Error types specific to CosmWasm contract analysis
#[derive(Debug, thiserror::Error)]
pub enum CosmosError {
    #[error("Invalid CosmWasm contract schema: {0}")]
    InvalidSchema(String),

    #[error("Unsupported storage pattern: {0}")]
    UnsupportedPattern(String),

    #[error("Contract analysis failed: {0}")]
    AnalysisFailed(String),

    #[error("Storage key generation failed: {0}")]
    KeyGenerationFailed(String),

    #[cfg(feature = "client")]
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Traverse core error: {0}")]
    TraverseCore(#[from] traverse_core::TraverseError),
}

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
