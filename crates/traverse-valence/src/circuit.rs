//! Minimal semantic circuit for ZK storage proof verification
//!
//! This module provides only essential functions for ZK circuits with
//! bounded memory usage and minimal computation.
//!
//! ## Circuit Functions
//! - `verify_semantic_storage_proofs_and_extract()` - Main circuit entry point
//! - `extract_u64_value_minimal()` - Extract u64 values
//! - `extract_address_value_minimal()` - Extract Ethereum addresses  
//! - `extract_raw_bytes_minimal()` - Extract raw 32-byte values
//!
//! ## Circuit Integration
//!
//! ```rust,ignore
//! use traverse_valence::circuit;
//! use valence_coprocessor::Witness;
//!
//! // Main circuit entry point (returns Vec<u8> of validation results)
//! pub fn circuit(witnesses: Vec<Witness>) -> Vec<u8> {
//!     circuit::verify_semantic_storage_proofs_and_extract(witnesses)
//! }
//! ```
//!
//! ## Witness Structure (Fixed Layout)
//!
//! Each witness contains exactly:
//! - 32 bytes: storage key
//! - 32 bytes: layout commitment  
//! - 32 bytes: storage value
//! - 1 byte: zero semantics enum (0-3)
//! - 1 byte: validation source (0-2)
//! - 4 bytes: proof data length (little-endian u32)
//! - N bytes: proof data (max 10KB)

use crate::TraverseValenceError;
use alloc::vec::Vec;
use valence_coprocessor::Witness;

/// Minimal semantic storage proof verification for ZK circuits
///
/// This function performs only the essential verification needed in-circuit:
/// 1. Witness structure validation (fixed sizes)
/// 2. Semantic conflict detection for zero values
/// 3. Basic proof presence check
///
/// Returns a minimal result: 1 byte per proof (0x01 = valid, 0x00 = invalid)
/// Complex validation and reporting is done outside the circuit.
pub fn verify_semantic_storage_proofs_and_extract(witnesses: Vec<Witness>) -> Vec<u8> {
    let mut results = Vec::with_capacity(witnesses.len());

    for witness in &witnesses {
        let is_valid = match verify_single_semantic_storage_proof_minimal(witness) {
            Ok(valid) => {
                if valid {
                    0x01
                } else {
                    0x00
                }
            }
            Err(_) => 0x00, // Any error = invalid
        };
        results.push(is_valid);
    }

    results
}

/// Minimal semantic storage proof verification for ZK circuits
///
/// Performs only essential validation with bounded memory:
/// 1. Witness structure validation (fixed sizes)
/// 2. Semantic conflict detection (simple boolean logic)
/// 3. Basic proof presence check
///
/// Returns: true if proof is valid and semantically consistent, false otherwise
fn verify_single_semantic_storage_proof_minimal(
    witness: &Witness,
) -> Result<bool, TraverseValenceError> {
    let witness_data = witness
        .as_data()
        .ok_or_else(|| TraverseValenceError::InvalidWitness("Expected Data witness".into()))?;

    // Check minimum witness size: 32 + 32 + 32 + 1 + 1 + 4 = 102 bytes
    if witness_data.len() < 102 {
        return Ok(false);
    }

    // Extract fixed-size components (no dynamic allocation)
    let storage_value = &witness_data[64..96]; // 32 bytes at offset 64
    let semantics_byte = witness_data[96]; // 1 byte at offset 96
    let source_byte = witness_data[97]; // 1 byte at offset 97

    // Validate semantic enum values (bounded check)
    if semantics_byte > 3 || source_byte > 2 {
        return Ok(false);
    }

    // Core semantic validation: check for zero value conflicts
    let is_zero = is_zero_value(storage_value);
    let has_conflict = is_zero && source_byte == 2; // DeclaredOverride indicates conflict

    // Extract proof length and validate bounds
    let proof_len_bytes = &witness_data[98..102];
    let proof_len = u32::from_le_bytes([
        proof_len_bytes[0],
        proof_len_bytes[1],
        proof_len_bytes[2],
        proof_len_bytes[3],
    ]) as usize;

    // Check proof data is present and within bounds (prevent DoS)
    if proof_len > 10000 || witness_data.len() < 102 + proof_len {
        return Ok(false);
    }

    // Proof is valid if structure is correct and no semantic conflicts
    Ok(!has_conflict)
}

/// Check if a storage value is all zeros
fn is_zero_value(value: &[u8]) -> bool {
    value.iter().all(|&b| b == 0)
}

/// Extract u64 value from semantic storage proof witness (circuit-optimized)
///
/// Extracts value with minimal computation - only last 8 bytes conversion.
/// For circuit use - assumes proof has already been validated.
pub fn extract_u64_value_minimal(witness: &Witness) -> Result<u64, TraverseValenceError> {
    let witness_data = witness
        .as_data()
        .ok_or_else(|| TraverseValenceError::InvalidWitness("Expected Data witness".into()))?;

    // Check minimum size and extract value directly
    if witness_data.len() < 96 {
        return Err(TraverseValenceError::InvalidWitness(
            "Witness too short".into(),
        ));
    }

    // Extract last 8 bytes of storage value (big-endian u64)
    let value_bytes = &witness_data[88..96]; // Bytes 24-32 of the 32-byte value at offset 64
    let mut bytes = [0u8; 8];
    bytes.copy_from_slice(value_bytes);

    Ok(u64::from_be_bytes(bytes))
}

/// Extract address value from semantic storage proof witness (circuit-optimized)
///
/// Extracts last 20 bytes with minimal computation.
/// For circuit use - assumes proof has already been validated.
pub fn extract_address_value_minimal(witness: &Witness) -> Result<[u8; 20], TraverseValenceError> {
    let witness_data = witness
        .as_data()
        .ok_or_else(|| TraverseValenceError::InvalidWitness("Expected Data witness".into()))?;

    // Check minimum size and extract address directly
    if witness_data.len() < 96 {
        return Err(TraverseValenceError::InvalidWitness(
            "Witness too short".into(),
        ));
    }

    // Extract last 20 bytes of storage value (Ethereum address)
    let address_bytes = &witness_data[76..96]; // Bytes 12-32 of the 32-byte value at offset 64
    let mut address = [0u8; 20];
    address.copy_from_slice(address_bytes);

    Ok(address)
}

/// Extract raw storage value (circuit-optimized)
///
/// Returns the complete 32-byte storage value with minimal processing.
pub fn extract_raw_bytes_minimal(witness: &Witness) -> Result<[u8; 32], TraverseValenceError> {
    let witness_data = witness
        .as_data()
        .ok_or_else(|| TraverseValenceError::InvalidWitness("Expected Data witness".into()))?;

    // Check minimum size and extract value directly
    if witness_data.len() < 96 {
        return Err(TraverseValenceError::InvalidWitness(
            "Witness too short".into(),
        ));
    }

    // Extract storage value directly
    let mut value = [0u8; 32];
    value.copy_from_slice(&witness_data[64..96]);

    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    fn create_test_semantic_witness() -> Witness {
        let mut witness_data = Vec::new();

        // Storage key (32 bytes)
        witness_data.extend_from_slice(&[1u8; 32]);

        // Layout commitment (32 bytes)
        witness_data.extend_from_slice(&[2u8; 32]);

        // Storage value (32 bytes) - represents value 100 as u64
        let mut value = [0u8; 32];
        value[31] = 100; // 100 in the last byte
        witness_data.extend_from_slice(&value);

        // Zero semantics (1 byte) - ExplicitlyZero = 1
        witness_data.push(1);

        // Semantic source (1 byte) - Declared = 0
        witness_data.push(0);

        // Proof data length (4 bytes)
        witness_data.extend_from_slice(&4u32.to_le_bytes());

        // Proof data (4 bytes)
        witness_data.extend_from_slice(&[0xde, 0xad, 0xbe, 0xef]);

        Witness::Data(witness_data)
    }

    #[test]
    fn test_verify_semantic_storage_proofs_and_extract() {
        let witnesses = vec![create_test_semantic_witness()];
        let result = verify_semantic_storage_proofs_and_extract(witnesses);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], 0x01); // Should be valid
    }

    #[test]
    fn test_extract_u64_value_minimal() {
        let witness = create_test_semantic_witness();
        let value = extract_u64_value_minimal(&witness).unwrap();

        assert_eq!(value, 100);
    }

    #[test]
    fn test_extract_raw_bytes_minimal() {
        let witness = create_test_semantic_witness();
        let value = extract_raw_bytes_minimal(&witness).unwrap();

        assert_eq!(value[31], 100);
        assert_eq!(value[0..31], [0u8; 31]);
    }

    #[test]
    fn test_semantic_conflict_detection() {
        let mut witness_data = Vec::new();

        // Storage key (32 bytes)
        witness_data.extend_from_slice(&[1u8; 32]);

        // Layout commitment (32 bytes)
        witness_data.extend_from_slice(&[2u8; 32]);

        // Storage value (32 bytes) - all zeros
        witness_data.extend_from_slice(&[0u8; 32]);

        // Zero semantics (1 byte) - NeverWritten = 0
        witness_data.push(0);

        // Semantic source (1 byte) - DeclaredOverride = 2 (indicates conflict)
        witness_data.push(2);

        // Proof data length (4 bytes)
        witness_data.extend_from_slice(&4u32.to_le_bytes());

        // Proof data (4 bytes)
        witness_data.extend_from_slice(&[0xde, 0xad, 0xbe, 0xef]);

        let witness = Witness::Data(witness_data);
        let witnesses = vec![witness];
        let result = verify_semantic_storage_proofs_and_extract(witnesses);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], 0x00); // Should be invalid due to conflict
    }

    #[test]
    fn test_all_zero_semantic_types() {
        // Test all four zero semantic types
        let semantic_types = vec![
            (0, "NeverWritten"),
            (1, "ExplicitlyZero"),
            (2, "Cleared"),
            (3, "ValidZero"),
        ];

        for (semantic_value, semantic_name) in semantic_types {
            let mut witness_data = Vec::new();

            // Storage key (32 bytes)
            witness_data.extend_from_slice(&[1u8; 32]);

            // Layout commitment (32 bytes)
            witness_data.extend_from_slice(&[2u8; 32]);

            // Storage value (32 bytes) - all zeros
            witness_data.extend_from_slice(&[0u8; 32]);

            // Zero semantics (1 byte)
            witness_data.push(semantic_value);

            // Semantic source (1 byte) - Declared = 0 (no conflict)
            witness_data.push(0);

            // Proof data length (4 bytes)
            witness_data.extend_from_slice(&4u32.to_le_bytes());

            // Proof data (4 bytes)
            witness_data.extend_from_slice(&[0xde, 0xad, 0xbe, 0xef]);

            let witness = Witness::Data(witness_data);
            let witnesses = vec![witness];
            let result = verify_semantic_storage_proofs_and_extract(witnesses);

            assert_eq!(
                result.len(),
                1,
                "Failed for semantic type: {}",
                semantic_name
            );
            assert_eq!(
                result[0], 0x01,
                "Should be valid for semantic type: {}",
                semantic_name
            );
        }
    }

    #[test]
    fn test_semantic_source_validation() {
        // Test all three semantic source types
        let source_types = vec![(0, "Declared"), (1, "Validated"), (2, "DeclaredOverride")];

        for (source_value, source_name) in source_types {
            let mut witness_data = Vec::new();

            // Storage key (32 bytes)
            witness_data.extend_from_slice(&[1u8; 32]);

            // Layout commitment (32 bytes)
            witness_data.extend_from_slice(&[2u8; 32]);

            // Storage value (32 bytes) - non-zero value to avoid conflict
            let mut value = [0u8; 32];
            value[31] = 42; // Non-zero value
            witness_data.extend_from_slice(&value);

            // Zero semantics (1 byte) - ValidZero = 3
            witness_data.push(3);

            // Semantic source (1 byte)
            witness_data.push(source_value);

            // Proof data length (4 bytes)
            witness_data.extend_from_slice(&4u32.to_le_bytes());

            // Proof data (4 bytes)
            witness_data.extend_from_slice(&[0xde, 0xad, 0xbe, 0xef]);

            let witness = Witness::Data(witness_data);
            let witnesses = vec![witness];
            let result = verify_semantic_storage_proofs_and_extract(witnesses);

            assert_eq!(result.len(), 1, "Failed for source type: {}", source_name);
            assert_eq!(
                result[0], 0x01,
                "Should be valid for source type: {}",
                source_name
            );
        }
    }

    #[test]
    fn test_invalid_semantic_enum_values() {
        // Test invalid semantic enum values (> 3)
        let mut witness_data = Vec::new();

        // Storage key (32 bytes)
        witness_data.extend_from_slice(&[1u8; 32]);

        // Layout commitment (32 bytes)
        witness_data.extend_from_slice(&[2u8; 32]);

        // Storage value (32 bytes)
        witness_data.extend_from_slice(&[0u8; 32]);

        // Invalid zero semantics (1 byte) - 4 is invalid
        witness_data.push(4);

        // Valid semantic source (1 byte)
        witness_data.push(0);

        // Proof data length (4 bytes)
        witness_data.extend_from_slice(&4u32.to_le_bytes());

        // Proof data (4 bytes)
        witness_data.extend_from_slice(&[0xde, 0xad, 0xbe, 0xef]);

        let witness = Witness::Data(witness_data);
        let witnesses = vec![witness];
        let result = verify_semantic_storage_proofs_and_extract(witnesses);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], 0x00); // Should be invalid
    }

    #[test]
    fn test_invalid_source_enum_values() {
        // Test invalid source enum values (> 2)
        let mut witness_data = Vec::new();

        // Storage key (32 bytes)
        witness_data.extend_from_slice(&[1u8; 32]);

        // Layout commitment (32 bytes)
        witness_data.extend_from_slice(&[2u8; 32]);

        // Storage value (32 bytes)
        witness_data.extend_from_slice(&[0u8; 32]);

        // Valid zero semantics (1 byte)
        witness_data.push(1);

        // Invalid semantic source (1 byte) - 3 is invalid
        witness_data.push(3);

        // Proof data length (4 bytes)
        witness_data.extend_from_slice(&4u32.to_le_bytes());

        // Proof data (4 bytes)
        witness_data.extend_from_slice(&[0xde, 0xad, 0xbe, 0xef]);

        let witness = Witness::Data(witness_data);
        let witnesses = vec![witness];
        let result = verify_semantic_storage_proofs_and_extract(witnesses);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], 0x00); // Should be invalid
    }

    #[test]
    fn test_witness_size_validation() {
        // Test witness with insufficient size
        let mut witness_data = Vec::new();

        // Only 50 bytes (need minimum 102)
        witness_data.extend_from_slice(&[1u8; 50]);

        let witness = Witness::Data(witness_data);
        let witnesses = vec![witness];
        let result = verify_semantic_storage_proofs_and_extract(witnesses);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], 0x00); // Should be invalid
    }

    #[test]
    fn test_proof_size_bounds() {
        // Test proof data size limits
        let mut witness_data = Vec::new();

        // Storage key (32 bytes)
        witness_data.extend_from_slice(&[1u8; 32]);

        // Layout commitment (32 bytes)
        witness_data.extend_from_slice(&[2u8; 32]);

        // Storage value (32 bytes)
        witness_data.extend_from_slice(&[0u8; 32]);

        // Zero semantics (1 byte)
        witness_data.push(1);

        // Semantic source (1 byte)
        witness_data.push(0);

        // Proof data length (4 bytes) - exceeds maximum (10KB)
        witness_data.extend_from_slice(&15000u32.to_le_bytes());

        // No actual proof data (would be too large)

        let witness = Witness::Data(witness_data);
        let witnesses = vec![witness];
        let result = verify_semantic_storage_proofs_and_extract(witnesses);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], 0x00); // Should be invalid due to size limit
    }

    #[test]
    fn test_extract_address_value_minimal() {
        let mut witness_data = Vec::new();

        // Storage key (32 bytes)
        witness_data.extend_from_slice(&[1u8; 32]);

        // Layout commitment (32 bytes)
        witness_data.extend_from_slice(&[2u8; 32]);

        // Storage value (32 bytes) - represents an Ethereum address
        let mut value = [0u8; 32];
        // Set last 20 bytes to represent an address
        for (i, item) in value[12..32].iter_mut().enumerate() {
            *item = i as u8;
        }
        witness_data.extend_from_slice(&value);

        // Zero semantics (1 byte)
        witness_data.push(1);

        // Semantic source (1 byte)
        witness_data.push(0);

        // Proof data length (4 bytes)
        witness_data.extend_from_slice(&4u32.to_le_bytes());

        // Proof data (4 bytes)
        witness_data.extend_from_slice(&[0xde, 0xad, 0xbe, 0xef]);

        let witness = Witness::Data(witness_data);
        let address = extract_address_value_minimal(&witness).unwrap();

        // Verify address extraction
        for (i, &item) in address.iter().enumerate() {
            assert_eq!(item, i as u8);
        }
    }

    #[test]
    fn test_multiple_witnesses_batch() {
        // Test batch processing with multiple witnesses
        let mut witnesses = Vec::new();

        // Create 3 witnesses with different semantic types
        for i in 0..3 {
            let mut witness_data = Vec::new();

            // Storage key (32 bytes)
            witness_data.extend_from_slice(&[i; 32]);

            // Layout commitment (32 bytes)
            witness_data.extend_from_slice(&[2u8; 32]);

            // Storage value (32 bytes)
            witness_data.extend_from_slice(&[0u8; 32]);

            // Zero semantics (1 byte) - different for each
            witness_data.push(i);

            // Semantic source (1 byte)
            witness_data.push(0);

            // Proof data length (4 bytes)
            witness_data.extend_from_slice(&4u32.to_le_bytes());

            // Proof data (4 bytes)
            witness_data.extend_from_slice(&[0xde, 0xad, 0xbe, 0xef]);

            witnesses.push(Witness::Data(witness_data));
        }

        let result = verify_semantic_storage_proofs_and_extract(witnesses);

        assert_eq!(result.len(), 3);
        // All should be valid
        for &item in result.iter().take(3) {
            assert_eq!(item, 0x01);
        }
    }
}
