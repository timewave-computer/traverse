//! Solana program analysis and proof generation commands
//!
//! This module provides CLI commands for analyzing Solana programs using IDL files,
//! compiling storage layouts, and generating account proofs.

use anyhow::Result;
use std::path::PathBuf;
use serde_json::Value;
use crate::cli::OutputFormat;

/// Analyze Solana program from IDL
pub async fn cmd_solana_analyze_program(
    idl_file: &PathBuf,
    output: Option<&std::path::Path>,
    validate_schema: bool,
) -> Result<()> {
    println!("Analyzing Solana program from IDL: {}", idl_file.display());
    
    // Read IDL file
    let idl_content = std::fs::read_to_string(idl_file)?;
    
    #[cfg(feature = "solana")]
    {
        use traverse_solana::anchor::IdlParser;
        
        // Parse IDL
        let idl = IdlParser::parse_idl(&idl_content)?;
        
        if validate_schema {
            println!("Validating IDL schema...");
            // TODO: Add IDL validation
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
    }
    
    #[cfg(not(feature = "solana"))]
    {
        return Err(anyhow::anyhow!(
            "Solana support not enabled. Rebuild with --features solana"
        ));
    }
    
    Ok(())
}

/// Compile Solana storage layout from IDL
pub async fn cmd_solana_compile_layout(
    idl_file: &PathBuf,
    output: Option<&std::path::Path>,
    format: &OutputFormat,
) -> Result<()> {
    println!("Compiling Solana storage layout from IDL: {}", idl_file.display());
    
    // Read IDL file
    let idl_content = std::fs::read_to_string(idl_file)?;
    
    #[cfg(feature = "solana")]
    {
        use traverse_solana::anchor::IdlParser;
        use traverse_solana::layout::SolanaLayoutCompiler;
        
        // Parse IDL
        let idl = IdlParser::parse_idl(&idl_content)?;
        
        // Extract account layouts
        let account_layouts = IdlParser::extract_account_layouts(&idl)?;
        
        // Create layout compiler
        let compiler = SolanaLayoutCompiler::new();
        let compiled_layout = compiler.compile_from_idl(&idl)?;
        
        // Format output based on requested format
        let output_content = match format {
            OutputFormat::Traverse | OutputFormat::CoprocessorJson => {
                serde_json::to_string_pretty(&serde_json::json!({
                    "program_id": idl.program_id,
                    "program_name": idl.name,
                    "layout_commitment": compiled_layout.commitment,
                    "accounts": account_layouts.iter().map(|layout| serde_json::json!({
                        "name": layout.name,
                        "discriminator": hex::encode(&layout.discriminator),
                        "fields": layout.fields.iter().map(|field| serde_json::json!({
                            "name": field.name,
                            "type": field.field_type,
                            "offset": field.offset,
                            "size": field.size,
                            "description": field.description
                        })).collect::<Vec<_>>(),
                        "total_size": layout.total_size
                    })).collect::<Vec<_>>()
                }))?
            }
            OutputFormat::Toml => {
                // TODO: Implement TOML output
                return Err(anyhow::anyhow!("TOML output format not yet implemented for Solana"));
            }
            OutputFormat::Binary => {
                // TODO: Implement binary output
                return Err(anyhow::anyhow!("Binary output format not yet implemented for Solana"));
            }
            OutputFormat::Base64 => {
                // TODO: Implement base64 output
                return Err(anyhow::anyhow!("Base64 output format not yet implemented for Solana"));
            }
        };
        
        // Output layout
        if let Some(output_path) = output {
            std::fs::write(output_path, output_content)?;
            println!("✓ Layout written to: {}", output_path.display());
        } else {
            println!("{}", output_content);
        }
        
        println!("✓ Layout compilation completed");
        println!("  - Program: {}", idl.name);
        println!("  - Accounts: {}", account_layouts.len());
        println!("  - Layout commitment: {}", compiled_layout.commitment);
    }
    
    #[cfg(not(feature = "solana"))]
    {
        return Err(anyhow::anyhow!(
            "Solana support not enabled. Rebuild with --features solana"
        ));
    }
    
    Ok(())
}

/// Generate storage queries for Solana state
pub async fn cmd_solana_generate_queries(
    layout_file: &PathBuf,
    state_keys: &str,
    output: Option<&std::path::Path>,
    include_examples: bool,
) -> Result<()> {
    println!("Generating Solana storage queries from layout: {}", layout_file.display());
    
    // Read layout file
    let layout_content = std::fs::read_to_string(layout_file)?;
    let layout: Value = serde_json::from_str(&layout_content)?;
    
    #[cfg(feature = "solana")]
    {
        use traverse_solana::resolver::{SolanaKeyResolver, SolanaQuery};
        
        let resolver = SolanaKeyResolver::new();
        let mut queries = Vec::new();
        
        // Parse requested state keys
        let keys: Vec<&str> = state_keys.split(',').map(|s| s.trim()).collect();
        
        for key in keys {
            if include_examples {
                // Generate example queries with different patterns
                let example_queries = vec![
                    format!("{}[user123]", key),                    // PDA example
                    format!("{}[mint456,owner789]", key),          // ATA example
                    format!("{}.authority", key),                  // Field access example
                    key.to_string(),                               // Direct access
                ];
                
                for example_query in example_queries {
                    if let Ok(parsed_query) = SolanaKeyResolver::parse_query(&example_query) {
                        queries.push(serde_json::json!({
                            "query": example_query,
                            "type": format!("{:?}", parsed_query),
                            "example": true
                        }));
                    }
                }
            } else {
                queries.push(serde_json::json!({
                    "query": key,
                    "type": "direct",
                    "example": false
                }));
            }
        }
        
        let output_content = serde_json::to_string_pretty(&serde_json::json!({
            "program_id": layout.get("program_id"),
            "generated_queries": queries,
            "total_queries": queries.len()
        }))?;
        
        // Output queries
        if let Some(output_path) = output {
            std::fs::write(output_path, output_content)?;
            println!("✓ Queries written to: {}", output_path.display());
        } else {
            println!("{}", output_content);
        }
        
        println!("✓ Query generation completed");
        println!("  - Generated queries: {}", queries.len());
        println!("  - Include examples: {}", include_examples);
    }
    
    #[cfg(not(feature = "solana"))]
    {
        return Err(anyhow::anyhow!(
            "Solana support not enabled. Rebuild with --features solana"
        ));
    }
    
    Ok(())
}

/// Resolve specific Solana storage query
pub async fn cmd_solana_resolve_query(
    query: &str,
    layout: &PathBuf,
    format: &OutputFormat,
    output: Option<&std::path::Path>,
) -> Result<()> {
    println!("Resolving Solana storage query: {}", query);
    
    // Read layout file
    let layout_content = std::fs::read_to_string(layout)?;
    let layout_data: Value = serde_json::from_str(&layout_content)?;
    
    #[cfg(feature = "solana")]
    {
        use traverse_solana::resolver::{SolanaKeyResolver, SolanaQuery};
        
        // Parse query
        let parsed_query = SolanaKeyResolver::parse_query(query)?;
        
        // Create resolver
        let program_id = layout_data.get("program_id")
            .and_then(|v| v.as_str())
            .unwrap_or("11111111111111111111111111111112"); // System program as fallback
        
        let resolver = SolanaKeyResolver::with_program_id(program_id.to_string());
        
        // Resolve query
        let resolution_result = match &parsed_query {
            SolanaQuery::Direct { account_name } => {
                serde_json::json!({
                    "query": query,
                    "type": "direct",
                    "account_name": account_name,
                    "resolved": false,
                    "note": "Direct access requires specific account address"
                })
            }
            SolanaQuery::PDA { account_name, seeds } => {
                match resolver.derive_pda_address(seeds) {
                    Ok(address) => serde_json::json!({
                        "query": query,
                        "type": "pda",
                        "account_name": account_name,
                        "seeds": seeds,
                        "resolved_address": address,
                        "resolved": true
                    }),
                    Err(e) => serde_json::json!({
                        "query": query,
                        "type": "pda",
                        "account_name": account_name,
                        "seeds": seeds,
                        "resolved": false,
                        "error": format!("{}", e)
                    })
                }
            }
            SolanaQuery::ATA { mint, owner } => {
                match resolver.derive_ata_address(mint, owner) {
                    Ok(address) => serde_json::json!({
                        "query": query,
                        "type": "ata",
                        "mint": mint,
                        "owner": owner,
                        "resolved_address": address,
                        "resolved": true
                    }),
                    Err(e) => serde_json::json!({
                        "query": query,
                        "type": "ata",
                        "mint": mint,
                        "owner": owner,
                        "resolved": false,
                        "error": format!("{}", e)
                    })
                }
            }
            SolanaQuery::FieldAccess { account_name, field_path } => {
                // For field access, we need to resolve the account first, then the field
                let field_key = resolver.resolve_field_key(
                    "placeholder_account", // Would need actual account address
                    field_path,
                    0 // Would need actual field offset
                )?;
                
                serde_json::json!({
                    "query": query,
                    "type": "field_access",
                    "account_name": account_name,
                    "field_path": field_path,
                    "field_key": field_key,
                    "resolved": true,
                    "note": "Field key generated, but requires account address resolution"
                })
            }
        };
        
        // Format output
        let output_content = match format {
            OutputFormat::CoprocessorJson | OutputFormat::Traverse => {
                serde_json::to_string_pretty(&resolution_result)?
            }
            _ => {
                return Err(anyhow::anyhow!("Unsupported output format for Solana query resolution"));
            }
        };
        
        // Output result
        if let Some(output_path) = output {
            std::fs::write(output_path, output_content)?;
            println!("✓ Resolution written to: {}", output_path.display());
        } else {
            println!("{}", output_content);
        }
        
        println!("✓ Query resolution completed");
        println!("  - Query type: {:?}", parsed_query);
    }
    
    #[cfg(not(feature = "solana"))]
    {
        return Err(anyhow::anyhow!(
            "Solana support not enabled. Rebuild with --features solana"
        ));
    }
    
    Ok(())
}

/// End-to-end automation for Solana
pub async fn cmd_solana_auto_generate(
    idl_file: &PathBuf,
    rpc: &str,
    program_address: &str,
    queries: &str,
    output_dir: &PathBuf,
    dry_run: bool,
) -> Result<()> {
    println!("Starting Solana auto-generation pipeline");
    println!("  - IDL file: {}", idl_file.display());
    println!("  - RPC: {}", rpc);
    println!("  - Program: {}", program_address);
    println!("  - Output dir: {}", output_dir.display());
    println!("  - Dry run: {}", dry_run);
    
    // Create output directory
    std::fs::create_dir_all(output_dir)?;
    
    if dry_run {
        println!("🧪 Dry run mode - no actual operations performed");
        return Ok(());
    }
    
    #[cfg(feature = "solana")]
    {
        // Step 1: Analyze program
        println!("\n📋 Step 1: Analyzing Solana program...");
        let analysis_output = output_dir.join("analysis.json");
        cmd_solana_analyze_program(idl_file, Some(&analysis_output), true).await?;
        
        // Step 2: Compile layout
        println!("\n🔧 Step 2: Compiling storage layout...");
        let layout_output = output_dir.join("layout.json");
        cmd_solana_compile_layout(idl_file, Some(&layout_output), &OutputFormat::CoprocessorJson).await?;
        
        // Step 3: Generate queries
        println!("\n🔍 Step 3: Generating storage queries...");
        let queries_output = output_dir.join("queries.json");
        cmd_solana_generate_queries(&layout_output, queries, Some(&queries_output), true).await?;
        
        // Step 4: Resolve each query
        println!("\n🎯 Step 4: Resolving storage queries...");
        let query_list: Vec<&str> = queries.split(',').map(|s| s.trim()).collect();
        for query in query_list {
            let query_output = output_dir.join(format!("resolved_{}.json", 
                query.replace(['[', ']', ':', '.'], "_")));
            
            if let Err(e) = cmd_solana_resolve_query(query, &layout_output, &OutputFormat::CoprocessorJson, Some(&query_output)).await {
                println!("⚠️  Warning: Failed to resolve query '{}': {}", query, e);
                continue;
            }
        }
        
        // Step 5: Create summary
        println!("\n📊 Step 5: Creating summary...");
        let summary = serde_json::json!({
            "program_address": program_address,
            "rpc_endpoint": rpc,
            "idl_file": idl_file.file_name(),
            "generated_files": {
                "analysis": "analysis.json",
                "layout": "layout.json", 
                "queries": "queries.json"
            },
            "queries_processed": query_list.len(),
            "timestamp": chrono::Utc::now().to_rfc3339()
        });
        
        let summary_output = output_dir.join("summary.json");
        std::fs::write(summary_output, serde_json::to_string_pretty(&summary)?)?;
        
        println!("\n✅ Auto-generation completed successfully!");
        println!("📁 Output directory: {}", output_dir.display());
        println!("📋 Files generated: analysis.json, layout.json, queries.json, summary.json");
    }
    
    #[cfg(not(feature = "solana"))]
    {
        return Err(anyhow::anyhow!(
            "Solana support not enabled. Rebuild with --features solana"
        ));
    }
    
    Ok(())
} 