//! Key and path types for storage queries
//!
//! This module contains types related to storage keys, resolved paths, and
//! coprocessor query payloads. These types are used for representing and
//! working with blockchain storage queries in a chain-independent way.

use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

/// Represents a storage key that can be either fixed or variable length
///
/// Different blockchain architectures use different key formats:
/// - Ethereum uses fixed 32-byte keys
/// - Cosmos/IAVL uses variable-length keys
/// - Other chains may have different requirements
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Key {
    /// Fixed 32-byte key (common for Ethereum)
    Fixed([u8; 32]),
    /// Variable length key (for other chains like Cosmos)
    Variable(Vec<u8>),
}

/// Semantic meaning of zero values in storage slots
///
/// This enum disambiguates what zero values mean, eliminating false positive
/// ambiguity in storage proofs.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ZeroSemantics {
    /// Slot was never written to (initial assumption)
    NeverWritten,
    /// Slot was intentionally set to zero
    ExplicitlyZero,
    /// Slot was previously non-zero but cleared
    Cleared,
    /// Zero is a valid operational state
    ValidZero,
}

/// Storage semantics metadata for disambiguating zero values
///
/// Contains both declared and validated semantics to handle dynamic
/// changes where blockchain state conflicts with developer declarations.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StorageSemantics {
    /// Current semantic meaning (resolved from declared vs validated)
    pub zero_meaning: ZeroSemantics,
    /// Original developer declaration
    pub declared_semantics: ZeroSemantics,
    /// Event-validated semantics (if validation was performed)
    pub validated_semantics: Option<ZeroSemantics>,
}

impl StorageSemantics {
    /// Create new storage semantics with declared meaning
    pub fn new(declared: ZeroSemantics) -> Self {
        Self {
            zero_meaning: declared,
            declared_semantics: declared,
            validated_semantics: None,
        }
    }

    /// Create semantics with validation override
    pub fn with_validation(declared: ZeroSemantics, validated: ZeroSemantics) -> Self {
        Self {
            zero_meaning: validated,
            declared_semantics: declared,
            validated_semantics: Some(validated),
        }
    }

    /// Check if there's a conflict between declared and validated semantics
    pub fn has_conflict(&self) -> bool {
        if let Some(validated) = self.validated_semantics {
            validated != self.declared_semantics
        } else {
            false
        }
    }

    /// Validate semantic field combinations
    ///
    /// Performs basic validation to ensure semantic combinations make sense.
    /// Returns an error if the semantic configuration is invalid.
    pub fn validate(&self) -> Result<(), &'static str> {
        // Ensure zero_meaning is consistent with the resolution logic
        let expected_meaning = self.validated_semantics.unwrap_or(self.declared_semantics);
        if self.zero_meaning != expected_meaning {
            return Err("Zero meaning should match the resolved semantic (validated or declared)");
        }

        Ok(())
    }
}

/// A resolved storage path with all necessary information for ZK verification
///
/// This struct contains everything needed to verify storage access in a ZK circuit:
/// - The storage key to look up
/// - Field offset and size for value extraction
/// - Layout commitment for circuit-ABI alignment verification
/// - Zero semantics for disambiguating zero values
///
/// # Circuit Usage
///
/// In ZK circuits, these paths are typically provided as constants generated
/// at compile time, ensuring deterministic behavior and preventing dynamic
/// allocations during proof generation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StaticKeyPath {
    /// Human-readable name for this path (should be static in circuits)
    pub name: &'static str,
    /// The storage key to query
    pub key: Key,
    /// Byte offset within the storage slot for packed fields
    pub offset: Option<u8>,
    /// Size of the field in bytes
    pub field_size: Option<u8>,
    /// Layout commitment ensuring circuit-layout alignment
    pub layout_commitment: [u8; 32],
    /// Zero semantics for this storage location
    pub zero_semantics: ZeroSemantics,
}

/// Semantic storage proof for ZK coprocessor verification
///
/// Contains all the information needed to verify a storage proof in a ZK circuit
/// with semantic disambiguation of zero values:
/// - The storage key that was queried
/// - The value returned from storage
/// - The Merkle proof demonstrating inclusion in the state trie
/// - Semantic meaning of the value
///
/// This replaces the ambiguous `CoprocessorQueryPayload` with semantic-aware
/// proof verification.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SemanticStorageProof {
    /// Storage key that was queried
    pub key: [u8; 32],
    /// Storage value returned by the query
    pub value: [u8; 32],
    /// Merkle proof path demonstrating inclusion
    pub proof: Vec<[u8; 32]>,
    /// Semantic meaning of the storage value
    pub semantics: StorageSemantics,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_variants() {
        let fixed_key = Key::Fixed([1u8; 32]);
        let variable_key = Key::Variable(alloc::vec![1, 2, 3, 4]);

        // Test that keys can be created
        match fixed_key {
            Key::Fixed(arr) => assert_eq!(arr[0], 1),
            _ => panic!("Expected fixed key"),
        }

        match variable_key {
            Key::Variable(vec) => assert_eq!(vec.len(), 4),
            _ => panic!("Expected variable key"),
        }
    }

    #[test]
    fn test_zero_semantics() {
        let semantics = StorageSemantics::new(ZeroSemantics::NeverWritten);
        assert_eq!(semantics.zero_meaning, ZeroSemantics::NeverWritten);
        assert_eq!(semantics.declared_semantics, ZeroSemantics::NeverWritten);
        assert_eq!(semantics.validated_semantics, None);
        assert!(!semantics.has_conflict());
    }

    #[test]
    fn test_semantic_conflict() {
        let semantics = StorageSemantics::with_validation(
            ZeroSemantics::NeverWritten,
            ZeroSemantics::ExplicitlyZero,
        );
        assert_eq!(semantics.zero_meaning, ZeroSemantics::ExplicitlyZero);
        assert_eq!(semantics.declared_semantics, ZeroSemantics::NeverWritten);
        assert_eq!(
            semantics.validated_semantics,
            Some(ZeroSemantics::ExplicitlyZero)
        );
        assert!(semantics.has_conflict());
    }

    #[test]
    fn test_semantic_storage_proof() {
        let semantics = StorageSemantics::new(ZeroSemantics::ExplicitlyZero);
        let proof = SemanticStorageProof {
            key: [0u8; 32],
            value: [0u8; 32],
            proof: alloc::vec![[1u8; 32], [2u8; 32]],
            semantics,
        };

        assert_eq!(proof.semantics.zero_meaning, ZeroSemantics::ExplicitlyZero);
        assert!(!proof.semantics.has_conflict());
    }

    #[test]
    fn test_semantic_validation() {
        // Valid semantics should pass validation
        let valid_semantics = StorageSemantics::new(ZeroSemantics::NeverWritten);
        assert!(valid_semantics.validate().is_ok());

        // Valid semantics with validation should pass
        let valid_with_validation = StorageSemantics::with_validation(
            ZeroSemantics::NeverWritten,
            ZeroSemantics::ExplicitlyZero,
        );
        assert!(valid_with_validation.validate().is_ok());

        // Matching declared and validated semantics should be valid (not an error)
        let matching_semantics = StorageSemantics::with_validation(
            ZeroSemantics::ExplicitlyZero,
            ZeroSemantics::ExplicitlyZero,
        );
        assert!(matching_semantics.validate().is_ok());
        assert!(!matching_semantics.has_conflict()); // No conflict when they match

        // Invalid semantics should fail validation
        let mut invalid_semantics = StorageSemantics::with_validation(
            ZeroSemantics::NeverWritten,
            ZeroSemantics::ExplicitlyZero,
        );
        invalid_semantics.zero_meaning = ZeroSemantics::Cleared; // Inconsistent with resolved
        assert!(invalid_semantics.validate().is_err());
    }

    #[test]
    fn test_semantic_proof_generation_all_types() {
        // Test NeverWritten semantic proof
        let never_written_semantics = StorageSemantics::new(ZeroSemantics::NeverWritten);
        let never_written_proof = SemanticStorageProof {
            key: [0u8; 32],
            value: [0u8; 32], // Zero value with NeverWritten semantic
            proof: alloc::vec![[1u8; 32], [2u8; 32]],
            semantics: never_written_semantics,
        };
        assert_eq!(
            never_written_proof.semantics.zero_meaning,
            ZeroSemantics::NeverWritten
        );
        assert!(!never_written_proof.semantics.has_conflict());

        // Test ExplicitlyZero semantic proof
        let explicitly_zero_semantics = StorageSemantics::new(ZeroSemantics::ExplicitlyZero);
        let explicitly_zero_proof = SemanticStorageProof {
            key: [1u8; 32],
            value: [0u8; 32], // Zero value with ExplicitlyZero semantic
            proof: alloc::vec![[3u8; 32], [4u8; 32]],
            semantics: explicitly_zero_semantics,
        };
        assert_eq!(
            explicitly_zero_proof.semantics.zero_meaning,
            ZeroSemantics::ExplicitlyZero
        );
        assert!(!explicitly_zero_proof.semantics.has_conflict());

        // Test Cleared semantic proof
        let cleared_semantics = StorageSemantics::new(ZeroSemantics::Cleared);
        let cleared_proof = SemanticStorageProof {
            key: [2u8; 32],
            value: [0u8; 32], // Zero value with Cleared semantic
            proof: alloc::vec![[5u8; 32], [6u8; 32]],
            semantics: cleared_semantics,
        };
        assert_eq!(cleared_proof.semantics.zero_meaning, ZeroSemantics::Cleared);
        assert!(!cleared_proof.semantics.has_conflict());

        // Test ValidZero semantic proof
        let valid_zero_semantics = StorageSemantics::new(ZeroSemantics::ValidZero);
        let valid_zero_proof = SemanticStorageProof {
            key: [3u8; 32],
            value: [0u8; 32], // Zero value with ValidZero semantic
            proof: alloc::vec![[7u8; 32], [8u8; 32]],
            semantics: valid_zero_semantics,
        };
        assert_eq!(
            valid_zero_proof.semantics.zero_meaning,
            ZeroSemantics::ValidZero
        );
        assert!(!valid_zero_proof.semantics.has_conflict());
    }

    #[test]
    fn test_semantic_proof_generation_with_validation() {
        // Test proof generation with semantic validation conflicts
        let conflict_semantics = StorageSemantics::with_validation(
            ZeroSemantics::NeverWritten,
            ZeroSemantics::ExplicitlyZero,
        );
        let conflict_proof = SemanticStorageProof {
            key: [4u8; 32],
            value: [0u8; 32],
            proof: alloc::vec![[9u8; 32], [10u8; 32]],
            semantics: conflict_semantics,
        };
        assert_eq!(
            conflict_proof.semantics.zero_meaning,
            ZeroSemantics::ExplicitlyZero
        );
        assert_eq!(
            conflict_proof.semantics.declared_semantics,
            ZeroSemantics::NeverWritten
        );
        assert_eq!(
            conflict_proof.semantics.validated_semantics,
            Some(ZeroSemantics::ExplicitlyZero)
        );
        assert!(conflict_proof.semantics.has_conflict());

        // Test proof generation with non-zero values
        let non_zero_semantics = StorageSemantics::new(ZeroSemantics::ValidZero);
        let non_zero_proof = SemanticStorageProof {
            key: [5u8; 32],
            value: [
                0x01, 0x02, 0x03, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00,
            ], // Non-zero value
            proof: alloc::vec![[11u8; 32], [12u8; 32]],
            semantics: non_zero_semantics,
        };
        assert_eq!(
            non_zero_proof.semantics.zero_meaning,
            ZeroSemantics::ValidZero
        );
        assert!(!non_zero_proof.semantics.has_conflict());
    }
}
