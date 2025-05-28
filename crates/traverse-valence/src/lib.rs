//! Traverse Valence Coprocessor Integration
//! 
//! This crate provides integration between the traverse storage path generator
//! and the valence coprocessor framework. It includes controller helpers for
//! creating witnesses, circuit helpers for proof verification, and domain
//! helpers for state proof validation.

#![no_std]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

use alloc::{string::String, vec::Vec};
use serde::{Deserialize, Serialize};

// Module declarations
pub mod controller;
pub mod circuit;
pub mod domain;
pub mod utils;

// Re-export key types and functions for convenience
pub use controller::{create_storage_witness, create_batch_storage_witnesses};
pub use circuit::{verify_storage_proof, extract_u64_value, extract_address_value};
pub use domain::{EthereumBlockHeader, ValidatedStateProof, validate_storage_proof, validate_ethereum_state_proof};
pub use utils::parse_hex_32;

/// Error type for valence coprocessor integration
#[derive(Debug)]
pub enum ValenceError {
    /// JSON parsing or serialization error
    Json(String),
    /// Invalid storage key format
    InvalidStorageKey(String),
    /// Proof verification failed
    ProofVerificationFailed(String),
    /// Layout commitment mismatch
    LayoutMismatch(String),
}

impl core::fmt::Display for ValenceError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ValenceError::Json(msg) => write!(f, "JSON error: {}", msg),
            ValenceError::InvalidStorageKey(msg) => write!(f, "Invalid storage key: {}", msg),
            ValenceError::ProofVerificationFailed(msg) => write!(f, "Proof verification failed: {}", msg),
            ValenceError::LayoutMismatch(msg) => write!(f, "Layout mismatch: {}", msg),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for ValenceError {}

/// Coprocessor-compatible storage query format (matches CLI output)
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CoprocessorStorageQuery {
    /// Original query string
    pub query: String,
    /// Pre-computed storage key (hex encoded)
    pub storage_key: String,
    /// Layout commitment for verification (hex encoded)
    pub layout_commitment: String,
    /// Field size in bytes
    pub field_size: Option<u8>,
    /// Byte offset within storage slot
    pub offset: Option<u8>,
}

/// Storage proof data from eth_getProof
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StorageProof {
    /// Storage key (hex encoded)
    pub key: String,
    /// Storage value (hex encoded)
    pub value: String,
    /// Merkle proof nodes (hex encoded)
    pub proof: Vec<String>,
}

/// Mock witness type (placeholder until we integrate actual valence-coprocessor)
#[derive(Debug, Clone)]
pub enum MockWitness {
    /// Raw data witness
    Data(Vec<u8>),
    /// Storage proof witness
    StateProof {
        /// Storage key
        key: [u8; 32],
        /// Storage value
        value: [u8; 32],
        /// Proof nodes
        proof: Vec<[u8; 32]>,
    },
} 