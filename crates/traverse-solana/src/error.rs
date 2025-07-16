//! Error types for Solana-specific operations
//!
//! This module defines all error types that can occur during Solana account analysis,
//! IDL parsing, key resolution, and proof generation operations.

use thiserror::Error;

/// Solana-specific errors for traverse operations
#[derive(Error, Debug)]
pub enum SolanaError {
    /// IDL parsing failed
    #[error("Invalid IDL: {0}")]
    InvalidIdl(String),
    
    /// IDL parsing failed (legacy variant)
    #[error("IDL parsing error: {0}")]
    IdlParsingError(String),
    
    /// Account not found on-chain
    #[error("Account not found: {0}")]
    AccountNotFound(String),
    
    /// Invalid program ID
    #[error("Invalid program ID: {0}")]
    InvalidProgramId(String),
    
    /// Program Derived Account derivation failed
    #[error("PDA derivation failed: {0}")]
    PdaDerivationFailed(String),
    
    /// Associated Token Account derivation failed
    #[error("ATA derivation failed: {0}")]
    AtaDerivationFailed(String),
    
    /// Account proof verification failed
    #[error("Proof verification failed: {0}")]
    ProofVerificationFailed(String),
    
    /// Network/RPC communication error
    #[error("Network error: {0}")]
    NetworkError(String),
    
    /// Invalid account data format
    #[error("Invalid account data: {0}")]
    InvalidAccountData(String),
    
    /// Anchor-specific parsing error
    #[error("Anchor parsing error: {0}")]
    AnchorError(String),
    
    /// Address parsing/validation error
    #[error("Address parsing error: {0}")]
    AddressParsingError(String),
    
    /// Invalid query format or parameters
    #[error("Invalid query: {0}")]
    InvalidQuery(String),
    
    /// Account parsing error
    #[error("Account parsing error: {0}")]
    AccountParsingError(String),
    
    /// RPC error response
    #[error("RPC error: {0}")]
    RpcError(String),
    
    /// Feature not enabled error
    #[error("Feature not enabled: {0}")]
    FeatureNotEnabled(String),
    
    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
    
    /// JSON serialization/deserialization error
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
    
    /// Hex encoding/decoding error
    #[error("Hex error: {0}")]
    HexError(#[from] hex::FromHexError),
    
    /// Base58 encoding/decoding error
    #[error("Base58 error: {0}")]
    Base58Error(String),
    
    /// Core traverse error propagation
    #[error("Core traverse error: {0}")]
    CoreError(#[from] traverse_core::TraverseError),
    
    /// General anyhow error for complex cases
    #[error("General error: {0}")]
    GeneralError(#[from] anyhow::Error),
    
    /// I/O errors during file operations
    #[cfg(feature = "std")]
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
    
    /// HTTP client errors during RPC calls
    #[cfg(feature = "client")]
    #[error("HTTP error: {0}")]
    HttpError(String),
    
    /// Solana SDK errors
    #[cfg(feature = "solana")]
    #[error("Solana SDK error: {0}")]
    SolanaError(String),
}

impl SolanaError {
    /// Create a network error from any error type
    pub fn network<E: std::fmt::Display>(error: E) -> Self {
        Self::NetworkError(error.to_string())
    }
    
    /// Create a base58 error from any error type
    pub fn base58<E: std::fmt::Display>(error: E) -> Self {
        Self::Base58Error(error.to_string())
    }
    
    /// Create an HTTP error from any error type (requires client feature)
    #[cfg(feature = "client")]
    pub fn http<E: std::fmt::Display>(error: E) -> Self {
        Self::HttpError(error.to_string())
    }
    
    // #[cfg(feature = "solana")] // Disabled temporarily
    // pub fn solana<E: std::fmt::Display>(error: E) -> Self {
    //     Self::SolanaError(error.to_string())
    // }
}

/// Convert reqwest errors to SolanaError (requires client feature)
#[cfg(feature = "client")]
impl From<reqwest::Error> for SolanaError {
    fn from(error: reqwest::Error) -> Self {
        Self::HttpError(error.to_string())
    }
}

/// Result type alias for Solana operations
pub type SolanaResult<T> = std::result::Result<T, SolanaError>;

/// Standard Result type alias
pub type Result<T> = std::result::Result<T, SolanaError>;