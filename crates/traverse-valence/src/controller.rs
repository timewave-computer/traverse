//! Controller helpers for witness generation
//! 
//! This module provides functions for creating storage witnesses from coprocessor
//! JSON arguments, supporting both single and batch witness generation.

use alloc::{vec::Vec, format};
use crate::{ValenceError, CoprocessorStorageQuery, StorageProof, MockWitness};
use crate::utils::parse_hex_32;

/// Create a storage witness from coprocessor JSON arguments
/// 
/// Expected JSON format:
/// ```json
/// {
///   "storage_query": {
///     "query": "_balances[0x742d35...]",
///     "storage_key": "c1f51986c7e9d391...",
///     "layout_commitment": "f6dc3c4a79e95565...",
///     "field_size": 32,
///     "offset": null
///   },
///   "storage_proof": {
///     "key": "c1f51986c7e9d391...",
///     "value": "00000000000003e8...",
///     "proof": ["abc123...", "def456..."]
///   }
/// }
/// ```
pub fn create_storage_witness(
    json_args: &serde_json::Value,
) -> Result<MockWitness, ValenceError> {
    let storage_query: CoprocessorStorageQuery = serde_json::from_value(
        json_args["storage_query"].clone()
    ).map_err(|e| ValenceError::Json(format!("{:?}", e)))?;
    
    let storage_proof: StorageProof = serde_json::from_value(
        json_args["storage_proof"].clone()
    ).map_err(|e| ValenceError::Json(format!("{:?}", e)))?;
    
    // Verify storage keys match
    if storage_query.storage_key != storage_proof.key {
        return Err(ValenceError::InvalidStorageKey(
            "Storage key mismatch between query and proof".into()
        ));
    }
    
    // Parse hex-encoded data
    let key = parse_hex_32(&storage_proof.key)?;
    let value = parse_hex_32(&storage_proof.value)?;
    let proof: Result<Vec<[u8; 32]>, _> = storage_proof.proof
        .iter()
        .map(|hex_str| parse_hex_32(hex_str))
        .collect();
    let proof = proof?;
    
    Ok(MockWitness::StateProof { key, value, proof })
}

/// Create multiple storage witnesses from a batch of queries
pub fn create_batch_storage_witnesses(
    json_args: &serde_json::Value,
) -> Result<Vec<MockWitness>, ValenceError> {
    let batch = json_args["storage_batch"].as_array()
        .ok_or_else(|| ValenceError::Json("Expected storage_batch array".into()))?;
    
    let mut witnesses = Vec::new();
    for item in batch {
        let witness = create_storage_witness(item)?;
        witnesses.push(witness);
    }
    
    Ok(witnesses)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_create_storage_witness() {
        let json_str = r#"{
            "storage_query": {
                "query": "_balances[0x742d35...]",
                "storage_key": "c1f51986c7e9d391993039c3c40e41ad9f26e1db9b80f8535a639eadeb1d1bd9",
                "layout_commitment": "f6dc3c4a79e95565b3cf38993f1a120c6a6b467796264e7fd9a9c8675616dd7a",
                "field_size": 32,
                "offset": null
            },
            "storage_proof": {
                "key": "c1f51986c7e9d391993039c3c40e41ad9f26e1db9b80f8535a639eadeb1d1bd9",
                "value": "0000000000000000000000000000000000000000000000000000000000000064",
                "proof": ["deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef"]
            }
        }"#;
        
        let json_args: serde_json::Value = serde_json::from_str(json_str).unwrap();
        let result = create_storage_witness(&json_args);
        assert!(result.is_ok());
        
        match result.unwrap() {
            MockWitness::StateProof { key, value, proof } => {
                assert_eq!(key.len(), 32);
                assert_eq!(value.len(), 32);
                assert_eq!(proof.len(), 1);
            }
            _ => panic!("Expected StateProof witness"),
        }
    }
} 