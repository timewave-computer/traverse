//! Traverse Valence Coprocessor Integration
//! 
//! This crate provides integration between the traverse storage path generator
//! and the valence coprocessor framework. It includes controller helpers for
//! witness creation, circuit helpers for proof verification, and domain helpers
//! for state validation.
//!
//! ## Architecture
//!
//! The valence coprocessor uses a three-tier architecture:
//! - **Controller**: Creates witnesses from JSON arguments
//! - **Circuit**: Verifies proofs and computes outputs
//! - **Domain**: Validates blockchain state proofs
//!
//! ## Example Usage
//!
//! ```rust,ignore
//! use traverse_valence::{controller, circuit, domain};
//! use valence_coprocessor::Witness;
//!
//! // Controller: Create witnesses from traverse output
//! let witnesses = controller::create_storage_witnesses(&json_args)?;
//!
//! // Circuit: Verify and extract values
//! let results = circuit::verify_and_extract(&witnesses)?;
//!
//! // Domain: Validate state proofs
//! let state_proof = domain::get_state_proof(&args)?;
//! ```

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

// Re-export the module contents at the crate root for convenience
pub use controller::*;
pub use circuit::*;
pub use domain::*;

/// Error type for valence coprocessor integration
#[derive(Debug)]
pub enum TraverseValenceError {
    /// JSON parsing or serialization error
    Json(String),
    /// Invalid storage key format
    InvalidStorageKey(String),
    /// Proof verification failed
    ProofVerificationFailed(String),
    /// Layout commitment mismatch
    LayoutMismatch(String),
    /// Invalid witness format
    InvalidWitness(String),
}

impl core::fmt::Display for TraverseValenceError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            TraverseValenceError::Json(msg) => write!(f, "JSON error: {}", msg),
            TraverseValenceError::InvalidStorageKey(msg) => write!(f, "Invalid storage key: {}", msg),
            TraverseValenceError::ProofVerificationFailed(msg) => write!(f, "Proof verification failed: {}", msg),
            TraverseValenceError::LayoutMismatch(msg) => write!(f, "Layout mismatch: {}", msg),
            TraverseValenceError::InvalidWitness(msg) => write!(f, "Invalid witness: {}", msg),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for TraverseValenceError {}

impl From<TraverseValenceError> for anyhow::Error {
    fn from(err: TraverseValenceError) -> Self {
        anyhow::anyhow!("{}", err)
    }
}

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