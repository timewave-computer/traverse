//! Error types for the traverse library
//!
//! This module defines the error types used throughout the traverse library.

use alloc::string::String;
#[cfg(feature = "serde")]
use alloc::format;
#[cfg(feature = "std")]
use std::io;

/// Error type for the traverse library
#[derive(Debug)]
#[cfg_attr(feature = "std", derive(thiserror::Error))]
pub enum TraverseError {
    /// I/O error
    #[cfg(feature = "std")]
    #[cfg_attr(feature = "std", error("I/O error: {0}"))]
    Io(#[from] io::Error),
    
    /// I/O error (no-std)
    #[cfg(not(feature = "std"))]
    #[cfg_attr(feature = "std", error("I/O error: {0}"))]
    Io(String),

    /// Serialization error
    #[cfg_attr(feature = "std", error("Serialization error: {0}"))]
    Serialization(String),

    /// Layout compilation error
    #[cfg_attr(feature = "std", error("Layout compilation error: {0}"))]
    LayoutCompilation(String),

    /// Key resolution error
    #[cfg_attr(feature = "std", error("Key resolution error: {0}"))]
    KeyResolution(String),

    /// Proof generation error
    #[cfg_attr(feature = "std", error("Proof generation error: {0}"))]
    ProofGeneration(String),

    /// Invalid input error
    #[cfg_attr(feature = "std", error("Invalid input: {0}"))]
    InvalidInput(String),

    /// Feature not supported error
    #[cfg_attr(feature = "std", error("Feature not supported: {0}"))]
    FeatureNotSupported(String),

    /// Configuration error
    #[cfg_attr(feature = "std", error("Configuration error: {0}"))]
    Configuration(String),

    /// External service error
    #[cfg_attr(feature = "std", error("External service error: {0}"))]
    ExternalService(String),

    /// Validation error
    #[cfg_attr(feature = "std", error("Validation error: {0}"))]
    Validation(String),

    /// Semantic error
    #[cfg_attr(feature = "std", error("Semantic error: {0}"))]
    Semantic(String),

    /// Circuit generation error
    #[cfg_attr(feature = "std", error("Circuit generation error: {0}"))]
    CircuitGeneration(String),

    /// Witness generation error
    #[cfg_attr(feature = "std", error("Witness generation error: {0}"))]
    WitnessGeneration(String),

    /// Memory allocation error
    #[cfg_attr(feature = "std", error("Memory allocation error: {0}"))]
    MemoryAllocation(String),

    /// Constraint violation error
    #[cfg_attr(feature = "std", error("Constraint violation: {0}"))]
    ConstraintViolation(String),

    /// Hash computation error
    #[cfg_attr(feature = "std", error("Hash computation error: {0}"))]
    HashComputation(String),

    /// Compatibility error
    #[cfg_attr(feature = "std", error("Compatibility error: {0}"))]
    Compatibility(String),
}

impl TraverseError {
    /// Create a new serialization error
    pub fn serialization(msg: impl Into<String>) -> Self {
        TraverseError::Serialization(msg.into())
    }

    /// Create a new layout compilation error
    pub fn layout_compilation(msg: impl Into<String>) -> Self {
        TraverseError::LayoutCompilation(msg.into())
    }

    /// Create a new key resolution error
    pub fn key_resolution(msg: impl Into<String>) -> Self {
        TraverseError::KeyResolution(msg.into())
    }

    /// Create a new proof generation error
    pub fn proof_generation(msg: impl Into<String>) -> Self {
        TraverseError::ProofGeneration(msg.into())
    }

    /// Create a new invalid input error
    pub fn invalid_input(msg: impl Into<String>) -> Self {
        TraverseError::InvalidInput(msg.into())
    }

    /// Create a new feature not supported error
    pub fn feature_not_supported(msg: impl Into<String>) -> Self {
        TraverseError::FeatureNotSupported(msg.into())
    }

    /// Create a new configuration error
    pub fn configuration(msg: impl Into<String>) -> Self {
        TraverseError::Configuration(msg.into())
    }

    /// Create a new external service error
    pub fn external_service(msg: impl Into<String>) -> Self {
        TraverseError::ExternalService(msg.into())
    }

    /// Create a new validation error
    pub fn validation(msg: impl Into<String>) -> Self {
        TraverseError::Validation(msg.into())
    }

    /// Create a new semantic error
    pub fn semantic(msg: impl Into<String>) -> Self {
        TraverseError::Semantic(msg.into())
    }

    /// Create a new circuit generation error
    pub fn circuit_generation(msg: impl Into<String>) -> Self {
        TraverseError::CircuitGeneration(msg.into())
    }

    /// Create a new witness generation error
    pub fn witness_generation(msg: impl Into<String>) -> Self {
        TraverseError::WitnessGeneration(msg.into())
    }

    /// Create a new memory allocation error
    pub fn memory_allocation(msg: impl Into<String>) -> Self {
        TraverseError::MemoryAllocation(msg.into())
    }

    /// Create a new constraint violation error
    pub fn constraint_violation(msg: impl Into<String>) -> Self {
        TraverseError::ConstraintViolation(msg.into())
    }

    /// Create a new hash computation error
    pub fn hash_computation(msg: impl Into<String>) -> Self {
        TraverseError::HashComputation(msg.into())
    }

    /// Create a new compatibility error
    pub fn compatibility(msg: impl Into<String>) -> Self {
        TraverseError::Compatibility(msg.into())
    }
}

#[cfg(feature = "serde")]
impl From<serde_json::Error> for TraverseError {
    fn from(err: serde_json::Error) -> Self {
        TraverseError::Serialization(format!("JSON error: {}", err))
    }
}
