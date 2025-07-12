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
//! - **Controller**: Creates witnesses from JSON arguments using `get_witnesses()`
//! - **Circuit**: Verifies storage proofs and generates ABI-encoded output using `circuit()`
//! - **Domain**: Validates blockchain state proofs and storage verification
//!
//! ## Message Flow Integration
//!
//! ```rust,ignore
//! use traverse_valence::circuit::{CircuitProcessor, CircuitWitness, FieldType};
//!
//! // Create minimal processor for ZK circuits
//! let processor = CircuitProcessor::new(layout_commitment, field_types);
//!
//! // Process witnesses with maximum efficiency
//! let results = processor.process_batch(&circuit_witnesses);
//! ```
//!
//! ## Storage Proof Integration
//!
//! This crate extends Valence coprocessors with Ethereum storage proof capabilities:
//!
//! 1. **External Setup**: Use `traverse-cli` to generate storage keys
//! 2. **External Fetching**: Use `eth_getProof` to fetch storage proofs  
//! 3. **Controller**: Combine traverse queries with storage proofs into witnesses
//! 4. **Circuit**: Verify proofs and extract typed values or generate complex messages
//!
//! ## ABI-Encoded Output
//!
//! The circuit helpers can generate ABI-encoded structures compatible with
//! the Valence Authorization contract ecosystem, following the same patterns
//! as other Valence coprocessor applications.
//!
//! ## Lightweight Alloy Integration
//!
//! This crate supports multiple levels of alloy integration:
//! - `lightweight-alloy`: Minimal alloy imports for reduced compilation time
//! - `full-alloy`: Complete alloy ecosystem for full functionality
//! - Fallback: Basic encoding without alloy dependencies
//!
//! ## Minimal Circuit Support
//!
//! For ZK circuits and constrained environments:
//! ```rust,ignore
//! use traverse_valence::circuit::{CircuitProcessor, CircuitWitness, FieldType};
//! 
//! // Create minimal processor for maximum efficiency
//! let processor = CircuitProcessor::new(layout_commitment, field_types);
//! 
//! // Process witnesses with no error handling overhead
//! let result = processor.process_witness(&witness); // Returns Valid/Invalid
//! ```

#![no_std]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

use alloc::{string::String, vec::Vec};
use serde::{Deserialize, Serialize};

// Module declarations
pub mod circuit;
pub mod controller;
pub mod domain;
pub mod messages;

// Lightweight ABI support
#[cfg(any(feature = "lightweight-alloy", feature = "full-alloy"))]
pub mod abi;

// Constrained environment support


// Re-export the module contents at the crate root for convenience
pub use circuit::{
    CircuitProcessor, CircuitResult, CircuitWitness,
    ExtractedValue, FieldType, ZeroSemantics
};
pub use controller::*;
pub use domain::*;
pub use messages::*;

// Re-export lightweight ABI when available
#[cfg(any(feature = "lightweight-alloy", feature = "full-alloy"))]
pub use abi::{AlloyAbiTypes, AbiValue, AbiType};


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
    /// ABI encoding/decoding error
    AbiError(String),
    /// Storage proof validation error
    StorageProofError(String),

}

impl core::fmt::Display for TraverseValenceError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            TraverseValenceError::Json(msg) => write!(f, "JSON error: {}", msg),
            TraverseValenceError::InvalidStorageKey(msg) => {
                write!(f, "Invalid storage key: {}", msg)
            }
            TraverseValenceError::ProofVerificationFailed(msg) => {
                write!(f, "Proof verification failed: {}", msg)
            }
            TraverseValenceError::LayoutMismatch(msg) => write!(f, "Layout mismatch: {}", msg),
            TraverseValenceError::InvalidWitness(msg) => write!(f, "Invalid witness: {}", msg),
            TraverseValenceError::AbiError(msg) => write!(f, "ABI error: {}", msg),
            TraverseValenceError::StorageProofError(msg) => {
                write!(f, "Storage proof error: {}", msg)
            }
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for TraverseValenceError {}



/// Coprocessor-compatible storage query format (matches traverse-cli output)
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

/// Complete storage verification request combining traverse query with blockchain proof
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StorageVerificationRequest {
    /// Storage query generated by traverse-cli
    pub storage_query: CoprocessorStorageQuery,
    /// Storage proof from eth_getProof
    pub storage_proof: StorageProof,
    /// Optional contract address for additional validation
    pub contract_address: Option<String>,
    /// Optional block number for proof validation
    pub block_number: Option<u64>,
}

/// Batch storage verification for multiple queries
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BatchStorageVerificationRequest {
    /// Multiple storage verification requests
    pub storage_batch: Vec<StorageVerificationRequest>,
    /// Common contract address (if all queries are for same contract)
    pub contract_address: Option<String>,
    /// Common block number (if all proofs are from same block)
    pub block_number: Option<u64>,
}

#[cfg(any(feature = "no-std", feature = "constrained", feature = "embedded"))]
pub mod constrained_prelude {
    //! Prelude for constrained environments
    //! Common imports for constrained environments
    
    pub use crate::{
        CircuitProcessor, CircuitWitness, CircuitResult,
        ExtractedValue, FieldType,
    };
    
    pub use traverse_core::{
        ConstrainedLayoutInfo, ConstrainedStorageEntry, ConstrainedFieldType,
        ConstrainedKeyResolver, MemoryUsage, ZeroSemantics,
    };
}
