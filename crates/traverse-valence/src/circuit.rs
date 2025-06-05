//! Circuit helpers for verifying storage proofs and extracting values
//! 
//! This module provides functions for verifying storage proofs within ZK circuits
//! and extracting typed values from proof data.

use alloc::vec::Vec;
use valence_coprocessor::Witness;
use crate::TraverseValenceError;

/// Verify storage proof and extract raw value bytes
pub fn verify_storage_proof(witness: &Witness) -> Result<Vec<u8>, TraverseValenceError> {
    let data = witness.as_data()
        .ok_or_else(|| TraverseValenceError::InvalidWitness("Expected data witness".into()))?;

    if data.len() < 32 {
        return Err(TraverseValenceError::ProofVerificationFailed(
            "Witness data too short (expected at least 32 bytes for storage key)".into()
        ));
    }

    // Extract value portion (everything after the 32-byte storage key)
    let value = if data.len() > 32 {
        data[32..].to_vec()
    } else {
        Vec::new()
    };

    Ok(value)
}

/// Extract u64 value from storage witness
pub fn extract_u64_value(witness: &Witness) -> Result<u64, TraverseValenceError> {
    let value = verify_storage_proof(witness)?;
    
    if value.len() < 8 {
        return Err(TraverseValenceError::ProofVerificationFailed(
            "Insufficient data for u64 extraction".into()
        ));
    }

    // Take last 8 bytes and convert from big-endian (Ethereum storage format)
    let mut bytes = [0u8; 8];
    let start = if value.len() >= 8 { value.len() - 8 } else { 0 };
    bytes.copy_from_slice(&value[start..start + 8]);
    
    Ok(u64::from_be_bytes(bytes))
}

/// Extract address value from storage witness
pub fn extract_address_value(witness: &Witness) -> Result<[u8; 20], TraverseValenceError> {
    let value = verify_storage_proof(witness)?;
    
    if value.len() < 20 {
        return Err(TraverseValenceError::ProofVerificationFailed(
            "Insufficient data for address extraction".into()
        ));
    }

    // Take last 20 bytes (Ethereum address format)
    let mut address = [0u8; 20];
    let start = if value.len() >= 20 { value.len() - 20 } else { 0 };
    address.copy_from_slice(&value[start..start + 20]);
    
    Ok(address)
}

/// Process multiple witnesses and extract u64 values
pub fn extract_multiple_u64_values(witnesses: &[Witness]) -> Result<Vec<u64>, TraverseValenceError> {
    let mut values = Vec::new();
    for witness in witnesses {
        let value = extract_u64_value(witness)?;
        values.push(value);
    }
    Ok(values)
} 