//! Solana-specific CLI binary
//!
//! This binary provides CLI access to Solana-specific functionality while using
//! the shared core CLI infrastructure. It only imports Solana dependencies to
//! avoid the k256 version conflict with Ethereum.

use clap::{Parser, Subcommand};
use serde_json::{json, Value};
use std::process;
use traverse_cli_core::{CommonArgs, CliResult, CliUtils, OutputFormat};

// Note: traverse_solana imports are used conditionally in commands.rs

mod commands;

/// Solana-specific CLI arguments
#[derive(Parser)]
#[command(name = "traverse-solana")]
#[command(about = "Solana ZK storage path generator")]
#[command(version)]
struct SolanaArgs {
    #[command(flatten)]
    common: CommonArgs,
    
    #[command(subcommand)]
    command: SolanaCommand,
}

/// Solana-specific commands
#[derive(Subcommand)]
enum SolanaCommand {
    /// Analyze a Solana program IDL
    AnalyzeProgram {
        /// Path to the IDL file
        idl_file: String,
        /// Program ID (optional)
        #[arg(long)]
        program_id: Option<String>,
        /// Enable deep analysis
        #[arg(long)]
        deep: bool,
    },
    
    /// Compile Solana account layout
    CompileLayout {
        /// Input IDL file path
        input: String,
        /// Output layout file path
        #[arg(short, long)]
        output: Option<String>,
    },
    
    /// Generate Solana account queries
    GenerateQueries {
        /// Layout file path
        layout: String,
        /// Query patterns
        patterns: Vec<String>,
    },
    
    /// Resolve Solana account query
    ResolveQuery {
        /// Query string to resolve
        query: String,
        /// Layout file path
        #[arg(short, long)]
        layout: String,
        /// Program ID
        #[arg(long)]
        program_id: Option<String>,
    },
    
    /// Generate Solana account proof
    GenerateProof {
        /// Account address
        #[arg(long)]
        account: String,
        /// Account query
        #[arg(long)]
        query: String,
        /// RPC endpoint
        #[arg(long)]
        rpc: String,
        /// Program ID
        #[arg(long)]
        program_id: String,
    },
    
    /// Auto-generate for Solana programs
    AutoGenerate {
        /// Configuration file path
        config: String,
        /// Output directory
        #[arg(short, long, default_value = "output")]
        output_dir: String,
    },
}

#[cfg(feature = "solana")]
async fn analyze_program(idl_file: &str, program_id: Option<&str>, deep: bool) -> CliResult<Value> {
    use std::path::Path;
    
    // Call the command implementation
    let result = commands::cmd_solana_analyze_program(
        Path::new(idl_file),
        None, // output handled by caller
        deep, // validate_schema
    ).await;
    
    match result {
        Ok(()) => {
            // For now, return a placeholder result since the command writes to output
            Ok(json!({
                "status": "success",
                "file": idl_file,
                "program_id": program_id.unwrap_or("not_specified"),
                "analyzed": true
            }))
        }
        Err(e) => Err(traverse_cli_core::CliError::Processing(e.to_string()))
    }
}

#[cfg(feature = "solana")]
async fn compile_layout(input: &str, output: Option<&str>) -> CliResult<()> {
    use std::path::Path;
    
    // Call the command implementation
    let result = commands::cmd_solana_compile_layout(
        Path::new(input),
        output.map(Path::new),
        &OutputFormat::Traverse,
    ).await;
    
    match result {
        Ok(()) => Ok(()),
        Err(e) => Err(traverse_cli_core::CliError::Processing(e.to_string()))
    }
}

#[cfg(feature = "solana")]
async fn resolve_query(query: &str, layout_file: &str, program_id: Option<&str>) -> CliResult<Value> {
    use std::path::Path;
    
    // Call the command implementation
    let result = commands::cmd_solana_resolve_query(
        query,
        Path::new(layout_file),
        &OutputFormat::CoprocessorJson,
        None, // output handled by caller
    ).await;
    
    match result {
        Ok(()) => {
            // For now, return a placeholder result since the command writes to output
            Ok(json!({
                "status": "success",
                "query": query,
                "layout_file": layout_file,
                "program_id": program_id.unwrap_or("not_specified"),
                "resolved": true
            }))
        }
        Err(e) => Err(traverse_cli_core::CliError::Processing(e.to_string()))
    }
}

#[cfg(not(feature = "solana"))]
async fn analyze_program(_idl_file: &str, _program_id: Option<&str>, _deep: bool) -> CliResult<Value> {
    Err(traverse_cli_core::CliError::Configuration(
        "Solana support not enabled. Build with --features solana".to_string()
    ))
}

#[cfg(not(feature = "solana"))]
async fn compile_layout(_input: &str, _output: Option<&str>) -> CliResult<()> {
    Err(traverse_cli_core::CliError::Configuration(
        "Solana support not enabled. Build with --features solana".to_string()
    ))
}

#[cfg(not(feature = "solana"))]
async fn resolve_query(_query: &str, _layout_file: &str, _program_id: Option<&str>) -> CliResult<Value> {
    Err(traverse_cli_core::CliError::Configuration(
        "Solana support not enabled. Build with --features solana".to_string()
    ))
}

async fn handle_command(args: SolanaArgs) -> CliResult<()> {
    // Set verbose mode
    if args.common.verbose {
        std::env::set_var("VERBOSE", "1");
    }
    
    match args.command {
        SolanaCommand::AnalyzeProgram { idl_file, program_id, deep } => {
            let result = analyze_program(&idl_file, program_id.as_deref(), deep).await?;
            let output = CliUtils::format_json(&result, &args.common.format)?;
            CliUtils::write_output(&output, args.common.output.as_deref())?;
        }
        
        SolanaCommand::CompileLayout { input, output } => {
            compile_layout(&input, output.as_deref()).await?;
        }
        
        SolanaCommand::GenerateQueries { layout, patterns } => {
            let result = json!({
                "layout": layout,
                "patterns": patterns,
                "note": "Query generation implementation would go here"
            });
            let output = CliUtils::format_json(&result, &args.common.format)?;
            CliUtils::write_output(&output, args.common.output.as_deref())?;
        }
        
        SolanaCommand::ResolveQuery { query, layout, program_id } => {
            let result = resolve_query(&query, &layout, program_id.as_deref()).await?;
            let output = CliUtils::format_json(&result, &args.common.format)?;
            CliUtils::write_output(&output, args.common.output.as_deref())?;
        }
        
        SolanaCommand::GenerateProof { account, query, rpc, program_id } => {
            let result = json!({
                "account": account,
                "query": query,
                "rpc": rpc,
                "program_id": program_id,
                "note": "Proof generation implementation would go here"
            });
            let output = CliUtils::format_json(&result, &args.common.format)?;
            CliUtils::write_output(&output, args.common.output.as_deref())?;
        }
        
        SolanaCommand::AutoGenerate { config, output_dir } => {
            let _config_data = CliUtils::load_config(&config)?;
            CliUtils::ensure_output_dir(&output_dir)?;
            
            let result = json!({
                "config": config,
                "output_dir": output_dir,
                "note": "Auto-generation implementation would go here"
            });
            let output = CliUtils::format_json(&result, &args.common.format)?;
            CliUtils::write_output(&output, args.common.output.as_deref())?;
        }
    }
    
    Ok(())
}

#[tokio::main]
async fn main() {
    let args = SolanaArgs::parse();
    
    if let Err(e) = handle_command(args).await {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
} 