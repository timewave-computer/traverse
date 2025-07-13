//! Traverse Valence Coprocessor Integration
//!
//! This crate provides integration between the traverse storage path generator
//! and the valence coprocessor framework. It includes controller helpers for
//! witness creation, circuit helpers for proof verification, and domain helpers
//! for state validation.
//!
//! ## Modular Architecture
//!
//! Following the valence-coprocessor-app pattern, this crate is designed to be
//! modular with separate controller, circuit, and domain components that can
//! be built independently based on your needs.
//!
//! ### For 3rd Party Developers
//!
//! You can generate minimal, custom crates containing only the functionality
//! you need for your specific storage queries:
//!
//! ```bash
//! # Generate a minimal controller crate for your queries
//! traverse-cli generate-controller --queries queries.json --output my-controller
//!
//! # Generate a minimal circuit crate for verification
//! traverse-cli generate-circuit --layout layout.json --output my-circuit
//! ```
//!
//! ## Controller Usage
//!
//! For witness generation in your valence app controller:
//!
//! ```toml
//! [dependencies]
//! traverse-valence = { version = "0.1", default-features = false, features = ["controller"] }
//! ```
//!
//! ```rust,ignore
//! use traverse_valence::controller::create_witness_from_request;
//! use traverse_valence::StorageVerificationRequest;
//! use valence_coprocessor::Witness;
//!
//! pub fn get_witnesses(request: StorageVerificationRequest) -> Result<Vec<Witness>, _> {
//!     let witness = create_witness_from_request(&request)?;
//!     Ok(vec![witness])
//! }
//! ```
//!
//! **Note**: Controllers are always `no_std` for maximum compatibility across environments.
//!
//! ## Circuit Usage
//!
//! For proof verification in your valence app circuit:
//!
//! ```toml
//! [dependencies]
//! traverse-valence = { version = "0.1", default-features = false, features = ["circuit"] }
//! ```
//!
//! ```rust,ignore
//! use traverse_valence::circuit::{CircuitProcessor, CircuitWitness};
//! use valence_coprocessor::Witness;
//!
//! pub fn circuit(witnesses: Vec<Witness>) -> Vec<u8> {
//!     let processor = CircuitProcessor::new(layout_commitment, field_types, field_semantics);
//!     let circuit_witnesses: Vec<CircuitWitness> = witnesses.into_iter()
//!         .map(|w| CircuitProcessor::parse_witness_from_bytes(w.as_data().unwrap()))
//!         .collect::<Result<Vec<_>, _>>()
//!         .expect("Failed to parse witnesses");
//!     
//!     let results = processor.process_batch(&circuit_witnesses);
//!     // Generate your ABI-encoded output based on results
//!     generate_abi_output(results)
//! }
//! ```
//!
//! **Note**: Circuits are always minimal/constrained for optimal ZK environments.
//!
//! ## Message Flow Integration
//!
//! ```rust,ignore
//! use traverse_valence::circuit::{CircuitProcessor, CircuitWitness, FieldType};
//!
//! // Create minimal processor for ZK circuits (always constrained)
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

// Conditional module compilation based on features
#[cfg(feature = "circuit")]
pub mod circuit;

#[cfg(feature = "controller")]
pub mod controller;

#[cfg(feature = "domain")]
pub mod domain;

// Always include messages as they're shared types
pub mod messages;

// Lightweight ABI support
#[cfg(any(feature = "lightweight-alloy", feature = "full-alloy"))]
pub mod abi;

// Code generation support
#[cfg(feature = "codegen")]
pub mod codegen;

// Minimal code generation support
#[cfg(feature = "codegen")]
pub mod minimal_codegen;

// Conditional re-exports based on enabled features
#[cfg(feature = "circuit")]
pub use circuit::{
    CircuitProcessor, CircuitResult, CircuitWitness,
    ExtractedValue, FieldType, ZeroSemantics
};

#[cfg(feature = "controller")]
pub use controller::*;

#[cfg(feature = "domain")]
pub use domain::*;

pub use messages::*;

// Re-export Solana types
pub use messages::{
    SolanaAccountQuery, SolanaAccountProof, SolanaAccountVerificationRequest,
    BatchSolanaAccountVerificationRequest, SolanaAccountValidationResult,
};

// Re-export lightweight ABI when available
#[cfg(any(feature = "lightweight-alloy", feature = "full-alloy"))]
pub use abi::{AlloyAbiTypes, AbiValue, AbiType};

// Re-export codegen when available
#[cfg(feature = "codegen")]
pub use codegen::{generate_controller_crate, generate_circuit_crate, CodegenOptions};

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
    /// Code generation error
    #[cfg(feature = "codegen")]
    CodegenError(String),
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
            #[cfg(feature = "codegen")]
            TraverseValenceError::CodegenError(msg) => write!(f, "Code generation error: {}", msg),
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

// Constrained environment prelude
#[cfg(any(feature = "no-std", feature = "constrained", feature = "embedded"))]
pub mod constrained_prelude {
    //! Prelude for constrained environments
    //! Common imports for constrained environments
    
    #[cfg(feature = "circuit")]
    pub use crate::{
        CircuitProcessor, CircuitWitness, CircuitResult,
        ExtractedValue, FieldType,
    };
    
    pub use traverse_core::{
        ConstrainedLayoutInfo, ConstrainedStorageEntry, ConstrainedFieldType,
        ConstrainedKeyResolver, MemoryUsage, ZeroSemantics,
    };
}
