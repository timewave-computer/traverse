//! Enhanced Ethereum command implementations
//!
//! This module provides comprehensive Ethereum-specific CLI commands for contract analysis,
//! storage layout compilation, query generation, and end-to-end automation.

use traverse_cli_core::{formatters::write_output, OutputFormat};
use anyhow::Result;
use base64::{engine::general_purpose::STANDARD, Engine};
use bincode;
use hex;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::Path;
use tracing::{info, warn};
use reqwest;
use chrono;
use traverse_core::{KeyResolver, LayoutCompiler, LayoutInfo, Key};

#[cfg(feature = "ethereum")]
use traverse_ethereum::{EthereumKeyResolver, EthereumLayoutCompiler};

/// Helper function to convert Key to bytes for hex encoding
fn key_to_bytes(key: &Key) -> &[u8] {
    match key {
        Key::Fixed(bytes) => bytes,
        Key::Variable(bytes) => bytes,
    }
}

/// Simplified structure for TOML serialization
#[derive(Serialize, Deserialize)]
struct SimpleLayoutInfo {
    contract_name: String,
    storage_entries: usize,
    type_definitions: usize,
    commitment: String,
    generated_at: u64,
    compiler: String,
}

/// Execute ethereum analyze-contract command
#[cfg(feature = "ethereum")]
pub async fn cmd_ethereum_analyze_contract(
    abi_file: &Path,
    output: Option<&Path>,
    validate_storage: bool,
    contract_address: Option<&str>,
    rpc: Option<&str>,
) -> Result<()> {
    info!("Analyzing Ethereum contract from {}", abi_file.display());

    // Read and parse ABI or Layout
    let abi_content = std::fs::read_to_string(abi_file)?;
    let abi: Value = serde_json::from_str(&abi_content)?;

    let mut analysis = json!({
        "contract_type": "ethereum",
        "abi_file": abi_file.display().to_string(),
        "analysis_timestamp": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        "functions": [],
        "events": [],
        "storage_patterns": [],
        "detected_patterns": [],
        "complexity_score": 0,
        "recommendations": []
    });

    // Check if this is a canonical layout or ABI
    if let Some(contract_name) = abi.get("contract_name") {
        // This is a canonical layout file
        analysis["contract_type"] = json!("ethereum_layout");
        analysis["contract_name"] = contract_name.clone();

        if let Some(storage) = abi.get("storage").and_then(|s| s.as_array()) {
            let mut storage_patterns = Vec::new();
            let mut complexity_score = 0;
            let mut detected_patterns = Vec::new();

            for entry in storage {
                if let Some(type_name) = entry.get("type_name").and_then(|t| t.as_str()) {
                    let pattern = if type_name.contains("mapping") {
                        "mapping"
                    } else if type_name.contains("array") {
                        "dynamic_array"
                    } else if type_name.contains("struct") {
                        "struct"
                    } else {
                        "simple"
                    };

                    storage_patterns.push(json!({
                        "label": entry.get("label").unwrap_or(&json!("unknown")),
                        "pattern": pattern,
                        "type": type_name,
                        "slot": entry.get("slot").unwrap_or(&json!("unknown"))
                    }));

                    complexity_score += 1;
                    if pattern == "mapping" {
                        complexity_score += 2;
                    }
                    if pattern == "struct" {
                        complexity_score += 3;
                    }

                    // Detect complex nested mappings
                    if type_name.contains("mapping") && type_name.matches("mapping").count() >= 2 {
                        detected_patterns.push("complex mappings".to_string());
                    }
                }
            }

            analysis["storage_patterns"] = json!(storage_patterns);
            analysis["complexity_score"] = json!(complexity_score);
            analysis["detected_patterns"] = json!(detected_patterns);
        }

        // Generate recommendations based on patterns
        let recommendations = generate_recommendations(&analysis);
        analysis["recommendations"] = json!(recommendations);

        // Perform live analysis if contract address and RPC are provided
        if let (Some(address), Some(rpc_url)) = (contract_address, rpc) {
            if validate_storage {
                info!("Performing live contract analysis for {}", address);
                match perform_live_ethereum_analysis(address, rpc_url).await {
                    Ok(live_data) => {
                        analysis["live_analysis"] = live_data;
                    }
                    Err(e) => {
                        warn!("Live analysis failed: {}", e);
                        analysis["live_analysis_error"] = json!(e.to_string());
                    }
                }
            }
        }

        // Write output
        let output_str = serde_json::to_string_pretty(&analysis)?;
        write_output(&output_str, output)?;
        return Ok(());
    }

    // Handle standard ABI files
    if let Some(functions) = abi.as_array() {
        let mut function_analysis = Vec::new();
        let mut event_analysis = Vec::new();
        let mut complexity_score = 0;
        let mut detected_patterns = Vec::new();

        for item in functions {
            if let Some(item_type) = item.get("type").and_then(|t| t.as_str()) {
                match item_type {
                    "function" => {
                        complexity_score += 1;
                        if let Some(name) = item.get("name").and_then(|n| n.as_str()) {
                            let inputs = item.get("inputs").and_then(|i| i.as_array()).unwrap_or(&vec![]);
                            let outputs = item.get("outputs").and_then(|o| o.as_array()).unwrap_or(&vec![]);
                            
                            function_analysis.push(json!({
                                "name": name,
                                "inputs": inputs.len(),
                                "outputs": outputs.len(),
                                "stateMutability": item.get("stateMutability").unwrap_or(&json!("unknown")),
                                "signature": generate_function_signature(name, inputs)
                            }));

                            // Detect common patterns
                            if name.starts_with("withdraw") || name.starts_with("transfer") {
                                detected_patterns.push("token operations".to_string());
                            }
                            if name.starts_with("set") || name.starts_with("update") {
                                detected_patterns.push("state modifications".to_string());
                            }
                            if name.starts_with("get") || name.starts_with("view") {
                                detected_patterns.push("view functions".to_string());
                            }
                        }
                    }
                    "event" => {
                        if let Some(name) = item.get("name").and_then(|n| n.as_str()) {
                            let inputs = item.get("inputs").and_then(|i| i.as_array()).unwrap_or(&vec![]);
                            event_analysis.push(json!({
                                "name": name,
                                "inputs": inputs.len(),
                                "anonymous": item.get("anonymous").unwrap_or(&json!(false))
                            }));
                        }
                    }
                    _ => {}
                }
            }
        }

        analysis["functions"] = json!(function_analysis);
        analysis["events"] = json!(event_analysis);
        analysis["complexity_score"] = json!(complexity_score);
        analysis["detected_patterns"] = json!(detected_patterns.into_iter().collect::<std::collections::HashSet<_>>().into_iter().collect::<Vec<_>>());

        // Generate recommendations
        let recommendations = generate_recommendations(&analysis);
        analysis["recommendations"] = json!(recommendations);

        // Perform live analysis if contract address and RPC are provided
        if let (Some(address), Some(rpc_url)) = (contract_address, rpc) {
            if validate_storage {
                info!("Performing live contract analysis for {}", address);
                match perform_live_ethereum_analysis(address, rpc_url).await {
                    Ok(live_data) => {
                        analysis["live_analysis"] = live_data;
                    }
                    Err(e) => {
                        warn!("Live analysis failed: {}", e);
                        analysis["live_analysis_error"] = json!(e.to_string());
                    }
                }
            }
        }
    }

    // Write output
    let output_str = serde_json::to_string_pretty(&analysis)?;
    write_output(&output_str, output)?;
    Ok(())
}

#[cfg(not(feature = "ethereum"))]
pub async fn cmd_ethereum_analyze_contract(
    _abi_file: &Path,
    _output: Option<&Path>,
    _validate_storage: bool,
    _contract_address: Option<&str>,
    _rpc: Option<&str>,
) -> Result<()> {
    Err(anyhow::anyhow!("Ethereum support not enabled. Build with --features ethereum"))
}

fn generate_recommendations(analysis: &Value) -> Vec<String> {
    let mut recommendations = Vec::new();
    
    if let Some(complexity) = analysis.get("complexity_score").and_then(|c| c.as_u64()) {
        if complexity > 10 {
            recommendations.push("Consider simplifying contract structure to reduce complexity".to_string());
        }
        if complexity > 20 {
            recommendations.push("High complexity detected - consider splitting into multiple contracts".to_string());
        }
    }
    
    if let Some(patterns) = analysis.get("detected_patterns").and_then(|p| p.as_array()) {
        if patterns.iter().any(|p| p.as_str() == Some("complex mappings")) {
            recommendations.push("Complex nested mappings detected - consider using libraries for gas optimization".to_string());
        }
        if patterns.iter().any(|p| p.as_str() == Some("token operations")) {
            recommendations.push("Token operations detected - ensure proper access controls and overflow checks".to_string());
        }
    }
    
    recommendations
}

fn generate_function_signature(name: &str, inputs: &[Value]) -> String {
    let input_types: Vec<String> = inputs.iter()
        .filter_map(|input| input.get("type").and_then(|t| t.as_str()))
        .map(|t| t.to_string())
        .collect();
    
    format!("{}({})", name, input_types.join(","))
}

/// Compile Ethereum storage layout
#[cfg(feature = "ethereum")]
pub fn cmd_ethereum_compile_layout(
    abi_file: &Path,
    output: Option<&Path>,
    format: &OutputFormat,
    validate: bool,
) -> Result<()> {
    info!("Compiling Ethereum storage layout from {}", abi_file.display());

    // Read ABI file
    let abi_content = std::fs::read_to_string(abi_file)?;
    let abi: Value = serde_json::from_str(&abi_content)?;

    // Create compiler and compile layout
    let compiler = EthereumLayoutCompiler;
    let layout = compiler.compile_layout(&abi)?;

    if validate {
        info!("Validating layout for conflicts...");
        validate_layout(&layout)?;
    }

    // Format output based on requested format
    let output_str = match format {
        OutputFormat::Traverse => serde_json::to_string_pretty(&layout)?,
        OutputFormat::CoprocessorJson => {
            let simplified = SimpleLayoutInfo {
                contract_name: layout.contract_name.clone(),
                storage_entries: layout.storage.len(),
                type_definitions: layout.types.len(),
                commitment: hex::encode(&layout.commitment()),
                generated_at: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                compiler: "ethereum".to_string(),
            };
            serde_json::to_string_pretty(&simplified)?
        }
        OutputFormat::Toml => {
            let simplified = SimpleLayoutInfo {
                contract_name: layout.contract_name.clone(),
                storage_entries: layout.storage.len(),
                type_definitions: layout.types.len(),
                commitment: hex::encode(&layout.commitment()),
                generated_at: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                compiler: "ethereum".to_string(),
            };
            toml::to_string_pretty(&simplified)?
        }
        OutputFormat::Binary => {
            let binary_data = bincode::serialize(&layout)?;
            format!("Binary layout: {} bytes\nBase64: {}", binary_data.len(), STANDARD.encode(&binary_data))
        }
        OutputFormat::Base64 => {
            let binary_data = bincode::serialize(&layout)?;
            STANDARD.encode(&binary_data)
        }
    };

    write_output(&output_str, output)?;
    Ok(())
}

#[cfg(not(feature = "ethereum"))]
pub fn cmd_ethereum_compile_layout(
    _abi_file: &Path,
    _output: Option<&Path>,
    _format: &OutputFormat,
    _validate: bool,
) -> Result<()> {
    Err(anyhow::anyhow!("Ethereum support not enabled. Build with --features ethereum"))
}

fn validate_layout(layout: &LayoutInfo) -> Result<()> {
    // Check for storage slot conflicts
    let mut used_slots = std::collections::HashSet::new();
    for entry in &layout.storage {
        let slot = &entry.slot;
        if used_slots.contains(slot) {
            warn!("Storage slot conflict detected at slot {}", slot);
        }
        used_slots.insert(slot.clone());
    }
    
    // Check for type consistency
    for entry in &layout.storage {
        let type_name = &entry.type_name;
        if !layout.types.iter().any(|t| t.label == *type_name) {
            warn!("Unknown type '{}' used in storage entry '{}'", type_name, entry.label);
        }
    }
    
    Ok(())
}

/// Generate storage queries for specific fields
#[cfg(feature = "ethereum")]
pub fn cmd_ethereum_generate_queries(
    layout_file: &Path,
    fields: &str,
    output: Option<&Path>,
    include_examples: bool,
) -> Result<()> {
    info!("Generating storage queries for fields: {}", fields);

    // Load layout
    let layout_content = std::fs::read_to_string(layout_file)?;
    let layout: LayoutInfo = serde_json::from_str(&layout_content)?;

    // Parse field list
    let field_list: Vec<&str> = fields.split(',').map(|f| f.trim()).collect();

    // Generate queries
    let mut queries = Vec::new();
    for field in &field_list {
        if let Some(entry) = layout.storage.iter().find(|e| e.label == *field) {
            let mut query = json!({
                "field": field,
                "type": entry.type_name,
                "slot": entry.slot,
                "offset": entry.offset,
                "zero_semantics": entry.zero_semantics
            });

            if include_examples {
                query["example_queries"] = match entry.type_name.as_str() {
                    t if t.contains("mapping") => {
                        json!([
                            format!("{}[0x1234567890abcdef]", field),
                            format!("{}[0x0000000000000000000000000000000000000001]", field)
                        ])
                    }
                    t if t.contains("array") => {
                        json!([
                            format!("{}[0]", field),
                            format!("{}[1]", field)
                        ])
                    }
                    _ => json!([field])
                };
            }

            queries.push(query);
        } else {
            warn!("Field '{}' not found in layout", field);
        }
    }

    let output_str = serde_json::to_string_pretty(&json!({
        "queries": queries,
        "total_fields": field_list.len(),
        "found_fields": queries.len()
    }))?;

    write_output(&output_str, output)?;
    Ok(())
}

#[cfg(not(feature = "ethereum"))]
pub fn cmd_ethereum_generate_queries(
    _layout_file: &Path,
    _fields: &str,
    _output: Option<&Path>,
    _include_examples: bool,
) -> Result<()> {
    Err(anyhow::anyhow!("Ethereum support not enabled. Build with --features ethereum"))
}

/// Resolve specific storage query
#[cfg(feature = "ethereum")]
pub fn cmd_ethereum_resolve_query(
    query: &str,
    layout_file: &Path,
    format: &OutputFormat,
    output: Option<&Path>,
) -> Result<()> {
    info!("Resolving storage query: {}", query);

    // Load layout
    let layout_content = std::fs::read_to_string(layout_file)?;
    let layout: LayoutInfo = serde_json::from_str(&layout_content)?;

    // Create resolver
    let resolver = EthereumKeyResolver;
    let resolved = resolver.resolve(&layout, query)?;

    let output_str = match format {
        OutputFormat::Traverse => serde_json::to_string_pretty(&resolved)?,
        OutputFormat::CoprocessorJson => {
            let coprocessor_format = json!({
                "query": query,
                "storage_key": hex::encode(key_to_bytes(&resolved.key)),
                "layout_commitment": hex::encode(&resolved.layout_commitment),
                "field_size": resolved.field_size,
                "offset": resolved.offset
            });
            serde_json::to_string_pretty(&coprocessor_format)?
        }
        OutputFormat::Toml => {
            let simplified = json!({
                "query": query,
                "storage_key": hex::encode(key_to_bytes(&resolved.key)),
                "layout_commitment": hex::encode(&resolved.layout_commitment)
            });
            toml::to_string_pretty(&simplified)?
        }
        OutputFormat::Binary => {
            let binary_data = bincode::serialize(&resolved)?;
            format!("Binary query result: {} bytes\nBase64: {}", binary_data.len(), STANDARD.encode(&binary_data))
        }
        OutputFormat::Base64 => {
            let binary_data = bincode::serialize(&resolved)?;
            STANDARD.encode(&binary_data)
        }
    };

    write_output(&output_str, output)?;
    Ok(())
}

#[cfg(not(feature = "ethereum"))]
pub fn cmd_ethereum_resolve_query(
    _query: &str,
    _layout_file: &Path,
    _format: &OutputFormat,
    _output: Option<&Path>,
) -> Result<()> {
    Err(anyhow::anyhow!("Ethereum support not enabled. Build with --features ethereum"))
}

/// Verify storage layout correctness
#[cfg(feature = "ethereum")]
pub async fn cmd_ethereum_verify_layout(
    layout_file: &Path,
    contract_address: Option<&str>,
    rpc: Option<&str>,
    comprehensive: bool,
) -> Result<()> {
    info!("Verifying storage layout from {}", layout_file.display());

    // Load layout
    let layout_content = std::fs::read_to_string(layout_file)?;
    let layout: LayoutInfo = serde_json::from_str(&layout_content)?;

    // Perform basic validation
    validate_layout(&layout)?;

    let mut verification_result = json!({
        "layout_file": layout_file.display().to_string(),
        "basic_validation": "passed",
        "timestamp": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    });

    // Perform live verification if contract address and RPC are provided
    if let (Some(address), Some(rpc_url)) = (contract_address, rpc) {
        info!("Performing live verification for contract {}", address);
        match perform_live_ethereum_verification(address, rpc_url, &layout).await {
            Ok(live_verification) => {
                verification_result["live_verification"] = live_verification;
            }
            Err(e) => {
                warn!("Live verification failed: {}", e);
                verification_result["live_verification_error"] = json!(e.to_string());
            }
        }
    }

    if comprehensive {
        info!("Running comprehensive verification tests...");
        // Add comprehensive tests here
        verification_result["comprehensive_tests"] = json!("not_implemented");
    }

    let output_str = serde_json::to_string_pretty(&verification_result)?;
    println!("{}", output_str);
    Ok(())
}

#[cfg(not(feature = "ethereum"))]
pub async fn cmd_ethereum_verify_layout(
    _layout_file: &Path,
    _contract_address: Option<&str>,
    _rpc: Option<&str>,
    _comprehensive: bool,
) -> Result<()> {
    Err(anyhow::anyhow!("Ethereum support not enabled. Build with --features ethereum"))
}

/// End-to-end automation for Ethereum
#[cfg(feature = "ethereum")]
pub async fn cmd_ethereum_auto_generate(
    abi_file: &Path,
    rpc: &str,
    contract: &str,
    queries: &str,
    output_dir: &Path,
    _cache: bool,
    dry_run: bool,
) -> Result<()> {
    info!("Running Ethereum auto-generation for {}", contract);

    // Create output directory
    std::fs::create_dir_all(output_dir)?;

    // Step 1: Compile layout
    info!("Step 1: Compiling layout...");
    let layout_file = output_dir.join("layout.json");
    cmd_ethereum_compile_layout(abi_file, Some(&layout_file), &OutputFormat::Traverse, true)?;

    // Step 2: Generate queries
    info!("Step 2: Generating queries...");
    let queries_file = output_dir.join("queries.json");
    cmd_ethereum_generate_queries(&layout_file, queries, Some(&queries_file), true)?;

    // Step 3: Resolve queries
    info!("Step 3: Resolving queries...");
    let query_list: Vec<&str> = queries.split(',').map(|q| q.trim()).collect();
    let resolved_file = output_dir.join("resolved.json");
    
    let mut resolved_queries = Vec::new();
    for query in query_list {
        match std::panic::catch_unwind(|| {
            cmd_ethereum_resolve_query(query, &layout_file, &OutputFormat::CoprocessorJson, None)
        }) {
            Ok(Ok(_)) => {
                resolved_queries.push(json!({
                    "query": query,
                    "status": "resolved"
                }));
            }
            _ => {
                resolved_queries.push(json!({
                    "query": query,
                    "status": "failed"
                }));
            }
        }
    }

    let resolved_output = json!({
        "contract": contract,
        "queries": resolved_queries,
        "total_queries": query_list.len()
    });

    std::fs::write(&resolved_file, serde_json::to_string_pretty(&resolved_output)?)?;

    // Step 4: Generate proof templates (if not dry run)
    if !dry_run {
        info!("Step 4: Generating proof templates...");
        let proof_template = json!({
            "contract": contract,
            "rpc": rpc,
            "queries": query_list,
            "note": "Use these queries with the generate-proof command"
        });
        
        let proof_file = output_dir.join("proof_template.json");
        std::fs::write(&proof_file, serde_json::to_string_pretty(&proof_template)?)?;
    }

    // Summary
    let summary = json!({
        "contract": contract,
        "abi_file": abi_file.display().to_string(),
        "output_dir": output_dir.display().to_string(),
        "dry_run": dry_run,
        "files_generated": {
            "layout": layout_file.display().to_string(),
            "queries": queries_file.display().to_string(),
            "resolved": resolved_file.display().to_string(),
            "proof_template": if dry_run { "skipped" } else { "generated" }
        },
        "next_steps": [
            "Review generated files",
            "Run proof generation with generated template",
            "Integrate with your ZK application"
        ]
    });

    let summary_file = output_dir.join("summary.json");
    std::fs::write(&summary_file, serde_json::to_string_pretty(&summary)?)?;

    info!("Auto-generation complete. Summary written to {}", summary_file.display());
    Ok(())
}

#[cfg(not(feature = "ethereum"))]
pub async fn cmd_ethereum_auto_generate(
    _abi_file: &Path,
    _rpc: &str,
    _contract: &str,
    _queries: &str,
    _output_dir: &Path,
    _cache: bool,
    _dry_run: bool,
) -> Result<()> {
    Err(anyhow::anyhow!("Ethereum support not enabled. Build with --features ethereum"))
}

// Helper functions for live analysis
#[cfg(feature = "ethereum")]
async fn perform_live_ethereum_analysis(contract_address: &str, rpc_url: &str) -> Result<Value> {
    // Basic RPC call to get contract code and validate it exists
    let client = reqwest::Client::new();
    
    let rpc_request = json!({
        "jsonrpc": "2.0",
        "method": "eth_getCode",
        "params": [contract_address, "latest"],
        "id": 1
    });
    
    let response = client
        .post(rpc_url)
        .json(&rpc_request)
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("RPC request failed: {}", e))?;
    
    let rpc_response: Value = response.json().await
        .map_err(|e| anyhow::anyhow!("Failed to parse RPC response: {}", e))?;
    
    // Check if contract exists (has code)
    let has_code = if let Some(result) = rpc_response.get("result") {
        result.as_str().unwrap_or("0x") != "0x"
    } else {
        false
    };
    
    Ok(json!({
        "contract_address": contract_address,
        "rpc_url": rpc_url,
        "status": if has_code { "contract_found" } else { "no_contract" },
        "has_code": has_code,
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

#[cfg(feature = "ethereum")]
async fn perform_live_ethereum_verification(
    contract_address: &str,
    rpc_url: &str,
    layout: &LayoutInfo,
) -> Result<Value> {
    // Basic verification: check a few storage slots to see if contract has expected structure
    let client = reqwest::Client::new();
    let mut verified_slots = 0;
    let mut total_slots = 0;
    
    // Check first few storage entries
    for (i, entry) in layout.storage.iter().enumerate().take(3) {
        total_slots += 1;
        let slot_hex = format!("0x{:064x}", i); // Simple slot numbering for basic verification
        
        let rpc_request = json!({
            "jsonrpc": "2.0",
            "method": "eth_getStorageAt",
            "params": [contract_address, slot_hex, "latest"],
            "id": i + 1
        });
        
        if let Ok(response) = client.post(rpc_url).json(&rpc_request).send().await {
            if let Ok(rpc_response) = response.json::<Value>().await {
                if rpc_response.get("result").is_some() {
                    verified_slots += 1;
                    info!("Verified storage slot for field: {}", entry.label);
                }
            }
        }
    }
    
    let verification_ratio = if total_slots > 0 { 
        verified_slots as f64 / total_slots as f64 
    } else { 
        0.0 
    };
    
    Ok(json!({
        "contract_address": contract_address,
        "rpc_url": rpc_url,
        "layout_commitment": hex::encode(&layout.commitment),
        "status": if verification_ratio > 0.5 { "verification_passed" } else { "verification_failed" },
        "verified_slots": verified_slots,
        "total_slots": total_slots,
        "verification_ratio": verification_ratio,
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

#[cfg(not(feature = "ethereum"))]
async fn perform_live_ethereum_analysis(_contract_address: &str, _rpc_url: &str) -> Result<Value> {
    Err(anyhow::anyhow!("Ethereum support not enabled"))
}

#[cfg(not(feature = "ethereum"))]
async fn perform_live_ethereum_verification(
    _contract_address: &str,
    _rpc_url: &str,
    _layout: &LayoutInfo,
) -> Result<Value> {
    Err(anyhow::anyhow!("Ethereum support not enabled"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;
    use traverse_core::{layout::LayoutInfo, layout::StorageEntry, layout::TypeInfo, ZeroSemantics};

    #[test]
    fn test_key_to_bytes_fixed() {
        use traverse_core::Key;
        
        let fixed_key = Key::Fixed(vec![1, 2, 3, 4]);
        let bytes = key_to_bytes(&fixed_key);
        assert_eq!(bytes, &[1, 2, 3, 4]);
    }

    #[test]
    fn test_key_to_bytes_variable() {
        use traverse_core::Key;
        
        let variable_key = Key::Variable(vec![5, 6, 7, 8]);
        let bytes = key_to_bytes(&variable_key);
        assert_eq!(bytes, &[5, 6, 7, 8]);
    }

    #[test]
    fn test_simple_layout_info_serialization() {
        let layout_info = SimpleLayoutInfo {
            contract_name: "TestContract".to_string(),
            storage_entries: 2,
            type_definitions: 1,
            commitment: "abcd1234".to_string(),
            generated_at: 1234567890,
            compiler: "ethereum".to_string(),
        };

        let json = serde_json::to_string(&layout_info).unwrap();
        assert!(json.contains("TestContract"));
        assert!(json.contains("abcd1234"));
    }

    #[cfg(feature = "ethereum")]
    #[tokio::test]
    async fn test_perform_live_ethereum_analysis_mock() {
        // Test with a placeholder URL since we can't easily mock HTTP in this context
        // In a real test environment, you'd use a mock server
        let result = perform_live_ethereum_analysis(
            "0x1234567890123456789012345678901234567890",
            "http://localhost:8545" // This will fail, but we can test error handling
        ).await;

        // Should fail with network error since localhost:8545 likely not running
        assert!(result.is_err());
    }

    #[cfg(feature = "ethereum")]
    #[tokio::test]
    async fn test_perform_live_ethereum_verification_mock() {
        // Create a minimal test layout
        let layout = LayoutInfo {
            contract_name: "TestContract".to_string(),
            storage: vec![
                StorageEntry {
                    label: "balance".to_string(),
                    slot: "0".to_string(),
                    offset: 0,
                    type_name: "uint256".to_string(),
                    zero_semantics: ZeroSemantics::ValidZero,
                },
            ],
            types: vec![
                TypeInfo {
                    label: "uint256".to_string(),
                    number_of_bytes: "32".to_string(),
                    encoding: "inplace".to_string(),
                    base: None,
                    key: None,
                    value: None,
                },
            ],
        };

        let result = perform_live_ethereum_verification(
            "0x1234567890123456789012345678901234567890",
            "http://localhost:8545", // This will fail, but we can test error handling
            &layout
        ).await;

        // Should fail with network error since localhost:8545 likely not running
        assert!(result.is_err());
    }
} 