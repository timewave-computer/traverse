//! Controller helpers for creating witnesses from traverse output
//! 
//! This module provides functions for creating valence coprocessor witnesses
//! from traverse-generated JSON data, following the standard Valence controller
//! patterns used in valence-coprocessor-app.
//!
//! ## Standard Valence Integration
//! 
//! ```rust,ignore
//! use traverse_valence::controller;
//! use serde_json::Value;
//! use valence_coprocessor::Witness;
//! 
//! // Standard Valence controller entry point
//! pub fn get_witnesses(args: Value) -> anyhow::Result<Vec<Witness>> {
//!     controller::create_storage_witnesses(&args)
//! }
//! ```

use alloc::{format, vec::Vec};
use serde_json::Value;
use valence_coprocessor::Witness;
use crate::{TraverseValenceError, StorageVerificationRequest, BatchStorageVerificationRequest};

/// Standard Valence controller entry point for storage proof verification
/// 
/// This function follows the Valence coprocessor pattern where the controller
/// receives JSON arguments and returns witnesses for the circuit.
/// 
/// ## Expected JSON Formats
/// 
/// **Single Storage Query:**
/// ```json
/// {
///   "storage_query": {
///     "query": "_balances[0x742d35...]",
///     "storage_key": "c1f51986c7e9d391993039c3c40e41ad9f26e1db9b80f8535a639eadeb1d1bd9",
///     "layout_commitment": "f6dc3c4a79e95565b3cf38993f1a120c6a6b467796264e7fd9a9c8675616dd7a"
///   },
///   "storage_proof": {
///     "key": "c1f51986c7e9d391993039c3c40e41ad9f26e1db9b80f8535a639eadeb1d1bd9",
///     "value": "0000000000000000000000000000000000000000000000000000000000000064",
///     "proof": ["deadbeef...", "cafebabe..."]
///   }
/// }
/// ```
/// 
/// **Batch Storage Queries:**
/// ```json
/// {
///   "storage_batch": [
///     { "storage_query": {...}, "storage_proof": {...} },
///     { "storage_query": {...}, "storage_proof": {...} }
///   ]
/// }
/// ```
pub fn create_storage_witnesses(json_args: &Value) -> Result<Vec<Witness>, TraverseValenceError> {
    // Check if this is a batch operation
    if let Some(storage_batch) = json_args.get("storage_batch") {
        create_batch_storage_witnesses_internal(storage_batch)
    } else {
        // Single storage verification
        let witness = create_single_storage_witness(json_args)?;
        Ok(alloc::vec![witness])
    }
}

/// Create a single storage witness from JSON arguments
/// 
/// Extracts storage key, layout commitment, and proof data to create
/// witnesses for circuit verification. The witness structure follows
/// the pattern: [storage_key, layout_commitment, storage_value, proof_data]
fn create_single_storage_witness(json_args: &Value) -> Result<Witness, TraverseValenceError> {
    let storage_query = json_args.get("storage_query")
        .ok_or_else(|| TraverseValenceError::Json("Missing storage_query field".into()))?;
    
    let storage_proof = json_args.get("storage_proof")
        .ok_or_else(|| TraverseValenceError::Json("Missing storage_proof field".into()))?;

    // Extract storage key from query (pre-computed by traverse-cli)
    let storage_key_str = storage_query.get("storage_key")
        .and_then(|v| v.as_str())
        .ok_or_else(|| TraverseValenceError::Json("Missing or invalid storage_key".into()))?;

    let storage_key = hex::decode(storage_key_str.strip_prefix("0x").unwrap_or(storage_key_str))
        .map_err(|e| TraverseValenceError::InvalidStorageKey(format!("Invalid hex: {:?}", e)))?;

    // Extract layout commitment for verification
    let layout_commitment_str = storage_query.get("layout_commitment")
        .and_then(|v| v.as_str())
        .ok_or_else(|| TraverseValenceError::Json("Missing or invalid layout_commitment".into()))?;

    let layout_commitment = hex::decode(layout_commitment_str.strip_prefix("0x").unwrap_or(layout_commitment_str))
        .map_err(|e| TraverseValenceError::LayoutMismatch(format!("Invalid commitment hex: {:?}", e)))?;

    // Extract proof value from eth_getProof response
    let value_str = storage_proof.get("value")
        .and_then(|v| v.as_str())
        .ok_or_else(|| TraverseValenceError::Json("Missing or invalid proof value".into()))?;

    let value = hex::decode(value_str.strip_prefix("0x").unwrap_or(value_str))
        .map_err(|e| TraverseValenceError::Json(format!("Invalid value hex: {:?}", e)))?;

    // Extract proof nodes for Merkle verification
    let proof_nodes = storage_proof.get("proof")
        .and_then(|v| v.as_array())
        .ok_or_else(|| TraverseValenceError::Json("Missing or invalid proof array".into()))?;

    // Serialize proof nodes 
    let mut proof_data = Vec::new();
    for node in proof_nodes {
        if let Some(node_str) = node.as_str() {
            let node_bytes = hex::decode(node_str.strip_prefix("0x").unwrap_or(node_str))
                .map_err(|e| TraverseValenceError::Json(format!("Invalid proof node hex: {:?}", e)))?;
            proof_data.extend_from_slice(&node_bytes);
        }
    }

    // Create witness data structure:
    // [32 bytes storage_key] + [32 bytes layout_commitment] + [32 bytes value] + [variable proof_data]
    let mut witness_data = Vec::new();
    
    // Ensure storage key is 32 bytes
    if storage_key.len() != 32 {
        return Err(TraverseValenceError::InvalidStorageKey("Storage key must be 32 bytes".into()));
    }
    witness_data.extend_from_slice(&storage_key);
    
    // Ensure layout commitment is 32 bytes
    if layout_commitment.len() != 32 {
        return Err(TraverseValenceError::LayoutMismatch("Layout commitment must be 32 bytes".into()));
    }
    witness_data.extend_from_slice(&layout_commitment);
    
    // Ensure value is 32 bytes (pad if necessary)
    let mut value_32 = [0u8; 32];
    if value.len() <= 32 {
        value_32[32 - value.len()..].copy_from_slice(&value);
    } else {
        value_32.copy_from_slice(&value[..32]);
    }
    witness_data.extend_from_slice(&value_32);
    
    // Add proof data length and proof data
    let proof_len = proof_data.len() as u32;
    witness_data.extend_from_slice(&proof_len.to_le_bytes());
    witness_data.extend_from_slice(&proof_data);

    Ok(Witness::Data(witness_data))
}

/// Create multiple storage witnesses from batch JSON arguments
fn create_batch_storage_witnesses_internal(storage_batch: &Value) -> Result<Vec<Witness>, TraverseValenceError> {
    let batch_array = storage_batch.as_array()
        .ok_or_else(|| TraverseValenceError::Json("storage_batch must be an array".into()))?;

    let mut witnesses = Vec::new();
    for (index, item) in batch_array.iter().enumerate() {
        let witness = create_single_storage_witness(item)
            .map_err(|e| TraverseValenceError::Json(format!("Batch item {}: {}", index, e)))?;
        witnesses.push(witness);
    }

    Ok(witnesses)
}

/// Extract storage verification request from JSON (convenience function)
pub fn extract_storage_verification_request(json_args: &Value) -> Result<StorageVerificationRequest, TraverseValenceError> {
    serde_json::from_value(json_args.clone())
        .map_err(|e| TraverseValenceError::Json(format!("Failed to parse StorageVerificationRequest: {:?}", e)))
}

/// Extract batch storage verification request from JSON (convenience function)  
pub fn extract_batch_storage_verification_request(json_args: &Value) -> Result<BatchStorageVerificationRequest, TraverseValenceError> {
    serde_json::from_value(json_args.clone())
        .map_err(|e| TraverseValenceError::Json(format!("Failed to parse BatchStorageVerificationRequest: {:?}", e)))
}

/// Prepare witnesses from storage verification request (alternative API)
pub fn prepare_witnesses_from_request(request: &StorageVerificationRequest) -> Result<Witness, TraverseValenceError> {
    let json_value = serde_json::to_value(request)
        .map_err(|e| TraverseValenceError::Json(format!("Failed to serialize request: {:?}", e)))?;
    
    create_single_storage_witness(&json_value)
}

/// Prepare witnesses from batch storage verification request (alternative API)
pub fn prepare_witnesses_from_batch_request(request: &BatchStorageVerificationRequest) -> Result<Vec<Witness>, TraverseValenceError> {
    let mut witnesses = Vec::new();
    
    for storage_request in &request.storage_batch {
        let witness = prepare_witnesses_from_request(storage_request)?;
        witnesses.push(witness);
    }
    
    Ok(witnesses)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_create_single_storage_witness() {
        let json_args = json!({
            "storage_query": {
                "query": "_balances[0x742d35Cc6aB8B23c0532C65C6b555f09F9d40894]",
                "storage_key": "c1f51986c7e9d391993039c3c40e41ad9f26e1db9b80f8535a639eadeb1d1bd9",
                "layout_commitment": "f6dc3c4a79e95565b3cf38993f1a120c6a6b467796264e7fd9a9c8675616dd7a"
            },
            "storage_proof": {
                "key": "c1f51986c7e9d391993039c3c40e41ad9f26e1db9b80f8535a639eadeb1d1bd9",
                "value": "0000000000000000000000000000000000000000000000000000000000000064",
                "proof": ["deadbeef", "cafebabe"]
            }
        });

        let witnesses = create_storage_witnesses(&json_args).unwrap();
        assert_eq!(witnesses.len(), 1);
        
        // Verify witness data structure
        if let Witness::Data(data) = &witnesses[0] {
            assert_eq!(data.len(), 32 + 32 + 32 + 4 + 8); // key + commitment + value + proof_len + proof_data
        } else {
            panic!("Expected Data witness");
        }
    }

    #[test]
    fn test_create_batch_storage_witnesses() {
        let json_args = json!({
            "storage_batch": [
                {
                    "storage_query": {
                        "query": "_balances[0x742d35...]",
                        "storage_key": "c1f51986c7e9d391993039c3c40e41ad9f26e1db9b80f8535a639eadeb1d1bd9",
                        "layout_commitment": "f6dc3c4a79e95565b3cf38993f1a120c6a6b467796264e7fd9a9c8675616dd7a"
                    },
                    "storage_proof": {
                        "key": "c1f51986c7e9d391993039c3c40e41ad9f26e1db9b80f8535a639eadeb1d1bd9",
                        "value": "0000000000000000000000000000000000000000000000000000000000000064",
                        "proof": ["deadbeef"]
                    }
                },
                {
                    "storage_query": {
                        "query": "_totalSupply",
                        "storage_key": "0000000000000000000000000000000000000000000000000000000000000001",
                        "layout_commitment": "f6dc3c4a79e95565b3cf38993f1a120c6a6b467796264e7fd9a9c8675616dd7a"
                    },
                    "storage_proof": {
                        "key": "0000000000000000000000000000000000000000000000000000000000000001",
                        "value": "00000000000000000000000000000000000000000000000000000000000003e8",
                        "proof": ["cafebabe"]
                    }
                }
            ]
        });

        let witnesses = create_storage_witnesses(&json_args).unwrap();
        assert_eq!(witnesses.len(), 2);
    }
} 