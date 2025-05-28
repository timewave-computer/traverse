//! CLI tool for chain-independent ZK storage path generation
//! 
//! This binary provides the `zkpath` command-line interface for compiling layouts,
//! resolving storage paths, and generating proofs.

use clap::Parser;
use anyhow::Result;

mod cli;
mod formatters;
mod commands;

use cli::{Cli, Commands};
use commands::{cmd_compile_layout, cmd_resolve, cmd_resolve_all, cmd_batch_resolve, cmd_generate_proof};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Initialize tracing
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(if cli.verbose { 
            tracing::Level::DEBUG 
        } else { 
            tracing::Level::INFO 
        })
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    match cli.command {
        Commands::CompileLayout { abi_file, output, chain } => {
            cmd_compile_layout(&abi_file, output.as_deref(), &chain)
        }
        
        Commands::Resolve { query, layout, output, format, chain } => {
            cmd_resolve(&query, &layout, output.as_deref(), &format, &chain)
        }
        
        Commands::ResolveAll { layout, output, format, chain } => {
            cmd_resolve_all(&layout, output.as_deref(), &format, &chain)
        }
        
        Commands::BatchResolve { queries_file, layout, output, format, chain } => {
            cmd_batch_resolve(&queries_file, &layout, output.as_deref(), &format, &chain)
        }
        
        Commands::GenerateProof { slot, rpc, contract, output } => {
            cmd_generate_proof(&slot, &rpc, &contract, output.as_deref())
        }
    }
} 