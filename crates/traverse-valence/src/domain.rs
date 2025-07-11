//! Semantic domain helpers for blockchain state validation
//!
//! This module provides semantic-aware functions for validating blockchain state proofs
//! and parsing block/state data for use in the valence coprocessor.
//! **Requires semantic metadata** - fails fast if semantics are missing.

use crate::TraverseValenceError;
use alloc::{format, vec::Vec};
use serde_json::Value;

/// Type alias for parsed semantic state proof data (key, value, proof, zero_semantics, semantic_source)
type ParsedSemanticStateProofData = (Vec<u8>, Vec<u8>, Vec<u8>, u8, u8);

/// Type alias for parsed block data (number, hash, state_root)  
type ParsedBlockData = (u64, Vec<u8>, Vec<u8>);

/// Ethereum block header for state validation
#[derive(Debug, Clone)]
pub struct EthereumBlockHeader {
    pub number: u64,
    pub state_root: [u8; 32],
    pub hash: [u8; 32],
}

/// Validated semantic state proof with metadata
#[derive(Debug, Clone)]
pub struct ValidatedSemanticStateProof {
    pub is_valid: bool,
    pub has_semantic_conflict: bool,
    pub block_header: EthereumBlockHeader,
    pub storage_key: [u8; 32],
    pub storage_value: Vec<u8>,
    pub zero_semantics: u8,
    pub semantic_source: u8,
}

/// Parse and validate semantic state proof data from JSON arguments
///
/// This function extracts and validates storage proof data with semantic metadata from JSON,
/// returning the parsed components for use by valence coprocessor functions.
/// **Requires semantic metadata** - fails fast if semantics are missing.
pub fn parse_semantic_state_proof_data(
    args: &Value,
) -> Result<ParsedSemanticStateProofData, TraverseValenceError> {
    #[cfg(feature = "valence-coprocessor-wasm")]
    {
        valence_coprocessor_wasm::abi::log!("parse_state_proof_data called")
            .map_err(|e| TraverseValenceError::Json(format!("Logging error: {:?}", e)))?;
    }

    // Extract storage proof data from JSON arguments
    let storage_proof = args
        .get("storage_proof")
        .ok_or_else(|| TraverseValenceError::Json("Missing storage_proof field".into()))?;

    let key_str = storage_proof
        .get("key")
        .and_then(|v| v.as_str())
        .ok_or_else(|| TraverseValenceError::Json("Missing storage key".into()))?;

    let value_str = storage_proof
        .get("value")
        .and_then(|v| v.as_str())
        .ok_or_else(|| TraverseValenceError::Json("Missing storage value".into()))?;

    let proof_nodes = storage_proof
        .get("proof")
        .and_then(|v| v.as_array())
        .ok_or_else(|| TraverseValenceError::Json("Missing proof array".into()))?;

    // Parse storage key
    let key_hex = key_str.strip_prefix("0x").unwrap_or(key_str);
    let key_bytes = hex::decode(key_hex).map_err(|e| {
        TraverseValenceError::InvalidStorageKey(format!("Invalid key hex: {:?}", e))
    })?;

    if key_bytes.len() != 32 {
        return Err(TraverseValenceError::InvalidStorageKey(
            "Storage key must be 32 bytes".into(),
        ));
    }

    // Parse storage value
    let value_hex = value_str.strip_prefix("0x").unwrap_or(value_str);
    let value_bytes = hex::decode(value_hex)
        .map_err(|e| TraverseValenceError::Json(format!("Invalid value hex: {:?}", e)))?;

    // Parse proof nodes (for Ethereum these are RLP-encoded nodes)
    let mut proof_data = Vec::new();
    for node in proof_nodes {
        if let Some(node_str) = node.as_str() {
            let node_hex = node_str.strip_prefix("0x").unwrap_or(node_str);
            let node_bytes = hex::decode(node_hex).map_err(|e| {
                TraverseValenceError::Json(format!("Invalid proof node hex: {:?}", e))
            })?;
            proof_data.extend_from_slice(&node_bytes);
        }
    }

    // Extract semantic metadata (required for semantic-first approach)
    let storage_query = args
        .get("storage_query")
        .ok_or_else(|| TraverseValenceError::Json("Missing storage_query field".into()))?;

    let zero_semantics = storage_query
        .get("zero_semantics")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| {
            TraverseValenceError::Json("Missing or invalid zero_semantics field".into())
        })?;

    let semantic_source = storage_query
        .get("semantic_source")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| {
            TraverseValenceError::Json("Missing or invalid semantic_source field".into())
        })?;

    // Validate semantic enum values
    if zero_semantics > 3 {
        return Err(TraverseValenceError::Json(
            "Invalid zero_semantics value (must be 0-3)".into(),
        ));
    }
    if semantic_source > 2 {
        return Err(TraverseValenceError::Json(
            "Invalid semantic_source value (must be 0-2)".into(),
        ));
    }

    // Return parsed components (key, value, proof_data, zero_semantics, semantic_source)
    Ok((
        key_bytes,
        value_bytes,
        proof_data,
        zero_semantics as u8,
        semantic_source as u8,
    ))
}

/// Parse and validate block data from JSON arguments
///
/// This function extracts and validates block information from JSON,
/// returning the parsed components for use by valence coprocessor functions.
/// Since ValidatedBlock construction is handled internally by valence-coprocessor,
/// this function provides validation and data extraction.
pub fn parse_block_data(args: &Value) -> Result<ParsedBlockData, TraverseValenceError> {
    #[cfg(feature = "valence-coprocessor-wasm")]
    {
        valence_coprocessor_wasm::abi::log!("parse_block_data called")
            .map_err(|e| TraverseValenceError::Json(format!("Logging error: {:?}", e)))?;
    }

    // Extract block information from JSON arguments
    let block_number = args
        .get("block_number")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);

    let block_hash_str = args
        .get("block_hash")
        .and_then(|v| v.as_str())
        .unwrap_or("0x0000000000000000000000000000000000000000000000000000000000000000");

    let state_root_str = args
        .get("state_root")
        .and_then(|v| v.as_str())
        .unwrap_or("0x0000000000000000000000000000000000000000000000000000000000000000");

    // Parse block hash
    let block_hash_hex = block_hash_str.strip_prefix("0x").unwrap_or(block_hash_str);
    let block_hash_bytes = hex::decode(block_hash_hex)
        .map_err(|e| TraverseValenceError::Json(format!("Invalid block hash hex: {:?}", e)))?;

    // Parse state root
    let state_root_hex = state_root_str.strip_prefix("0x").unwrap_or(state_root_str);
    let state_root_bytes = hex::decode(state_root_hex)
        .map_err(|e| TraverseValenceError::Json(format!("Invalid state root hex: {:?}", e)))?;

    // Return parsed components (block_number, block_hash, state_root)
    Ok((block_number, block_hash_bytes, state_root_bytes))
}

/// Validate Ethereum semantic state proof
pub fn validate_ethereum_semantic_state_proof(
    storage_proof: &Value,
    block_header: &EthereumBlockHeader,
    zero_semantics: u8,
    semantic_source: u8,
) -> Result<ValidatedSemanticStateProof, TraverseValenceError> {
    let key_str = storage_proof
        .get("key")
        .and_then(|v| v.as_str())
        .ok_or_else(|| TraverseValenceError::Json("Missing storage key".into()))?;

    let storage_key = {
        let key_hex = key_str.strip_prefix("0x").unwrap_or(key_str);
        if key_hex.len() != 64 {
            return Err(TraverseValenceError::InvalidStorageKey(
                "Storage key must be 32 bytes (64 hex chars)".into(),
            ));
        }

        let key_bytes = hex::decode(key_hex).map_err(|e| {
            TraverseValenceError::InvalidStorageKey(format!("Invalid hex: {:?}", e))
        })?;

        let mut key_array = [0u8; 32];
        key_array.copy_from_slice(&key_bytes);
        key_array
    };

    let value_str = storage_proof
        .get("value")
        .and_then(|v| v.as_str())
        .ok_or_else(|| TraverseValenceError::Json("Missing storage value".into()))?;

    let storage_value = hex::decode(value_str.strip_prefix("0x").unwrap_or(value_str))
        .map_err(|e| TraverseValenceError::Json(format!("Invalid value hex: {:?}", e)))?;

    // Semantic conflict detection - check for zero value conflicts
    let is_zero_value = storage_value.iter().all(|&b| b == 0);
    let has_semantic_conflict = is_zero_value && semantic_source == 2; // DeclaredOverride indicates conflict

    // Perform basic Merkle-Patricia trie verification
    let is_valid = verify_storage_proof_inclusion(storage_proof, &block_header.state_root)?;

    Ok(ValidatedSemanticStateProof {
        is_valid,
        has_semantic_conflict,
        block_header: block_header.clone(),
        storage_key,
        storage_value,
        zero_semantics,
        semantic_source,
    })
}

/// Basic Merkle-Patricia trie proof verification
///
/// This is a simplified verification that checks the basic structure of the proof.
/// A full implementation would need complete MPT verification logic.
fn verify_storage_proof_inclusion(
    storage_proof: &Value,
    state_root: &[u8; 32],
) -> Result<bool, TraverseValenceError> {
    // Extract proof array
    let proof_array = storage_proof
        .get("proof")
        .and_then(|v| v.as_array())
        .ok_or_else(|| TraverseValenceError::Json("Missing proof array".into()))?;

    // Basic validation checks
    if proof_array.is_empty() {
        return Ok(false); // Empty proof is invalid
    }

    // Check that all proof nodes are valid hex strings
    for node in proof_array {
        if let Some(node_str) = node.as_str() {
            let node_hex = node_str.strip_prefix("0x").unwrap_or(node_str);
            hex::decode(node_hex)
                .map_err(|_| TraverseValenceError::Json("Invalid proof node hex".into()))?;
        } else {
            return Ok(false); // Non-string node is invalid
        }
    }

    // For demonstration, perform basic structural validation
    // A full implementation would:
    // 1. Parse each proof node as RLP-encoded data
    // 2. Traverse the trie path using the storage key
    // 3. Verify each level's hash matches the next level's parent
    // 4. Ensure the final leaf contains the expected value

    // Basic heuristics for proof validity
    let proof_length = proof_array.len();
    let has_reasonable_depth = (1..=64).contains(&proof_length); // Reasonable trie depth

    // Check state root is not all zeros (indicates real block)
    let state_root_nonzero = !state_root.iter().all(|&b| b == 0);

    // Simple validation: proof should have reasonable structure
    Ok(has_reasonable_depth && state_root_nonzero)
}
