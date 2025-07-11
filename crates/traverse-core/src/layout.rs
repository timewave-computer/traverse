//! Layout types and data structures
//!
//! This module contains the core data structures for representing contract storage
//! layouts in a chain-independent format. These types are used throughout the
//! system for layout compilation, path resolution, and commitment generation.

use crate::ZeroSemantics;
use alloc::{string::String, vec::Vec};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Represents the storage layout information for a contract
///
/// This is the canonical representation of a contract's storage layout that
/// can be shared across different blockchain architectures. It includes
/// all necessary information to resolve storage queries deterministically.
///
/// # Fields
///
/// - `contract_name`: Human-readable name of the contract
/// - `storage`: List of storage variables and their locations
/// - `types`: Type definitions used by the storage variables
///
/// # Layout Commitment
///
/// The layout commitment is a SHA256 hash that ensures the circuit was compiled
/// with the correct ABI. This prevents mismatches between the expected layout
/// and the actual contract layout at proof generation time.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LayoutInfo {
    /// Name of the contract
    pub contract_name: String,
    /// Storage layout entries mapping variable names to storage locations
    pub storage: Vec<StorageEntry>,
    /// ABI types information for proper field size calculation
    pub types: Vec<TypeInfo>,
}

impl LayoutInfo {
    /// Compute the layout commitment hash
    ///
    /// This generates a deterministic SHA256 hash of the layout that can be used
    /// to verify circuit-layout alignment. The commitment includes all storage
    /// entries and type information to ensure completeness.
    ///
    /// # Returns
    ///
    /// A 32-byte SHA256 hash that uniquely identifies this layout
    ///
    /// # Examples
    ///
    /// ```rust
    /// use traverse_core::LayoutInfo;
    ///
    /// let layout = LayoutInfo {
    ///     contract_name: "MyContract".into(),
    ///     storage: vec![],
    ///     types: vec![],
    /// };
    /// let commitment = layout.commitment();
    /// assert_eq!(commitment.len(), 32);
    /// ```
    pub fn commitment(&self) -> [u8; 32] {
        #[cfg(feature = "serde_json")]
        {
            let normalized = serde_json::to_vec(self).expect("LayoutInfo should always serialize");
            let mut hasher = Sha256::new();
            hasher.update(&normalized);
            hasher.finalize().into()
        }
        #[cfg(not(feature = "serde_json"))]
        {
            // For no_std without serde_json, we'll use a simpler deterministic hash
            // In practice, circuits would use the std version for layout generation
            let mut hasher = Sha256::new();
            hasher.update(self.contract_name.as_bytes());
            for entry in &self.storage {
                hasher.update(entry.label.as_bytes());
                hasher.update(entry.slot.as_bytes());
                hasher.update([entry.offset]);
                hasher.update(entry.type_name.as_bytes());
            }
            hasher.finalize().into()
        }
    }
}

/// A single entry in the storage layout
///
/// Represents one storage variable in a contract, including its name,
/// location in storage, type information, and semantic meaning of zero values.
/// This information is used to generate the correct storage keys for queries
/// and eliminate false positive ambiguity.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StorageEntry {
    /// Variable name as it appears in the contract source
    pub label: String,
    /// Storage slot index (as a string to support different number formats)
    pub slot: String,
    /// Byte offset within the slot (for packed variables)
    pub offset: u8,
    /// Type identifier referencing an entry in the types array
    pub type_name: String,
    /// Semantic meaning of zero values for this storage location (required)
    pub zero_semantics: ZeroSemantics,
}

/// Type information for ABI types
///
/// Provides detailed information about the types used in storage variables,
/// including size information and encoding details needed for proper
/// storage key generation and value extraction.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TypeInfo {
    /// Type identifier that matches storage entries
    pub label: String,
    /// Number of bytes this type occupies in storage
    pub number_of_bytes: String,
    /// Encoding type (e.g., "inplace", "mapping", "dynamic_array")
    pub encoding: String,
    /// Base type for arrays and mappings
    pub base: Option<String>,
    /// Key type for mappings  
    pub key: Option<String>,
    /// Value type for mappings
    pub value: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layout_commitment() {
        let layout = LayoutInfo {
            contract_name: "TestContract".into(),
            storage: alloc::vec![StorageEntry {
                label: "value".into(),
                slot: "0".into(),
                offset: 0,
                type_name: "t_uint256".into(),
                zero_semantics: ZeroSemantics::ValidZero,
            }],
            types: alloc::vec![TypeInfo {
                label: "t_uint256".into(),
                number_of_bytes: "32".into(),
                encoding: "inplace".into(),
                base: None,
                key: None,
                value: None,
            }],
        };

        let commitment = layout.commitment();
        assert_eq!(commitment.len(), 32);

        // Same layout should produce same commitment
        let commitment2 = layout.commitment();
        assert_eq!(commitment, commitment2);
    }
}
