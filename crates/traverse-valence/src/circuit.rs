//! Circuit helpers for proof verification
//! 
//! This module provides functions for verifying storage proofs within circuits
//! and extracting typed values from the storage data.

use alloc::vec::Vec;
use crate::{ValenceError, CoprocessorStorageQuery, MockWitness};
use crate::utils::parse_hex_32;

/// Verify a storage proof witness and extract the field value
/// 
/// This function:
/// 1. Verifies the layout commitment matches expected
/// 2. Validates the storage key derivation
/// 3. Extracts the field value based on offset and size
/// 
/// Returns the extracted field value as bytes
pub fn verify_storage_proof(
    witness: &MockWitness,
    expected_commitment: &[u8; 32],
    query: &CoprocessorStorageQuery,
) -> Result<Vec<u8>, ValenceError> {
    let (key, value, _proof) = match witness {
        MockWitness::StateProof { key, value, proof } => (key, value, proof),
        _ => return Err(ValenceError::ProofVerificationFailed(
            "Expected StateProof witness".into()
        )),
    };
    
    // Verify layout commitment
    let query_commitment = parse_hex_32(&query.layout_commitment)?;
    if &query_commitment != expected_commitment {
        return Err(ValenceError::LayoutMismatch(
            "Layout commitment mismatch".into()
        ));
    }
    
    // Verify storage key matches query
    let query_key = parse_hex_32(&query.storage_key)?;
    if key != &query_key {
        return Err(ValenceError::ProofVerificationFailed(
            "Storage key mismatch".into()
        ));
    }
    
    // Extract field value based on offset and size
    let field_size = query.field_size.unwrap_or(32) as usize;
    let offset = query.offset.unwrap_or(0) as usize;
    
    if offset + field_size > 32 {
        return Err(ValenceError::ProofVerificationFailed(
            "Field extends beyond storage slot".into()
        ));
    }
    
    let field_value = &value[offset..offset + field_size];
    Ok(field_value.to_vec())
}

/// Extract a u64 value from storage proof (common for balances, amounts)
pub fn extract_u64_value(
    witness: &MockWitness,
    expected_commitment: &[u8; 32],
    query: &CoprocessorStorageQuery,
) -> Result<u64, ValenceError> {
    let field_bytes = verify_storage_proof(witness, expected_commitment, query)?;
    
    if field_bytes.len() < 8 {
        return Err(ValenceError::ProofVerificationFailed(
            "Insufficient bytes for u64 extraction".into()
        ));
    }
    
    // Extract u64 from the last 8 bytes (big-endian storage)
    let mut u64_bytes = [0u8; 8];
    let start = field_bytes.len() - 8;
    u64_bytes.copy_from_slice(&field_bytes[start..]);
    
    Ok(u64::from_be_bytes(u64_bytes))
}

/// Extract an address value from storage proof  
pub fn extract_address_value(
    witness: &MockWitness,
    expected_commitment: &[u8; 32],
    query: &CoprocessorStorageQuery,
) -> Result<[u8; 20], ValenceError> {
    let field_bytes = verify_storage_proof(witness, expected_commitment, query)?;
    
    if field_bytes.len() < 20 {
        return Err(ValenceError::ProofVerificationFailed(
            "Insufficient bytes for address extraction".into()
        ));
    }
    
    // Extract address from the last 20 bytes
    let mut address_bytes = [0u8; 20];
    let start = field_bytes.len() - 20;
    address_bytes.copy_from_slice(&field_bytes[start..]);
    
    Ok(address_bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CoprocessorStorageQuery;
    use alloc::vec;
    
    #[test]
    fn test_verify_storage_proof() {
        let witness = MockWitness::StateProof {
            key: [1u8; 32],
            value: [0xFF; 32], // All ones for testing
            proof: vec![[2u8; 32]],
        };
        
        let commitment = [3u8; 32];
        let query = CoprocessorStorageQuery {
            query: "test".into(),
            storage_key: hex::encode([1u8; 32]),
            layout_commitment: hex::encode([3u8; 32]),
            field_size: Some(32),
            offset: None,
        };
        
        let result = verify_storage_proof(&witness, &commitment, &query);
        assert!(result.is_ok());
        
        let extracted = result.unwrap();
        assert_eq!(extracted.len(), 32);
        assert_eq!(extracted, vec![0xFF; 32]);
    }
    
    #[test]
    fn test_extract_u64_value() {
        let mut value = [0u8; 32];
        value[24..32].copy_from_slice(&100u64.to_be_bytes()); // Put 100 at the end
        
        let witness = MockWitness::StateProof {
            key: [1u8; 32],
            value,
            proof: vec![[2u8; 32]],
        };
        
        let commitment = [3u8; 32];
        let query = CoprocessorStorageQuery {
            query: "balance".into(),
            storage_key: hex::encode([1u8; 32]),
            layout_commitment: hex::encode([3u8; 32]),
            field_size: Some(32),
            offset: None,
        };
        
        let result = extract_u64_value(&witness, &commitment, &query);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 100u64);
    }
} 