//! Solana program analysis and proof generation commands
//!
//! This module provides CLI commands for analyzing Solana programs using IDL files,
//! compiling storage layouts, and generating account proofs.

use anyhow::Result;
use std::path::Path;
use traverse_cli_core::{formatters::write_output, OutputFormat};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

/// Analyze Solana program from IDL
#[cfg(feature = "solana")]
pub async fn cmd_solana_analyze_program(
    idl_file: &Path,
    output: Option<&Path>,
    validate_schema: bool,
) -> Result<()> {
    println!("Analyzing Solana program from IDL: {}", idl_file.display());
    
    // Check if the IDL file exists before attempting to read it
    if !idl_file.exists() {
        return Err(anyhow::anyhow!(
            "IDL file does not exist: {}",
            idl_file.display()
        ));
    }
    
    // Read IDL file
    let idl_content = std::fs::read_to_string(idl_file)
        .map_err(|e| anyhow::anyhow!("Failed to read IDL file '{}': {}", idl_file.display(), e))?;
    
    #[cfg(feature = "anchor")]
    use traverse_solana::anchor::IdlParser;
    
    #[cfg(feature = "anchor")]
    {
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
            "constants": idl.constants.len()
        });
        
        // Format and write output
        let output_content = serde_json::to_string_pretty(&analysis)?;
        write_output(&output_content, output)?;
        
        if output.is_some() {
            println!("✓ Program analysis completed and written to file");
        } else {
            println!("✓ Program analysis completed");
        }
        
        return Ok(());
    }
    
    #[cfg(not(feature = "anchor"))]
    {
        return Err(anyhow::anyhow!("Anchor feature not enabled. Enable it to parse IDL files."));
    }
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
    
    // Check if the IDL file exists before attempting to read it
    if !idl_file.exists() {
        return Err(anyhow::anyhow!(
            "IDL file does not exist: {}",
            idl_file.display()
        ));
    }
    
    // Read IDL file
    let idl_content = std::fs::read_to_string(idl_file)
        .map_err(|e| anyhow::anyhow!("Failed to read IDL file '{}': {}", idl_file.display(), e))?;
    
    #[cfg(feature = "anchor")]
    use traverse_solana::{SolanaLayoutCompiler, anchor::IdlParser};
    #[cfg(not(feature = "anchor"))]
    use traverse_solana::SolanaLayoutCompiler;
    
    // Parse IDL
    #[cfg(feature = "anchor")]
    let _idl = IdlParser::parse_idl(&idl_content)?;
    
    #[cfg(not(feature = "anchor"))]
    return Err(anyhow::anyhow!("Anchor feature not enabled. Enable it to parse IDL files."));
    
    // Create compiler and compile layout
    #[cfg(feature = "anchor")]
    let layout = {
        let compiler = SolanaLayoutCompiler::new();
        compiler.compile_from_idl(&idl_content)?
    };
    
    #[cfg(not(feature = "anchor"))]
    let layout: traverse_solana::layout::SolanaLayout = unreachable!();
    
    // Format output based on requested format
    let output_str = match format {
        OutputFormat::Traverse => serde_json::to_string_pretty(&layout)?,
        OutputFormat::CoprocessorJson => {
            let simplified = serde_json::json!({
                "program_id": layout.program_id,
                "accounts": layout.accounts.len(),
                "instructions": layout.instructions.len(),
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
                "program_id": layout.program_id,
                "accounts": layout.accounts.len(),
                "instructions": layout.instructions.len()
            });
            toml::to_string_pretty(&simplified)?
        }
        OutputFormat::Binary => {
            let binary_data = bincode::serialize(&layout)?;
            format!("Binary layout: {} bytes\nBase64: {}", binary_data.len(), BASE64.encode(&binary_data))
        }
        OutputFormat::Base64 => {
            let binary_data = bincode::serialize(&layout)?;
            BASE64.encode(&binary_data)
        }
    };
    
    write_output(&output_str, output)?;
    
    println!("✓ Storage layout compiled");
    println!("  - Program ID: {}", layout.program_id);
    println!("  - Accounts: {}", layout.accounts.len());
    println!("  - Instructions: {}", layout.instructions.len());
    
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
    
    // Check if the layout file exists before attempting to read it
    if !layout_file.exists() {
        return Err(anyhow::anyhow!(
            "Layout file does not exist: {}",
            layout_file.display()
        ));
    }
    
    // Load layout
    let layout_content = std::fs::read_to_string(layout_file)
        .map_err(|e| anyhow::anyhow!("Failed to read layout file '{}': {}", layout_file.display(), e))?;
    let layout: traverse_core::LayoutInfo = serde_json::from_str(&layout_content)
        .map_err(|e| anyhow::anyhow!("Failed to parse layout file '{}': {}", layout_file.display(), e))?;
    
    // Parse key list
    let key_list: Vec<&str> = state_keys.split(',').map(|k| k.trim()).collect();
    
    // Generate queries
    let mut queries = Vec::new();
    for key in &key_list {
        if let Some(entry) = layout.storage.iter().find(|e| e.label == *key) {
            let mut query = serde_json::json!({
                "account": key,
                "type": entry.type_name,
                "slot": entry.slot,
                "offset": entry.offset
            });
            
            if include_examples {
                query["example_queries"] = match entry.type_name.as_str() {
                    t if t.contains("pubkey") => {
                        serde_json::json!([
                            format!("{}[9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM]", key),
                            format!("{}[11111111111111111111111111111112]", key)
                        ])
                    }
                    t if t.contains("array") => {
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
    
    // Check if the layout file exists before attempting to read it
    if !layout_file.exists() {
        return Err(anyhow::anyhow!(
            "Layout file does not exist: {}",
            layout_file.display()
        ));
    }
    
    // Load layout
    let layout_content = std::fs::read_to_string(layout_file)
        .map_err(|e| anyhow::anyhow!("Failed to read layout file '{}': {}", layout_file.display(), e))?;
    let _layout: traverse_core::LayoutInfo = serde_json::from_str(&layout_content)
        .map_err(|e| anyhow::anyhow!("Failed to parse layout file '{}': {}", layout_file.display(), e))?;
    
    // Create resolver
    use traverse_solana::SolanaKeyResolver;
    
    let resolver = SolanaKeyResolver::new();
    let parsed_query = SolanaKeyResolver::parse_query(query)?;
    let resolved_address = resolver.resolve_account_address(&parsed_query)?;
    
    // Create a resolved structure for compatibility
    let resolved = serde_json::json!({
        "address": resolved_address,
        "query": query
    });
    
    let output_str = match format {
        OutputFormat::Traverse => serde_json::to_string_pretty(&resolved)?,
        OutputFormat::CoprocessorJson => {
            let coprocessor_format = serde_json::json!({
                "query": query,
                "resolved_address": resolved["address"].as_str().unwrap_or(""),
                "layout_commitment": "not_implemented",
                "field_size": 0,
                "offset": 0
            });
            serde_json::to_string_pretty(&coprocessor_format)?
        }
        OutputFormat::Toml => {
            let simplified = serde_json::json!({
                "query": query,
                "account_key": resolved["address"].as_str().unwrap_or(""),
                "layout_commitment": "not_implemented"
            });
            toml::to_string_pretty(&simplified)?
        }
        OutputFormat::Binary => {
            // For binary/base64, we need to serialize the JSON value
            let binary_data = resolved.to_string().as_bytes().to_vec();
            format!("Binary query result: {} bytes\nBase64: {}", binary_data.len(), BASE64.encode(&binary_data))
        }
        OutputFormat::Base64 => {
            // For binary/base64, we need to serialize the JSON value
            let binary_data = resolved.to_string().as_bytes().to_vec();
            BASE64.encode(&binary_data)
        }
    };
    
    write_output(&output_str, output)?;
    
    println!("✓ Query resolved");
    println!("  - Query: {}", query);
    println!("  - Account address: {}", resolved["address"].as_str().unwrap_or(""));
    
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
    
    // Check if the IDL file exists before attempting to process it
    if !idl_file.exists() {
        return Err(anyhow::anyhow!(
            "IDL file does not exist: {}",
            idl_file.display()
        ));
    }
    
    // Validate basic parameters
    if rpc.trim().is_empty() {
        return Err(anyhow::anyhow!(
            "RPC URL cannot be empty. Please provide a valid Solana RPC endpoint."
        ));
    }
    
    if program_address.trim().is_empty() {
        return Err(anyhow::anyhow!(
            "Program address cannot be empty. Please provide a valid Solana program address."
        ));
    }
    
    if queries.trim().is_empty() {
        return Err(anyhow::anyhow!(
            "Queries cannot be empty. Please provide comma-separated query patterns."
        ));
    }
    
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
    for query in &query_list {
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
#[cfg(feature = "anchor")]
fn validate_idl_schema(idl: &traverse_solana::anchor::SolanaIdl) -> Result<()> {
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
        
        use traverse_solana::anchor::IdlAccountType;
        match &account.account_type {
            IdlAccountType::Struct { fields } => {
                if fields.is_empty() {
                    return Err(anyhow::anyhow!("Account '{}' has no fields", account.name));
                }
                
                // Check for duplicate field names
                let mut field_names = std::collections::HashSet::new();
                for field in fields {
                    if field.name.is_empty() {
                        return Err(anyhow::anyhow!("Account '{}' has field with empty name", account.name));
                    }
                    
                    if !field_names.insert(&field.name) {
                        return Err(anyhow::anyhow!("Account '{}' has duplicate field '{}'", account.name, field.name));
                    }
                }
            }
            IdlAccountType::Enum { variants } => {
                if variants.is_empty() {
                    return Err(anyhow::anyhow!("Account '{}' enum has no variants", account.name));
                }
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

#[cfg(not(feature = "anchor"))]
fn validate_idl_schema(_idl: &()) -> Result<()> {
    Err(anyhow::anyhow!("Anchor support not enabled"))
}

#[cfg(all(test, feature = "anchor"))]
mod tests {
    use super::*;

    #[cfg(feature = "solana")]
    #[test]
    fn test_validate_idl_schema_valid() {
        use traverse_solana::anchor::{SolanaIdl, IdlField, IdlAccount, IdlInstruction, IdlAccountType};

        let idl = SolanaIdl {
            program_id: "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM".to_string(),
            version: "0.1.0".to_string(),
            name: "test_program".to_string(),
            accounts: vec![
                IdlAccount {
                    name: "UserAccount".to_string(),
                    discriminator: None,
                    account_type: IdlAccountType::Struct {
                        fields: vec![
                            IdlField {
                            name: "authority".to_string(),
                            field_type: "publicKey".to_string(),
                        },
                        IdlField {
                            name: "balance".to_string(),
                            field_type: "u64".to_string(),
                        },
                    ],
                    },
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
        use traverse_solana::anchor::{SolanaIdl, IdlField, IdlAccount, IdlAccountType};

        let idl = SolanaIdl {
            program_id: "".to_string(), // Empty program ID
            version: "0.1.0".to_string(),
            name: "test_program".to_string(),
            accounts: vec![
                IdlAccount {
                    name: "UserAccount".to_string(),
                    discriminator: None,
                    account_type: IdlAccountType::Struct {
                        fields: vec![
                            IdlField {
                            name: "authority".to_string(),
                            field_type: "publicKey".to_string(),
                        },
                    ],
                    },
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
        use traverse_solana::anchor::{SolanaIdl, IdlField, IdlAccount, IdlAccountType};

        let idl = SolanaIdl {
            program_id: "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM".to_string(),
            version: "0.1.0".to_string(),
            name: "test_program".to_string(),
            accounts: vec![
                IdlAccount {
                    name: "UserAccount".to_string(),
                    discriminator: None,
                    account_type: IdlAccountType::Struct {
                        fields: vec![
                            IdlField {
                            name: "authority".to_string(),
                            field_type: "publicKey".to_string(),
                        },
                        IdlField {
                            name: "authority".to_string(), // Duplicate field name
                            field_type: "u64".to_string(),
                        },
                    ],
                    },
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

    #[cfg(feature = "solana")]
    #[tokio::test]
    async fn test_file_existence_checks() {
        use std::path::Path;
        use tempfile::{NamedTempFile, TempDir};
        
        // Test missing IDL file in analyze_program
        let result = cmd_solana_analyze_program(
            Path::new("nonexistent.idl.json"),
            None,
            false,
        ).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("IDL file does not exist"));

        // Test missing IDL file in compile_layout
        let result = cmd_solana_compile_layout(
            Path::new("nonexistent.idl.json"),
            None,
            &traverse_core::OutputFormat::Json,
        ).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("IDL file does not exist"));

        // Test missing layout file in generate_queries
        let result = cmd_solana_generate_queries(
            Path::new("nonexistent_layout.json"),
            "balance",
            None,
            false,
        ).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Layout file does not exist"));

        // Test missing layout file in resolve_query
        let result = cmd_solana_resolve_query(
            "balance",
            Path::new("nonexistent_layout.json"),
            &traverse_core::OutputFormat::Json,
            None,
        ).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Layout file does not exist"));

        // Test with valid temporary files (should not crash, though may fail for other reasons)
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        
        // Create a minimal valid IDL file
        let idl = serde_json::json!({
            "version": "0.1.0",
            "name": "test_program",
            "programId": "11111111111111111111111111111111",
            "instructions": [],
            "accounts": [],
            "types": [],
            "events": [],
            "errors": [],
            "constants": []
        });
        
        let mut temp_idl = NamedTempFile::new().expect("Failed to create temp IDL file");
        std::fs::write(temp_idl.path(), serde_json::to_string_pretty(&idl).unwrap())
            .expect("Failed to write temp IDL file");

        // Test analyze_program with valid file (should not crash)
        let result = cmd_solana_analyze_program(
            temp_idl.path(),
            None,
            false, // Don't validate schema to avoid other validation errors
        ).await;
        // Should not fail due to file existence issues
        assert!(result.is_ok() || !result.unwrap_err().to_string().contains("IDL file does not exist"));

        // Create a minimal valid layout file for other tests
        let layout = serde_json::json!({
            "contract_name": "test_program",
            "entries": [],
            "types": [],
            "semantic_entries": []
        });
        
        let mut temp_layout = NamedTempFile::new().expect("Failed to create temp layout file");
        std::fs::write(temp_layout.path(), serde_json::to_string_pretty(&layout).unwrap())
            .expect("Failed to write temp layout file");

        // Test generate_queries with valid file (should not crash)
        let result = cmd_solana_generate_queries(
            temp_layout.path(),
            "balance",
            None,
            false,
        ).await;
        // Should not fail due to file existence issues
        assert!(result.is_ok() || !result.unwrap_err().to_string().contains("Layout file does not exist"));
    }

    #[cfg(feature = "solana")]
    #[tokio::test]
    async fn test_auto_generate_parameter_validation() {
        use std::path::Path;
        use tempfile::{NamedTempFile, TempDir};
        
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let output_dir = temp_dir.path().join("output");
        
        // Create a minimal valid IDL file
        let idl = serde_json::json!({
            "version": "0.1.0",
            "name": "test_program",
            "programId": "11111111111111111111111111111111",
            "instructions": [],
            "accounts": [],
            "types": [],
            "events": [],
            "errors": [],
            "constants": []
        });
        let mut temp_idl = NamedTempFile::new().expect("Failed to create temp IDL file");
        std::fs::write(temp_idl.path(), serde_json::to_string_pretty(&idl).unwrap())
            .expect("Failed to write temp IDL file");

        // Test missing IDL file
        let result = cmd_solana_auto_generate(
            Path::new("nonexistent.idl.json"),
            "https://api.mainnet-beta.solana.com",
            "11111111111111111111111111111111",
            "balance",
            &output_dir,
            true, // dry run
        ).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("IDL file does not exist"));

        // Test empty RPC
        let result = cmd_solana_auto_generate(
            temp_idl.path(),
            "",
            "11111111111111111111111111111111",
            "balance",
            &output_dir,
            true,
        ).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("RPC URL cannot be empty"));

        // Test empty program address
        let result = cmd_solana_auto_generate(
            temp_idl.path(),
            "https://api.mainnet-beta.solana.com",
            "",
            "balance",
            &output_dir,
            true,
        ).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Program address cannot be empty"));

        // Test empty queries
        let result = cmd_solana_auto_generate(
            temp_idl.path(),
            "https://api.mainnet-beta.solana.com",
            "11111111111111111111111111111111",
            "",
            &output_dir,
            true,
        ).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Queries cannot be empty"));

        // Test valid parameters (dry run should succeed)
        let result = cmd_solana_auto_generate(
            temp_idl.path(),
            "https://api.mainnet-beta.solana.com",
            "11111111111111111111111111111111",
            "balance",
            &output_dir,
            true, // dry run
        ).await;
        // Should succeed in dry run mode (or fail for other reasons, not parameter validation)
        assert!(
            result.is_ok() || 
            (!result.as_ref().unwrap_err().to_string().contains("IDL file does not exist") &&
             !result.as_ref().unwrap_err().to_string().contains("cannot be empty"))
        );
    }
}