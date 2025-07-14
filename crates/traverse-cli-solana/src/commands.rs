//! Solana program analysis and proof generation commands
//!
//! This module provides CLI commands for analyzing Solana programs using IDL files,
//! compiling storage layouts, and generating account proofs.

use anyhow::Result;
use std::path::{Path, PathBuf};
use serde_json::Value;
use traverse_cli_core::{formatters::write_output, OutputFormat};

/// Analyze Solana program from IDL
#[cfg(feature = "solana")]
pub async fn cmd_solana_analyze_program(
    idl_file: &Path,
    output: Option<&Path>,
    validate_schema: bool,
) -> Result<()> {
    println!("Analyzing Solana program from IDL: {}", idl_file.display());
    
    // Read IDL file
    let idl_content = std::fs::read_to_string(idl_file)?;
    
    use traverse_solana::anchor::{IdlParser, SolanaIdl};
    
    // Parse IDL
    let idl = IdlParser::parse_idl(&idl_content)?;
    
    if validate_schema {
        println!("Validating IDL schema...");
        validate_idl_schema(&idl)?;
        println!("✓ IDL validation passed");
    }
    
    // Extract analysis information
    let analysis = serde_json::json!({
        "program_id": idl.program_id,
        "program_name": idl.name,
        "version": idl.version,
        "instructions": idl.instructions.len(),
        "accounts": idl.accounts.len(),
        "types": idl.types.len(),
        "events": idl.events.len(),
        "errors": idl.errors.len(),
        "constants": idl.constants.len(),
        "account_layouts": {
            let layouts = IdlParser::extract_account_layouts(&idl)?;
            layouts.iter().map(|layout| serde_json::json!({
                "name": layout.name,
                "discriminator": layout.discriminator,
                "fields": layout.fields.len(),
                "total_size": layout.total_size
            })).collect::<Vec<_>>()
        },
        "pdas": {
            let pdas = IdlParser::extract_pdas(&idl);
            pdas.iter().map(|pda| serde_json::json!({
                "account_name": pda.account_name,
                "seeds": pda.seeds.len(),
                "program_id": pda.program_id
            })).collect::<Vec<_>>()
        }
    });
    
    // Output analysis
    let output_content = serde_json::to_string_pretty(&analysis)?;
    
    if let Some(output_path) = output {
        std::fs::write(output_path, output_content)?;
        println!("✓ Analysis written to: {}", output_path.display());
    } else {
        println!("{}", output_content);
    }
    
    println!("✓ Program analysis completed");
    println!("  - Program ID: {}", idl.program_id);
    println!("  - Instructions: {}", idl.instructions.len());
    println!("  - Accounts: {}", idl.accounts.len());
    println!("  - Account layouts: {}", IdlParser::extract_account_layouts(&idl)?.len());
    println!("  - PDAs: {}", IdlParser::extract_pdas(&idl).len());
    
    Ok(())
}

#[cfg(not(feature = "solana"))]
pub async fn cmd_solana_analyze_program(
    _idl_file: &Path,
    _output: Option<&Path>,
    _validate_schema: bool,
) -> Result<()> {
    Err(anyhow::anyhow!("Solana support not enabled. Build with --features solana"))
}

/// Compile Solana storage layout from IDL
#[cfg(feature = "solana")]
pub async fn cmd_solana_compile_layout(
    idl_file: &Path,
    output: Option<&Path>,
    format: &OutputFormat,
) -> Result<()> {
    println!("Compiling Solana storage layout from IDL: {}", idl_file.display());
    
    // Read IDL file
    let idl_content = std::fs::read_to_string(idl_file)?;
    
    use traverse_solana::{SolanaLayoutCompiler, anchor::IdlParser};
    use traverse_core::LayoutCompiler;
    
    // Parse IDL
    let idl = IdlParser::parse_idl(&idl_content)?;
    
    // Create compiler and compile layout
    let compiler = SolanaLayoutCompiler::new();
    let layout = compiler.compile_from_idl(&idl)?;
    
    // Format output based on requested format
    let output_str = match format {
        OutputFormat::Traverse => serde_json::to_string_pretty(&layout)?,
        OutputFormat::CoprocessorJson => {
            let simplified = serde_json::json!({
                "program_name": layout.contract_name,
                "accounts": layout.storage.len(),
                "types": layout.types.len(),
                "commitment": hex::encode(&layout.commitment),
                "generated_at": std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                "compiler": "solana"
            });
            serde_json::to_string_pretty(&simplified)?
        }
        OutputFormat::Toml => {
            let simplified = serde_json::json!({
                "program_name": layout.contract_name,
                "accounts": layout.storage.len(),
                "types": layout.types.len(),
                "commitment": hex::encode(&layout.commitment)
            });
            toml::to_string_pretty(&simplified)?
        }
        OutputFormat::Binary => {
            let binary_data = bincode::serialize(&layout)?;
            format!("Binary layout: {} bytes\nBase64: {}", binary_data.len(), base64::engine::general_purpose::STANDARD.encode(&binary_data))
        }
        OutputFormat::Base64 => {
            let binary_data = bincode::serialize(&layout)?;
            base64::engine::general_purpose::STANDARD.encode(&binary_data)
        }
    };
    
    write_output(&output_str, output)?;
    
    println!("✓ Storage layout compiled");
    println!("  - Program: {}", layout.contract_name);
    println!("  - Accounts: {}", layout.storage.len());
    println!("  - Types: {}", layout.types.len());
    
    Ok(())
}

#[cfg(not(feature = "solana"))]
pub async fn cmd_solana_compile_layout(
    _idl_file: &Path,
    _output: Option<&Path>,
    _format: &OutputFormat,
) -> Result<()> {
    Err(anyhow::anyhow!("Solana support not enabled. Build with --features solana"))
}

/// Generate storage queries for Solana state
#[cfg(feature = "solana")]
pub async fn cmd_solana_generate_queries(
    layout_file: &Path,
    state_keys: &str,
    output: Option<&Path>,
    include_examples: bool,
) -> Result<()> {
    println!("Generating Solana storage queries for keys: {}", state_keys);
    
    // Load layout
    let layout_content = std::fs::read_to_string(layout_file)?;
    let layout: traverse_core::LayoutInfo = serde_json::from_str(&layout_content)?;
    
    // Parse key list
    let key_list: Vec<&str> = state_keys.split(',').map(|k| k.trim()).collect();
    
    // Generate queries
    let mut queries = Vec::new();
    for key in key_list {
        if let Some(entry) = layout.storage.iter().find(|e| e.label == key) {
            let mut query = serde_json::json!({
                "account": key,
                "type": entry.type_name,
                "discriminator": entry.slot, // In Solana, slot represents discriminator
                "fields": entry.size
            });
            
            if include_examples {
                query["example_queries"] = match entry.type_name.as_deref() {
                    Some(t) if t.contains("pubkey") => {
                        serde_json::json!([
                            format!("{}[9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM]", key),
                            format!("{}[11111111111111111111111111111112]", key)
                        ])
                    }
                    Some(t) if t.contains("array") => {
                        serde_json::json!([
                            format!("{}[0]", key),
                            format!("{}[1]", key)
                        ])
                    }
                    _ => serde_json::json!([key])
                };
            }
            
            queries.push(query);
        } else {
            println!("Warning: Account '{}' not found in layout", key);
        }
    }
    
    let output_json = serde_json::json!({
        "queries": queries,
        "total_keys": key_list.len(),
        "found_keys": queries.len()
    });
    
    let output_str = serde_json::to_string_pretty(&output_json)?;
    write_output(&output_str, output)?;
    
    println!("✓ Generated {} queries for {} keys", queries.len(), key_list.len());
    
    Ok(())
}

#[cfg(not(feature = "solana"))]
pub async fn cmd_solana_generate_queries(
    _layout_file: &Path,
    _state_keys: &str,
    _output: Option<&Path>,
    _include_examples: bool,
) -> Result<()> {
    Err(anyhow::anyhow!("Solana support not enabled. Build with --features solana"))
}

/// Resolve specific Solana storage query
#[cfg(feature = "solana")]
pub async fn cmd_solana_resolve_query(
    query: &str,
    layout_file: &Path,
    format: &OutputFormat,
    output: Option<&Path>,
) -> Result<()> {
    println!("Resolving Solana storage query: {}", query);
    
    // Load layout
    let layout_content = std::fs::read_to_string(layout_file)?;
    let layout: traverse_core::LayoutInfo = serde_json::from_str(&layout_content)?;
    
    // Create resolver
    use traverse_solana::SolanaKeyResolver;
    use traverse_core::KeyResolver;
    
    let resolver = SolanaKeyResolver::new();
    let resolved = resolver.resolve(query, &layout)?;
    
    let output_str = match format {
        OutputFormat::Traverse => serde_json::to_string_pretty(&resolved)?,
        OutputFormat::CoprocessorJson => {
            let coprocessor_format = serde_json::json!({
                "query": query,
                "account_key": hex::encode(&resolved.key),
                "layout_commitment": hex::encode(&resolved.layout_commitment),
                "field_size": resolved.field_size,
                "offset": resolved.offset
            });
            serde_json::to_string_pretty(&coprocessor_format)?
        }
        OutputFormat::Toml => {
            let simplified = serde_json::json!({
                "query": query,
                "account_key": hex::encode(&resolved.key),
                "layout_commitment": hex::encode(&resolved.layout_commitment)
            });
            toml::to_string_pretty(&simplified)?
        }
        OutputFormat::Binary => {
            let binary_data = bincode::serialize(&resolved)?;
            format!("Binary query result: {} bytes\nBase64: {}", binary_data.len(), base64::engine::general_purpose::STANDARD.encode(&binary_data))
        }
        OutputFormat::Base64 => {
            let binary_data = bincode::serialize(&resolved)?;
            base64::engine::general_purpose::STANDARD.encode(&binary_data)
        }
    };
    
    write_output(&output_str, output)?;
    
    println!("✓ Query resolved");
    println!("  - Query: {}", query);
    println!("  - Account key: {}", hex::encode(&resolved.key));
    
    Ok(())
}

#[cfg(not(feature = "solana"))]
pub async fn cmd_solana_resolve_query(
    _query: &str,
    _layout_file: &Path,
    _format: &OutputFormat,
    _output: Option<&Path>,
) -> Result<()> {
    Err(anyhow::anyhow!("Solana support not enabled. Build with --features solana"))
}

/// End-to-end automation for Solana
#[cfg(feature = "solana")]
pub async fn cmd_solana_auto_generate(
    idl_file: &Path,
    rpc: &str,
    program_address: &str,
    queries: &str,
    output_dir: &Path,
    dry_run: bool,
) -> Result<()> {
    println!("Running Solana auto-generation for program: {}", program_address);
    
    // Create output directory
    std::fs::create_dir_all(output_dir)?;
    
    // Step 1: Compile layout
    println!("Step 1: Compiling layout...");
    let layout_file = output_dir.join("layout.json");
    cmd_solana_compile_layout(idl_file, Some(&layout_file), &OutputFormat::Traverse).await?;
    
    // Step 2: Generate queries
    println!("Step 2: Generating queries...");
    let queries_file = output_dir.join("queries.json");
    cmd_solana_generate_queries(&layout_file, queries, Some(&queries_file), true).await?;
    
    // Step 3: Resolve queries
    println!("Step 3: Resolving queries...");
    let query_list: Vec<&str> = queries.split(',').map(|q| q.trim()).collect();
    let resolved_file = output_dir.join("resolved.json");
    
    let mut resolved_queries = Vec::new();
    for query in query_list {
        match cmd_solana_resolve_query(query, &layout_file, &OutputFormat::CoprocessorJson, None).await {
            Ok(()) => {
                resolved_queries.push(serde_json::json!({
                    "query": query,
                    "status": "resolved"
                }));
            }
            Err(_) => {
                resolved_queries.push(serde_json::json!({
                    "query": query,
                    "status": "failed"
                }));
            }
        }
    }
    
    let resolved_output = serde_json::json!({
        "program": program_address,
        "queries": resolved_queries,
        "total_queries": query_list.len()
    });
    
    std::fs::write(&resolved_file, serde_json::to_string_pretty(&resolved_output)?)?;
    
    // Step 4: Generate proof templates (if not dry run)
    if !dry_run {
        println!("Step 4: Generating proof templates...");
        let proof_template = serde_json::json!({
            "program": program_address,
            "rpc": rpc,
            "queries": query_list,
            "note": "Use these queries with the generate-proof command"
        });
        
        let proof_file = output_dir.join("proof_template.json");
        std::fs::write(&proof_file, serde_json::to_string_pretty(&proof_template)?)?;
    }
    
    // Summary
    let summary = serde_json::json!({
        "program": program_address,
        "idl_file": idl_file.display().to_string(),
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
    
    println!("✓ Auto-generation complete. Summary written to {}", summary_file.display());
    Ok(())
}

#[cfg(not(feature = "solana"))]
pub async fn cmd_solana_auto_generate(
    _idl_file: &Path,
    _rpc: &str,
    _program_address: &str,
    _queries: &str,
    _output_dir: &Path,
    _dry_run: bool,
) -> Result<()> {
    Err(anyhow::anyhow!("Solana support not enabled. Build with --features solana"))
}

/// Validate IDL schema for correctness and completeness
#[cfg(feature = "solana")]
fn validate_idl_schema(idl: &SolanaIdl) -> Result<()> {
    // Basic validation checks
    
    // 1. Check program ID is valid
    if idl.program_id.is_empty() {
        return Err(anyhow::anyhow!("IDL missing program_id"));
    }
    
    // 2. Check for at least one account type
    if idl.accounts.is_empty() {
        return Err(anyhow::anyhow!("IDL has no account types defined"));
    }
    
    // 3. Validate account structures
    for account in &idl.accounts {
        if account.name.is_empty() {
            return Err(anyhow::anyhow!("Account with empty name found"));
        }
        
        if account.fields.is_empty() {
            return Err(anyhow::anyhow!("Account '{}' has no fields", account.name));
        }
        
        // Check for duplicate field names
        let mut field_names = std::collections::HashSet::new();
        for field in &account.fields {
            if field.name.is_empty() {
                return Err(anyhow::anyhow!("Account '{}' has field with empty name", account.name));
            }
            
            if !field_names.insert(&field.name) {
                return Err(anyhow::anyhow!("Account '{}' has duplicate field '{}'", account.name, field.name));
            }
        }
    }
    
    // 4. Validate instruction definitions if present
    for instruction in &idl.instructions {
        if instruction.name.is_empty() {
            return Err(anyhow::anyhow!("Instruction with empty name found"));
        }
    }
    
    Ok(())
}

#[cfg(not(feature = "solana"))]
fn validate_idl_schema(_idl: &()) -> Result<()> {
    Err(anyhow::anyhow!("Solana support not enabled"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "solana")]
    #[test]
    fn test_validate_idl_schema_valid() {
        use traverse_solana::anchor::{SolanaIdl, IdlAccountField, IdlAccount, IdlInstruction};

        let idl = SolanaIdl {
            program_id: "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM".to_string(),
            version: "0.1.0".to_string(),
            name: "test_program".to_string(),
            accounts: vec![
                IdlAccount {
                    name: "UserAccount".to_string(),
                    fields: vec![
                        IdlAccountField {
                            name: "authority".to_string(),
                            field_type: "publicKey".to_string(),
                        },
                        IdlAccountField {
                            name: "balance".to_string(),
                            field_type: "u64".to_string(),
                        },
                    ],
                }
            ],
            instructions: vec![
                IdlInstruction {
                    name: "initialize".to_string(),
                    accounts: vec![],
                    args: vec![],
                }
            ],
            types: vec![],
            errors: vec![],
            events: vec![],
        };

        let result = validate_idl_schema(&idl);
        assert!(result.is_ok());
    }

    #[cfg(feature = "solana")]
    #[test]
    fn test_validate_idl_schema_missing_program_id() {
        use traverse_solana::anchor::{SolanaIdl, IdlAccountField, IdlAccount};

        let idl = SolanaIdl {
            program_id: "".to_string(), // Empty program ID
            version: "0.1.0".to_string(),
            name: "test_program".to_string(),
            accounts: vec![
                IdlAccount {
                    name: "UserAccount".to_string(),
                    fields: vec![
                        IdlAccountField {
                            name: "authority".to_string(),
                            field_type: "publicKey".to_string(),
                        },
                    ],
                }
            ],
            instructions: vec![],
            types: vec![],
            errors: vec![],
            events: vec![],
        };

        let result = validate_idl_schema(&idl);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("missing program_id"));
    }

    #[cfg(feature = "solana")]
    #[test]
    fn test_validate_idl_schema_no_accounts() {
        use traverse_solana::anchor::SolanaIdl;

        let idl = SolanaIdl {
            program_id: "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM".to_string(),
            version: "0.1.0".to_string(),
            name: "test_program".to_string(),
            accounts: vec![], // No accounts
            instructions: vec![],
            types: vec![],
            errors: vec![],
            events: vec![],
        };

        let result = validate_idl_schema(&idl);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("no account types defined"));
    }

    #[cfg(feature = "solana")]
    #[test]
    fn test_validate_idl_schema_duplicate_fields() {
        use traverse_solana::anchor::{SolanaIdl, IdlAccountField, IdlAccount};

        let idl = SolanaIdl {
            program_id: "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM".to_string(),
            version: "0.1.0".to_string(),
            name: "test_program".to_string(),
            accounts: vec![
                IdlAccount {
                    name: "UserAccount".to_string(),
                    fields: vec![
                        IdlAccountField {
                            name: "authority".to_string(),
                            field_type: "publicKey".to_string(),
                        },
                        IdlAccountField {
                            name: "authority".to_string(), // Duplicate field name
                            field_type: "u64".to_string(),
                        },
                    ],
                }
            ],
            instructions: vec![],
            types: vec![],
            errors: vec![],
            events: vec![],
        };

        let result = validate_idl_schema(&idl);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("duplicate field"));
    }

    #[cfg(not(feature = "solana"))]
    #[test]
    fn test_validate_idl_schema_disabled() {
        let result = validate_idl_schema(&());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Solana support not enabled"));
    }
}