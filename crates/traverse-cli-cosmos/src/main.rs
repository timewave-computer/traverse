//! Cosmos-specific CLI binary
//!
//! This binary provides CLI access to Cosmos-specific functionality while using
//! the shared core CLI infrastructure. It only imports Cosmos dependencies to
//! avoid the k256 version conflict with Ethereum.

use clap::{Parser, Subcommand};
use serde_json::{json, Value};
use std::process;
use traverse_cli_core::{CommonArgs, CliResult, CliUtils, OutputFormat};

#[cfg(feature = "cosmos")]
use traverse_cosmos::{
    CosmosLayoutCompiler, CosmosKeyResolver, CosmosProofFetcher,
    CosmosError, contract::ContractSchema
};

mod commands;

/// Cosmos-specific CLI arguments
#[derive(Parser)]
#[command(name = "traverse-cosmos")]
#[command(about = "Cosmos ZK storage path generator")]
#[command(version)]
struct CosmosArgs {
    #[command(flatten)]
    common: CommonArgs,
    
    #[command(subcommand)]
    command: CosmosCommand,
}

/// Cosmos-specific commands
#[derive(Subcommand)]
enum CosmosCommand {
    /// Analyze a Cosmos contract schema
    AnalyzeContract {
        /// Path to the contract schema file
        schema_file: String,
        /// Contract address (optional)
        #[arg(long)]
        address: Option<String>,
        /// Enable deep analysis
        #[arg(long)]
        deep: bool,
    },
    
    /// Compile Cosmos contract layout
    CompileLayout {
        /// Input schema file path
        input: String,
        /// Output layout file path
        #[arg(short, long)]
        output: Option<String>,
    },
    
    /// Generate Cosmos contract queries
    GenerateQueries {
        /// Layout file path
        layout: String,
        /// Query patterns
        patterns: Vec<String>,
    },
    
    /// Resolve Cosmos contract query
    ResolveQuery {
        /// Query string to resolve
        query: String,
        /// Layout file path
        #[arg(short, long)]
        layout: String,
        /// Contract address
        #[arg(long)]
        address: Option<String>,
    },
    
    /// Generate Cosmos contract proof
    GenerateProof {
        /// Contract address
        #[arg(long)]
        address: String,
        /// Contract query
        #[arg(long)]
        query: String,
        /// RPC endpoint
        #[arg(long)]
        rpc: String,
        /// Chain ID
        #[arg(long)]
        chain_id: String,
    },
    
    /// Auto-generate for Cosmos contracts
    AutoGenerate {
        /// Configuration file path
        config: String,
        /// Output directory
        #[arg(short, long, default_value = "output")]
        output_dir: String,
    },
}

#[cfg(feature = "cosmos")]
async fn analyze_contract(schema_file: &str, address: Option<&str>, deep: bool) -> CliResult<Value> {
    use std::path::Path;
    
    // Call the command implementation
    let result = commands::cmd_cosmos_analyze_contract(
        Path::new(schema_file),
        None, // output handled by caller
        deep, // validate_schema
        address,
        None, // rpc
    ).await;
    
    match result {
        Ok(()) => {
            // For now, return a placeholder result since the command writes to output
            Ok(json!({
                "status": "success",
                "file": schema_file,
                "address": address.unwrap_or("not_specified"),
                "analyzed": true
            }))
        }
        Err(e) => Err(traverse_cli_core::CliError::Processing(e.to_string()))
    }
}

#[cfg(feature = "cosmos")]
fn compile_layout(input: &str, output: Option<&str>) -> CliResult<()> {
    use std::path::Path;
    
    // Call the command implementation
    let result = commands::cmd_cosmos_compile_layout(
        Path::new(input),
        output.map(Path::new),
        &OutputFormat::Traverse,
    );
    
    match result {
        Ok(()) => Ok(()),
        Err(e) => Err(traverse_cli_core::CliError::Processing(e.to_string()))
    }
}

#[cfg(feature = "cosmos")]
fn resolve_query(query: &str, layout_file: &str, address: Option<&str>) -> CliResult<Value> {
    use std::path::Path;
    
    // Call the command implementation
    let result = commands::cmd_cosmos_resolve_query(
        query,
        Path::new(layout_file),
        &OutputFormat::CoprocessorJson,
        None, // output handled by caller
    );
    
    match result {
        Ok(()) => {
            // For now, return a placeholder result since the command writes to output
            Ok(json!({
                "status": "success",
                "query": query,
                "layout_file": layout_file,
                "address": address.unwrap_or("not_specified"),
                "resolved": true
            }))
        }
        Err(e) => Err(traverse_cli_core::CliError::Processing(e.to_string()))
    }
}

#[cfg(not(feature = "cosmos"))]
fn analyze_contract(_schema_file: &str, _address: Option<&str>, _deep: bool) -> CliResult<Value> {
    Err(traverse_cli_core::CliError::Configuration(
        "Cosmos support not enabled. Build with --features cosmos".to_string()
    ))
}

#[cfg(not(feature = "cosmos"))]
fn compile_layout(_input: &str, _output: Option<&str>) -> CliResult<()> {
    Err(traverse_cli_core::CliError::Configuration(
        "Cosmos support not enabled. Build with --features cosmos".to_string()
    ))
}

#[cfg(not(feature = "cosmos"))]
fn resolve_query(_query: &str, _layout_file: &str, _address: Option<&str>) -> CliResult<Value> {
    Err(traverse_cli_core::CliError::Configuration(
        "Cosmos support not enabled. Build with --features cosmos".to_string()
    ))
}

async fn handle_command(args: CosmosArgs) -> CliResult<()> {
    // Set verbose mode
    if args.common.verbose {
        std::env::set_var("VERBOSE", "1");
    }
    
    match args.command {
        CosmosCommand::AnalyzeContract { schema_file, address, deep } => {
            let result = analyze_contract(&schema_file, address.as_deref(), deep).await?;
            let output = CliUtils::format_json(&result, &args.common.format)?;
            CliUtils::write_output(&output, args.common.output.as_deref())?;
        }
        
        CosmosCommand::CompileLayout { input, output } => {
            compile_layout(&input, output.as_deref())?;
        }
        
        CosmosCommand::GenerateQueries { layout, patterns } => {
            let result = json!({
                "layout": layout,
                "patterns": patterns,
                "note": "Query generation implementation would go here"
            });
            let output = CliUtils::format_json(&result, &args.common.format)?;
            CliUtils::write_output(&output, args.common.output.as_deref())?;
        }
        
        CosmosCommand::ResolveQuery { query, layout, address } => {
            let result = resolve_query(&query, &layout, address.as_deref())?;
            let output = CliUtils::format_json(&result, &args.common.format)?;
            CliUtils::write_output(&output, args.common.output.as_deref())?;
        }
        
        CosmosCommand::GenerateProof { address, query, rpc, chain_id } => {
            let result = json!({
                "address": address,
                "query": query,
                "rpc": rpc,
                "chain_id": chain_id,
                "note": "Proof generation implementation would go here"
            });
            let output = CliUtils::format_json(&result, &args.common.format)?;
            CliUtils::write_output(&output, args.common.output.as_deref())?;
        }
        
        CosmosCommand::AutoGenerate { config, output_dir } => {
            let config_data = CliUtils::load_config(&config)?;
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
    let args = CosmosArgs::parse();
    
    if let Err(e) = handle_command(args).await {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
} 