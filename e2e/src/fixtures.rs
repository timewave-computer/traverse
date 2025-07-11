//! Test fixtures for end-to-end CLI testing with semantic examples
//!
//! This module provides sample contract files, configurations, and test data
//! needed to comprehensively test all CLI functionality with semantic storage proofs.

use anyhow::Result;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// Test fixtures containing all necessary test data with semantic examples
pub struct TestFixtures {
    /// Temporary directory for test files
    pub temp_dir: TempDir,
    /// Sample Ethereum ABI files with semantic metadata
    pub ethereum_abis: HashMap<String, PathBuf>,
    /// Sample CosmWasm message schemas with semantic metadata
    pub cosmos_schemas: HashMap<String, PathBuf>,
    /// Configuration files for batch operations
    pub config_files: HashMap<String, PathBuf>,
    /// Query files for testing
    pub query_files: HashMap<String, PathBuf>,
    /// Mock RPC responses
    pub mock_responses: HashMap<String, Value>,
    /// Semantic test data for various zero meaning types
    pub semantic_test_data: HashMap<String, Value>,
    /// Mock event data for validation testing
    pub mock_event_data: HashMap<String, Value>,
}

impl TestFixtures {
    /// Create new test fixtures with all necessary test data including semantic examples
    pub async fn new() -> Result<Self> {
        let temp_dir = TempDir::new()?;
        let base_path = temp_dir.path();

        // Create subdirectories
        fs::create_dir_all(base_path.join("ethereum"))?;
        fs::create_dir_all(base_path.join("cosmos"))?;
        fs::create_dir_all(base_path.join("configs"))?;
        fs::create_dir_all(base_path.join("queries"))?;
        fs::create_dir_all(base_path.join("outputs"))?;
        fs::create_dir_all(base_path.join("semantic_tests"))?;
        fs::create_dir_all(base_path.join("mock_events"))?;

        let mut fixtures = Self {
            temp_dir,
            ethereum_abis: HashMap::new(),
            cosmos_schemas: HashMap::new(),
            config_files: HashMap::new(),
            query_files: HashMap::new(),
            mock_responses: HashMap::new(),
            semantic_test_data: HashMap::new(),
            mock_event_data: HashMap::new(),
        };

        // Create test files
        fixtures.create_ethereum_fixtures().await?;
        fixtures.create_cosmos_fixtures().await?;
        fixtures.create_config_fixtures().await?;
        fixtures.create_query_fixtures().await?;
        fixtures.create_mock_responses().await?;
        fixtures.create_semantic_test_data().await?;
        fixtures.create_mock_event_data().await?;

        Ok(fixtures)
    }

    /// Get path to a test file
    pub fn path(&self, relative_path: &str) -> PathBuf {
        self.temp_dir.path().join(relative_path)
    }

    /// Get CLI binary path
    pub fn cli_path(&self) -> PathBuf {
        let exe_name = if cfg!(windows) {
            "traverse-cli.exe"
        } else {
            "traverse-cli"
        };

        // Look for the binary relative to the workspace root
        let current_dir = std::env::current_dir().unwrap();
        let workspace_root = current_dir.parent().unwrap_or(&current_dir);

        let debug_path = workspace_root.join("target/debug").join(exe_name);
        let release_path = workspace_root.join("target/release").join(exe_name);

        if release_path.exists() {
            release_path
        } else {
            debug_path
        }
    }

    /// Get semantic test data for a specific zero meaning type
    #[allow(dead_code)]
    pub fn get_semantic_test_data(&self, semantic_type: &str) -> Option<&Value> {
        self.semantic_test_data.get(semantic_type)
    }

    /// Get mock event data for validation testing
    #[allow(dead_code)]
    pub fn get_mock_event_data(&self, event_type: &str) -> Option<&Value> {
        self.mock_event_data.get(event_type)
    }

    /// Create Ethereum layout test fixtures with semantic metadata
    async fn create_ethereum_fixtures(&mut self) -> Result<()> {
        // ERC20 Token Layout with semantic metadata
        let erc20_layout = json!({
            "contract_name": "MockERC20",
            "storage": [
                {
                    "label": "_balances",
                    "slot": "0",
                    "offset": 0,
                    "type_name": "t_mapping(t_address,t_uint256)",
                    "zero_semantics": "never_written"
                },
                {
                    "label": "_allowances",
                    "slot": "1",
                    "offset": 0,
                    "type_name": "t_mapping(t_address,t_mapping(t_address,t_uint256))",
                    "zero_semantics": "never_written"
                },
                {
                    "label": "_totalSupply",
                    "slot": "2",
                    "offset": 0,
                    "type_name": "t_uint256",
                    "zero_semantics": "explicitly_zero"
                },
                {
                    "label": "totalSupply",
                    "slot": "2",
                    "offset": 0,
                    "type_name": "t_uint256",
                    "zero_semantics": "explicitly_zero"
                },
                {
                    "label": "_name",
                    "slot": "3",
                    "offset": 0,
                    "type_name": "t_string_storage",
                    "zero_semantics": "never_written"
                },
                {
                    "label": "name",
                    "slot": "3",
                    "offset": 0,
                    "type_name": "t_string_storage",
                    "zero_semantics": "never_written"
                },
                {
                    "label": "_symbol",
                    "slot": "4",
                    "offset": 0,
                    "type_name": "t_string_storage",
                    "zero_semantics": "never_written"
                },
                {
                    "label": "symbol",
                    "slot": "4",
                    "offset": 0,
                    "type_name": "t_string_storage",
                    "zero_semantics": "never_written"
                },
                {
                    "label": "_decimals",
                    "slot": "5",
                    "offset": 0,
                    "type_name": "t_uint8",
                    "zero_semantics": "valid_zero"
                },
                {
                    "label": "decimals",
                    "slot": "5",
                    "offset": 0,
                    "type_name": "t_uint8",
                    "zero_semantics": "valid_zero"
                },
                {
                    "label": "owner",
                    "slot": "6",
                    "offset": 0,
                    "type_name": "t_address",
                    "zero_semantics": "never_written"
                },
                {
                    "label": "paused",
                    "slot": "6",
                    "offset": 20,
                    "type_name": "t_bool",
                    "zero_semantics": "explicitly_zero"
                },
                {
                    "label": "balanceOf",
                    "slot": "0",
                    "offset": 0,
                    "type_name": "t_mapping(t_address,t_uint256)",
                    "zero_semantics": "never_written"
                }
            ],
            "types": [
                {
                    "label": "t_address",
                    "number_of_bytes": "20",
                    "encoding": "inplace"
                },
                {
                    "label": "t_bool",
                    "number_of_bytes": "1",
                    "encoding": "inplace"
                },
                {
                    "label": "t_uint8",
                    "number_of_bytes": "1",
                    "encoding": "inplace"
                },
                {
                    "label": "t_uint256",
                    "number_of_bytes": "32",
                    "encoding": "inplace"
                },
                {
                    "label": "t_string_storage",
                    "number_of_bytes": "32",
                    "encoding": "dynamic_array"
                },
                {
                    "label": "t_mapping(t_address,t_uint256)",
                    "number_of_bytes": "32",
                    "encoding": "mapping",
                    "key": "t_address",
                    "value": "t_uint256"
                },
                {
                    "label": "t_mapping(t_address,t_mapping(t_address,t_uint256))",
                    "number_of_bytes": "32",
                    "encoding": "mapping",
                    "key": "t_address",
                    "value": "t_mapping(t_address,t_uint256)"
                }
            ]
        });

        let erc20_path = self.path("ethereum/erc20_layout.json");
        fs::write(&erc20_path, serde_json::to_string_pretty(&erc20_layout)?)?;
        self.ethereum_abis.insert("erc20".to_string(), erc20_path);

        // Complex DeFi contract layout with semantic metadata demonstrating all types
        let defi_layout = json!({
            "contract_name": "MockDeFi",
            "storage": [
                {
                    "label": "userInfo",
                    "slot": "0",
                    "offset": 0,
                    "type_name": "t_mapping(t_address,t_struct(UserInfo))",
                    "zero_semantics": "never_written"
                },
                {
                    "label": "poolInfo",
                    "slot": "1",
                    "offset": 0,
                    "type_name": "t_mapping(t_uint256,t_struct(PoolInfo))",
                    "zero_semantics": "never_written"
                },
                {
                    "label": "userBalances",
                    "slot": "2",
                    "offset": 0,
                    "type_name": "t_mapping(t_address,t_mapping(t_uint256,t_uint256))",
                    "zero_semantics": "valid_zero"
                },
                {
                    "label": "totalDeposits",
                    "slot": "3",
                    "offset": 0,
                    "type_name": "t_uint256",
                    "zero_semantics": "explicitly_zero"
                },
                {
                    "label": "emergencyStop",
                    "slot": "4",
                    "offset": 0,
                    "type_name": "t_bool",
                    "zero_semantics": "explicitly_zero"
                },
                {
                    "label": "previousBalance",
                    "slot": "5",
                    "offset": 0,
                    "type_name": "t_uint256",
                    "zero_semantics": "cleared"
                }
            ],
            "types": [
                {
                    "label": "t_address",
                    "number_of_bytes": "20",
                    "encoding": "inplace"
                },
                {
                    "label": "t_uint256",
                    "number_of_bytes": "32",
                    "encoding": "inplace"
                },
                {
                    "label": "t_bool",
                    "number_of_bytes": "1",
                    "encoding": "inplace"
                },
                {
                    "label": "t_struct(UserInfo)",
                    "number_of_bytes": "96",
                    "encoding": "inplace"
                },
                {
                    "label": "t_struct(PoolInfo)",
                    "number_of_bytes": "64",
                    "encoding": "inplace"
                },
                {
                    "label": "t_mapping(t_address,t_struct(UserInfo))",
                    "number_of_bytes": "32",
                    "encoding": "mapping",
                    "key": "t_address",
                    "value": "t_struct(UserInfo)"
                },
                {
                    "label": "t_mapping(t_uint256,t_struct(PoolInfo))",
                    "number_of_bytes": "32",
                    "encoding": "mapping",
                    "key": "t_uint256",
                    "value": "t_struct(PoolInfo)"
                },
                {
                    "label": "t_mapping(t_address,t_mapping(t_uint256,t_uint256))",
                    "number_of_bytes": "32",
                    "encoding": "mapping",
                    "key": "t_address",
                    "value": "t_mapping(t_uint256,t_uint256)"
                }
            ]
        });

        let defi_path = self.path("ethereum/defi_layout.json");
        fs::write(&defi_path, serde_json::to_string_pretty(&defi_layout)?)?;
        self.ethereum_abis.insert("defi".to_string(), defi_path);

        // Create semantic conflict test layout (declared never_written but actually written)
        let conflict_layout = json!({
            "contract_name": "ConflictTest",
            "storage": [
                {
                    "label": "conflictSlot",
                    "slot": "0",
                    "offset": 0,
                    "type_name": "t_uint256",
                    "zero_semantics": "never_written"
                },
                {
                    "label": "validSlot",
                    "slot": "1",
                    "offset": 0,
                    "type_name": "t_uint256",
                    "zero_semantics": "explicitly_zero"
                }
            ],
            "types": [
                {
                    "label": "t_uint256",
                    "number_of_bytes": "32",
                    "encoding": "inplace"
                }
            ]
        });

        let conflict_path = self.path("ethereum/conflict_layout.json");
        fs::write(
            &conflict_path,
            serde_json::to_string_pretty(&conflict_layout)?,
        )?;
        self.ethereum_abis
            .insert("conflict".to_string(), conflict_path);

        Ok(())
    }

    /// Create CosmWasm contract layout test fixtures with semantic metadata
    async fn create_cosmos_fixtures(&mut self) -> Result<()> {
        // CW20 Token Layout with semantic metadata
        let cw20_layout = json!({
            "contract_name": "MockCW20",
            "storage": [
                {
                    "label": "config",
                    "slot": "config",
                    "offset": 0,
                    "type_name": "t_cw20_config",
                    "zero_semantics": "never_written"
                },
                {
                    "label": "balances",
                    "slot": "balances",
                    "offset": 0,
                    "type_name": "t_map(t_addr,t_uint128)",
                    "zero_semantics": "never_written"
                },
                {
                    "label": "allowances",
                    "slot": "allowances",
                    "offset": 0,
                    "type_name": "t_map(t_addr,t_map(t_addr,t_uint128))",
                    "zero_semantics": "never_written"
                },
                {
                    "label": "total_supply",
                    "slot": "total_supply",
                    "offset": 0,
                    "type_name": "t_uint128",
                    "zero_semantics": "explicitly_zero"
                },
                {
                    "label": "minter",
                    "slot": "minter",
                    "offset": 0,
                    "type_name": "t_option(t_addr)",
                    "zero_semantics": "valid_zero"
                }
            ],
            "types": [
                {
                    "label": "t_addr",
                    "number_of_bytes": "32",
                    "encoding": "cosmwasm_addr"
                },
                {
                    "label": "t_uint128",
                    "number_of_bytes": "16",
                    "encoding": "inplace"
                },
                {
                    "label": "t_string",
                    "number_of_bytes": "32",
                    "encoding": "dynamic"
                },
                {
                    "label": "t_cw20_config",
                    "number_of_bytes": "128",
                    "encoding": "cosmwasm_item"
                },
                {
                    "label": "t_map(t_addr,t_uint128)",
                    "number_of_bytes": "32",
                    "encoding": "cosmwasm_map",
                    "key": "t_addr",
                    "value": "t_uint128"
                },
                {
                    "label": "t_map(t_addr,t_map(t_addr,t_uint128))",
                    "number_of_bytes": "32",
                    "encoding": "cosmwasm_map",
                    "key": "t_addr",
                    "value": "t_map(t_addr,t_uint128)"
                },
                {
                    "label": "t_option(t_addr)",
                    "number_of_bytes": "33",
                    "encoding": "option"
                }
            ]
        });

        let cw20_path = self.path("cosmos/cw20_layout.json");
        fs::write(&cw20_path, serde_json::to_string_pretty(&cw20_layout)?)?;
        self.cosmos_schemas.insert("cw20".to_string(), cw20_path);

        // CW721 NFT Layout with semantic metadata
        let cw721_layout = json!({
            "contract_name": "MockCW721",
            "storage": [
                {
                    "label": "contract_info",
                    "slot": "contract_info",
                    "offset": 0,
                    "type_name": "t_cw721_config",
                    "zero_semantics": "never_written"
                },
                {
                    "label": "tokens",
                    "slot": "tokens",
                    "offset": 0,
                    "type_name": "t_map(t_string,t_token_info)",
                    "zero_semantics": "never_written"
                },
                {
                    "label": "operators",
                    "slot": "operators",
                    "offset": 0,
                    "type_name": "t_map(t_addr,t_map(t_addr,t_bool))",
                    "zero_semantics": "never_written"
                },
                {
                    "label": "num_tokens",
                    "slot": "num_tokens",
                    "offset": 0,
                    "type_name": "t_uint64",
                    "zero_semantics": "explicitly_zero"
                }
            ],
            "types": [
                {
                    "label": "t_addr",
                    "number_of_bytes": "32",
                    "encoding": "cosmwasm_addr"
                },
                {
                    "label": "t_string",
                    "number_of_bytes": "32",
                    "encoding": "dynamic"
                },
                {
                    "label": "t_uint64",
                    "number_of_bytes": "8",
                    "encoding": "inplace"
                },
                {
                    "label": "t_bool",
                    "number_of_bytes": "1",
                    "encoding": "inplace"
                },
                {
                    "label": "t_cw721_config",
                    "number_of_bytes": "128",
                    "encoding": "cosmwasm_item"
                },
                {
                    "label": "t_token_info",
                    "number_of_bytes": "256",
                    "encoding": "cosmwasm_item"
                },
                {
                    "label": "t_map(t_string,t_token_info)",
                    "number_of_bytes": "32",
                    "encoding": "cosmwasm_map",
                    "key": "t_string",
                    "value": "t_token_info"
                },
                {
                    "label": "t_map(t_addr,t_map(t_addr,t_bool))",
                    "number_of_bytes": "32",
                    "encoding": "cosmwasm_map",
                    "key": "t_addr",
                    "value": "t_map(t_addr,t_bool)"
                }
            ]
        });

        let cw721_path = self.path("cosmos/cw721_layout.json");
        fs::write(&cw721_path, serde_json::to_string_pretty(&cw721_layout)?)?;
        self.cosmos_schemas.insert("cw721".to_string(), cw721_path);

        Ok(())
    }

    /// Create configuration files for batch testing with semantic metadata
    async fn create_config_fixtures(&mut self) -> Result<()> {
        // Batch configuration for mixed chains with semantic specifications
        let batch_config = json!({
            "contracts": [
                {
                    "file": self.ethereum_abis.get("erc20").unwrap(),
                    "chain": "ethereum",
                    "address": "0xA0b86a33E6Cc3b3c7bC8F1DCCF0e6a8F71c1c0123",
                    "queries": ["totalSupply", "name", "symbol"],
                    "zero_semantics": "never_written"
                },
                {
                    "file": self.cosmos_schemas.get("cw20").unwrap(),
                    "chain": "cosmos",
                    "address": "cosmos1contract123",
                    "queries": ["token_info", "config"],
                    "zero_semantics": "explicitly_zero"
                }
            ],
            "rpc_endpoints": {
                "ethereum": "https://mainnet.infura.io/v3/test",
                "cosmos": "https://rpc.cosmos.network"
            },
            "output": {
                "base_dir": self.path("outputs/batch"),
                "separate_chains": true,
                "include_metadata": true
            },
            "semantic_validation": {
                "enabled": true,
                "indexer_service": "mock",
                "event_depth": 1000
            }
        });

        let batch_config_path = self.path("configs/batch_config.json");
        fs::write(
            &batch_config_path,
            serde_json::to_string_pretty(&batch_config)?,
        )?;
        self.config_files
            .insert("batch".to_string(), batch_config_path);

        // Semantic validation configuration
        let semantic_config = json!({
            "semantic_specifications": [
                {
                    "contract": "0xA0b86a33E6Cc3b3c7bC8F1DCCF0e6a8F71c1c0123",
                    "slot": "0",
                    "declared_semantics": "never_written",
                    "override_semantics": "explicitly_zero"
                },
                {
                    "contract": "0xA0b86a33E6Cc3b3c7bC8F1DCCF0e6a8F71c1c0123",
                    "slot": "1",
                    "declared_semantics": "never_written",
                    "override_semantics": null
                }
            ],
            "validation_rules": {
                "strict_mode": true,
                "require_event_validation": false,
                "allow_semantic_overrides": true
            }
        });

        let semantic_config_path = self.path("configs/semantic_config.json");
        fs::write(
            &semantic_config_path,
            serde_json::to_string_pretty(&semantic_config)?,
        )?;
        self.config_files
            .insert("semantic".to_string(), semantic_config_path);

        Ok(())
    }

    /// Create query files for testing with semantic specifications
    async fn create_query_fixtures(&mut self) -> Result<()> {
        // Ethereum queries with semantic metadata
        let eth_queries = json!({
            "queries": [
                {
                    "query": "totalSupply",
                    "zero_semantics": "explicitly_zero"
                },
                {
                    "query": "balanceOf[0x742d35Cc6aB8B23c0532C65C6b555f09F9d40894]",
                    "zero_semantics": "never_written"
                },
                {
                    "query": "name",
                    "zero_semantics": "never_written"
                },
                {
                    "query": "symbol",
                    "zero_semantics": "never_written"
                },
                {
                    "query": "decimals",
                    "zero_semantics": "valid_zero"
                }
            ]
        });

        let eth_queries_path = self.path("queries/ethereum_queries.json");
        fs::write(
            &eth_queries_path,
            serde_json::to_string_pretty(&eth_queries)?,
        )?;
        self.query_files
            .insert("ethereum".to_string(), eth_queries_path);

        // Cosmos queries with semantic metadata
        let cosmos_queries = json!({
            "queries": [
                {
                    "query": "config",
                    "zero_semantics": "never_written"
                },
                {
                    "query": "token_info",
                    "zero_semantics": "never_written"
                },
                {
                    "query": "balance.cosmos1abc123",
                    "zero_semantics": "never_written"
                },
                {
                    "query": "all_accounts",
                    "zero_semantics": "never_written"
                }
            ]
        });

        let cosmos_queries_path = self.path("queries/cosmos_queries.json");
        fs::write(
            &cosmos_queries_path,
            serde_json::to_string_pretty(&cosmos_queries)?,
        )?;
        self.query_files
            .insert("cosmos".to_string(), cosmos_queries_path);

        // Conflict test queries
        let conflict_queries = json!({
            "queries": [
                {
                    "query": "conflictSlot",
                    "zero_semantics": "never_written",
                    "expected_conflict": true
                },
                {
                    "query": "validSlot",
                    "zero_semantics": "explicitly_zero",
                    "expected_conflict": false
                }
            ]
        });

        let conflict_queries_path = self.path("queries/conflict_queries.json");
        fs::write(
            &conflict_queries_path,
            serde_json::to_string_pretty(&conflict_queries)?,
        )?;
        self.query_files
            .insert("conflict".to_string(), conflict_queries_path);

        Ok(())
    }

    /// Create mock RPC responses for testing with semantic data
    async fn create_mock_responses(&mut self) -> Result<()> {
        // Mock Ethereum storage proof with semantic metadata
        self.mock_responses.insert(
            "eth_proof".to_string(),
            json!({
                "jsonrpc": "2.0",
                "id": 1,
                "result": {
                    "accountProof": ["0x123", "0x456"],
                    "balance": "0x0",
                    "codeHash": "0x789",
                    "nonce": "0x0",
                    "storageHash": "0xabc",
                    "storageProof": [{
                        "key": "0x0000000000000000000000000000000000000000000000000000000000000000",
                        "value": "0x152d02c7e14af6800000",
                        "proof": ["0xdef", "0x012"]
                    }]
                }
            }),
        );

        // Mock Cosmos query response with semantic metadata
        self.mock_responses.insert(
            "cosmos_query".to_string(),
            json!({
                "result": {
                    "response": {
                        "code": 0,
                        "value": "eyJ0b3RhbF9zdXBwbHkiOiIxMDAwMDAwMDAwIn0=", // base64 encoded
                        "proofOps": [{
                            "type": "iavl:v",
                            "key": "Y29uZmlnOg==",
                            "data": "proof_data_here"
                        }],
                        "height": "12345"
                    }
                }
            }),
        );

        // Mock semantic storage proof responses
        self.mock_responses.insert(
            "semantic_proof_never_written".to_string(),
            json!({
                "semantic_storage_proof": {
                    "key": "0x0000000000000000000000000000000000000000000000000000000000000000",
                    "value": "0x0000000000000000000000000000000000000000000000000000000000000000",
                    "proof": ["0x789", "0xabc"],
                    "semantics": {
                        "declared_semantics": "never_written",
                        "validated_semantics": null,
                        "semantic_source": "declared"
                    }
                }
            }),
        );

        self.mock_responses.insert(
            "semantic_proof_explicitly_zero".to_string(),
            json!({
                "semantic_storage_proof": {
                    "key": "0x0000000000000000000000000000000000000000000000000000000000000002",
                    "value": "0x0000000000000000000000000000000000000000000000000000000000000000",
                    "proof": ["0x123", "0x456"],
                    "semantics": {
                        "declared_semantics": "explicitly_zero",
                        "validated_semantics": "explicitly_zero",
                        "semantic_source": "validated"
                    }
                }
            }),
        );

        Ok(())
    }

    /// Create semantic test data for various zero meaning types
    async fn create_semantic_test_data(&mut self) -> Result<()> {
        // Test data for never_written semantics
        let never_written_data = json!({
            "description": "Storage slot that has never been written to",
            "test_cases": [
                {
                    "contract": "0x1234567890123456789012345678901234567890",
                    "slot": "0x0",
                    "expected_value": "0x0000000000000000000000000000000000000000000000000000000000000000",
                    "zero_semantics": "never_written",
                    "should_validate": true
                },
                {
                    "contract": "0x1234567890123456789012345678901234567890",
                    "slot": "0x1",
                    "expected_value": "0x0000000000000000000000000000000000000000000000000000000000000000",
                    "zero_semantics": "never_written",
                    "should_validate": true
                }
            ]
        });

        let never_written_path = self.path("semantic_tests/never_written.json");
        fs::write(
            &never_written_path,
            serde_json::to_string_pretty(&never_written_data)?,
        )?;
        self.semantic_test_data
            .insert("never_written".to_string(), never_written_data);

        // Test data for explicitly_zero semantics
        let explicitly_zero_data = json!({
            "description": "Storage slot that was intentionally set to zero",
            "test_cases": [
                {
                    "contract": "0x1234567890123456789012345678901234567890",
                    "slot": "0x2",
                    "expected_value": "0x0000000000000000000000000000000000000000000000000000000000000000",
                    "zero_semantics": "explicitly_zero",
                    "should_validate": true
                },
                {
                    "contract": "0x1234567890123456789012345678901234567890",
                    "slot": "0x3",
                    "expected_value": "0x0000000000000000000000000000000000000000000000000000000000000000",
                    "zero_semantics": "explicitly_zero",
                    "should_validate": true
                }
            ]
        });

        let explicitly_zero_path = self.path("semantic_tests/explicitly_zero.json");
        fs::write(
            &explicitly_zero_path,
            serde_json::to_string_pretty(&explicitly_zero_data)?,
        )?;
        self.semantic_test_data
            .insert("explicitly_zero".to_string(), explicitly_zero_data);

        // Test data for cleared semantics
        let cleared_data = json!({
            "description": "Storage slot that was previously non-zero but cleared",
            "test_cases": [
                {
                    "contract": "0x1234567890123456789012345678901234567890",
                    "slot": "0x4",
                    "expected_value": "0x0000000000000000000000000000000000000000000000000000000000000000",
                    "zero_semantics": "cleared",
                    "should_validate": true,
                    "previous_value": "0x152d02c7e14af6800000"
                },
                {
                    "contract": "0x1234567890123456789012345678901234567890",
                    "slot": "0x5",
                    "expected_value": "0x0000000000000000000000000000000000000000000000000000000000000000",
                    "zero_semantics": "cleared",
                    "should_validate": true,
                    "previous_value": "0x0000000000000000000000000000000000000000000000000000000000000001"
                }
            ]
        });

        let cleared_path = self.path("semantic_tests/cleared.json");
        fs::write(&cleared_path, serde_json::to_string_pretty(&cleared_data)?)?;
        self.semantic_test_data
            .insert("cleared".to_string(), cleared_data);

        // Test data for valid_zero semantics
        let valid_zero_data = json!({
            "description": "Storage slot where zero is a valid operational state",
            "test_cases": [
                {
                    "contract": "0x1234567890123456789012345678901234567890",
                    "slot": "0x6",
                    "expected_value": "0x0000000000000000000000000000000000000000000000000000000000000000",
                    "zero_semantics": "valid_zero",
                    "should_validate": true
                },
                {
                    "contract": "0x1234567890123456789012345678901234567890",
                    "slot": "0x7",
                    "expected_value": "0x0000000000000000000000000000000000000000000000000000000000000000",
                    "zero_semantics": "valid_zero",
                    "should_validate": true
                }
            ]
        });

        let valid_zero_path = self.path("semantic_tests/valid_zero.json");
        fs::write(
            &valid_zero_path,
            serde_json::to_string_pretty(&valid_zero_data)?,
        )?;
        self.semantic_test_data
            .insert("valid_zero".to_string(), valid_zero_data);

        // Test data for semantic conflicts
        let conflict_data = json!({
            "description": "Test cases for semantic conflicts between declared and validated semantics",
            "test_cases": [
                {
                    "contract": "0x1234567890123456789012345678901234567890",
                    "slot": "0x8",
                    "expected_value": "0x0000000000000000000000000000000000000000000000000000000000000000",
                    "declared_semantics": "never_written",
                    "validated_semantics": "explicitly_zero",
                    "should_conflict": true,
                    "description": "Declared never_written but indexer shows write events"
                },
                {
                    "contract": "0x1234567890123456789012345678901234567890",
                    "slot": "0x9",
                    "expected_value": "0x0000000000000000000000000000000000000000000000000000000000000000",
                    "declared_semantics": "never_written",
                    "validated_semantics": "cleared",
                    "should_conflict": true,
                    "description": "Declared never_written but indexer shows previous non-zero value"
                }
            ]
        });

        let conflict_path = self.path("semantic_tests/conflicts.json");
        fs::write(
            &conflict_path,
            serde_json::to_string_pretty(&conflict_data)?,
        )?;
        self.semantic_test_data
            .insert("conflicts".to_string(), conflict_data);

        Ok(())
    }

    /// Create mock event data for validation testing
    async fn create_mock_event_data(&mut self) -> Result<()> {
        // Mock event data for never_written validation
        let never_written_events = json!({
            "contract": "0x1234567890123456789012345678901234567890",
            "slot": "0x0",
            "events": [],
            "description": "No events found - confirms never_written semantics"
        });

        let never_written_events_path = self.path("mock_events/never_written.json");
        fs::write(
            &never_written_events_path,
            serde_json::to_string_pretty(&never_written_events)?,
        )?;
        self.mock_event_data
            .insert("never_written".to_string(), never_written_events);

        // Mock event data for explicitly_zero validation
        let explicitly_zero_events = json!({
            "contract": "0x1234567890123456789012345678901234567890",
            "slot": "0x2",
            "events": [
                {
                    "block_number": 18500000,
                    "transaction_hash": "0xabc123",
                    "event_type": "SSTORE",
                    "old_value": "0x0000000000000000000000000000000000000000000000000000000000000000",
                    "new_value": "0x0000000000000000000000000000000000000000000000000000000000000000"
                }
            ],
            "description": "Write event setting value to zero - confirms explicitly_zero semantics"
        });

        let explicitly_zero_events_path = self.path("mock_events/explicitly_zero.json");
        fs::write(
            &explicitly_zero_events_path,
            serde_json::to_string_pretty(&explicitly_zero_events)?,
        )?;
        self.mock_event_data
            .insert("explicitly_zero".to_string(), explicitly_zero_events);

        // Mock event data for cleared validation
        let cleared_events = json!({
            "contract": "0x1234567890123456789012345678901234567890",
            "slot": "0x4",
            "events": [
                {
                    "block_number": 18400000,
                    "transaction_hash": "0xdef456",
                    "event_type": "SSTORE",
                    "old_value": "0x0000000000000000000000000000000000000000000000000000000000000000",
                    "new_value": "0x152d02c7e14af6800000"
                },
                {
                    "block_number": 18500000,
                    "transaction_hash": "0x789abc",
                    "event_type": "SSTORE",
                    "old_value": "0x152d02c7e14af6800000",
                    "new_value": "0x0000000000000000000000000000000000000000000000000000000000000000"
                }
            ],
            "description": "First non-zero then zero - confirms cleared semantics"
        });

        let cleared_events_path = self.path("mock_events/cleared.json");
        fs::write(
            &cleared_events_path,
            serde_json::to_string_pretty(&cleared_events)?,
        )?;
        self.mock_event_data
            .insert("cleared".to_string(), cleared_events);

        // Mock event data for valid_zero validation
        let valid_zero_events = json!({
            "contract": "0x1234567890123456789012345678901234567890",
            "slot": "0x6",
            "events": [
                {
                    "block_number": 18500000,
                    "transaction_hash": "0x123def",
                    "event_type": "SSTORE",
                    "old_value": "0x0000000000000000000000000000000000000000000000000000000000000000",
                    "new_value": "0x0000000000000000000000000000000000000000000000000000000000000000"
                }
            ],
            "description": "Zero value write - confirms valid_zero semantics"
        });

        let valid_zero_events_path = self.path("mock_events/valid_zero.json");
        fs::write(
            &valid_zero_events_path,
            serde_json::to_string_pretty(&valid_zero_events)?,
        )?;
        self.mock_event_data
            .insert("valid_zero".to_string(), valid_zero_events);

        // Mock event data for semantic conflicts
        let conflict_events = json!({
            "contract": "0x1234567890123456789012345678901234567890",
            "slot": "0x8",
            "events": [
                {
                    "block_number": 18500000,
                    "transaction_hash": "0xconflict123",
                    "event_type": "SSTORE",
                    "old_value": "0x0000000000000000000000000000000000000000000000000000000000000000",
                    "new_value": "0x0000000000000000000000000000000000000000000000000000000000000000"
                }
            ],
            "description": "Write event found but declared never_written - semantic conflict"
        });

        let conflict_events_path = self.path("mock_events/conflicts.json");
        fs::write(
            &conflict_events_path,
            serde_json::to_string_pretty(&conflict_events)?,
        )?;
        self.mock_event_data
            .insert("conflicts".to_string(), conflict_events);

        Ok(())
    }
}
