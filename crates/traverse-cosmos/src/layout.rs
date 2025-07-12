//! CosmWasm storage layout compilation
//!
//! This module converts CosmWasm contract schemas into the canonical LayoutInfo
//! format used by the traverse system for ZK coprocessor integration.

use crate::{contract::CosmWasmContract, CosmosError};
use std::path::Path;
use traverse_core::{LayoutCompiler, LayoutInfo, StorageEntry, TraverseError, TypeInfo};

/// CosmWasm layout compiler that converts contract schemas to canonical format
///
/// This implementation analyzes CosmWasm message schemas and infers storage
/// layouts based on common CosmWasm patterns and conventions.
///
/// # Usage
///
/// ```rust,ignore
/// use traverse_cosmos::CosmosLayoutCompiler;
/// use traverse_core::LayoutCompiler;
///
/// let compiler = CosmosLayoutCompiler;
/// let layout = compiler.compile_layout(Path::new("contract_schema.json"))?;
/// ```
pub struct CosmosLayoutCompiler;

impl CosmosLayoutCompiler {
    /// Generate storage entries for CosmWasm contract
    ///
    /// Maps CosmWasm storage patterns to canonical storage entries
    fn generate_storage_entries(
        contract: &CosmWasmContract,
    ) -> Result<Vec<StorageEntry>, CosmosError> {
        let mut entries = Vec::new();
        let analysis = contract.analyze()?;

        for (index, storage_var) in analysis.storage_variables.iter().enumerate() {
            let entry = StorageEntry {
                label: storage_var.name.clone(),
                slot: index.to_string(), // CosmWasm uses namespace-based keys, not slots
                offset: 0,               // CosmWasm doesn't have slot offsets like Ethereum
                type_name: Self::cosmos_to_canonical_type(&storage_var.value_type),
                zero_semantics: traverse_core::ZeroSemantics::NeverWritten,
            };
            entries.push(entry);

            // Generate additional entries for map-type storage
            if matches!(storage_var.storage_type, crate::contract::StorageType::Map) {
                let map_entry = StorageEntry {
                    label: format!("{}[key]", storage_var.name),
                    slot: format!("map_{}", index),
                    offset: 0,
                    type_name: format!(
                        "t_mapping_bytes_{}",
                        Self::cosmos_to_canonical_type(&storage_var.value_type)
                    ),
                    zero_semantics: traverse_core::ZeroSemantics::ValidZero,
                };
                entries.push(map_entry);
            }
        }

        Ok(entries)
    }

    /// Generate type information for CosmWasm types
    fn generate_type_info(contract: &CosmWasmContract) -> Result<Vec<TypeInfo>, CosmosError> {
        let mut types = Vec::new();
        let analysis = contract.analyze()?;

        // Add standard CosmWasm types
        types.extend(Self::standard_cosmwasm_types());

        // Add contract-specific types
        for storage_var in &analysis.storage_variables {
            let type_name = Self::cosmos_to_canonical_type(&storage_var.value_type);

            if !types.iter().any(|t| t.label == type_name) {
                let type_info = TypeInfo {
                    label: type_name.clone(),
                    number_of_bytes: Self::get_type_size(&storage_var.value_type).to_string(),
                    encoding: match storage_var.storage_type {
                        crate::contract::StorageType::Item => "inplace".to_string(),
                        crate::contract::StorageType::Map => "mapping".to_string(),
                        _ => "inplace".to_string(),
                    },
                    base: None,
                    key: if matches!(storage_var.storage_type, crate::contract::StorageType::Map) {
                        Some("t_bytes".to_string())
                    } else {
                        None
                    },
                    value: if matches!(storage_var.storage_type, crate::contract::StorageType::Map)
                    {
                        Some(Self::cosmos_to_canonical_type(&storage_var.value_type))
                    } else {
                        None
                    },
                };
                types.push(type_info);
            }

            // Add mapping type for map storage
            if matches!(storage_var.storage_type, crate::contract::StorageType::Map) {
                let mapping_type = format!(
                    "t_mapping_bytes_{}",
                    Self::cosmos_to_canonical_type(&storage_var.value_type)
                );

                if !types.iter().any(|t| t.label == mapping_type) {
                    let mapping_info = TypeInfo {
                        label: mapping_type,
                        number_of_bytes: "32".to_string(), // Standard mapping size
                        encoding: "mapping".to_string(),
                        base: None,
                        key: Some("t_bytes".to_string()),
                        value: Some(Self::cosmos_to_canonical_type(&storage_var.value_type)),
                    };
                    types.push(mapping_info);
                }
            }
        }

        Ok(types)
    }

    /// Convert CosmWasm type names to canonical format
    fn cosmos_to_canonical_type(cosmos_type: &str) -> String {
        match cosmos_type {
            "Uint128" => "t_uint128".to_string(),
            "Uint64" => "t_uint64".to_string(),
            "Uint32" => "t_uint32".to_string(),
            "String" => "t_string".to_string(),
            "Addr" => "t_address".to_string(),
            "Bool" => "t_bool".to_string(),
            "Binary" => "t_bytes".to_string(),
            "Decimal" => "t_decimal".to_string(),
            _ => format!("t_{}", cosmos_type.to_lowercase()),
        }
    }

    /// Get the size in bytes for a CosmWasm type
    fn get_type_size(cosmos_type: &str) -> u32 {
        match cosmos_type {
            "Uint128" => 16,
            "Uint64" => 8,
            "Uint32" => 4,
            "Bool" => 1,
            "Addr" => 32, // CosmWasm addresses are represented as strings but stored as 32 bytes
            "String" | "Binary" => 32, // Variable length, but we use 32 as standard
            "Decimal" => 16, // Decimal is typically 128 bits
            _ => 32,      // Default size for unknown types
        }
    }

    /// Generate standard CosmWasm type definitions
    fn standard_cosmwasm_types() -> Vec<TypeInfo> {
        vec![
            TypeInfo {
                label: "t_uint128".to_string(),
                number_of_bytes: "16".to_string(),
                encoding: "inplace".to_string(),
                base: None,
                key: None,
                value: None,
            },
            TypeInfo {
                label: "t_uint64".to_string(),
                number_of_bytes: "8".to_string(),
                encoding: "inplace".to_string(),
                base: None,
                key: None,
                value: None,
            },
            TypeInfo {
                label: "t_uint32".to_string(),
                number_of_bytes: "4".to_string(),
                encoding: "inplace".to_string(),
                base: None,
                key: None,
                value: None,
            },
            TypeInfo {
                label: "t_bool".to_string(),
                number_of_bytes: "1".to_string(),
                encoding: "inplace".to_string(),
                base: None,
                key: None,
                value: None,
            },
            TypeInfo {
                label: "t_address".to_string(),
                number_of_bytes: "32".to_string(),
                encoding: "inplace".to_string(),
                base: None,
                key: None,
                value: None,
            },
            TypeInfo {
                label: "t_string".to_string(),
                number_of_bytes: "32".to_string(),
                encoding: "bytes".to_string(),
                base: None,
                key: None,
                value: None,
            },
            TypeInfo {
                label: "t_bytes".to_string(),
                number_of_bytes: "32".to_string(),
                encoding: "bytes".to_string(),
                base: None,
                key: None,
                value: None,
            },
            TypeInfo {
                label: "t_decimal".to_string(),
                number_of_bytes: "16".to_string(),
                encoding: "inplace".to_string(),
                base: None,
                key: None,
                value: None,
            },
        ]
    }
}

impl LayoutCompiler for CosmosLayoutCompiler {
    /// Compile CosmWasm contract schema to canonical layout format
    ///
    /// Expects a JSON file containing CosmWasm message schemas or a combined
    /// schema file with instantiate, execute, and query message definitions.
    ///
    /// # Arguments
    ///
    /// * `schema_path` - Path to the CosmWasm schema JSON file
    ///
    /// # Returns
    ///
    /// * `Ok(LayoutInfo)` - Successfully compiled layout
    /// * `Err(TraverseError)` - Failed to read, parse, or compile the schema
    ///
    /// # Errors
    ///
    /// - `TraverseError::Io` - File cannot be read
    /// - `TraverseError::Serialization` - Invalid JSON format
    /// - `TraverseError::InvalidLayout` - Invalid CosmWasm schema
    fn compile_layout(&self, schema_path: &Path) -> Result<LayoutInfo, TraverseError> {
        // Try to parse as a combined schema file first
        let content = std::fs::read_to_string(schema_path)?;

        // Attempt to parse as a combined schema containing multiple message types
        let combined_schema: Result<serde_json::Value, _> = serde_json::from_str(&content);

        let contract = match combined_schema {
            Ok(schema) => {
                // Check if this is a combined schema with separate message types
                if let Some(obj) = schema.as_object() {
                    let instantiate_msg = obj.get("instantiate").cloned();
                    let execute_msg = obj.get("execute").cloned();
                    let query_msg = obj.get("query").cloned();

                    CosmWasmContract {
                        name: schema_path
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("UnknownContract")
                            .to_string(),
                        instantiate_msg,
                        execute_msg,
                        query_msg,
                        storage_patterns: Vec::new(),
                        metadata: crate::contract::ContractMetadata {
                            version: None,
                            description: None,
                            dependencies: Vec::new(),
                            features: Vec::new(),
                        },
                    }
                } else {
                    // Single message schema - try to infer type
                    let msg_type = schema_path
                        .file_name()
                        .and_then(|s| s.to_str())
                        .unwrap_or("");

                    let (instantiate_msg, execute_msg, query_msg) =
                        if msg_type.contains("instantiate") {
                            (Some(schema), None, None)
                        } else if msg_type.contains("execute") {
                            (None, Some(schema), None)
                        } else if msg_type.contains("query") {
                            (None, None, Some(schema))
                        } else {
                            // Assume it's an execute message if unknown
                            (None, Some(schema), None)
                        };

                    CosmWasmContract {
                        name: schema_path
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("UnknownContract")
                            .to_string(),
                        instantiate_msg,
                        execute_msg,
                        query_msg,
                        storage_patterns: Vec::new(),
                        metadata: crate::contract::ContractMetadata {
                            version: None,
                            description: None,
                            dependencies: Vec::new(),
                            features: Vec::new(),
                        },
                    }
                }
            }
            Err(e) => {
                return Err(TraverseError::Serialization(format!(
                    "Invalid JSON schema: {}",
                    e
                )));
            }
        };

        // Generate storage entries and type information
        let storage = Self::generate_storage_entries(&contract).map_err(|e| {
                            TraverseError::InvalidInput(format!("Failed to generate storage: {}", e))
        })?;

        let types = Self::generate_type_info(&contract).map_err(|e| {
                            TraverseError::InvalidInput(format!("Failed to generate types: {}", e))
        })?;

        let layout = LayoutInfo {
            contract_name: contract.name,
            storage,
            types,
        };

        Ok(layout)
    }
}
