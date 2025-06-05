//! Controller helpers for creating witnesses from traverse output
//! 
//! This module provides functions for creating valence coprocessor witnesses
//! from traverse-generated JSON data.

use alloc::{format, vec::Vec};
use serde_json::Value;
use valence_coprocessor::Witness;
use crate::TraverseValenceError;

/// Create a single storage witness from JSON arguments
/// 
/// Expected JSON format:
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
pub fn create_storage_witness(json_args: &Value) -> Result<Witness, TraverseValenceError> {
    let storage_query = json_args.get("storage_query")
        .ok_or_else(|| TraverseValenceError::Json("Missing storage_query field".into()))?;
    
    let storage_proof = json_args.get("storage_proof")
        .ok_or_else(|| TraverseValenceError::Json("Missing storage_proof field".into()))?;

    // Extract storage key from query
    let storage_key_str = storage_query.get("storage_key")
        .and_then(|v| v.as_str())
        .ok_or_else(|| TraverseValenceError::Json("Missing or invalid storage_key".into()))?;

    let storage_key = hex::decode(storage_key_str)
        .map_err(|e| TraverseValenceError::InvalidStorageKey(format!("Invalid hex: {:?}", e)))?;

    // Extract proof value
    let value_str = storage_proof.get("value")
        .and_then(|v| v.as_str())
        .ok_or_else(|| TraverseValenceError::Json("Missing or invalid proof value".into()))?;

    let value = hex::decode(value_str.strip_prefix("0x").unwrap_or(value_str))
        .map_err(|e| TraverseValenceError::Json(format!("Invalid value hex: {:?}", e)))?;

    // Create witness data combining key and value
    let mut witness_data = Vec::new();
    witness_data.extend_from_slice(&storage_key);
    witness_data.extend_from_slice(&value);

    Ok(Witness::Data(witness_data))
}

/// Create multiple storage witnesses from batch JSON arguments
pub fn create_batch_storage_witnesses(json_args: &Value) -> Result<Vec<Witness>, TraverseValenceError> {
    let storage_batch = json_args.get("storage_batch")
        .and_then(|v| v.as_array())
        .ok_or_else(|| TraverseValenceError::Json("Missing or invalid storage_batch array".into()))?;

    let mut witnesses = Vec::new();
    for item in storage_batch {
        let witness = create_storage_witness(item)?;
        witnesses.push(witness);
    }

    Ok(witnesses)
} 