//! Unified commands that work across both Ethereum and Cosmos chains
//! 
//! This module provides high-level commands that can automatically detect chain type
//! and orchestrate end-to-end workflows for both Ethereum and Cosmos.

use anyhow::Result;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use std::fs;

use crate::commands::ethereum::cmd_ethereum_auto_generate;
use crate::commands::cosmos::cmd_cosmos_auto_generate;

/// Chain type detected from contract file
#[derive(Debug, Clone, PartialEq)]
pub enum ChainType {
    Ethereum,
    Cosmos,
}

/// Configuration structure for batch processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchConfig {
    /// Contracts to process
    pub contracts: Vec<ContractConfig>,
    /// Default RPC endpoints
    pub rpc_endpoints: RpcEndpoints,
    /// Output configuration
    pub output: OutputConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractConfig {
    /// Path to contract file
    pub file: PathBuf,
    /// Chain type (optional, will auto-detect if not specified)
    pub chain: Option<String>,
    /// Contract address
    pub address: String,
    /// Queries to generate
    pub queries: Vec<String>,
    /// Chain-specific RPC override
    pub rpc: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcEndpoints {
    pub ethereum: Option<String>,
    pub cosmos: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    /// Base output directory
    pub base_dir: PathBuf,
    /// Whether to create separate subdirectories per chain
    pub separate_chains: bool,
    /// Whether to include metadata files
    pub include_metadata: bool,
}

/// Auto-generate everything from contract files with chain detection
pub async fn cmd_auto_generate(
    contract_file: &Path,
    rpc_ethereum: Option<&str>,
    rpc_cosmos: Option<&str>,
    contract_ethereum: Option<&str>,
    contract_cosmos: Option<&str>,
    queries_file: Option<&Path>,
    output_dir: &Path,
    dry_run: bool,
) -> Result<()> {
    println!("🚀 Starting unified auto-generation...");
    
    // Auto-detect chain type
    let chain_type = detect_chain_type(contract_file)?;
    let chain_name = match chain_type {
        ChainType::Ethereum => "ethereum",
        ChainType::Cosmos => "cosmos",
    };
    println!("🔍 Detected chain type: {}", chain_name);
    
    // Load queries if specified
    let queries = if let Some(queries_path) = queries_file {
        load_queries_from_file(queries_path)?
    } else {
        // Use default queries based on chain type
        get_default_queries(&chain_type)
    };
    
    // Create output directory
    fs::create_dir_all(output_dir)?;
    
    // In dry-run mode, create placeholder files for testing
    if dry_run {
        println!("   📋 Creating placeholder files for testing...");
        
        // Create basic layout file
        let layout_placeholder = serde_json::json!({
            "contract_name": "placeholder",
            "generated_at": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            "dry_run": true
        });
        fs::write(output_dir.join("layout.json"), serde_json::to_string_pretty(&layout_placeholder)?)?;
        
        // Create basic queries file
        let queries_placeholder = serde_json::json!({
            "queries": queries,
            "generated_at": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            "dry_run": true
        });
        fs::write(output_dir.join("queries.json"), serde_json::to_string_pretty(&queries_placeholder)?)?;
        
        // Create resolved queries file
        let resolved_placeholder = serde_json::json!({
            "resolved_queries": [],
            "generated_at": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            "dry_run": true
        });
        fs::write(output_dir.join("resolved_queries.json"), serde_json::to_string_pretty(&resolved_placeholder)?)?;
        
        println!("   ✅ Placeholder files created for testing");
    }
    
    match chain_type {
        ChainType::Ethereum => {
            let rpc = rpc_ethereum.ok_or_else(|| anyhow::anyhow!("Ethereum RPC endpoint required"))?;
            let contract = contract_ethereum.ok_or_else(|| anyhow::anyhow!("Ethereum contract address required"))?;
            
            println!("📈 Processing Ethereum contract...");
            cmd_ethereum_auto_generate(
                contract_file,
                rpc,
                contract,
                &queries.join(","),
                output_dir,
                true, // enable caching
                dry_run,
            ).await
        }
        ChainType::Cosmos => {
            let rpc = rpc_cosmos.ok_or_else(|| anyhow::anyhow!("Cosmos RPC endpoint required"))?;
            let contract = contract_cosmos.ok_or_else(|| anyhow::anyhow!("Cosmos contract address required"))?;
            
            println!("🌌 Processing Cosmos contract...");
            cmd_cosmos_auto_generate(
                contract_file,
                rpc,
                contract,
                &queries.join(","),
                output_dir,
                dry_run,
            )
        }
    }
}

/// Batch generate proofs for multiple contracts
pub async fn cmd_batch_generate(
    config_file: &Path,
    parallel: usize,
    output_dir: &Path,
    dry_run: bool,
) -> Result<()> {
    println!("🔄 Starting batch generation...");
    
    // Load configuration
    let config_content = fs::read_to_string(config_file)?;
    let config: BatchConfig = toml::from_str(&config_content)?;
    
    // Create base output directory
    fs::create_dir_all(output_dir)?;
    
    println!("📊 Processing {} contracts with {} parallel workers", 
             config.contracts.len(), parallel);
    
    // Process contracts in batches
    let chunk_size = (config.contracts.len() + parallel - 1) / parallel;
    let mut futures = Vec::new();
    
    for (batch_idx, chunk) in config.contracts.chunks(chunk_size).enumerate() {
        let chunk = chunk.to_vec();
        let config_clone = config.clone();
        let output_dir = output_dir.to_owned();
        
        let future = async move {
            println!("⚡ Starting batch {} with {} contracts", batch_idx, chunk.len());
            
            for contract in chunk {
                let result = process_single_contract(&contract, &config_clone, &output_dir, dry_run).await;
                if let Err(e) = result {
                    eprintln!("❌ Error processing {}: {}", contract.file.display(), e);
                } else {
                    println!("✅ Completed {}", contract.file.display());
                }
            }
        };
        
        futures.push(future);
    }
    
    // Wait for all batches to complete
    futures::future::join_all(futures).await;
    
    println!("🎉 Batch generation completed!");
    Ok(())
}

/// Watch mode for continuous proof generation
pub async fn cmd_watch(
    watch_dir: &Path,
    config_file: &Path,
    webhook: Option<&str>,
) -> Result<()> {
    println!("👀 Starting watch mode on {}", watch_dir.display());
    
    // For now, implement a simple polling-based watcher
    // In a full implementation, this would use filesystem events
    println!("⚠️  Watch mode is not yet implemented - would monitor {} for changes", watch_dir.display());
    
    if let Some(webhook_url) = webhook {
        println!("🔗 Would send notifications to: {}", webhook_url);
    }
    
    // Load configuration
    let _config_content = fs::read_to_string(config_file)?;
    
    println!("💡 This would continuously monitor contract files and auto-generate proofs");
    println!("💡 Implementation needed: filesystem watcher + event processing");
    
    Ok(())
}

/// Detect chain type from contract file extension and content
fn detect_chain_type(contract_file: &Path) -> Result<ChainType> {
    let file_name = contract_file.file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| anyhow::anyhow!("Invalid file name"))?;
    
    // Check file extension patterns
    if file_name.ends_with(".abi.json") || file_name.contains("abi") {
        return Ok(ChainType::Ethereum);
    }
    
    if file_name.ends_with("_msg.json") || file_name.contains("msg") {
        return Ok(ChainType::Cosmos);
    }
    
    // Check file content for more sophisticated detection
    let content = fs::read_to_string(contract_file)?;
    let json_value: serde_json::Value = serde_json::from_str(&content)?;
    
    // Ethereum ABI files typically have an array with "type" fields
    if let Some(array) = json_value.as_array() {
        if array.iter().any(|item| {
            item.get("type").and_then(|t| t.as_str()).map_or(false, |s| {
                s == "function" || s == "event" || s == "constructor"
            })
        }) {
            return Ok(ChainType::Ethereum);
        }
    }
    
    // CosmWasm message files typically have specific schema structures
    if json_value.get("$schema").is_some() || 
       json_value.get("title").is_some() ||
       json_value.get("properties").is_some() {
        return Ok(ChainType::Cosmos);
    }
    
    // Default to Ethereum if we can't determine
    println!("⚠️  Could not auto-detect chain type, defaulting to Ethereum");
    Ok(ChainType::Ethereum)
}

/// Load queries from a YAML file
fn load_queries_from_file(queries_file: &Path) -> Result<Vec<String>> {
    let content = fs::read_to_string(queries_file)?;
    
    // Try YAML first
    if let Ok(yaml_value) = serde_yaml::from_str::<serde_yaml::Value>(&content) {
        if let Some(queries) = yaml_value.get("queries") {
            if let Some(queries_array) = queries.as_sequence() {
                return Ok(queries_array.iter()
                    .filter_map(|q| q.as_str().map(|s| s.to_string()))
                    .collect());
            }
        }
    }
    
    // Try TOML
    if let Ok(toml_value) = toml::from_str::<toml::Value>(&content) {
        if let Some(queries) = toml_value.get("queries") {
            if let Some(queries_array) = queries.as_array() {
                return Ok(queries_array.iter()
                    .filter_map(|q| q.as_str().map(|s| s.to_string()))
                    .collect());
            }
        }
    }
    
    // Fall back to line-by-line text file
    Ok(content.lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| line.to_string())
        .collect())
}

/// Get default queries based on chain type
fn get_default_queries(chain_type: &ChainType) -> Vec<String> {
    match chain_type {
        ChainType::Ethereum => vec![
            "totalSupply".to_string(),
            "name".to_string(),
            "symbol".to_string(),
            "decimals".to_string(),
        ],
        ChainType::Cosmos => vec![
            "config".to_string(),
            "total_supply".to_string(),
        ],
    }
}

/// Process a single contract in batch mode
async fn process_single_contract(
    contract: &ContractConfig,
    config: &BatchConfig,
    base_output_dir: &Path,
    dry_run: bool,
) -> Result<()> {
    // Determine chain type
    let chain_type = if let Some(chain_str) = &contract.chain {
        match chain_str.to_lowercase().as_str() {
            "ethereum" | "eth" => ChainType::Ethereum,
            "cosmos" | "cosm" => ChainType::Cosmos,
            _ => detect_chain_type(&contract.file)?,
        }
    } else {
        detect_chain_type(&contract.file)?
    };
    
    // Create output directory for this contract
    let contract_name = contract.file.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown");
    
    let output_dir = if config.output.separate_chains {
        base_output_dir.join(format!("{:?}", chain_type).to_lowercase()).join(contract_name)
    } else {
        base_output_dir.join(contract_name)
    };
    
    fs::create_dir_all(&output_dir)?;
    
    // Get RPC endpoint
    let rpc = contract.rpc.as_deref().or_else(|| {
        match chain_type {
            ChainType::Ethereum => config.rpc_endpoints.ethereum.as_deref(),
            ChainType::Cosmos => config.rpc_endpoints.cosmos.as_deref(),
        }
    }).ok_or_else(|| anyhow::anyhow!("No RPC endpoint configured for {:?}", chain_type))?;
    
    // Process based on chain type
    match chain_type {
        ChainType::Ethereum => {
            cmd_ethereum_auto_generate(
                &contract.file,
                rpc,
                &contract.address,
                &contract.queries.join(","),
                &output_dir,
                true, // enable caching
                dry_run,
            ).await
        }
        ChainType::Cosmos => {
            cmd_cosmos_auto_generate(
                &contract.file,
                rpc,
                &contract.address,
                &contract.queries.join(","),
                &output_dir,
                dry_run,
            )
        }
    }
} 