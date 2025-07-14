//! Cosmos-specific CLI binary
//!
//! This binary provides CLI access to Cosmos-specific functionality while using
//! the shared core CLI infrastructure. It only imports Cosmos dependencies to
//! avoid the k256 version conflict with Ethereum.

use anyhow::Result;
use clap::{Args, Command, Parser, Subcommand};
use serde_json::{json, Value};
use std::path::Path;
use traverse_core::OutputFormat;

#[cfg(feature = "cosmos")]
use traverse_cosmos::{
    CosmosLayoutCompiler, CosmosKeyResolver, CosmosProofFetcher,
    CosmosError, contract::ContractSchema
};

mod commands;

/// Cosmos-specific CLI arguments
#[derive(Debug, Parser)]
#[command(name = "traverse-cosmos")]
#[command(about = "Traverse CLI for Cosmos blockchain analysis")]
#[command(version)]
struct CosmosArgs {
    #[command(flatten)]
    common: CommonArgs,
    
    #[command(subcommand)]
    command: CosmosCommand,
}

#[derive(Debug, Args)]
struct CommonArgs {
    #[arg(long, global = true)]
    verbose: bool,
}

/// Cosmos-specific commands
#[derive(Debug, Subcommand)]
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

type CliResult<T> = Result<T, Box<dyn std::error::Error>>;

fn write_output(content: &str, output_path: Option<&str>) -> CliResult<()> {
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
                "schema_file": schema_file,
                "address": address,
                "deep_analysis": deep
            }))
        }
        Err(e) => Ok(json!({
            "status": "error",
            "error": e.to_string()
        }))
    }
}

#[cfg(feature = "cosmos")]
fn compile_layout(input: &str, output: Option<&str>) -> CliResult<()> {
    use std::path::Path;
    
    let output_path = output.map(Path::new);
    commands::cmd_cosmos_compile_layout(
        Path::new(input),
        output_path,
        &OutputFormat::Json,
    )?;
    
    Ok(())
}

#[cfg(feature = "cosmos")]
fn resolve_query(query: &str, layout_file: &str, address: Option<&str>) -> CliResult<Value> {
    use std::path::Path;
    
    let result = commands::cmd_cosmos_resolve_query(
        query,
        Path::new(layout_file),
        &OutputFormat::Json,
        None, // output handled by caller
    );
    
    match result {
        Ok(()) => Ok(json!({
            "status": "success",
            "query": query,
            "layout_file": layout_file,
            "address": address
        })),
        Err(e) => Ok(json!({
            "status": "error",
            "error": e.to_string()
        }))
    }
}

async fn handle_command(args: CosmosArgs) -> CliResult<()> {
    match args.command {
        CosmosCommand::AnalyzeContract { schema_file, address, deep } => {
            #[cfg(feature = "cosmos")]
            {
                let result = analyze_contract(&schema_file, address.as_deref(), deep).await?;
                println!("{}", serde_json::to_string_pretty(&result)?);
            }
            
            #[cfg(not(feature = "cosmos"))]
            {
                eprintln!("Error: Cosmos support not enabled.");
                eprintln!("This binary was built without Cosmos support.");
                eprintln!("Please use a build with the 'cosmos' feature enabled.");
                std::process::exit(1);
            }
        }
        
        CosmosCommand::CompileLayout { input, output } => {
            #[cfg(feature = "cosmos")]
            {
                compile_layout(&input, output.as_deref())?;
            }
            
            #[cfg(not(feature = "cosmos"))]
            {
                eprintln!("Error: Cosmos support not enabled.");
                eprintln!("This binary was built without Cosmos support.");
                eprintln!("Please use a build with the 'cosmos' feature enabled.");
                std::process::exit(1);
            }
        }
        
        CosmosCommand::GenerateQueries { layout, patterns } => {
            #[cfg(feature = "cosmos")]
            {
                let patterns_str = patterns.join(",");
                commands::cmd_cosmos_generate_queries(
                    Path::new(&layout),
                    &patterns_str,
                    None,
                    true, // include_examples
                )?;
            }
            
            #[cfg(not(feature = "cosmos"))]
            {
                eprintln!("Error: Cosmos support not enabled.");
                eprintln!("This binary was built without Cosmos support.");  
                eprintln!("Please use a build with the 'cosmos' feature enabled.");
                std::process::exit(1);
            }
        }
        
        CosmosCommand::ResolveQuery { query, layout, address } => {
            #[cfg(feature = "cosmos")]
            {
                let result = resolve_query(&query, &layout, address.as_deref())?;
                println!("{}", serde_json::to_string_pretty(&result)?);
            }
            
            #[cfg(not(feature = "cosmos"))]
            {
                eprintln!("Error: Cosmos support not enabled.");
                eprintln!("This binary was built without Cosmos support.");
                eprintln!("Please use a build with the 'cosmos' feature enabled.");
                std::process::exit(1);
            }
        }
        
        CosmosCommand::GenerateProof { address, query, rpc, chain_id } => {
            #[cfg(feature = "cosmos")]
            {
                println!("Generating proof for contract {} with query: {}", address, query);
                println!("RPC: {}, Chain ID: {}", rpc, chain_id);
                // Implementation would go here
            }
            
            #[cfg(not(feature = "cosmos"))]
            {
                eprintln!("Error: Cosmos support not enabled.");
                eprintln!("This binary was built without Cosmos support.");
                eprintln!("Please use a build with the 'cosmos' feature enabled.");
                std::process::exit(1);
            }
        }
        
        CosmosCommand::AutoGenerate { config, output_dir } => {
            #[cfg(feature = "cosmos")]
            {
                println!("Auto-generating from config: {} to {}", config, output_dir);
                // Implementation would go here
            }
            
            #[cfg(not(feature = "cosmos"))]
            {
                eprintln!("Error: Cosmos support not enabled.");
                eprintln!("This binary was built without Cosmos support.");
                eprintln!("Please use a build with the 'cosmos' feature enabled.");
                std::process::exit(1);
            }
        }
    }
    
    Ok(())
}

#[tokio::main]
async fn main() {
    let args = CosmosArgs::parse();
    
    if let Err(e) = handle_command(args).await {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
} 