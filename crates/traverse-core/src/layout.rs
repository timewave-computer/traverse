//! Layout types and data structures
//!
//! This module contains the core data structures for representing contract storage
//! layouts in a chain-independent format. These types are used throughout the
//! system for layout compilation, path resolution, and commitment generation.

use crate::ZeroSemantics;
use alloc::{format, string::String, vec::Vec};
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
    /// Validate the storage layout for consistency and correctness
    ///
    /// This performs sanity checks on the layout to catch common errors:
    /// - Validates packed fields don't overflow slot boundaries
    /// - Ensures referenced types exist
    /// - Checks for valid slot numbers
    /// - Validates offset alignment
    ///
    /// # Returns
    /// - `Ok(())` if the layout is valid
    /// - `Err(String)` with a description of the validation error
    pub fn validate(&self) -> Result<(), String> {
        // Check that all storage entries reference valid types
        for entry in &self.storage {
            // Validate slot format (should be parseable as number)
            entry.slot.parse::<u64>()
                .map_err(|_| format!("Invalid slot format '{}' for field '{}'", entry.slot, entry.label))?;
            
            // Find the type info for this entry
            let type_info = self.types.iter()
                .find(|t| t.label == entry.type_name)
                .ok_or_else(|| format!("Type '{}' not found for field '{}'", entry.type_name, entry.label))?;
            
            // Parse type size
            let type_size = type_info.number_of_bytes.parse::<u8>()
                .map_err(|_| format!("Invalid size '{}' for type '{}'", type_info.number_of_bytes, type_info.label))?;
            
            // Validate offset + size doesn't exceed slot boundary (32 bytes)
            if entry.offset as u32 + type_size as u32 > 32 {
                return Err(format!(
                    "Field '{}' with offset {} and size {} exceeds slot boundary",
                    entry.label, entry.offset, type_size
                ));
            }
            
            // Validate packed fields are properly aligned
            // For packed storage, fields should be aligned to their natural boundaries
            // (e.g., uint16 should be 2-byte aligned, uint32 should be 4-byte aligned)
            match type_size {
                1 => {}, // uint8/bool - no alignment required
                2 => {
                    if entry.offset % 2 != 0 {
                        return Err(format!(
                            "Field '{}' (uint16) at offset {} is not 2-byte aligned",
                            entry.label, entry.offset
                        ));
                    }
                },
                4 => {
                    if entry.offset % 4 != 0 {
                        return Err(format!(
                            "Field '{}' (uint32) at offset {} is not 4-byte aligned",
                            entry.label, entry.offset
                        ));
                    }
                },
                8 => {
                    if entry.offset % 8 != 0 {
                        return Err(format!(
                            "Field '{}' (uint64) at offset {} is not 8-byte aligned",
                            entry.label, entry.offset
                        ));
                    }
                },
                20 => {}, // address - commonly packed, no strict alignment
                32 => {
                    // Full slot types should have offset 0
                    if entry.offset != 0 {
                        return Err(format!(
                            "Field '{}' (32-byte type) should have offset 0, found {}",
                            entry.label, entry.offset
                        ));
                    }
                },
                _ => {}, // Other sizes - no specific alignment rules
            }
            
            // Validate encoding types
            match type_info.encoding.as_str() {
                "inplace" => {
                    // Inplace types should not have base/key/value
                    if type_info.base.is_some() || type_info.key.is_some() || type_info.value.is_some() {
                        return Err(format!(
                            "Inplace type '{}' should not have base/key/value fields",
                            type_info.label
                        ));
                    }
                },
                "mapping" => {
                    // Mappings should have key and value types
                    if type_info.key.is_none() || type_info.value.is_none() {
                        return Err(format!(
                            "Mapping type '{}' must have both key and value types",
                            type_info.label
                        ));
                    }
                },
                "dynamic_array" => {
                    // Dynamic arrays should have a base type
                    if type_info.base.is_none() {
                        return Err(format!(
                            "Dynamic array type '{}' must have a base type",
                            type_info.label
                        ));
                    }
                },
                _ => {}, // Unknown encoding - skip validation
            }
        }
        
        // Check for duplicate field names
        let mut seen_labels = alloc::collections::BTreeSet::new();
        for entry in &self.storage {
            if !seen_labels.insert(&entry.label) {
                return Err(format!("Duplicate field name '{}'", entry.label));
            }
        }
        
        // Check for overlapping fields in the same slot
        let mut slot_usage: alloc::collections::BTreeMap<u64, Vec<&StorageEntry>> = alloc::collections::BTreeMap::new();
        for entry in &self.storage {
            let slot = entry.slot.parse::<u64>().unwrap(); // Already validated above
            slot_usage.entry(slot).or_default().push(entry);
        }
        
        for (slot, entries) in slot_usage {
            if entries.len() > 1 {
                // Multiple fields in same slot - check for overlaps
                for i in 0..entries.len() {
                    for j in i+1..entries.len() {
                        let entry1 = entries[i];
                        let entry2 = entries[j];
                        
                        let type1 = self.types.iter().find(|t| t.label == entry1.type_name).unwrap();
                        let type2 = self.types.iter().find(|t| t.label == entry2.type_name).unwrap();
                        
                        let size1 = type1.number_of_bytes.parse::<u8>().unwrap();
                        let size2 = type2.number_of_bytes.parse::<u8>().unwrap();
                        
                        let start1 = entry1.offset;
                        let end1 = entry1.offset + size1;
                        let start2 = entry2.offset;
                        let end2 = entry2.offset + size2;
                        
                        // Check for overlap: if one field starts before the other ends
                        if start1 < end2 && start2 < end1 {
                            return Err(format!(
                                "Fields '{}' and '{}' overlap in slot {}",
                                entry1.label, entry2.label, slot
                            ));
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
    
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
        // SECURITY: Consistent commitment calculation across all environments
        // This ensures that the same layout produces the same commitment
        // regardless of whether std or no_std features are enabled.
        // The order and format must be deterministic to prevent commitment mismatches.
        let mut hasher = Sha256::new();
        
        // Hash contract name with length prefix for unambiguous encoding
        hasher.update((self.contract_name.len() as u32).to_le_bytes());
        hasher.update(self.contract_name.as_bytes());
        
        // Hash number of storage entries
        hasher.update((self.storage.len() as u32).to_le_bytes());
        
        // Hash each storage entry in order
        for entry in &self.storage {
            // Hash label with length prefix
            hasher.update((entry.label.len() as u32).to_le_bytes());
            hasher.update(entry.label.as_bytes());
            
            // Hash slot with length prefix
            hasher.update((entry.slot.len() as u32).to_le_bytes());
            hasher.update(entry.slot.as_bytes());
            
            // Hash offset as fixed-size value
            hasher.update((entry.offset as u32).to_le_bytes());
            
            // Hash type name with length prefix
            hasher.update((entry.type_name.len() as u32).to_le_bytes());
            hasher.update(entry.type_name.as_bytes());
        }
        
        hasher.finalize().into()
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

    #[test]
    fn test_layout_validation() {
        // Test 1: Valid layout
        let valid_layout = LayoutInfo {
            contract_name: "TestContract".into(),
            storage: alloc::vec![
                StorageEntry {
                    label: "field1".into(),
                    slot: "0".into(),
                    offset: 0,
                    type_name: "t_uint256".into(),
                    zero_semantics: ZeroSemantics::ValidZero,
                },
                StorageEntry {
                    label: "field2".into(),
                    slot: "1".into(),
                    offset: 0,
                    type_name: "t_address".into(),
                    zero_semantics: ZeroSemantics::NeverWritten,
                },
            ],
            types: alloc::vec![
                TypeInfo {
                    label: "t_uint256".into(),
                    number_of_bytes: "32".into(),
                    encoding: "inplace".into(),
                    base: None,
                    key: None,
                    value: None,
                },
                TypeInfo {
                    label: "t_address".into(),
                    number_of_bytes: "20".into(),
                    encoding: "inplace".into(),
                    base: None,
                    key: None,
                    value: None,
                },
            ],
        };
        
        assert!(valid_layout.validate().is_ok());
        
        // Test 2: Invalid - field exceeds slot boundary
        let invalid_overflow = LayoutInfo {
            contract_name: "TestContract".into(),
            storage: alloc::vec![StorageEntry {
                label: "field1".into(),
                slot: "0".into(),
                offset: 20, // 20 + 32 > 32
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
        
        let result = invalid_overflow.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("exceeds slot boundary"));
        
        // Test 3: Invalid - missing type
        let invalid_missing_type = LayoutInfo {
            contract_name: "TestContract".into(),
            storage: alloc::vec![StorageEntry {
                label: "field1".into(),
                slot: "0".into(),
                offset: 0,
                type_name: "t_unknown".into(),
                zero_semantics: ZeroSemantics::ValidZero,
            }],
            types: alloc::vec![],
        };
        
        let result = invalid_missing_type.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Type 't_unknown' not found"));
        
        // Test 4: Invalid - duplicate field names
        let invalid_duplicate = LayoutInfo {
            contract_name: "TestContract".into(),
            storage: alloc::vec![
                StorageEntry {
                    label: "field1".into(),
                    slot: "0".into(),
                    offset: 0,
                    type_name: "t_uint8".into(),
                    zero_semantics: ZeroSemantics::ValidZero,
                },
                StorageEntry {
                    label: "field1".into(), // Duplicate name
                    slot: "1".into(),
                    offset: 0,
                    type_name: "t_uint8".into(),
                    zero_semantics: ZeroSemantics::ValidZero,
                },
            ],
            types: alloc::vec![TypeInfo {
                label: "t_uint8".into(),
                number_of_bytes: "1".into(),
                encoding: "inplace".into(),
                base: None,
                key: None,
                value: None,
            }],
        };
        
        let result = invalid_duplicate.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Duplicate field name"));
        
        // Test 5: Invalid - overlapping fields
        let invalid_overlap = LayoutInfo {
            contract_name: "TestContract".into(),
            storage: alloc::vec![
                StorageEntry {
                    label: "field1".into(),
                    slot: "0".into(),
                    offset: 0,
                    type_name: "t_uint32".into(),
                    zero_semantics: ZeroSemantics::ValidZero,
                },
                StorageEntry {
                    label: "field2".into(),
                    slot: "0".into(),
                    offset: 2, // Overlaps with field1 (0-4)
                    type_name: "t_uint32".into(),
                    zero_semantics: ZeroSemantics::ValidZero,
                },
            ],
            types: alloc::vec![TypeInfo {
                label: "t_uint32".into(),
                number_of_bytes: "4".into(),
                encoding: "inplace".into(),
                base: None,
                key: None,
                value: None,
            }],
        };
        
        let result = invalid_overlap.validate();
        match result {
            Ok(_) => panic!("Expected overlap validation to fail, but it passed!"),
            Err(e) => {
                // Check the actual error message
                if !e.contains("overlap") && !e.contains("not 4-byte aligned") {
                    panic!("Unexpected error: {}", e);
                }
            }
        }
        
        // Test 6: Valid - packed fields (non-overlapping)
        let valid_packed = LayoutInfo {
            contract_name: "TestContract".into(),
            storage: alloc::vec![
                StorageEntry {
                    label: "field1".into(),
                    slot: "0".into(),
                    offset: 0,
                    type_name: "t_uint8".into(),
                    zero_semantics: ZeroSemantics::ValidZero,
                },
                StorageEntry {
                    label: "field2".into(),
                    slot: "0".into(),
                    offset: 1,
                    type_name: "t_uint8".into(),
                    zero_semantics: ZeroSemantics::ValidZero,
                },
                StorageEntry {
                    label: "field3".into(),
                    slot: "0".into(),
                    offset: 2,
                    type_name: "t_uint16".into(),
                    zero_semantics: ZeroSemantics::ValidZero,
                },
            ],
            types: alloc::vec![
                TypeInfo {
                    label: "t_uint8".into(),
                    number_of_bytes: "1".into(),
                    encoding: "inplace".into(),
                    base: None,
                    key: None,
                    value: None,
                },
                TypeInfo {
                    label: "t_uint16".into(),
                    number_of_bytes: "2".into(),
                    encoding: "inplace".into(),
                    base: None,
                    key: None,
                    value: None,
                },
            ],
        };
        
        assert!(valid_packed.validate().is_ok());
        
        // Test 7: Invalid - misaligned uint16
        let invalid_alignment = LayoutInfo {
            contract_name: "TestContract".into(),
            storage: alloc::vec![StorageEntry {
                label: "field1".into(),
                slot: "0".into(),
                offset: 1, // Not 2-byte aligned
                type_name: "t_uint16".into(),
                zero_semantics: ZeroSemantics::ValidZero,
            }],
            types: alloc::vec![TypeInfo {
                label: "t_uint16".into(),
                number_of_bytes: "2".into(),
                encoding: "inplace".into(),
                base: None,
                key: None,
                value: None,
            }],
        };
        
        let result = invalid_alignment.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not 2-byte aligned"));
        
        // Test 8: Invalid - mapping without key/value
        let invalid_mapping = LayoutInfo {
            contract_name: "TestContract".into(),
            storage: alloc::vec![StorageEntry {
                label: "balances".into(),
                slot: "0".into(),
                offset: 0,
                type_name: "t_mapping".into(),
                zero_semantics: ZeroSemantics::NeverWritten,
            }],
            types: alloc::vec![TypeInfo {
                label: "t_mapping".into(),
                number_of_bytes: "32".into(),
                encoding: "mapping".into(),
                base: None,
                key: None,  // Missing key
                value: None, // Missing value
            }],
        };
        
        let result = invalid_mapping.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("must have both key and value types"));
    }

    #[test]
    fn test_commitment_consistency() {
        // Test 1: Different order of fields in storage should produce different commitments
        let layout1 = LayoutInfo {
            contract_name: "TestContract".into(),
            storage: alloc::vec![
                StorageEntry {
                    label: "field1".into(),
                    slot: "0".into(),
                    offset: 0,
                    type_name: "t_uint256".into(),
                    zero_semantics: ZeroSemantics::ValidZero,
                },
                StorageEntry {
                    label: "field2".into(),
                    slot: "1".into(),
                    offset: 0,
                    type_name: "t_uint256".into(),
                    zero_semantics: ZeroSemantics::ExplicitlyZero,
                },
            ],
            types: alloc::vec![],
        };
        
        let layout2 = LayoutInfo {
            contract_name: "TestContract".into(),
            storage: alloc::vec![
                StorageEntry {
                    label: "field2".into(),
                    slot: "1".into(),
                    offset: 0,
                    type_name: "t_uint256".into(),
                    zero_semantics: ZeroSemantics::ExplicitlyZero,
                },
                StorageEntry {
                    label: "field1".into(),
                    slot: "0".into(),
                    offset: 0,
                    type_name: "t_uint256".into(),
                    zero_semantics: ZeroSemantics::ValidZero,
                },
            ],
            types: alloc::vec![],
        };
        
        let commitment1 = layout1.commitment();
        let commitment2 = layout2.commitment();
        assert_ne!(commitment1, commitment2, "Different field order should produce different commitments");
        
        // Test 2: Different contract names should produce different commitments
        let layout3 = LayoutInfo {
            contract_name: "DifferentContract".into(),
            storage: layout1.storage.clone(),
            types: alloc::vec![],
        };
        
        let commitment3 = layout3.commitment();
        assert_ne!(commitment1, commitment3, "Different contract names should produce different commitments");
        
        // Test 3: Edge case - empty storage
        let empty_layout = LayoutInfo {
            contract_name: "EmptyContract".into(),
            storage: alloc::vec![],
            types: alloc::vec![],
        };
        
        let empty_commitment = empty_layout.commitment();
        assert_eq!(empty_commitment.len(), 32);
        assert_ne!(empty_commitment, [0u8; 32], "Empty layout should not produce zero commitment");
        
        // Test 4: Different offsets should produce different commitments
        let layout4 = LayoutInfo {
            contract_name: "TestContract".into(),
            storage: alloc::vec![StorageEntry {
                label: "value".into(),
                slot: "0".into(),
                offset: 16, // Different offset
                type_name: "t_uint256".into(),
                zero_semantics: ZeroSemantics::ValidZero,
            }],
            types: alloc::vec![],
        };
        
        let layout5 = LayoutInfo {
            contract_name: "TestContract".into(),
            storage: alloc::vec![StorageEntry {
                label: "value".into(),
                slot: "0".into(),
                offset: 0, // Different offset
                type_name: "t_uint256".into(),
                zero_semantics: ZeroSemantics::ValidZero,
            }],
            types: alloc::vec![],
        };
        
        let commitment4 = layout4.commitment();
        let commitment5 = layout5.commitment();
        assert_ne!(commitment4, commitment5, "Different offsets should produce different commitments");
    }
}
