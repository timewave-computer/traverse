//! Ethereum-specific CLI binary
//!
//! This binary provides CLI access to Ethereum-specific functionality while using
//! the shared core CLI infrastructure. It only imports Ethereum dependencies to
//! avoid the k256 version conflict with Solana.

use clap::{Parser, Subcommand};
use serde_json::{json, Value};
use std::process;
use traverse_cli_core::{CommonArgs, CliResult, CliUtils, OutputFormat};

#[cfg(feature = "ethereum")]
use traverse_ethereum::{
    EthereumLayoutCompiler, EthereumKeyResolver, EthereumProofFetcher,
    EthereumError, abi::AbiInfo
};

mod commands;

/// Ethereum-specific CLI arguments
#[derive(Parser)]
#[command(name = "traverse-ethereum")]
#[command(about = "Ethereum ZK storage path generator")]
#[command(version)]
struct EthereumArgs {
    #[command(flatten)]
    common: CommonArgs,
    
    #[command(subcommand)]
    command: EthereumCommand,
}

/// Ethereum-specific commands
#[derive(Subcommand)]
enum EthereumCommand {
    /// Analyze an Ethereum contract ABI
    AnalyzeContract {
        /// Path to the ABI file
        abi_file: String,
        /// Contract address (optional)
        #[arg(long)]
        address: Option<String>,
        /// Enable deep analysis
        #[arg(long)]
        deep: bool,
    },
    
    /// Compile Ethereum storage layout
    CompileLayout {
        /// Input ABI file path
        input: String,
        /// Output layout file path
        #[arg(short, long)]
        output: Option<String>,
    },
    
    /// Generate Ethereum storage queries
    GenerateQueries {
        /// Layout file path
        layout: String,
        /// Query patterns
        patterns: Vec<String>,
    },
    
    /// Resolve Ethereum storage query
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
    
    /// Generate Ethereum proof
    GenerateProof {
        /// Contract address
        #[arg(long)]
        address: String,
        /// Storage slot query
        #[arg(long)]
        query: String,
        /// RPC endpoint
        #[arg(long)]
        rpc: String,
        /// Block number (latest if not specified)
        #[arg(long)]
        block: Option<u64>,
    },
    
    /// Auto-generate for Ethereum contracts
    AutoGenerate {
        /// Configuration file path
        config: String,
        /// Output directory
        #[arg(short, long, default_value = "output")]
        output_dir: String,
    },
}

#[cfg(feature = "ethereum")]
async fn analyze_contract(abi_file: &str, address: Option<&str>, deep: bool) -> CliResult<Value> {
    use std::path::Path;
    
    // Call the command implementation
    let result = commands::cmd_ethereum_analyze_contract(
        Path::new(abi_file),
        None, // output handled by caller
        deep, // validate_storage
        address,
        None, // rpc
    ).await;
    
    match result {
        Ok(()) => {
            // For now, return a placeholder result since the command writes to output
            Ok(json!({
                "status": "success",
                "file": abi_file,
                "analyzed": true
            }))
        }
        Err(e) => Err(traverse_cli_core::CliError::Processing(e.to_string()))
    }
}

#[cfg(feature = "ethereum")]
fn compile_layout(input: &str, output: Option<&str>) -> CliResult<()> {
    use std::path::Path;
    
    // Call the command implementation
    let result = commands::cmd_ethereum_compile_layout(
        Path::new(input),
        output.map(Path::new),
        &OutputFormat::Traverse,
        true, // validate
    );
    
    match result {
        Ok(()) => Ok(()),
        Err(e) => Err(traverse_cli_core::CliError::Processing(e.to_string()))
    }
}

#[cfg(feature = "ethereum")]
async fn resolve_query(query: &str, layout_file: &str, address: Option<&str>) -> CliResult<Value> {
    use std::path::Path;
    
    // Call the command implementation
    let result = commands::cmd_ethereum_resolve_query(
        query,
        Path::new(layout_file),
        &OutputFormat::CoprocessorJson,
        None, // output handled by caller
        address,
        None, // rpc
    ).await;
    
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

#[cfg(not(feature = "ethereum"))]
async fn analyze_contract(_abi_file: &str, _address: Option<&str>, _deep: bool) -> CliResult<Value> {
    Err(traverse_cli_core::CliError::Configuration(
        "Ethereum support not enabled. Build with --features ethereum".to_string()
    ))
}

#[cfg(not(feature = "ethereum"))]
fn compile_layout(_input: &str, _output: Option<&str>) -> CliResult<()> {
    Err(traverse_cli_core::CliError::Configuration(
        "Ethereum support not enabled. Build with --features ethereum".to_string()
    ))
}

#[cfg(not(feature = "ethereum"))]
async fn resolve_query(_query: &str, _layout_file: &str, _address: Option<&str>) -> CliResult<Value> {
    Err(traverse_cli_core::CliError::Configuration(
        "Ethereum support not enabled. Build with --features ethereum".to_string()
    ))
}

async fn handle_command(args: EthereumArgs) -> CliResult<()> {
    // Set verbose mode
    if args.common.verbose {
        std::env::set_var("VERBOSE", "1");
    }
    
    match args.command {
        EthereumCommand::AnalyzeContract { abi_file, address, deep } => {
            let result = analyze_contract(&abi_file, address.as_deref(), deep).await?;
            let output = CliUtils::format_json(&result, &args.common.format)?;
            CliUtils::write_output(&output, args.common.output.as_deref())?;
        }
        
        EthereumCommand::CompileLayout { input, output } => {
            compile_layout(&input, output.as_deref())?;
        }
        
        EthereumCommand::GenerateQueries { layout, patterns } => {
            let result = json!({
                "layout": layout,
                "patterns": patterns,
                "note": "Query generation implementation would go here"
            });
            let output = CliUtils::format_json(&result, &args.common.format)?;
            CliUtils::write_output(&output, args.common.output.as_deref())?;
        }
        
        EthereumCommand::ResolveQuery { query, layout, address } => {
            let result = resolve_query(&query, &layout, address.as_deref()).await?;
            let output = CliUtils::format_json(&result, &args.common.format)?;
            CliUtils::write_output(&output, args.common.output.as_deref())?;
        }
        
        EthereumCommand::GenerateProof { address, query, rpc, block } => {
            let result = json!({
                "address": address,
                "query": query,
                "rpc": rpc,
                "block": block,
                "note": "Proof generation implementation would go here"
            });
            let output = CliUtils::format_json(&result, &args.common.format)?;
            CliUtils::write_output(&output, args.common.output.as_deref())?;
        }
        
        EthereumCommand::AutoGenerate { config, output_dir } => {
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
    let args = EthereumArgs::parse();
    
    if let Err(e) = handle_command(args).await {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
} 