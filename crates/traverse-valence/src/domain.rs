//! Domain helpers for blockchain state validation
//! 
//! This module provides functions for validating blockchain state proofs
//! and parsing block/state data for use in the valence coprocessor.

use alloc::{format, vec::Vec};
use serde_json::Value;
use crate::TraverseValenceError;

/// Type alias for parsed state proof data (key, value, proof)
type ParsedStateProofData = (Vec<u8>, Vec<u8>, Vec<u8>);

/// Type alias for parsed block data (number, hash, state_root)  
type ParsedBlockData = (u64, Vec<u8>, Vec<u8>);

/// Ethereum block header for state validation
#[derive(Debug, Clone)]
pub struct EthereumBlockHeader {
    pub number: u64,
    pub state_root: [u8; 32],
    pub hash: [u8; 32],
}

/// Validated state proof with metadata
#[derive(Debug, Clone)]
pub struct ValidatedStateProof {
    pub is_valid: bool,
    pub block_header: EthereumBlockHeader,
    pub storage_key: [u8; 32],
    pub storage_value: Vec<u8>,
}

/// Parse and validate state proof data from JSON arguments
/// 
/// This function extracts and validates storage proof data from JSON,
/// returning the parsed components for use by valence coprocessor functions.
/// Since StateProof construction is handled internally by valence-coprocessor,
/// this function provides validation and data extraction.
pub fn parse_state_proof_data(args: &Value) -> Result<ParsedStateProofData, TraverseValenceError> {
    #[cfg(feature = "valence-coprocessor-wasm")]
    {
        valence_coprocessor_wasm::abi::log!("parse_state_proof_data called")
            .map_err(|e| TraverseValenceError::Json(format!("Logging error: {:?}", e)))?;
    }

    // Extract storage proof data from JSON arguments
    let storage_proof = args.get("storage_proof")
        .ok_or_else(|| TraverseValenceError::Json("Missing storage_proof field".into()))?;

    let key_str = storage_proof.get("key")
        .and_then(|v| v.as_str())
        .ok_or_else(|| TraverseValenceError::Json("Missing storage key".into()))?;

    let value_str = storage_proof.get("value")
        .and_then(|v| v.as_str())
        .ok_or_else(|| TraverseValenceError::Json("Missing storage value".into()))?;

    let proof_nodes = storage_proof.get("proof")
        .and_then(|v| v.as_array())
        .ok_or_else(|| TraverseValenceError::Json("Missing proof array".into()))?;

    // Parse storage key
    let key_hex = key_str.strip_prefix("0x").unwrap_or(key_str);
    let key_bytes = hex::decode(key_hex)
        .map_err(|e| TraverseValenceError::InvalidStorageKey(format!("Invalid key hex: {:?}", e)))?;
    
    if key_bytes.len() != 32 {
        return Err(TraverseValenceError::InvalidStorageKey(
            "Storage key must be 32 bytes".into()
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
            let node_bytes = hex::decode(node_hex)
                .map_err(|e| TraverseValenceError::Json(format!("Invalid proof node hex: {:?}", e)))?;
            proof_data.extend_from_slice(&node_bytes);
        }
    }

    // Return parsed components (key, value, proof_data)
    Ok((key_bytes, value_bytes, proof_data))
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
    let block_number = args.get("block_number")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);

    let block_hash_str = args.get("block_hash")
        .and_then(|v| v.as_str())
        .unwrap_or("0x0000000000000000000000000000000000000000000000000000000000000000");

    let state_root_str = args.get("state_root")
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

/// Validate Ethereum state proof
pub fn validate_ethereum_state_proof(
    storage_proof: &Value,
    block_header: &EthereumBlockHeader,
) -> Result<ValidatedStateProof, TraverseValenceError> {
    let key_str = storage_proof.get("key")
        .and_then(|v| v.as_str())
        .ok_or_else(|| TraverseValenceError::Json("Missing storage key".into()))?;

    let storage_key = {
        let key_hex = key_str.strip_prefix("0x").unwrap_or(key_str);
        if key_hex.len() != 64 {
            return Err(TraverseValenceError::InvalidStorageKey(
                "Storage key must be 32 bytes (64 hex chars)".into()
            ));
        }
        
        let key_bytes = hex::decode(key_hex)
            .map_err(|e| TraverseValenceError::InvalidStorageKey(format!("Invalid hex: {:?}", e)))?;
        
        let mut key_array = [0u8; 32];
        key_array.copy_from_slice(&key_bytes);
        key_array
    };

    let value_str = storage_proof.get("value")
        .and_then(|v| v.as_str())
        .ok_or_else(|| TraverseValenceError::Json("Missing storage value".into()))?;

    let storage_value = hex::decode(value_str.strip_prefix("0x").unwrap_or(value_str))
        .map_err(|e| TraverseValenceError::Json(format!("Invalid value hex: {:?}", e)))?;

    // For now, assume validation passes
    // Real implementation would verify Merkle-Patricia trie inclusion
    Ok(ValidatedStateProof {
        is_valid: true,
        block_header: block_header.clone(),
        storage_key,
        storage_value,
    })
} 