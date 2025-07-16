//! Lightweight Alloy Selective Imports
//!
//! This module provides selective imports from the alloy ecosystem, importing only
//! the specific components we need for ABI encoding/decoding and basic type functionality.
//! This approach minimizes dependencies and compilation time while maintaining 
//! type compatibility with the alloy ecosystem.

#[cfg(feature = "std")]
use std::{format, string::String, vec, vec::Vec};

#[cfg(not(feature = "std"))]
use alloc::{format, string::String, vec, vec::Vec};

use serde::{Deserialize, Serialize};

// =============================================================================
// Selective Alloy Imports
// =============================================================================

// Core primitive types - these are the foundation
#[cfg(feature = "lightweight-alloy")]
pub use alloy_primitives::{Address, B256, U256, Bytes, FixedBytes, Uint};

// ABI encoding/decoding functionality
#[cfg(feature = "lightweight-alloy")]
pub use alloy_sol_types::{sol, SolValue, SolType, SolCall, SolEvent, SolError};

// Essential RPC types for storage proofs
#[cfg(feature = "lightweight-alloy")]
pub use alloy_rpc_types_eth::{
    EIP1186AccountProofResponse, EIP1186StorageProof, Block, Transaction, TransactionReceipt,
};

// =============================================================================
// Lightweight Error Types
// =============================================================================

/// Lightweight error type for alloy operations
#[derive(Debug, thiserror::Error)]
pub enum LightweightAlloyError {
    #[error("Invalid address: {0}")]
    InvalidAddress(String),
    
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),
    
    #[error("RPC error: {0}")]
    RpcError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("ABI encoding error: {0}")]
    AbiError(String),
    
    #[error("No proof returned")]
    NoProofReturned,
    
    #[error("Feature not enabled: {0}")]
    FeatureNotEnabled(String),
}

impl From<LightweightAlloyError> for traverse_core::TraverseError {
    fn from(err: LightweightAlloyError) -> Self {
        traverse_core::TraverseError::ProofGeneration(format!("Lightweight alloy error: {}", err))
    }
}

impl From<serde_json::Error> for LightweightAlloyError {
    fn from(err: serde_json::Error) -> Self {
        LightweightAlloyError::SerializationError(format!("JSON error: {}", err))
    }
}

// =============================================================================
// Simplified Storage Proof Types
// =============================================================================

/// Simplified storage proof response using basic types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageProofResponse {
    #[serde(with = "hex")]
    pub key: [u8; 32],
    #[serde(with = "hex")]
    pub value: [u8; 32],
    pub proof_nodes: Vec<String>, // Hex-encoded proof nodes
}

impl StorageProofResponse {
    /// Convert to traverse core semantic storage proof
    pub fn to_semantic_proof(
        &self,
        semantics: traverse_core::StorageSemantics,
    ) -> traverse_core::SemanticStorageProof {
        // Convert hex-encoded proof nodes to fixed-size arrays
        let proof_nodes: Vec<[u8; 32]> = self.proof_nodes
            .iter()
            .map(|node_hex| {
                let node_bytes = hex::decode(node_hex.strip_prefix("0x").unwrap_or(node_hex))
                    .unwrap_or_default();
                
                if node_bytes.len() == 32 {
                    let mut array = [0u8; 32];
                    array.copy_from_slice(&node_bytes);
                    array
                } else if node_bytes.len() < 32 {
                    // Pad with zeros
                    let mut array = [0u8; 32];
                    array[32 - node_bytes.len()..].copy_from_slice(&node_bytes);
                    array
                } else {
                    // Hash longer nodes to fit 32 bytes
                    #[cfg(feature = "ethereum")]
                    {
                        use tiny_keccak::{Hasher, Keccak};
                        let mut keccak = Keccak::v256();
                        keccak.update(&node_bytes);
                        let mut array = [0u8; 32];
                        keccak.finalize(&mut array);
                        array
                    }
                    #[cfg(not(feature = "ethereum"))]
                    {
                        // Without ethereum feature, just truncate
                        let mut array = [0u8; 32];
                        array.copy_from_slice(&node_bytes[..32]);
                        array
                    }
                }
            })
            .collect();

        traverse_core::SemanticStorageProof {
            key: self.key,
            value: self.value,
            proof: proof_nodes,
            semantics,
        }
    }
}

// =============================================================================
// ABI Encoding Utilities
// =============================================================================

/// Lightweight ABI encoder using alloy-sol-types
#[cfg(feature = "lightweight-alloy")]
pub struct LightweightAbi;

#[cfg(feature = "lightweight-alloy")]
impl LightweightAbi {
    /// Encode a simple function call
    pub fn encode_function_call<T: SolCall>(call: &T) -> Result<Vec<u8>, LightweightAlloyError> {
        Ok(call.abi_encode())
    }

    /// Decode a function return value
    pub fn decode_function_return<T>(data: &[u8]) -> Result<T, LightweightAlloyError> 
    where
        T: SolValue + From<<T::SolType as SolType>::RustType>,
    {
        T::abi_decode(data, true)
            .map_err(|e| LightweightAlloyError::AbiError(format!("{}", e)))
    }

    /// Generate function selector for a function signature
    pub fn function_selector(signature: &str) -> [u8; 4] {
        #[cfg(feature = "ethereum")]
        {
            use tiny_keccak::{Hasher, Keccak};
            let mut keccak = Keccak::v256();
            keccak.update(signature.as_bytes());
            let mut hash = [0u8; 32];
            keccak.finalize(&mut hash);
            [hash[0], hash[1], hash[2], hash[3]]
        }
        #[cfg(not(feature = "ethereum"))]
        {
            // Without ethereum feature, return zeros
            let _ = signature; // Suppress unused warning
            [0u8; 4]
        }
    }
}

// =============================================================================
// Fallback Implementations
// =============================================================================

// When alloy features are not enabled, provide basic fallback types
#[cfg(not(feature = "lightweight-alloy"))]
mod fallback {
    use super::*;

    /// Fallback address type (hex string)
    pub type Address = String;

    /// Fallback B256 type
    pub type B256 = [u8; 32];

    /// Fallback U256 type
    pub type U256 = [u8; 32];

    /// Fallback bytes type
    pub type Bytes = Vec<u8>;

    /// Fallback storage proof response
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct StorageProofResponse {
        #[serde(with = "hex")]
        pub key: [u8; 32],
        #[serde(with = "hex")]
        pub value: [u8; 32],
        pub proof_nodes: Vec<String>,
    }

    /// Fallback error type
    #[derive(Debug, thiserror::Error)]
    pub enum LightweightAlloyError {
        #[error("Alloy features not enabled")]
        NotAvailable,
    }

    /// Fallback ABI encoder
    pub struct LightweightAbi;

    impl LightweightAbi {
        /// Generate function selector for a function signature
        pub fn function_selector(signature: &str) -> [u8; 4] {
            #[cfg(feature = "ethereum")]
            {
                use tiny_keccak::{Hasher, Keccak};
                let mut keccak = Keccak::v256();
                keccak.update(signature.as_bytes());
                let mut hash = [0u8; 32];
                keccak.finalize(&mut hash);
                [hash[0], hash[1], hash[2], hash[3]]
            }
            #[cfg(not(feature = "ethereum"))]
            {
                // Without ethereum feature, return zeros
                let _ = signature; // Suppress unused warning
                [0u8; 4]
            }
        }
    }
}

// Re-export fallback types when alloy is not available
#[cfg(not(feature = "lightweight-alloy"))]
pub use fallback::*;

// =============================================================================
// Utility Functions
// =============================================================================

/// Check if alloy features are available
pub fn alloy_features_available() -> bool {
    cfg!(feature = "lightweight-alloy")
}

/// Get a list of available alloy features
pub fn available_features() -> Vec<&'static str> {
    #[cfg(feature = "lightweight-alloy")]
    {
        vec!["lightweight-alloy"]
    }
    #[cfg(not(feature = "lightweight-alloy"))]
    {
        vec![]
    }
}

/// Convert hex string to Address
pub fn parse_address(addr_str: &str) -> Result<Address, LightweightAlloyError> {
    #[cfg(feature = "lightweight-alloy")]
    {
        addr_str.parse()
            .map_err(|e| LightweightAlloyError::InvalidAddress(format!("{}", e)))
    }
    
    #[cfg(not(feature = "lightweight-alloy"))]
    {
        if addr_str.len() == 42 && addr_str.starts_with("0x") {
            Ok(addr_str.to_string())
        } else {
            Err(LightweightAlloyError::InvalidAddress("Invalid address format".to_string()))
        }
    }
}

/// Convert hex string to B256
pub fn parse_b256(hex_str: &str) -> Result<B256, LightweightAlloyError> {
    #[cfg(feature = "lightweight-alloy")]
    {
        hex_str.parse()
            .map_err(|e| LightweightAlloyError::InvalidAddress(format!("{}", e)))
    }
    
    #[cfg(not(feature = "lightweight-alloy"))]
    {
        let hex_str = hex_str.strip_prefix("0x").unwrap_or(hex_str);
        if hex_str.len() == 64 {
            let mut bytes = [0u8; 32];
            hex::decode_to_slice(hex_str, &mut bytes)
                .map_err(|e| LightweightAlloyError::InvalidAddress(format!("{}", e)))?;
            Ok(bytes)
        } else {
            Err(LightweightAlloyError::InvalidAddress("Invalid B256 format".to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_address_parsing() {
        let addr = "0x742d35Cc6634C0532925a3b8D97C2e0D8b2D9C00";
        let result = parse_address(addr);
        assert!(result.is_ok());
    }

    #[test]
    fn test_b256_parsing() {
        let hex = "0x0000000000000000000000000000000000000000000000000000000000000001";
        let result = parse_b256(hex);
        assert!(result.is_ok());
    }

    #[test]
    fn test_feature_availability() {
        let features = available_features();
        assert!(!features.is_empty() || !alloy_features_available());
    }

    #[test]
    fn test_abi_encoding() {
        let selector = LightweightAbi::function_selector("transfer(address,uint256)");
        assert_eq!(selector, [0xa9, 0x05, 0x9c, 0xbb]);
    }
} 