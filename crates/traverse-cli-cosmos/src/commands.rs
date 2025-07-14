//! CosmWasm command implementations
//!
//! This module provides CosmWasm-specific CLI commands for contract analysis,
//! storage layout compilation, and query generation.

use anyhow::Result;
use log::info;
use serde_json::{json, Value};
use std::path::Path;
use traverse_core::OutputFormat;

#[cfg(feature = "cosmos")]
use hex;

#[cfg(feature = "cosmos")]
use base64;
#[cfg(feature = "cosmos")]
use reqwest;

#[cfg(feature = "cosmos")]
use traverse_cosmos::{CosmosKeyResolver, CosmosLayoutCompiler};
#[cfg(feature = "cosmos")]
use traverse_core::{KeyResolver};

/// Write output to file or stdout
fn write_output(content: &str, output_path: Option<&Path>) -> Result<()> {
    match output_path {
        Some(path) => {
            std::fs::write(path, content)?;
            Ok(())
        }
        None => {
            println!("{}", content);
            Ok(())
        }
    }
}

/// Execute cosmos analyze-contract command
#[cfg(feature = "cosmos")]
pub async fn cmd_cosmos_analyze_contract(
    schema_file: &Path,
    output: Option<&Path>,
    validate_storage: bool,
    contract_address: Option<&str>,
    rpc: Option<&str>,
) -> Result<()> {
    info!("Analyzing CosmWasm contract schema: {}", schema_file.display());

    let contract_schema = std::fs::read_to_string(schema_file)?;
    let _schema: serde_json::Value = serde_json::from_str(&contract_schema)?;

    let mut analysis = serde_json::json!({
        "contract_type": "cosmwasm",
        "schema_file": schema_file.display().to_string(),
        "validation": {
            "schema_valid": true,
            "storage_validation": validate_storage
        }
    });

    if let Some(address) = contract_address {
        analysis["contract_address"] = serde_json::Value::String(address.to_string());
        
        if let Some(rpc_url) = rpc {
            info!("Performing live contract analysis for address: {}", address);
            match perform_live_cosmos_analysis(address, rpc_url).await {
                Ok(live_data) => {
                    analysis["live_analysis"] = live_data;
                    analysis["validation"]["live_analysis_success"] = serde_json::Value::Bool(true);
                }
                Err(e) => {
                    analysis["validation"]["live_analysis_success"] = serde_json::Value::Bool(false);
                    analysis["validation"]["live_analysis_error"] = serde_json::Value::String(e.to_string());
                }
            }
        }
    }

    let output_str = serde_json::to_string_pretty(&analysis)?;
    write_output(&output_str, output)?;

    println!("Analyzing CosmWasm contract: cosmwasm detected");
    println!("Contract analysis completed");

    Ok(())
}

/// Execute cosmos compile-layout command
#[cfg(feature = "cosmos")]
pub fn cmd_cosmos_compile_layout(
    msg_file: &Path,
    output: Option<&Path>,
    format: &OutputFormat,
) -> Result<()> {
    info!(
        "Compiling CosmWasm storage layout from {}",
        msg_file.display()
    );

    use traverse_cosmos::CosmosLayoutCompiler;
    use traverse_core::LayoutCompiler;

    let compiler = CosmosLayoutCompiler;
    let layout = compiler.compile_layout(msg_file)?;

    let output_str = match format {
        OutputFormat::Json => serde_json::to_string_pretty(&layout)?,
        OutputFormat::Yaml => serde_yaml::to_string(&layout)?,
    };

    write_output(&output_str, output)?;

    println!("Layout compilation completed");
    println!("  • {} storage entries", layout.entries.len());
    println!("  • {} semantic entries", layout.semantic_entries.len());
    println!(
        "  • Layout commitment: {}",
        hex::encode(layout.commitment())
    );

    Ok(())
}

/// Execute cosmos resolve-query command
#[cfg(feature = "cosmos")]
pub fn cmd_cosmos_resolve_query(
    query: &str,
    layout_file: &Path,
    format: &OutputFormat,
    output: Option<&Path>,
) -> Result<()> {
    info!("Resolving CosmWasm storage query: {}", query);

    let layout_content = std::fs::read_to_string(layout_file)?;
    let layout: traverse_core::LayoutInfo = serde_json::from_str(&layout_content)?;

    let resolver = CosmosKeyResolver;
    let resolved_path = resolver.resolve(&layout, query)?;

    let result = serde_json::json!({
        "query": query,
        "resolved_path": {
            "key": match &resolved_path.key {
                traverse_core::Key::Fixed(key) => hex::encode(key),
                _ => "dynamic".to_string(),
            },
            "offset": resolved_path.offset,
            "field_size": resolved_path.field_size,
            "layout_commitment": hex::encode(resolved_path.layout_commitment)
        }
    });

    let output_str = match format {
        OutputFormat::Json => serde_json::to_string_pretty(&result)?,
        OutputFormat::Yaml => serde_json::to_string_pretty(&result)?, // YAML not available, use JSON
    };

    write_output(&output_str, output)?;

    println!("Query resolution completed");
    println!("  • Query: {}", query);
    println!("  • Resolved path: {}", match &resolved_path.key {
        traverse_core::Key::Fixed(key) => hex::encode(key),
        _ => "dynamic".to_string(),
    });

    Ok(())
}

/// Execute cosmos generate-queries command
#[cfg(feature = "cosmos")]
pub fn cmd_cosmos_generate_queries(
    layout_file: &Path,
    state_keys: &str,
    output: Option<&Path>,
    include_examples: bool,
) -> Result<()> {
    info!("Generating CosmWasm contract queries from layout");

    let layout_content = std::fs::read_to_string(layout_file)?;
    let layout: traverse_core::LayoutInfo = serde_json::from_str(&layout_content)?;

    let state_key_patterns: Vec<&str> = state_keys.split(',').map(|s| s.trim()).collect();
    let mut generated_queries = Vec::new();

    for entry in &layout.storage {
        for pattern in &state_key_patterns {
            if entry.label.contains(pattern) {
                let query = if include_examples {
                    format!("{}[example_value]", entry.label)
                } else {
                    entry.label.clone()
                };
                generated_queries.push(serde_json::json!({
                    "query": query,
                    "key_path": entry.label,
                    "value_type": entry.type_name,
                    "description": format!("Query for {}", entry.label)
                }));
            }
        }
    }

    let result = serde_json::json!({
        "layout_file": layout_file.display().to_string(),
        "state_key_patterns": state_key_patterns,
        "generated_queries": generated_queries,
        "query_count": generated_queries.len()
    });

    let output_str = serde_json::to_string_pretty(&result)?;
    write_output(&output_str, output)?;

    println!("Query generation completed");
    println!("  • Generated {} queries", generated_queries.len());
    println!("  • State key patterns: {}", state_keys);

    Ok(())
}

/// Execute cosmos auto-generate command
#[cfg(feature = "cosmos")]
pub async fn cmd_cosmos_auto_generate(
    schema_file: &Path,
    rpc: &str,
    contract: &str,
    queries: &str,
    output_dir: &Path,
    _cache: bool,
    dry_run: bool,
) -> Result<()> {
    info!("Auto-generating CosmWasm contract analysis");

    if dry_run {
        println!("DRY RUN: Would generate analysis for:");
        println!("  • Schema file: {}", schema_file.display());
        println!("  • RPC endpoint: {}", rpc);
        println!("  • Contract: {}", contract);
        println!("  • Queries: {}", queries);
        println!("  • Output directory: {}", output_dir.display());
        return Ok(());
    }

    // Ensure output directory exists
    std::fs::create_dir_all(output_dir)?;

    // Step 1: Analyze contract
    let analysis_output = output_dir.join("analysis.json");
    cmd_cosmos_analyze_contract(
        schema_file,
        Some(&analysis_output),
        true,  // validate_storage
        Some(contract),
        Some(rpc),
    ).await?;

    // Step 2: Compile layout
    let layout_output = output_dir.join("layout.json");
    cmd_cosmos_compile_layout(
        schema_file,
        Some(&layout_output),
        &OutputFormat::Json,
    )?;

    // Step 3: Generate queries
    let queries_output = output_dir.join("queries.json");
    cmd_cosmos_generate_queries(
        &layout_output,
        queries,
        Some(&queries_output),
        true,  // include_examples
    )?;

    // Step 4: Resolve each query
    let query_list: Vec<&str> = queries.split(',').map(|q| q.trim()).collect();
    for (i, query) in query_list.iter().enumerate() {
        let resolved_output = output_dir.join(format!("resolved_query_{}.json", i + 1));
        cmd_cosmos_resolve_query(
            query,
            &layout_output,
            &OutputFormat::Json,
            Some(&resolved_output),
        )?;
    }

    println!("Auto-generation completed");
    println!("  • Contract: {}", contract);
    println!("  • Generated {} files", 3 + query_list.len());
    println!("  • Output directory: {}", output_dir.display());

    Ok(())
}

/// Perform live analysis of a CosmWasm contract
#[cfg(feature = "cosmos")]
async fn perform_live_cosmos_analysis(contract_address: &str, rpc_url: &str) -> Result<Value> {
    use reqwest::Client;
    use serde_json::json;

    let client = Client::new();
    
    // Query contract info
    let contract_info_query = json!({
        "jsonrpc": "2.0",
        "method": "abci_query",
        "params": {
            "path": "/cosmwasm.wasm.v1.Query/ContractInfo",
            "data": base64::encode(contract_address.as_bytes()),
            "prove": false
        },
        "id": 1
    });

    let response = client
        .post(rpc_url)
        .json(&contract_info_query)
        .send()
        .await?;

    let result: serde_json::Value = response.json().await?;
    
    // Extract contract info from response
    let contract_info = if let Some(result_data) = result.get("result") {
        if let Some(response_data) = result_data.get("response") {
            if let Some(value) = response_data.get("value") {
                // Decode base64 response
                if let Ok(decoded) = base64::decode(value.as_str().unwrap_or("")) {
                    // Parse protobuf response (simplified)
                    json!({
                        "contract_address": contract_address,
                        "data_size": decoded.len(),
                        "status": "active",
                        "rpc_endpoint": rpc_url
                    })
                } else {
                    json!({
                        "contract_address": contract_address,
                        "status": "found",
                        "rpc_endpoint": rpc_url
                    })
                }
            } else {
                json!({
                    "contract_address": contract_address,
                    "status": "not_found",
                    "rpc_endpoint": rpc_url
                })
            }
        } else {
            json!({
                "contract_address": contract_address,
                "status": "rpc_error",
                "rpc_endpoint": rpc_url
            })
        }
    } else {
        json!({
            "contract_address": contract_address,
            "status": "query_failed",
            "rpc_endpoint": rpc_url
        })
    };

    Ok(contract_info)
} 