//! Test fixtures for end-to-end CLI testing
//!
//! This module provides sample contract files, configurations, and test data
//! needed to comprehensively test all CLI functionality.

use anyhow::Result;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// Test fixtures containing all necessary test data
pub struct TestFixtures {
    /// Temporary directory for test files
    pub temp_dir: TempDir,
    /// Sample Ethereum ABI files
    pub ethereum_abis: HashMap<String, PathBuf>,
    /// Sample CosmWasm message schemas
    pub cosmos_schemas: HashMap<String, PathBuf>,
    /// Configuration files for batch operations
    pub config_files: HashMap<String, PathBuf>,
    /// Query files for testing
    pub query_files: HashMap<String, PathBuf>,
    /// Mock RPC responses
    pub mock_responses: HashMap<String, Value>,
}

impl TestFixtures {
    /// Create new test fixtures with all necessary test data
    pub async fn new() -> Result<Self> {
        let temp_dir = TempDir::new()?;
        let base_path = temp_dir.path();
        
        // Create subdirectories
        fs::create_dir_all(base_path.join("ethereum"))?;
        fs::create_dir_all(base_path.join("cosmos"))?;
        fs::create_dir_all(base_path.join("configs"))?;
        fs::create_dir_all(base_path.join("queries"))?;
        fs::create_dir_all(base_path.join("outputs"))?;
        
        let mut fixtures = Self {
            temp_dir,
            ethereum_abis: HashMap::new(),
            cosmos_schemas: HashMap::new(),
            config_files: HashMap::new(),
            query_files: HashMap::new(),
            mock_responses: HashMap::new(),
        };
        
        // Create test files
        fixtures.create_ethereum_fixtures().await?;
        fixtures.create_cosmos_fixtures().await?;
        fixtures.create_config_fixtures().await?;
        fixtures.create_query_fixtures().await?;
        fixtures.create_mock_responses().await?;
        
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
    
    /// Create Ethereum layout test fixtures (using canonical LayoutInfo format)
    async fn create_ethereum_fixtures(&mut self) -> Result<()> {
        // ERC20 Token Layout (canonical format)
        let erc20_layout = json!({
            "contract_name": "MockERC20",
            "storage": [
                {
                    "label": "_balances",
                    "slot": "0",
                    "offset": 0,
                    "type_name": "t_mapping(t_address,t_uint256)"
                },
                {
                    "label": "_allowances", 
                    "slot": "1",
                    "offset": 0,
                    "type_name": "t_mapping(t_address,t_mapping(t_address,t_uint256))"
                },
                {
                    "label": "_totalSupply",
                    "slot": "2", 
                    "offset": 0,
                    "type_name": "t_uint256"
                },
                {
                    "label": "totalSupply",
                    "slot": "2", 
                    "offset": 0,
                    "type_name": "t_uint256"
                },
                {
                    "label": "_name",
                    "slot": "3",
                    "offset": 0,
                    "type_name": "t_string_storage"
                },
                {
                    "label": "name",
                    "slot": "3",
                    "offset": 0,
                    "type_name": "t_string_storage"
                },
                {
                    "label": "_symbol",
                    "slot": "4",
                    "offset": 0,
                    "type_name": "t_string_storage"
                },
                {
                    "label": "symbol",
                    "slot": "4",
                    "offset": 0,
                    "type_name": "t_string_storage"
                },
                {
                    "label": "_decimals",
                    "slot": "5",
                    "offset": 0,
                    "type_name": "t_uint8"
                },
                {
                    "label": "decimals",
                    "slot": "5",
                    "offset": 0,
                    "type_name": "t_uint8"
                },
                {
                    "label": "owner",
                    "slot": "6",
                    "offset": 0,
                    "type_name": "t_address"
                },
                {
                    "label": "paused",
                    "slot": "6",
                    "offset": 20,
                    "type_name": "t_bool"
                },
                {
                    "label": "balanceOf",
                    "slot": "0",
                    "offset": 0,
                    "type_name": "t_mapping(t_address,t_uint256)"
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
        
        // Complex DeFi contract layout with mappings and structs
        let defi_layout = json!({
            "contract_name": "MockDeFi",
            "storage": [
                {
                    "label": "userInfo",
                    "slot": "0",
                    "offset": 0,
                    "type_name": "t_mapping(t_address,t_struct(UserInfo))"
                },
                {
                    "label": "poolInfo",
                    "slot": "1",
                    "offset": 0,
                    "type_name": "t_mapping(t_uint256,t_struct(PoolInfo))"
                },
                {
                    "label": "userBalances",
                    "slot": "2",
                    "offset": 0,
                    "type_name": "t_mapping(t_address,t_mapping(t_uint256,t_uint256))"
                },
                {
                    "label": "totalDeposits",
                    "slot": "3",
                    "offset": 0,
                    "type_name": "t_uint256"
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
        
        Ok(())
    }
    
    /// Create CosmWasm contract layout test fixtures (using canonical LayoutInfo format)
    async fn create_cosmos_fixtures(&mut self) -> Result<()> {
        // CW20 Token Layout
        let cw20_layout = json!({
            "contract_name": "MockCW20",
            "storage": [
                {
                    "label": "config",
                    "slot": "config",
                    "offset": 0,
                    "type_name": "t_cw20_config"
                },
                {
                    "label": "balances",
                    "slot": "balances",
                    "offset": 0,
                    "type_name": "t_map(t_addr,t_uint128)"
                },
                {
                    "label": "allowances",
                    "slot": "allowances",
                    "offset": 0,
                    "type_name": "t_map(t_addr,t_map(t_addr,t_uint128))"
                },
                {
                    "label": "total_supply",
                    "slot": "total_supply",
                    "offset": 0,
                    "type_name": "t_uint128"
                },
                {
                    "label": "minter",
                    "slot": "minter",
                    "offset": 0,
                    "type_name": "t_option(t_addr)"
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
        
        // CW721 NFT Layout
        let cw721_layout = json!({
            "contract_name": "MockCW721",
            "storage": [
                {
                    "label": "contract_info",
                    "slot": "contract_info",
                    "offset": 0,
                    "type_name": "t_cw721_config"
                },
                {
                    "label": "tokens",
                    "slot": "tokens",
                    "offset": 0,
                    "type_name": "t_map(t_string,t_token_info)"
                },
                {
                    "label": "operators",
                    "slot": "operators",
                    "offset": 0,
                    "type_name": "t_map(t_addr,t_map(t_addr,t_bool))"
                },
                {
                    "label": "num_tokens",
                    "slot": "num_tokens",
                    "offset": 0,
                    "type_name": "t_uint64"
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
    
    /// Create configuration files for batch testing
    async fn create_config_fixtures(&mut self) -> Result<()> {
        // Batch configuration for mixed chains
        let batch_config = json!({
            "contracts": [
                {
                    "file": self.ethereum_abis.get("erc20").unwrap(),
                    "chain": "ethereum",
                    "address": "0xA0b86a33E6Cc3b3c7bC8F1DCCF0e6a8F71c1c0123",
                    "queries": ["totalSupply", "name", "symbol"]
                },
                {
                    "file": self.cosmos_schemas.get("cw20").unwrap(),
                    "chain": "cosmos", 
                    "address": "cosmos1contract123",
                    "queries": ["token_info", "config"]
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
            }
        });
        
        let batch_config_path = self.path("configs/batch_config.toml");
        fs::write(&batch_config_path, toml::to_string_pretty(&batch_config)?)?;
        self.config_files.insert("batch".to_string(), batch_config_path);
        
        // Watch configuration
        let watch_config = json!({
            "watch_patterns": ["*.abi.json", "*_msg.json"],
            "output_dir": self.path("outputs/watch"),
            "webhook_url": "http://localhost:3000/proof-ready",
            "debounce_ms": 1000
        });
        
        let watch_config_path = self.path("configs/watch_config.toml");
        fs::write(&watch_config_path, toml::to_string_pretty(&watch_config)?)?;
        self.config_files.insert("watch".to_string(), watch_config_path);
        
        Ok(())
    }
    
    /// Create query files for testing
    async fn create_query_fixtures(&mut self) -> Result<()> {
        // Ethereum queries YAML
        let eth_queries = json!({
            "queries": [
                "totalSupply",
                "balanceOf[0x742d35Cc6aB8B23c0532C65C6b555f09F9d40894]",
                "name",
                "symbol",
                "decimals"
            ]
        });
        
        let eth_queries_path = self.path("queries/ethereum_queries.yaml");
        fs::write(&eth_queries_path, serde_yaml::to_string(&eth_queries)?)?;
        self.query_files.insert("ethereum".to_string(), eth_queries_path);
        
        // Cosmos queries TOML
        let cosmos_queries_toml = r#"
queries = [
    "config",
    "token_info", 
    "balance.cosmos1abc123",
    "all_accounts"
]
"#;
        
        let cosmos_queries_path = self.path("queries/cosmos_queries.toml");
        fs::write(&cosmos_queries_path, cosmos_queries_toml)?;
        self.query_files.insert("cosmos".to_string(), cosmos_queries_path);
        
        // Simple text queries
        let simple_queries = "totalSupply\nname\nsymbol\ndecimals\n";
        let simple_queries_path = self.path("queries/simple_queries.txt");
        fs::write(&simple_queries_path, simple_queries)?;
        self.query_files.insert("simple".to_string(), simple_queries_path);
        
        Ok(())
    }
    
    /// Create mock RPC responses for testing
    async fn create_mock_responses(&mut self) -> Result<()> {
        // Mock Ethereum storage proof
        self.mock_responses.insert("eth_proof".to_string(), json!({
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
        }));
        
        // Mock Cosmos query response
        self.mock_responses.insert("cosmos_query".to_string(), json!({
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
        }));
        
        Ok(())
    }
} 