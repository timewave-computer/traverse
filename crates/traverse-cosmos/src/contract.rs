//! CosmWasm contract analysis and schema parsing
//!
//! This module provides functionality to analyze CosmWasm contracts from their
//! message schemas and identify storage patterns for ZK coprocessor integration.

use crate::CosmosError;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Represents a CosmWasm contract with its message schema and storage patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CosmWasmContract {
    /// Contract name
    pub name: String,
    /// Instantiate message schema
    pub instantiate_msg: Option<Value>,
    /// Execute message schema
    pub execute_msg: Option<Value>,
    /// Query message schema
    pub query_msg: Option<Value>,
    /// Identified storage patterns
    pub storage_patterns: Vec<StoragePattern>,
    /// Contract metadata
    pub metadata: ContractMetadata,
}

/// Analysis results for a CosmWasm contract
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractAnalysis {
    /// Original contract
    pub contract: CosmWasmContract,
    /// Detected storage variables
    pub storage_variables: Vec<StorageVariable>,
    /// Identified message patterns
    pub message_patterns: Vec<MessagePattern>,
    /// Complexity metrics
    pub complexity: ComplexityMetrics,
    /// Recommendations for optimization
    pub recommendations: Vec<String>,
}

/// Storage pattern detected in CosmWasm contract
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoragePattern {
    /// Pattern type (Map, Item, IndexedMap, etc.)
    pub pattern_type: StorageType,
    /// Storage key or namespace
    pub key: String,
    /// Value type information
    pub value_type: String,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Types of CosmWasm storage patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageType {
    /// Simple key-value storage
    Item,
    /// Map storage with dynamic keys
    Map,
    /// Indexed map with multiple indices
    IndexedMap,
    /// Snapshot map for historical data
    SnapshotMap,
    /// Multi-index storage
    MultiIndex,
    /// Custom storage pattern
    Custom(String),
}

/// Storage variable identified in contract
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageVariable {
    /// Variable name
    pub name: String,
    /// Storage type
    pub storage_type: StorageType,
    /// Key path for access
    pub key_path: String,
    /// Value type
    pub value_type: String,
    /// Whether it's queryable
    pub queryable: bool,
}

/// Message pattern identified in contract
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessagePattern {
    /// Pattern type (StandardToken, NFT, DAO, etc.)
    pub pattern_type: String,
    /// Associated messages
    pub messages: Vec<String>,
    /// Confidence level (0.0 - 1.0)
    pub confidence: f32,
}

/// Contract metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractMetadata {
    /// Contract version
    pub version: Option<String>,
    /// Description
    pub description: Option<String>,
    /// Dependencies
    pub dependencies: Vec<String>,
    /// Features
    pub features: Vec<String>,
}

/// Complexity metrics for contract analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityMetrics {
    /// Number of execute messages
    pub execute_msg_count: usize,
    /// Number of query messages
    pub query_msg_count: usize,
    /// Number of storage variables
    pub storage_var_count: usize,
    /// Estimated complexity score
    pub complexity_score: u32,
}

impl CosmWasmContract {
    /// Parse a CosmWasm contract from JSON schema files
    pub fn from_schema_files(
        instantiate_path: Option<&str>,
        execute_path: Option<&str>,
        query_path: Option<&str>,
    ) -> Result<Self, CosmosError> {
        let instantiate_msg = if let Some(path) = instantiate_path {
            let content = std::fs::read_to_string(path).map_err(|e| {
                CosmosError::InvalidSchema(format!("Failed to read instantiate schema: {}", e))
            })?;
            Some(serde_json::from_str(&content)?)
        } else {
            None
        };

        let execute_msg = if let Some(path) = execute_path {
            let content = std::fs::read_to_string(path).map_err(|e| {
                CosmosError::InvalidSchema(format!("Failed to read execute schema: {}", e))
            })?;
            Some(serde_json::from_str(&content)?)
        } else {
            None
        };

        let query_msg = if let Some(path) = query_path {
            let content = std::fs::read_to_string(path).map_err(|e| {
                CosmosError::InvalidSchema(format!("Failed to read query schema: {}", e))
            })?;
            Some(serde_json::from_str(&content)?)
        } else {
            None
        };

        Ok(CosmWasmContract {
            name: "UnknownContract".to_string(),
            instantiate_msg,
            execute_msg,
            query_msg,
            storage_patterns: Vec::new(),
            metadata: ContractMetadata {
                version: None,
                description: None,
                dependencies: Vec::new(),
                features: Vec::new(),
            },
        })
    }

    /// Analyze the contract and identify patterns
    pub fn analyze(&self) -> Result<ContractAnalysis, CosmosError> {
        let mut message_patterns = Vec::new();
        let mut complexity = ComplexityMetrics {
            execute_msg_count: 0,
            query_msg_count: 0,
            storage_var_count: 0,
            complexity_score: 0,
        };

        // Analyze execute messages
        if let Some(ref execute_msg) = self.execute_msg {
            let execute_analysis = Self::analyze_execute_messages(execute_msg)?;
            complexity.execute_msg_count = execute_analysis.len();
            message_patterns.extend(execute_analysis);
        }

        // Analyze query messages
        if let Some(ref query_msg) = self.query_msg {
            let query_analysis = Self::analyze_query_messages(query_msg)?;
            complexity.query_msg_count = query_analysis.len();
        }

        // Infer storage patterns from messages
        let storage_variables =
            Self::infer_storage_from_messages(&self.execute_msg, &self.query_msg)?;
        complexity.storage_var_count = storage_variables.len();
        complexity.complexity_score = self.calculate_complexity_score(&complexity);

        // Generate recommendations
        let recommendations = Self::generate_recommendations(&complexity, &message_patterns);

        Ok(ContractAnalysis {
            contract: self.clone(),
            storage_variables,
            message_patterns,
            complexity,
            recommendations,
        })
    }

    /// Analyze execute messages to identify patterns
    fn analyze_execute_messages(execute_msg: &Value) -> Result<Vec<MessagePattern>, CosmosError> {
        let mut patterns = Vec::new();

        // Look for common CosmWasm patterns in execute messages
        if let Some(obj) = execute_msg.as_object() {
            for (key, _value) in obj {
                match key.to_lowercase().as_str() {
                    "transfer" | "send" | "transfer_from" => {
                        patterns.push(MessagePattern {
                            pattern_type: "CW20Token".to_string(),
                            messages: vec![key.clone()],
                            confidence: 0.8,
                        });
                    }
                    "mint" | "burn" => {
                        patterns.push(MessagePattern {
                            pattern_type: "MintableBurnable".to_string(),
                            messages: vec![key.clone()],
                            confidence: 0.7,
                        });
                    }
                    "approve" | "increase_allowance" | "decrease_allowance" => {
                        patterns.push(MessagePattern {
                            pattern_type: "Approval".to_string(),
                            messages: vec![key.clone()],
                            confidence: 0.8,
                        });
                    }
                    _ => {}
                }
            }
        }

        Ok(patterns)
    }

    /// Analyze query messages
    fn analyze_query_messages(query_msg: &Value) -> Result<Vec<String>, CosmosError> {
        let mut queries = Vec::new();

        if let Some(obj) = query_msg.as_object() {
            for (key, _value) in obj {
                queries.push(key.clone());
            }
        }

        Ok(queries)
    }

    /// Infer storage variables from message patterns
    fn infer_storage_from_messages(
        execute_msg: &Option<Value>,
        _query_msg: &Option<Value>,
    ) -> Result<Vec<StorageVariable>, CosmosError> {
        let mut storage_vars = Vec::new();

        // Infer from execute messages
        if let Some(execute) = execute_msg {
            if let Some(obj) = execute.as_object() {
                for (key, _value) in obj {
                    match key.to_lowercase().as_str() {
                        "transfer" | "send" | "transfer_from" => {
                            storage_vars.push(StorageVariable {
                                name: "balances".to_string(),
                                storage_type: StorageType::Map,
                                key_path: "balances".to_string(),
                                value_type: "Uint128".to_string(),
                                queryable: true,
                            });
                        }
                        "approve" => {
                            storage_vars.push(StorageVariable {
                                name: "allowances".to_string(),
                                storage_type: StorageType::Map,
                                key_path: "allowances".to_string(),
                                value_type: "Uint128".to_string(),
                                queryable: true,
                            });
                        }
                        _ => {}
                    }
                }
            }
        }

        // Add common storage variables
        storage_vars.push(StorageVariable {
            name: "config".to_string(),
            storage_type: StorageType::Item,
            key_path: "config".to_string(),
            value_type: "Config".to_string(),
            queryable: true,
        });

        Ok(storage_vars)
    }

    /// Calculate complexity score based on metrics
    fn calculate_complexity_score(&self, metrics: &ComplexityMetrics) -> u32 {
        let base_score = metrics.execute_msg_count as u32 * 10
            + metrics.query_msg_count as u32 * 5
            + metrics.storage_var_count as u32 * 15;

        // Add bonus for complex patterns
        let pattern_bonus = self.storage_patterns.len() as u32 * 20;

        base_score + pattern_bonus
    }

    /// Generate optimization recommendations
    fn generate_recommendations(
        complexity: &ComplexityMetrics,
        patterns: &[MessagePattern],
    ) -> Vec<String> {
        let mut recommendations = Vec::new();

        if complexity.complexity_score > 200 {
            recommendations.push(
                "Consider splitting into multiple contracts for better modularity".to_string(),
            );
        }

        if complexity.storage_var_count > 10 {
            recommendations
                .push("Review storage layout for optimization opportunities".to_string());
        }

        if patterns.iter().any(|p| p.pattern_type == "CW20Token") {
            recommendations.push(
                "Consider using standard CW20 implementation for better compatibility".to_string(),
            );
        }

        if recommendations.is_empty() {
            recommendations
                .push("Contract structure looks good for ZK coprocessor integration".to_string());
        }

        recommendations
    }
}
