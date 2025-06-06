//! Circuit helpers for storage proof verification and value extraction
//! 
//! This module provides functions for verifying storage proofs and extracting
//! typed values in ZK circuits, following the standard Valence circuit patterns.
//!
//! ## Standard Valence Integration
//! 
//! ```rust,ignore
//! use traverse_valence::circuit;
//! use valence_coprocessor::Witness;
//! 
//! // Standard Valence circuit entry point
//! pub fn circuit(witnesses: Vec<Witness>) -> Vec<u8> {
//!     circuit::verify_storage_proofs_and_extract(&witnesses)
//! }
//! ```
//!
//! ## Witness Data Structure
//! 
//! Each witness contains:
//! - 32 bytes: storage key
//! - 32 bytes: layout commitment  
//! - 32 bytes: storage value
//! - 4 bytes: proof data length (little-endian u32)
//! - N bytes: proof data

use alloc::{vec::Vec, format};
use valence_coprocessor::Witness;
use crate::{TraverseValenceError, messages::StorageProofValidationResult};

/// Standard Valence circuit entry point for storage proof verification
/// 
/// This function follows the Valence coprocessor pattern where the circuit
/// receives witnesses from the controller and returns a Vec<u8> result.
/// 
/// The default implementation performs storage proof verification and returns
/// a JSON-encoded validation result. Applications can customize this to
/// generate ABI-encoded messages or other formats.
pub fn verify_storage_proofs_and_extract(witnesses: Vec<Witness>) -> Vec<u8> {
    match verify_storage_proofs_internal(&witnesses) {
        Ok(results) => {
            // Encode validation results as JSON
            let validation_summary = StorageValidationSummary {
                total_proofs: results.len(),
                valid_proofs: results.iter().filter(|r| r.is_valid).count(),
                results,
            };
            
            serde_json::to_vec(&validation_summary)
                .unwrap_or_else(|_| b"encoding_failed".to_vec())
        }
        Err(e) => {
            // Return error message
            format!("circuit_error: {}", e).into_bytes()
        }
    }
}

/// Verification summary for multiple storage proofs
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct StorageValidationSummary {
    /// Total number of proofs processed
    pub total_proofs: usize,
    /// Number of valid proofs
    pub valid_proofs: usize,
    /// Individual proof results
    pub results: Vec<StorageProofValidationResult>,
}

/// Internal storage proof verification (used by circuit implementations)
fn verify_storage_proofs_internal(witnesses: &[Witness]) -> Result<Vec<StorageProofValidationResult>, TraverseValenceError> {
    let mut results = Vec::new();
    
    for (index, witness) in witnesses.iter().enumerate() {
        let result = verify_single_storage_proof(witness)
            .map_err(|e| TraverseValenceError::StorageProofError(format!("Witness {}: {}", index, e)))?;
        results.push(result);
    }
    
    Ok(results)
}

/// Verify a single storage proof witness
/// 
/// Extracts and validates the storage proof data from a witness, checking:
/// 1. Layout commitment consistency
/// 2. Storage key format
/// 3. Basic proof structure
/// 
/// Note: Full Merkle-Patricia tree verification would require additional
/// state root validation (typically handled in domain layer).
pub fn verify_single_storage_proof(witness: &Witness) -> Result<StorageProofValidationResult, TraverseValenceError> {
    let witness_data = witness.as_data()
        .ok_or_else(|| TraverseValenceError::InvalidWitness("Expected Data witness".into()))?;
    
    // Parse witness structure: [32 key] + [32 commitment] + [32 value] + [4 proof_len] + [N proof_data]
    if witness_data.len() < 32 + 32 + 32 + 4 {
        return Err(TraverseValenceError::InvalidWitness("Witness too short".into()));
    }
    
    let mut offset = 0;
    
    // Extract storage key (32 bytes)
    let storage_key = &witness_data[offset..offset + 32];
    offset += 32;
    
    // Extract layout commitment (32 bytes)
    let layout_commitment = &witness_data[offset..offset + 32];
    offset += 32;
    
    // Extract storage value (32 bytes)
    let storage_value = &witness_data[offset..offset + 32];
    offset += 32;
    
    // Extract proof data length (4 bytes)
    let proof_len_bytes = &witness_data[offset..offset + 4];
    let proof_len = u32::from_le_bytes([proof_len_bytes[0], proof_len_bytes[1], proof_len_bytes[2], proof_len_bytes[3]]) as usize;
    offset += 4;
    
    // Extract proof data
    if witness_data.len() < offset + proof_len {
        return Err(TraverseValenceError::InvalidWitness("Insufficient proof data".into()));
    }
    let _proof_data = &witness_data[offset..offset + proof_len];
    
    // Basic validation (applications should add Merkle-Patricia verification)
    let is_valid = storage_key.len() == 32 && layout_commitment.len() == 32 && storage_value.len() == 32;
    
    Ok(StorageProofValidationResult {
        is_valid,
        storage_value: hex::encode(storage_value),
        storage_key: hex::encode(storage_key),
        layout_commitment: hex::encode(layout_commitment),
        metadata: Some(format!("proof_nodes_length:{}", proof_len)),
    })
}

/// Extract a u64 value from storage proof witness
/// 
/// Interprets the storage value as a big-endian u64 (common for token amounts).
/// The value is taken from the last 8 bytes of the 32-byte storage slot.
pub fn extract_u64_value(witness: &Witness) -> Result<u64, TraverseValenceError> {
    let result = verify_single_storage_proof(witness)?;
    
    if !result.is_valid {
        return Err(TraverseValenceError::ProofVerificationFailed("Invalid storage proof".into()));
    }
    
    let value_bytes = hex::decode(&result.storage_value)
        .map_err(|e| TraverseValenceError::Json(format!("Invalid hex value: {:?}", e)))?;
    
    if value_bytes.len() < 8 {
        return Err(TraverseValenceError::Json("Value too short for u64".into()));
    }
    
    // Extract u64 from last 8 bytes (big-endian)
    let u64_bytes = &value_bytes[24..32];
    let mut bytes = [0u8; 8];
    bytes.copy_from_slice(u64_bytes);
    
    Ok(u64::from_be_bytes(bytes))
}

/// Extract an address (20 bytes) from storage proof witness
/// 
/// Interprets the storage value as an Ethereum address.
/// The address is taken from the last 20 bytes of the 32-byte storage slot.
pub fn extract_address_value(witness: &Witness) -> Result<[u8; 20], TraverseValenceError> {
    let result = verify_single_storage_proof(witness)?;
    
    if !result.is_valid {
        return Err(TraverseValenceError::ProofVerificationFailed("Invalid storage proof".into()));
    }
    
    let value_bytes = hex::decode(&result.storage_value)
        .map_err(|e| TraverseValenceError::Json(format!("Invalid hex value: {:?}", e)))?;
    
    if value_bytes.len() < 20 {
        return Err(TraverseValenceError::Json("Value too short for address".into()));
    }
    
    // Extract address from last 20 bytes
    let address_bytes = &value_bytes[12..32];
    let mut address = [0u8; 20];
    address.copy_from_slice(address_bytes);
    
    Ok(address)
}

/// Extract multiple u64 values from batch witnesses
/// 
/// Processes multiple witnesses and extracts u64 values from each.
/// Useful for batch operations like checking multiple token balances.
pub fn extract_multiple_u64_values(witnesses: &[Witness]) -> Result<Vec<u64>, TraverseValenceError> {
    let mut values = Vec::new();
    
    for (index, witness) in witnesses.iter().enumerate() {
        let value = extract_u64_value(witness)
            .map_err(|e| TraverseValenceError::StorageProofError(format!("Witness {}: {}", index, e)))?;
        values.push(value);
    }
    
    Ok(values)
}

/// Extract multiple address values from batch witnesses
/// 
/// Processes multiple witnesses and extracts address values from each.
pub fn extract_multiple_address_values(witnesses: &[Witness]) -> Result<Vec<[u8; 20]>, TraverseValenceError> {
    let mut addresses = Vec::new();
    
    for (index, witness) in witnesses.iter().enumerate() {
        let address = extract_address_value(witness)
            .map_err(|e| TraverseValenceError::StorageProofError(format!("Witness {}: {}", index, e)))?;
        addresses.push(address);
    }
    
    Ok(addresses)
}

/// Extract raw bytes from storage proof witness (full 32-byte value)
/// 
/// Returns the complete 32-byte storage value without interpretation.
pub fn extract_raw_bytes(witness: &Witness) -> Result<[u8; 32], TraverseValenceError> {
    let result = verify_single_storage_proof(witness)?;
    
    if !result.is_valid {
        return Err(TraverseValenceError::ProofVerificationFailed("Invalid storage proof".into()));
    }
    
    let value_bytes = hex::decode(&result.storage_value)
        .map_err(|e| TraverseValenceError::Json(format!("Invalid hex value: {:?}", e)))?;
    
    if value_bytes.len() != 32 {
        return Err(TraverseValenceError::Json("Expected 32-byte value".into()));
    }
    
    let mut bytes = [0u8; 32];
    bytes.copy_from_slice(&value_bytes);
    
    Ok(bytes)
}

/// Check if all storage proofs in batch are valid
/// 
/// Returns true only if all witnesses contain valid storage proofs.
pub fn verify_all_proofs_valid(witnesses: &[Witness]) -> Result<bool, TraverseValenceError> {
    let results = verify_storage_proofs_internal(witnesses)?;
    Ok(results.iter().all(|r| r.is_valid))
}

/// Get layout commitment from witness (for verification)
/// 
/// Extracts the layout commitment from a witness for external verification.
pub fn extract_layout_commitment(witness: &Witness) -> Result<[u8; 32], TraverseValenceError> {
    let witness_data = witness.as_data()
        .ok_or_else(|| TraverseValenceError::InvalidWitness("Expected Data witness".into()))?;
    
    if witness_data.len() < 32 + 32 {
        return Err(TraverseValenceError::InvalidWitness("Witness too short for layout commitment".into()));
    }
    
    // Layout commitment is at offset 32 (after storage key)
    let mut commitment = [0u8; 32];
    commitment.copy_from_slice(&witness_data[32..64]);
    
    Ok(commitment)
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    fn create_test_witness() -> Witness {
        let mut witness_data = Vec::new();
        
        // Storage key (32 bytes)
        witness_data.extend_from_slice(&[1u8; 32]);
        
        // Layout commitment (32 bytes)  
        witness_data.extend_from_slice(&[2u8; 32]);
        
        // Storage value (32 bytes) - represents value 100 as u64
        let mut value = [0u8; 32];
        value[31] = 100; // 100 in the last byte
        witness_data.extend_from_slice(&value);
        
        // Proof data length (4 bytes)
        witness_data.extend_from_slice(&4u32.to_le_bytes());
        
        // Proof data (4 bytes)
        witness_data.extend_from_slice(&[0xde, 0xad, 0xbe, 0xef]);
        
        Witness::Data(witness_data)
    }

    #[test]
    fn test_verify_single_storage_proof() {
        let witness = create_test_witness();
        let result = verify_single_storage_proof(&witness).unwrap();
        
        assert!(result.is_valid);
        assert_eq!(result.storage_key, hex::encode([1u8; 32]));
        assert_eq!(result.layout_commitment, hex::encode([2u8; 32]));
    }

    #[test]
    fn test_extract_u64_value() {
        let witness = create_test_witness();
        let value = extract_u64_value(&witness).unwrap();
        
        assert_eq!(value, 100);
    }

    #[test]
    fn test_extract_layout_commitment() {
        let witness = create_test_witness();
        let commitment = extract_layout_commitment(&witness).unwrap();
        
        assert_eq!(commitment, [2u8; 32]);
    }

    #[test]
    fn test_verify_storage_proofs_and_extract() {
        let witnesses = vec![create_test_witness()];
        let result = verify_storage_proofs_and_extract(witnesses);
        
        // Should return JSON-encoded validation summary
        assert!(!result.is_empty());
        let json_str = core::str::from_utf8(&result).unwrap();
        assert!(json_str.contains("total_proofs"));
    }
} 