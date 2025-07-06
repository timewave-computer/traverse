//! Ethereum key resolver for translating storage queries into storage keys
//!
//! This module provides the core functionality for converting human-readable
//! storage queries (like `balances[0x123...]`) into deterministic Ethereum
//! storage keys using Solidity's storage layout rules.

use tiny_keccak::{Hasher, Keccak};
use traverse_core::{Key, KeyResolver, LayoutInfo, StaticKeyPath, TraverseError};

/// Ethereum key resolver that implements Solidity storage key derivation
///
/// This implementation handles Ethereum-specific storage key calculation:
/// - Simple storage variables use their slot index as the key
/// - Mappings use `keccak256(key ++ slot)` derivation  
/// - Packed variables include offset information
///
/// # Storage Key Derivation
///
/// ## Simple Variables
/// Storage slot is used directly as a 32-byte key (big-endian).
///
/// ## Mappings
/// Uses Solidity's standard: `keccak256(abi.encode(key, slot))`
/// - Key is left-padded to 32 bytes
/// - Slot is right-padded to 32 bytes (big-endian)
///
/// ## Packed Variables  
/// Uses the same slot key but includes byte offset information
/// for proper field extraction.
///
/// # Examples
///
/// ```rust,ignore
/// use traverse_ethereum::EthereumKeyResolver;
/// use traverse_core::KeyResolver;
///
/// let resolver = EthereumKeyResolver;
///
/// // Simple field access
/// let path = resolver.resolve(&layout, "totalSupply")?;
///
/// // Mapping access
/// let path = resolver.resolve(&layout, "balances[0x742d35Cc6634C0532925a3b8D97C2e0D8b2D9C]")?;
/// ```
pub struct EthereumKeyResolver;

/// Represents different types of storage queries
#[derive(Debug)]
enum QueryParts {
    /// Simple field access (e.g., "owner", "totalSupply")
    Field { field_name: String },
    /// Single mapping access (e.g., "balances[0x123...]")
    Mapping { field_name: String, key: Vec<u8> },
    /// Nested mapping access (e.g., "allowances[0x123...][0x456...]")
    NestedMapping {
        field_name: String,
        keys: Vec<Vec<u8>>,
    },
    /// Array access (e.g., "items[5]")
    Array { field_name: String, index: u64 },
    /// Struct field access (e.g., "user.balance")
    StructField {
        struct_name: String,
        field_name: String,
    },
    /// Dynamic array/string length access (e.g., "name.length")
    DynamicLength { field_name: String },
    /// Dynamic array/string data access (e.g., "name.data")
    DynamicData { field_name: String },
}

impl EthereumKeyResolver {
    /// Compute keccak256 hash
    ///
    /// # Arguments
    ///
    /// * `data` - Input data to hash
    ///
    /// # Returns
    ///
    /// 32-byte Keccak256 hash of the input
    fn keccak256(data: &[u8]) -> [u8; 32] {
        let mut hasher = Keccak::v256();
        let mut output = [0u8; 32];
        hasher.update(data);
        hasher.finalize(&mut output);
        output
    }

    /// Derive storage key for array element: keccak256(slot) + index
    ///
    /// For dynamic arrays in Solidity:
    /// - Array length is stored at the slot
    /// - Array elements are stored at keccak256(slot) + index
    ///
    /// # Arguments
    ///
    /// * `slot` - The storage slot of the array declaration
    /// * `index` - The array index to access
    ///
    /// # Returns
    ///
    /// 32-byte storage key for the array element
    fn derive_array_key(slot: u64, index: u64) -> [u8; 32] {
        // First, get the base location: keccak256(slot)
        let mut slot_bytes = [0u8; 32];
        slot_bytes[24..].copy_from_slice(&slot.to_be_bytes());
        let base_key = Self::keccak256(&slot_bytes);

        // Convert base key to u256 and add index
        let mut result = base_key;

        // Add index to the base key (treating as big-endian u256)
        let mut carry = index;
        for i in (0..32).rev() {
            let sum = result[i] as u64 + carry;
            result[i] = sum as u8;
            carry = sum >> 8;
            if carry == 0 {
                break;
            }
        }

        result
    }

    /// Derive storage key for nested mapping: keccak256(key ++ previous_key)
    ///
    /// For nested mappings, each level uses the full 32-byte result from the previous
    /// level as the "slot" parameter.
    ///
    /// # Arguments
    ///
    /// * `map_key` - The mapping key (e.g., address, uint256)
    /// * `previous_key` - The 32-byte result from the previous mapping level
    ///
    /// # Returns
    ///
    /// 32-byte storage key for the nested mapping entry
    fn derive_nested_mapping_key(map_key: &[u8], previous_key: &[u8; 32]) -> [u8; 32] {
        let mut data = Vec::new();

        // Pad key to 32 bytes (left-padded for addresses, right-padded for others)
        if map_key.len() <= 32 {
            let mut padded_key = [0u8; 32];
            let start = 32 - map_key.len();
            padded_key[start..].copy_from_slice(map_key);
            data.extend_from_slice(&padded_key);
        } else {
            data.extend_from_slice(map_key);
        }

        // Use the full 32-byte previous key directly
        data.extend_from_slice(previous_key);

        Self::keccak256(&data)
    }

    /// Derives storage key for mapping entries using Ethereum's standard keccak256(key ++ slot)
    ///
    /// # Arguments
    /// * `key` - The mapping key (address, number, etc.)
    /// * `slot` - The storage slot number of the mapping
    ///
    /// # Returns
    ///
    /// 32-byte storage key for the mapping entry
    pub fn derive_mapping_key(key: &[u8], slot: u64) -> [u8; 32] {
        let mut data = Vec::new();

        // Pad key to 32 bytes (left-padded for addresses, right-padded for others)
        if key.len() <= 32 {
            let mut padded_key = [0u8; 32];
            let start = 32 - key.len();
            padded_key[start..].copy_from_slice(key);
            data.extend_from_slice(&padded_key);
        } else {
            data.extend_from_slice(key);
        }

        // Pad slot to 32 bytes (big-endian)
        let mut padded_slot = [0u8; 32];
        padded_slot[24..].copy_from_slice(&slot.to_be_bytes());
        data.extend_from_slice(&padded_slot);

        Self::keccak256(&data)
    }

    /// Parse a query that may include nested mappings, arrays, and struct access
    ///
    /// # Arguments
    ///
    /// * `query` - The storage query string
    ///
    /// # Returns
    ///
    /// * `Ok(QueryParts)` - Successfully parsed query
    /// * `Err(TraverseError)` - Invalid query format
    ///
    /// # Supported Formats
    ///
    /// - Simple fields: `"owner"`, `"totalSupply"`
    /// - Single mappings: `"balances[0x742d35...]"`
    /// - Nested mappings: `"allowances[0x123...][0x456...]"`
    /// - Array indexing: `"items[5]"`
    /// - Struct fields: `"user.balance"` (future support)
    ///
    /// # Errors
    ///
    /// - Invalid mapping syntax (missing brackets)
    /// - Invalid hex encoding in mapping keys
    /// - Unsupported query patterns
    fn parse_query(&self, query: &str) -> Result<QueryParts, TraverseError> {
        // Handle struct field access (dot notation)
        if query.contains('.') {
            let parts: Vec<&str> = query.split('.').collect();
            if parts.len() != 2 {
                return Err(TraverseError::InvalidQuery(format!(
                    "Invalid struct field access: {}. Expected format: struct.field",
                    query
                )));
            }

            let struct_name = parts[0].trim().to_string();
            let field_name = parts[1].trim().to_string();

            // Check if the struct part has array/mapping access
            if struct_name.contains('[') {
                return Err(TraverseError::InvalidQuery(
                    "Complex struct access with arrays/mappings not yet implemented".to_string(),
                ));
            }

            // Check for special dynamic array/string access
            match field_name.as_str() {
                "length" => {
                    return Ok(QueryParts::DynamicLength {
                        field_name: struct_name,
                    })
                }
                "data" => {
                    return Ok(QueryParts::DynamicData {
                        field_name: struct_name,
                    })
                }
                _ => {
                    return Ok(QueryParts::StructField {
                        struct_name,
                        field_name,
                    })
                }
            }
        }

        // Check for array/mapping access patterns
        if query.contains('[') && query.contains(']') {
            // Count the number of bracket pairs to determine nesting level
            let open_brackets = query.matches('[').count();
            let close_brackets = query.matches(']').count();

            if open_brackets != close_brackets {
                return Err(TraverseError::InvalidQuery(format!(
                    "Mismatched brackets in query: {}",
                    query
                )));
            }

            // Extract field name (everything before first '[')
            let field_name = query
                .split('[')
                .next()
                .ok_or_else(|| {
                    TraverseError::InvalidQuery(format!("Invalid query format: {}", query))
                })?
                .trim()
                .to_string();

            // Extract all keys from bracket pairs
            let mut keys = Vec::new();
            let mut is_array_access = true;
            let mut current = query;

            while let Some(start) = current.find('[') {
                let end = current.find(']').ok_or_else(|| {
                    TraverseError::InvalidQuery(format!("Unclosed bracket in query: {}", query))
                })?;

                let key_str = &current[start + 1..end];

                // Check if this looks like a numeric index (array) or hex key (mapping)
                if key_str.trim().parse::<u64>().is_ok() {
                    // Numeric key - could be array access
                    let key_bytes = self.parse_key(key_str)?;
                    keys.push(key_bytes);
                } else {
                    // Non-numeric key - definitely mapping access
                    is_array_access = false;
                    let key_bytes = self.parse_key(key_str)?;
                    keys.push(key_bytes);
                }

                // Move past this bracket pair
                current = &current[end + 1..];
            }

            if keys.is_empty() {
                return Err(TraverseError::InvalidQuery(format!(
                    "No keys found in mapping query: {}",
                    query
                )));
            }

            // Determine the query type based on the analysis
            if keys.len() == 1 && is_array_access {
                // Single numeric key - could be array access
                let key_str = &query[query.find('[').unwrap() + 1..query.find(']').unwrap()];
                if let Ok(index) = key_str.trim().parse::<u64>() {
                    Ok(QueryParts::Array { field_name, index })
                } else {
                    // Fallback to mapping if parsing as number fails
                    Ok(QueryParts::Mapping {
                        field_name,
                        key: keys.into_iter().next().unwrap(),
                    })
                }
            } else if keys.len() == 1 {
                // Single mapping
                Ok(QueryParts::Mapping {
                    field_name,
                    key: keys.into_iter().next().unwrap(),
                })
            } else {
                // Nested mapping
                Ok(QueryParts::NestedMapping { field_name, keys })
            }
        } else {
            // Simple field access
            Ok(QueryParts::Field {
                field_name: query.to_string(),
            })
        }
    }

    /// Parse a key from a string, supporting both hex addresses and numeric indices
    ///
    /// # Arguments
    ///
    /// * `key_str` - The key string to parse
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<u8>)` - Parsed key bytes
    /// * `Err(TraverseError)` - Invalid key format
    ///
    /// # Supported Formats
    ///
    /// - Hex addresses: `0x742d35...` or `742d35...`
    /// - Decimal numbers: `123`, `0`
    /// - Hex numbers: `0x1a2b`
    fn parse_key(&self, key_str: &str) -> Result<Vec<u8>, TraverseError> {
        let key_str = key_str.trim();

        // Try parsing as hex (with or without 0x prefix)
        if key_str.starts_with("0x") || key_str.len() == 40 || key_str.len() == 64 {
            let hex_str = key_str.strip_prefix("0x").unwrap_or(key_str);
            hex::decode(hex_str)
                .map_err(|e| TraverseError::InvalidQuery(format!("Invalid hex key: {}", e)))
        } else {
            // Try parsing as decimal number
            if let Ok(num) = key_str.parse::<u64>() {
                Ok(num.to_be_bytes().to_vec())
            } else if let Ok(num) = key_str.parse::<u32>() {
                Ok(num.to_be_bytes().to_vec())
            } else {
                // Try as hex without 0x prefix
                hex::decode(key_str).map_err(|e| {
                    TraverseError::InvalidQuery(format!(
                        "Could not parse key '{}' as hex or number: {}",
                        key_str, e
                    ))
                })
            }
        }
    }

    /// Generate example keys for different mapping key types
    fn generate_example_keys(&self, key_type: &str) -> Vec<String> {
        match key_type {
            "t_address" => vec![
                "0x742d35Cc6634C0532925a3b8D97C2e0D8b2D9C".to_string(),
                "0xdAC17F958D2ee523a2206206994597C13D831ec7".to_string(), // USDT
                "0xA0b86a33E6Cc3b3c7bC8F1DCCF0e6a8F71c1c0123".to_string(), // Example address
            ],
            "t_uint256" | "t_uint128" | "t_uint64" | "t_uint32" => {
                vec!["0".to_string(), "1".to_string(), "42".to_string()]
            }
            "t_bytes32" => vec![
                "0x0000000000000000000000000000000000000000000000000000000000000001".to_string(),
                "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string(),
            ],
            "t_string" => vec![
                "0x48656c6c6f".to_string(), // "Hello" in hex
                "0x576f726c64".to_string(), // "World" in hex
            ],
            _ => vec!["0".to_string(), "1".to_string()],
        }
    }
}

impl KeyResolver for EthereumKeyResolver {
    fn resolve(&self, layout: &LayoutInfo, query: &str) -> Result<StaticKeyPath, TraverseError> {
        let query_parts = self.parse_query(query)?;

        match query_parts {
            QueryParts::Field { field_name } => {
                // Find the storage entry
                let entry = layout
                    .storage
                    .iter()
                    .find(|s| s.label == field_name)
                    .ok_or_else(|| {
                        TraverseError::PathResolution(format!("Field not found: {}", field_name))
                    })?;

                let slot = entry
                    .slot
                    .parse::<u64>()
                    .map_err(|e| TraverseError::PathResolution(format!("Invalid slot: {}", e)))?;

                let mut key_bytes = [0u8; 32];
                key_bytes[24..].copy_from_slice(&slot.to_be_bytes());

                // Get type info for field size
                let type_info = layout
                    .types
                    .iter()
                    .find(|t| t.label == entry.type_name)
                    .ok_or_else(|| {
                        TraverseError::PathResolution(format!(
                            "Type not found: {}",
                            entry.type_name
                        ))
                    })?;

                let field_size = type_info.number_of_bytes.parse::<u8>().ok();

                Ok(StaticKeyPath {
                    name: Box::leak(query.to_string().into_boxed_str()),
                    key: Key::Fixed(key_bytes),
                    offset: if entry.offset > 0 {
                        Some(entry.offset)
                    } else {
                        None
                    },
                    field_size,
                    layout_commitment: layout.commitment(),
                    zero_semantics: entry.zero_semantics,
                })
            }
            QueryParts::Mapping { field_name, key } => {
                // Find the mapping storage entry
                let entry = layout
                    .storage
                    .iter()
                    .find(|s| s.label == field_name)
                    .ok_or_else(|| {
                        TraverseError::PathResolution(format!("Mapping not found: {}", field_name))
                    })?;

                let slot = entry
                    .slot
                    .parse::<u64>()
                    .map_err(|e| TraverseError::PathResolution(format!("Invalid slot: {}", e)))?;

                let storage_key = Self::derive_mapping_key(&key, slot);

                // Get type info for value size
                let type_info = layout
                    .types
                    .iter()
                    .find(|t| t.label == entry.type_name)
                    .ok_or_else(|| {
                        TraverseError::PathResolution(format!(
                            "Type not found: {}",
                            entry.type_name
                        ))
                    })?;

                let field_size = if let Some(value_type) = &type_info.value {
                    layout
                        .types
                        .iter()
                        .find(|t| t.label == *value_type)
                        .and_then(|t| t.number_of_bytes.parse::<u8>().ok())
                } else {
                    type_info.number_of_bytes.parse::<u8>().ok()
                };

                Ok(StaticKeyPath {
                    name: Box::leak(query.to_string().into_boxed_str()),
                    key: Key::Fixed(storage_key),
                    offset: None, // Mappings typically don't have offsets
                    field_size,
                    layout_commitment: layout.commitment(),
                    zero_semantics: entry.zero_semantics,
                })
            }
            QueryParts::NestedMapping { field_name, keys } => {
                // Find the mapping storage entry
                let entry = layout
                    .storage
                    .iter()
                    .find(|s| s.label == field_name)
                    .ok_or_else(|| {
                        TraverseError::PathResolution(format!("Mapping not found: {}", field_name))
                    })?;

                let slot = entry
                    .slot
                    .parse::<u64>()
                    .map_err(|e| TraverseError::PathResolution(format!("Invalid slot: {}", e)))?;

                // For nested mappings like allowances[owner][spender]:
                // 1. Derive key1 = keccak256(owner ++ slot)
                // 2. Derive key2 = keccak256(spender ++ key1)
                let mut current_key = Self::derive_mapping_key(&keys[0], slot);

                // Chain each subsequent key derivation
                for key in &keys[1..] {
                    current_key = Self::derive_nested_mapping_key(key, &current_key);
                }

                // Get type info for value size
                let type_info = layout
                    .types
                    .iter()
                    .find(|t| t.label == entry.type_name)
                    .ok_or_else(|| {
                        TraverseError::PathResolution(format!(
                            "Type not found: {}",
                            entry.type_name
                        ))
                    })?;

                let field_size = if let Some(value_type) = &type_info.value {
                    layout
                        .types
                        .iter()
                        .find(|t| t.label == *value_type)
                        .and_then(|t| t.number_of_bytes.parse::<u8>().ok())
                } else {
                    type_info.number_of_bytes.parse::<u8>().ok()
                };

                Ok(StaticKeyPath {
                    name: Box::leak(query.to_string().into_boxed_str()),
                    key: Key::Fixed(current_key),
                    offset: None, // Mappings typically don't have offsets
                    field_size,
                    layout_commitment: layout.commitment(),
                    zero_semantics: entry.zero_semantics,
                })
            }
            QueryParts::Array { field_name, index } => {
                // Find the array storage entry
                let entry = layout
                    .storage
                    .iter()
                    .find(|s| s.label == field_name)
                    .ok_or_else(|| {
                        TraverseError::PathResolution(format!("Array not found: {}", field_name))
                    })?;

                let slot = entry
                    .slot
                    .parse::<u64>()
                    .map_err(|e| TraverseError::PathResolution(format!("Invalid slot: {}", e)))?;

                let array_key = Self::derive_array_key(slot, index);

                // Get type info for value size
                let type_info = layout
                    .types
                    .iter()
                    .find(|t| t.label == entry.type_name)
                    .ok_or_else(|| {
                        TraverseError::PathResolution(format!(
                            "Type not found: {}",
                            entry.type_name
                        ))
                    })?;

                let field_size = type_info.number_of_bytes.parse::<u8>().ok();

                Ok(StaticKeyPath {
                    name: Box::leak(query.to_string().into_boxed_str()),
                    key: Key::Fixed(array_key),
                    offset: None, // Arrays typically don't have offsets
                    field_size,
                    layout_commitment: layout.commitment(),
                    zero_semantics: entry.zero_semantics,
                })
            }
            QueryParts::StructField {
                struct_name,
                field_name,
            } => {
                // Find the storage entry
                let entry = layout
                    .storage
                    .iter()
                    .find(|s| s.label == struct_name)
                    .ok_or_else(|| {
                        TraverseError::PathResolution(format!("Struct not found: {}", struct_name))
                    })?;

                let slot = entry
                    .slot
                    .parse::<u64>()
                    .map_err(|e| TraverseError::PathResolution(format!("Invalid slot: {}", e)))?;

                let mut key_bytes = [0u8; 32];
                key_bytes[24..].copy_from_slice(&slot.to_be_bytes());

                // Get type info for field size
                let type_info = layout
                    .types
                    .iter()
                    .find(|t| t.label == field_name)
                    .ok_or_else(|| {
                        TraverseError::PathResolution(format!("Field not found: {}", field_name))
                    })?;

                let field_size = type_info.number_of_bytes.parse::<u8>().ok();

                Ok(StaticKeyPath {
                    name: Box::leak(query.to_string().into_boxed_str()),
                    key: Key::Fixed(key_bytes),
                    offset: None,
                    field_size,
                    layout_commitment: layout.commitment(),
                    zero_semantics: entry.zero_semantics,
                })
            }
            QueryParts::DynamicLength { field_name } => {
                // For dynamic arrays and strings, the length is stored at the slot itself
                let entry = layout
                    .storage
                    .iter()
                    .find(|s| s.label == field_name)
                    .ok_or_else(|| {
                        TraverseError::PathResolution(format!(
                            "Dynamic field not found: {}",
                            field_name
                        ))
                    })?;

                let slot = entry
                    .slot
                    .parse::<u64>()
                    .map_err(|e| TraverseError::PathResolution(format!("Invalid slot: {}", e)))?;

                let mut key_bytes = [0u8; 32];
                key_bytes[24..].copy_from_slice(&slot.to_be_bytes());

                Ok(StaticKeyPath {
                    name: Box::leak(query.to_string().into_boxed_str()),
                    key: Key::Fixed(key_bytes),
                    offset: None,
                    field_size: Some(32), // Length is stored as uint256
                    layout_commitment: layout.commitment(),
                    zero_semantics: entry.zero_semantics,
                })
            }
            QueryParts::DynamicData { field_name } => {
                // For dynamic arrays and strings, the data starts at keccak256(slot)
                let entry = layout
                    .storage
                    .iter()
                    .find(|s| s.label == field_name)
                    .ok_or_else(|| {
                        TraverseError::PathResolution(format!(
                            "Dynamic field not found: {}",
                            field_name
                        ))
                    })?;

                let slot = entry
                    .slot
                    .parse::<u64>()
                    .map_err(|e| TraverseError::PathResolution(format!("Invalid slot: {}", e)))?;

                // Data starts at keccak256(slot)
                let mut slot_bytes = [0u8; 32];
                slot_bytes[24..].copy_from_slice(&slot.to_be_bytes());
                let data_key = Self::keccak256(&slot_bytes);

                Ok(StaticKeyPath {
                    name: Box::leak(query.to_string().into_boxed_str()),
                    key: Key::Fixed(data_key),
                    offset: None,
                    field_size: Some(32), // Data is stored in 32-byte chunks
                    layout_commitment: layout.commitment(),
                    zero_semantics: entry.zero_semantics,
                })
            }
        }
    }

    fn resolve_all(&self, layout: &LayoutInfo) -> Result<Vec<StaticKeyPath>, TraverseError> {
        let mut paths = Vec::new();

        for entry in &layout.storage {
            // Get type information to determine if this is a mapping
            if let Some(type_info) = layout.types.iter().find(|t| t.label == entry.type_name) {
                if type_info.encoding == "mapping" {
                    // Generate example mapping paths with common example keys
                    let example_keys = self.generate_example_keys(
                        type_info.key.as_ref().unwrap_or(&"t_address".to_string()),
                    );

                    for example_key in example_keys {
                        let query = format!("{}[{}]", entry.label, example_key);
                        if let Ok(path) = self.resolve(layout, &query) {
                            paths.push(path);
                        }
                    }
                } else {
                    // Simple field - resolve directly
                    if let Ok(path) = self.resolve(layout, &entry.label) {
                        paths.push(path);
                    }
                }
            } else {
                // Fallback for entries without type info
                if let Ok(path) = self.resolve(layout, &entry.label) {
                    paths.push(path);
                }
            }
        }

        Ok(paths)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keccak256() {
        let result = EthereumKeyResolver::keccak256(b"hello");
        // Known keccak256("hello") = 0x1c8aff950685c2ed4bc3174f3472287b56d9517b9c948127319a09a7a36deac8
        let expected =
            hex::decode("1c8aff950685c2ed4bc3174f3472287b56d9517b9c948127319a09a7a36deac8")
                .unwrap();
        assert_eq!(result.to_vec(), expected);
    }

    #[test]
    fn test_parse_simple_query() {
        let resolver = EthereumKeyResolver;
        let result = resolver.parse_query("owner").unwrap();
        match result {
            QueryParts::Field { field_name } => assert_eq!(field_name, "owner"),
            _ => panic!("Expected field query"),
        }
    }

    #[test]
    fn test_parse_mapping_query() {
        let resolver = EthereumKeyResolver;
        let result = resolver.parse_query("balances[0x1234]").unwrap();
        match result {
            QueryParts::Mapping { field_name, key } => {
                assert_eq!(field_name, "balances");
                assert_eq!(key, hex::decode("1234").unwrap());
            }
            _ => panic!("Expected mapping query"),
        }
    }

    #[test]
    fn test_parse_nested_mapping_query() {
        let resolver = EthereumKeyResolver;
        let result = resolver.parse_query("allowances[0x1234][0x5678]").unwrap();
        match result {
            QueryParts::NestedMapping { field_name, keys } => {
                assert_eq!(field_name, "allowances");
                assert_eq!(keys.len(), 2);
                assert_eq!(keys[0], hex::decode("1234").unwrap());
                assert_eq!(keys[1], hex::decode("5678").unwrap());
            }
            _ => panic!("Expected nested mapping query"),
        }
    }

    #[test]
    fn test_parse_numeric_keys() {
        let resolver = EthereumKeyResolver;

        // Test decimal number parsing
        let key = resolver.parse_key("123").unwrap();
        assert_eq!(key, 123u64.to_be_bytes().to_vec());

        // Test hex number parsing
        let key = resolver.parse_key("0x1a2b").unwrap();
        assert_eq!(key, hex::decode("1a2b").unwrap());
    }

    #[test]
    fn test_nested_mapping_key_derivation() {
        let key1 = [1u8; 32];
        let key2 = b"test";
        let result = EthereumKeyResolver::derive_nested_mapping_key(key2, &key1);

        // Should be keccak256(padded_key2 ++ key1)
        let mut expected_input = Vec::new();
        let mut padded_key2 = [0u8; 32];
        padded_key2[28..].copy_from_slice(key2); // Right-pad for non-address
        expected_input.extend_from_slice(&padded_key2);
        expected_input.extend_from_slice(&key1);

        let expected = EthereumKeyResolver::keccak256(&expected_input);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_parse_array_query() {
        let resolver = EthereumKeyResolver;
        let result = resolver.parse_query("items[5]").unwrap();
        match result {
            QueryParts::Array { field_name, index } => {
                assert_eq!(field_name, "items");
                assert_eq!(index, 5);
            }
            _ => panic!("Expected array query"),
        }
    }

    #[test]
    fn test_array_key_derivation() {
        let slot = 3u64;
        let index = 5u64;
        let result = EthereumKeyResolver::derive_array_key(slot, index);

        // Should be keccak256(slot) + index
        let mut slot_bytes = [0u8; 32];
        slot_bytes[24..].copy_from_slice(&slot.to_be_bytes());
        let base_key = EthereumKeyResolver::keccak256(&slot_bytes);

        // Manually add index to base_key
        let mut expected = base_key;
        let mut carry = index;
        for i in (0..32).rev() {
            let sum = expected[i] as u64 + carry;
            expected[i] = sum as u8;
            carry = sum >> 8;
            if carry == 0 {
                break;
            }
        }

        assert_eq!(result, expected);
    }

    #[test]
    fn test_distinguish_array_vs_mapping() {
        let resolver = EthereumKeyResolver;

        // Numeric key should be parsed as array
        let result = resolver.parse_query("items[123]").unwrap();
        assert!(matches!(result, QueryParts::Array { .. }));

        // Hex key should be parsed as mapping
        let result = resolver.parse_query("balances[0x1234]").unwrap();
        assert!(matches!(result, QueryParts::Mapping { .. }));

        // Long hex address should be parsed as mapping
        let result = resolver
            .parse_query("balances[742d35Cc6634C0532925a3b8D97C2e0D8b2D9C]")
            .unwrap();
        assert!(matches!(result, QueryParts::Mapping { .. }));
    }

    #[test]
    fn test_parse_struct_field_query() {
        let resolver = EthereumKeyResolver;
        let result = resolver.parse_query("user.balance").unwrap();
        match result {
            QueryParts::StructField {
                struct_name,
                field_name,
            } => {
                assert_eq!(struct_name, "user");
                assert_eq!(field_name, "balance");
            }
            _ => panic!("Expected struct field query"),
        }
    }

    #[test]
    fn test_invalid_struct_queries() {
        let resolver = EthereumKeyResolver;

        // Too many dots
        let result = resolver.parse_query("user.profile.name");
        assert!(result.is_err());

        // Complex struct access not supported yet
        let result = resolver.parse_query("users[0].balance");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_dynamic_length_query() {
        let resolver = EthereumKeyResolver;
        let result = resolver.parse_query("name.length").unwrap();
        match result {
            QueryParts::DynamicLength { field_name } => {
                assert_eq!(field_name, "name");
            }
            _ => panic!("Expected dynamic length query"),
        }
    }

    #[test]
    fn test_parse_dynamic_data_query() {
        let resolver = EthereumKeyResolver;
        let result = resolver.parse_query("name.data").unwrap();
        match result {
            QueryParts::DynamicData { field_name } => {
                assert_eq!(field_name, "name");
            }
            _ => panic!("Expected dynamic data query"),
        }
    }
}
