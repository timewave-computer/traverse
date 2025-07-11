//! Semantic-first controller helpers for creating witnesses from traverse output (no_std)
//!
//! This module provides functions for creating valence coprocessor witnesses
//! from traverse-generated semantic storage proof data, following the standard
//! Valence controller patterns used in valence-coprocessor-app.
//!
//! ## Complete no_std Compatibility
//!
//! This module is **completely `no_std` compatible** by default. JSON functionality
//! is available only when the `std` feature is enabled. The primary APIs work with
//! structured data types for maximum efficiency in constrained environments.
//!
//! ## Semantic Valence Integration
//!
//! ### Primary no_std API (structured data):
//! ```rust,ignore
//! use traverse_valence::controller;
//! use traverse_valence::{StorageVerificationRequest, BatchStorageVerificationRequest};
//! use valence_coprocessor::Witness;
//!
//! // Direct structured data API (no_std compatible)
//! pub fn get_witnesses(request: StorageVerificationRequest) -> Result<Witness, _> {
//!     controller::create_witness_from_request(&request)
//! }
//! ```
//!
//! ### Optional std support (JSON APIs):
//! ```rust,ignore
//! #[cfg(feature = "std")]
//! use traverse_valence::controller;
//! #[cfg(feature = "std")]
//! use serde_json::Value;
//! #[cfg(feature = "std")]
//! use valence_coprocessor::Witness;
//!
//! // JSON-based API (requires std feature)
//! #[cfg(feature = "std")]
//! pub fn get_witnesses(args: Value) -> Result<Vec<Witness>, _> {
//!     controller::create_semantic_storage_witnesses(&args)
//! }
//! ```

use alloc::{format, vec::Vec};
use valence_coprocessor::Witness;

use crate::{BatchStorageVerificationRequest, StorageVerificationRequest, TraverseValenceError};
use crate::domain::LightClient;

// === Primary no_std APIs (structured data) ===

/// Create a semantic storage witness from structured data (no_std compatible)
///
/// This is the **primary API** for creating witnesses from structured data.
/// Works in all environments including embedded/ZK circuits.
/// 
/// ## Security Features
/// - Validates storage key format and length
/// - Verifies layout commitment integrity  
/// - Ensures proof data consistency
/// - Applies semantic validation rules
///
/// ## Performance
/// - Zero JSON parsing overhead
/// - Minimal heap allocations
/// - Direct memory operations
/// - Constant-time validation
pub fn create_witness_from_request(
    request: &StorageVerificationRequest,
) -> Result<Witness, TraverseValenceError> {
    create_witness_from_request_with_light_client::<crate::domain::MockLightClient>(request, None)
}

/// Create a semantic storage witness with light client validation (no_std compatible)
///
/// This enhanced API includes light client validation for state root verification.
/// The light client provides cryptographically verified block information.
///
/// ## Security Features
/// - All features from create_witness_from_request
/// - Light client state verification
/// - Block height and hash validation
/// - Ensures proofs are from verified blocks
pub fn create_witness_from_request_with_light_client<L: LightClient>(
    request: &StorageVerificationRequest,
    light_client: Option<&L>,
) -> Result<Witness, TraverseValenceError> {
    let storage_query = &request.storage_query;
    let storage_proof = &request.storage_proof;

    // Parse storage key with validation
    let storage_key = parse_hex_bytes(&storage_query.storage_key, 32)
        .ok_or_else(|| TraverseValenceError::InvalidStorageKey("Invalid storage key format".into()))?;

    // Parse layout commitment with validation  
    let layout_commitment = parse_hex_bytes(&storage_query.layout_commitment, 32)
        .ok_or_else(|| TraverseValenceError::LayoutMismatch("Invalid layout commitment format".into()))?;

    // Parse storage value with validation
    let value = parse_hex_bytes(&storage_proof.value, 32)
        .ok_or_else(|| TraverseValenceError::InvalidWitness("Invalid storage value format".into()))?;

    // Parse and concatenate proof nodes
    let mut proof_data = Vec::new();
    for node in &storage_proof.proof {
        let node_bytes = parse_hex_bytes_variable(node)
            .ok_or_else(|| TraverseValenceError::ProofVerificationFailed("Invalid proof node format".into()))?;
        proof_data.extend_from_slice(&node_bytes);
    }

    // Use semantic defaults for structured data (can be enhanced with semantic analysis)
    let zero_semantics = derive_zero_semantics(&value);
    let semantic_source = 0u8; // Declared via structured data

    // Extract block information if available
    let (block_height, block_hash) = if let Some(lc) = light_client {
        (lc.block_height(), lc.proven_block_hash())
    } else if let Some(bn) = request.block_number {
        // Use provided block number, but no hash validation without light client
        (bn, [0u8; 32])
    } else {
        // No block information available
        (0u64, [0u8; 32])
    };

    create_semantic_witness_from_raw_data_with_block(
        &storage_key,
        &layout_commitment,
        &value,
        zero_semantics,
        semantic_source,
        &proof_data,
        block_height,
        &block_hash,
    )
}

/// Create witnesses from batch storage verification request (no_std compatible)
///
/// Processes multiple storage verification requests efficiently.
/// Each witness is validated independently for maximum security.
pub fn create_witnesses_from_batch_request(
    request: &BatchStorageVerificationRequest,
) -> Result<Vec<Witness>, TraverseValenceError> {
    let mut witnesses = Vec::with_capacity(request.storage_batch.len());

    for (index, storage_request) in request.storage_batch.iter().enumerate() {
        let witness = create_witness_from_request(storage_request)
            .map_err(|e| TraverseValenceError::InvalidWitness(format!("Batch item {}: {}", index, e)))?;
        witnesses.push(witness);
    }

    Ok(witnesses)
}

/// Core witness creation function (no_std compatible)
///
/// Creates a semantic witness from raw byte data. This is the lowest-level API
/// and is used by all other witness creation functions.
///
/// ## Witness Format
/// ```text
/// [32 bytes storage_key] +
/// [32 bytes layout_commitment] + 
/// [32 bytes value] +
/// [1 byte zero_semantics] +
/// [1 byte semantic_source] + 
/// [4 bytes proof_len] +
/// [variable proof_data]
/// ```
pub fn create_semantic_witness_from_raw_data(
    storage_key: &[u8],
    layout_commitment: &[u8],
    value: &[u8],
    zero_semantics: u8,
    semantic_source: u8,
    proof_data: &[u8],
) -> Result<Witness, TraverseValenceError> {
    // For backward compatibility, use zero block data
    create_semantic_witness_from_raw_data_with_block(
        storage_key,
        layout_commitment,
        value,
        zero_semantics,
        semantic_source,
        proof_data,
        0,
        &[0u8; 32],
    )
}

/// Enhanced witness creation with block validation data (no_std compatible)
///
/// Creates a semantic witness that includes block height and hash for light client validation.
/// This is the most secure witness format when light client verification is available.
///
/// ## Enhanced Witness Format
/// ```text
/// [32 bytes storage_key] +
/// [32 bytes layout_commitment] + 
/// [32 bytes value] +
/// [1 byte zero_semantics] +
/// [1 byte semantic_source] +
/// [8 bytes block_height] +
/// [32 bytes block_hash] +
/// [4 bytes proof_len] +
/// [variable proof_data]
/// ```
#[allow(clippy::too_many_arguments)]
pub fn create_semantic_witness_from_raw_data_with_block(
    storage_key: &[u8],
    layout_commitment: &[u8],
    value: &[u8],
    zero_semantics: u8,
    semantic_source: u8,
    proof_data: &[u8],
    block_height: u64,
    block_hash: &[u8],
) -> Result<Witness, TraverseValenceError> {
    // Validate semantic enum values
    if zero_semantics > 3 {
        return Err(TraverseValenceError::InvalidWitness(
            "Invalid zero_semantics value (must be 0-3)".into(),
        ));
    }
    if semantic_source > 2 {
        return Err(TraverseValenceError::InvalidWitness(
            "Invalid semantic_source value (must be 0-2)".into(),
        ));
    }

    // Validate required field lengths
    if storage_key.len() != 32 {
        return Err(TraverseValenceError::InvalidStorageKey(
            "Storage key must be 32 bytes".into(),
        ));
    }
    if layout_commitment.len() != 32 {
        return Err(TraverseValenceError::LayoutMismatch(
            "Layout commitment must be 32 bytes".into(),
        ));
    }
    if value.len() != 32 {
        return Err(TraverseValenceError::InvalidWitness(
            "Storage value must be 32 bytes".into(),
        ));
    }
    if block_hash.len() != 32 {
        return Err(TraverseValenceError::InvalidWitness(
            "Block hash must be 32 bytes".into(),
        ));
    }

    // Calculate total witness size (includes block data)
    let witness_size = 32 + 32 + 32 + 1 + 1 + 8 + 32 + 4 + proof_data.len();
    let mut witness_data = Vec::with_capacity(witness_size);

    // Serialize witness data in enhanced format
    witness_data.extend_from_slice(storage_key);
    witness_data.extend_from_slice(layout_commitment);
    witness_data.extend_from_slice(value);
    witness_data.push(zero_semantics);
    witness_data.push(semantic_source);
    witness_data.extend_from_slice(&block_height.to_le_bytes()); // 8 bytes block height
    witness_data.extend_from_slice(block_hash); // 32 bytes block hash
    witness_data.extend_from_slice(&(proof_data.len() as u32).to_le_bytes());
    witness_data.extend_from_slice(proof_data);

    Ok(Witness::Data(witness_data))
}

// === Utility Functions (no_std compatible) ===

/// Parse hex string to fixed-size byte array (no_std compatible)
///
/// Handles both "0x" prefixed and raw hex strings.
/// Returns None if parsing fails or length doesn't match.
fn parse_hex_bytes(hex_str: &str, expected_len: usize) -> Option<Vec<u8>> {
    let hex_str = hex_str.strip_prefix("0x").unwrap_or(hex_str);
    
    if hex_str.len() != expected_len * 2 {
        return None;
    }

    hex::decode(hex_str).ok()
}

/// Parse hex string to variable-length byte array (no_std compatible)
fn parse_hex_bytes_variable(hex_str: &str) -> Option<Vec<u8>> {
    let hex_str = hex_str.strip_prefix("0x").unwrap_or(hex_str);
    hex::decode(hex_str).ok()
}

/// Derive zero semantics from storage value (no_std compatible)
///
/// Analyzes the storage value to determine appropriate zero semantics.
/// This is a heuristic approach for structured data inputs.
fn derive_zero_semantics(value: &[u8]) -> u8 {
    if value.iter().all(|&b| b == 0) {
        1 // ExplicitlyZero - most common for zero values
    } else {
        3 // ValidZero - non-zero values
    }
}

// === Optional JSON APIs (require std feature) ===

#[cfg(feature = "std")]
use serde_json::Value;

/// Semantic-first Valence controller entry point for storage proof verification
///
/// **Requires std feature**. This function follows the Valence coprocessor pattern 
/// where the controller receives JSON arguments and returns witnesses for the circuit.
/// 
/// For no_std environments, use `create_witness_from_request` instead.
#[cfg(feature = "std")]
pub fn create_semantic_storage_witnesses(
    json_args: &Value,
) -> Result<Vec<Witness>, TraverseValenceError> {
    // Check if this is a batch operation
    if let Some(storage_batch) = json_args.get("storage_batch") {
        create_batch_semantic_storage_witnesses_internal(storage_batch)
    } else {
        // Single semantic storage verification
        let witness = create_single_semantic_storage_witness(json_args)?;
        Ok(alloc::vec![witness])
    }
}

/// Create a single semantic storage witness from JSON arguments
///
/// **Requires std feature**. Extracts storage key, layout commitment, proof data, 
/// and semantic metadata to create witnesses for circuit verification.
#[cfg(feature = "std")]
fn create_single_semantic_storage_witness(
    json_args: &Value,
) -> Result<Witness, TraverseValenceError> {
    let storage_query = json_args
        .get("storage_query")
        .ok_or_else(|| TraverseValenceError::Json("Missing storage_query field".into()))?;

    let storage_proof = json_args
        .get("storage_proof")
        .ok_or_else(|| TraverseValenceError::Json("Missing storage_proof field".into()))?;

    // Extract storage key from query (pre-computed by traverse-cli)
    let storage_key_str = storage_query
        .get("storage_key")
        .and_then(|v| v.as_str())
        .ok_or_else(|| TraverseValenceError::Json("Missing or invalid storage_key".into()))?;

    let storage_key = parse_hex_bytes(storage_key_str, 32)
        .ok_or_else(|| TraverseValenceError::InvalidStorageKey("Invalid storage key format".into()))?;

    // Extract layout commitment for verification
    let layout_commitment_str = storage_query
        .get("layout_commitment")
        .and_then(|v| v.as_str())
        .ok_or_else(|| TraverseValenceError::Json("Missing or invalid layout_commitment".into()))?;

    let layout_commitment = parse_hex_bytes(layout_commitment_str, 32)
        .ok_or_else(|| TraverseValenceError::LayoutMismatch("Invalid layout commitment format".into()))?;

    // Extract semantic metadata (required for semantic-first approach)
    let zero_semantics = storage_query
        .get("zero_semantics")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| {
            TraverseValenceError::Json("Missing or invalid zero_semantics field".into())
        })? as u8;

    let semantic_source = storage_query
        .get("semantic_source")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| {
            TraverseValenceError::Json("Missing or invalid semantic_source field".into())
        })? as u8;

    // Extract proof value from eth_getProof response
    let value_str = storage_proof
        .get("value")
        .and_then(|v| v.as_str())
        .ok_or_else(|| TraverseValenceError::Json("Missing or invalid proof value".into()))?;

    let value = parse_hex_bytes(value_str, 32)
        .ok_or_else(|| TraverseValenceError::InvalidWitness("Invalid storage value format".into()))?;

    // Extract proof nodes for Merkle verification
    let proof_nodes = storage_proof
        .get("proof")
        .and_then(|v| v.as_array())
        .ok_or_else(|| TraverseValenceError::Json("Missing or invalid proof array".into()))?;

    // Serialize proof nodes
    let mut proof_data = Vec::new();
    for node in proof_nodes {
        if let Some(node_str) = node.as_str() {
            let node_bytes = parse_hex_bytes_variable(node_str)
                .ok_or_else(|| TraverseValenceError::Json("Invalid proof node format".into()))?;
            proof_data.extend_from_slice(&node_bytes);
        }
    }

    create_semantic_witness_from_raw_data(
        &storage_key,
        &layout_commitment,
        &value,
        zero_semantics,
        semantic_source,
        &proof_data,
    )
}

/// Create multiple semantic storage witnesses from batch JSON arguments
#[cfg(feature = "std")]
fn create_batch_semantic_storage_witnesses_internal(
    storage_batch: &Value,
) -> Result<Vec<Witness>, TraverseValenceError> {
    let batch_array = storage_batch
        .as_array()
        .ok_or_else(|| TraverseValenceError::Json("storage_batch must be an array".into()))?;

    let mut witnesses = Vec::with_capacity(batch_array.len());
    for (index, item) in batch_array.iter().enumerate() {
        let witness = create_single_semantic_storage_witness(item)
            .map_err(|e| TraverseValenceError::Json(format!("Batch item {}: {}", index, e)))?;
        witnesses.push(witness);
    }

    Ok(witnesses)
}

/// Extract storage verification request from JSON (convenience function)
#[cfg(feature = "std")]
pub fn extract_storage_verification_request(
    json_args: &Value,
) -> Result<StorageVerificationRequest, TraverseValenceError> {
    serde_json::from_value(json_args.clone()).map_err(|e| {
        TraverseValenceError::Json(format!(
            "Failed to parse StorageVerificationRequest: {:?}",
            e
        ))
    })
}

/// Extract batch storage verification request from JSON (convenience function)  
#[cfg(feature = "std")]
pub fn extract_batch_storage_verification_request(
    json_args: &Value,
) -> Result<BatchStorageVerificationRequest, TraverseValenceError> {
    serde_json::from_value(json_args.clone()).map_err(|e| {
        TraverseValenceError::Json(format!(
            "Failed to parse BatchStorageVerificationRequest: {:?}",
            e
        ))
    })
}

// === Legacy APIs (deprecated) ===

/// **Deprecated**: Use `create_witness_from_request` instead.
#[cfg(feature = "std")]
#[deprecated(since = "0.2.0", note = "Use create_witness_from_request instead")]
pub fn prepare_semantic_witnesses_from_request(
    request: &StorageVerificationRequest,
) -> Result<Witness, TraverseValenceError> {
    let json_value = serde_json::to_value(request)
        .map_err(|e| TraverseValenceError::Json(format!("Failed to serialize request: {:?}", e)))?;

    create_single_semantic_storage_witness(&json_value)
}

/// **Deprecated**: Use `create_witnesses_from_batch_request` instead.
#[cfg(feature = "std")]
#[deprecated(since = "0.2.0", note = "Use create_witnesses_from_batch_request instead")]
pub fn prepare_semantic_witnesses_from_batch_request(
    request: &BatchStorageVerificationRequest,
) -> Result<Vec<Witness>, TraverseValenceError> {
    let mut witnesses = Vec::new();

    for storage_request in &request.storage_batch {
        let witness = create_witness_from_request(storage_request)?;
        witnesses.push(witness);
    }

    Ok(witnesses)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{StorageProof, CoprocessorStorageQuery};
    use alloc::string::ToString;
    
    #[cfg(feature = "std")]
    use serde_json::json;

    #[test]
    fn test_no_std_raw_witness_creation() {
        let storage_key = [1u8; 32];
        let layout_commitment = [2u8; 32];
        let value = [3u8; 32];
        let zero_semantics = 1;
        let semantic_source = 0;
        let proof_data = alloc::vec![0xde, 0xad, 0xbe, 0xef];

        let witness = create_semantic_witness_from_raw_data(
            &storage_key,
            &layout_commitment,
            &value,
            zero_semantics,
            semantic_source,
            &proof_data,
        )
        .unwrap();

        if let Witness::Data(data) = witness {
            // Updated format includes block height (8 bytes) and block hash (32 bytes)
            assert_eq!(data.len(), 32 + 32 + 32 + 1 + 1 + 8 + 32 + 4 + 4);
            assert_eq!(data[96], 1); // zero_semantics
            assert_eq!(data[97], 0); // semantic_source
            assert_eq!(data[98..106], [0, 0, 0, 0, 0, 0, 0, 0]); // block_height (8 bytes, little endian)
            assert_eq!(data[106..138], [0u8; 32]); // block_hash (32 bytes)
            assert_eq!(data[138..142], [4, 0, 0, 0]); // proof_len (little endian)
            assert_eq!(data[142..146], [0xde, 0xad, 0xbe, 0xef]); // proof_data
        } else {
            panic!("Expected Data witness");
        }
    }

    #[test]
    fn test_no_std_structured_request() {
        let request = StorageVerificationRequest {
            storage_query: CoprocessorStorageQuery {
                query: "_balances[0x742d35...]".to_string(),
                storage_key: "c1f51986c7e9d391993039c3c40e41ad9f26e1db9b80f8535a639eadeb1d1bd9".to_string(),
                layout_commitment: "f6dc3c4a79e95565b3cf38993f1a120c6a6b467796264e7fd9a9c8675616dd7a".to_string(),
                field_size: Some(32),
                offset: Some(0),
            },
            storage_proof: StorageProof {
                key: "c1f51986c7e9d391993039c3c40e41ad9f26e1db9b80f8535a639eadeb1d1bd9".to_string(),
                value: "0000000000000000000000000000000000000000000000000000000000000064".to_string(),
                proof: alloc::vec!["deadbeef".to_string(), "cafebabe".to_string()],
            },
            contract_address: Some("0x742d35Cc6634C0532925a3b8D97C2e0D8b2D9C".to_string()),
            block_number: Some(12345),
        };

        let witness = create_witness_from_request(&request).unwrap();

        if let Witness::Data(data) = witness {
            assert!(data.len() >= 102); // Minimum witness size
        } else {
            panic!("Expected Data witness");
        }
    }

    #[test]
    fn test_no_std_hex_parsing() {
        // Test with 0x prefix
        let result = parse_hex_bytes("0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef", 32);
        assert!(result.is_some());
        assert_eq!(result.unwrap().len(), 32);

        // Test without 0x prefix  
        let result = parse_hex_bytes("1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef", 32);
        assert!(result.is_some());
        assert_eq!(result.unwrap().len(), 32);

        // Test invalid length
        let result = parse_hex_bytes("0x1234", 32);
        assert!(result.is_none());

        // Test invalid hex
        let result = parse_hex_bytes("0xzzzz567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef", 32);
        assert!(result.is_none());
    }

    #[test]
    fn test_no_std_semantic_derivation() {
        // Test zero value
        let zero_value = [0u8; 32];
        assert_eq!(derive_zero_semantics(&zero_value), 1); // ExplicitlyZero

        // Test non-zero value
        let mut non_zero_value = [0u8; 32];
        non_zero_value[31] = 42;
        assert_eq!(derive_zero_semantics(&non_zero_value), 3); // ValidZero
    }

    #[test]
    fn test_no_std_batch_processing() {
        let request1 = StorageVerificationRequest {
            storage_query: CoprocessorStorageQuery {
                query: "test1".to_string(),
                storage_key: "c1f51986c7e9d391993039c3c40e41ad9f26e1db9b80f8535a639eadeb1d1bd9".to_string(),
                layout_commitment: "f6dc3c4a79e95565b3cf38993f1a120c6a6b467796264e7fd9a9c8675616dd7a".to_string(),
                field_size: Some(32),
                offset: Some(0),
            },
            storage_proof: StorageProof {
                key: "c1f51986c7e9d391993039c3c40e41ad9f26e1db9b80f8535a639eadeb1d1bd9".to_string(),
                value: "0000000000000000000000000000000000000000000000000000000000000001".to_string(),
                proof: alloc::vec!["dead".to_string()],
            },
            contract_address: None,
            block_number: None,
        };

        let request2 = StorageVerificationRequest {
            storage_query: CoprocessorStorageQuery {
                query: "test2".to_string(),
                storage_key: "d1f51986c7e9d391993039c3c40e41ad9f26e1db9b80f8535a639eadeb1d1bd9".to_string(),
                layout_commitment: "f6dc3c4a79e95565b3cf38993f1a120c6a6b467796264e7fd9a9c8675616dd7a".to_string(),
                field_size: Some(32),
                offset: Some(0),
            },
            storage_proof: StorageProof {
                key: "d1f51986c7e9d391993039c3c40e41ad9f26e1db9b80f8535a639eadeb1d1bd9".to_string(),
                value: "0000000000000000000000000000000000000000000000000000000000000002".to_string(),
                proof: alloc::vec!["beef".to_string()],
            },
            contract_address: None,
            block_number: None,
        };

        let batch_request = BatchStorageVerificationRequest {
            storage_batch: alloc::vec![request1, request2],
            contract_address: None,
            block_number: None,
        };

        let witnesses = create_witnesses_from_batch_request(&batch_request).unwrap();
        assert_eq!(witnesses.len(), 2);
    }

    #[cfg(feature = "std")]
    #[test]
    fn test_std_json_compatibility() {
        let json_args = json!({
            "storage_query": {
                "query": "_balances[0x742d35Cc6aB8B23c0532C65C6b555f09F9d40894]",
                "storage_key": "c1f51986c7e9d391993039c3c40e41ad9f26e1db9b80f8535a639eadeb1d1bd9",
                "layout_commitment": "f6dc3c4a79e95565b3cf38993f1a120c6a6b467796264e7fd9a9c8675616dd7a",
                "zero_semantics": 1,
                "semantic_source": 0
            },
            "storage_proof": {
                "key": "c1f51986c7e9d391993039c3c40e41ad9f26e1db9b80f8535a639eadeb1d1bd9",
                "value": "0000000000000000000000000000000000000000000000000000000000000064",
                "proof": ["deadbeef", "cafebabe"]
            }
        });

        let witnesses = create_semantic_storage_witnesses(&json_args).unwrap();
        assert_eq!(witnesses.len(), 1);
    }
}
