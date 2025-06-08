//! CLI tool for chain-independent ZK storage path generation
//! 
//! This binary provides the `zkpath` command-line interface for compiling layouts,
//! resolving storage paths, and generating proofs.

use clap::Parser;
use anyhow::Result;

mod cli;
mod formatters;
mod commands;

use cli::{Cli, Commands, EthereumCommands, CosmosCommands};
use commands::{cmd_compile_layout, cmd_resolve, cmd_resolve_all, cmd_batch_resolve, cmd_generate_proof};
use commands::{cmd_auto_generate, cmd_batch_generate, cmd_watch};
use commands::ethereum::{
    cmd_ethereum_analyze_contract, cmd_ethereum_compile_layout, cmd_ethereum_generate_queries,
    cmd_ethereum_resolve_query, cmd_ethereum_verify_layout, cmd_ethereum_auto_generate
};
use commands::cosmos::{
    cmd_cosmos_analyze_contract, cmd_cosmos_compile_layout, cmd_cosmos_resolve_query,
    cmd_cosmos_generate_queries, cmd_cosmos_auto_generate
};

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
        
        Commands::GenerateProof { slot, rpc, contract, output, block_number: _ } => {
            cmd_generate_proof(&slot, &rpc, &contract, output.as_deref()).await
        }
        
        Commands::Ethereum(ethereum_cmd) => {
            match ethereum_cmd {
                EthereumCommands::AnalyzeContract { abi_file, output, validate_storage, contract_address, rpc } => {
                    cmd_ethereum_analyze_contract(&abi_file, output.as_deref(), validate_storage, 
                                                  contract_address.as_deref(), rpc.as_deref()).await
                }
                EthereumCommands::CompileLayout { abi_file, output, format, validate } => {
                    cmd_ethereum_compile_layout(&abi_file, output.as_deref(), &format, validate)
                }
                EthereumCommands::GenerateQueries { layout_file, fields, output, include_examples } => {
                    cmd_ethereum_generate_queries(&layout_file, &fields, output.as_deref(), include_examples)
                }
                EthereumCommands::ResolveQuery { query, layout, format, output } => {
                    cmd_ethereum_resolve_query(&query, &layout, &format, output.as_deref())
                }
                EthereumCommands::VerifyLayout { layout_file, contract, rpc, comprehensive } => {
                    cmd_ethereum_verify_layout(&layout_file, contract.as_deref(), rpc.as_deref(), comprehensive).await
                }
                EthereumCommands::GenerateProof { contract, slot, rpc, output, block_number: _, validate_proof: _ } => {
                    // Use the existing proof generation logic but with Ethereum-specific enhancements
                    cmd_generate_proof(&slot, &rpc, &contract, output.as_deref()).await
                }
                EthereumCommands::AutoGenerate { abi_file, rpc, contract, queries, output_dir, cache, dry_run } => {
                    cmd_ethereum_auto_generate(&abi_file, &rpc, &contract, &queries, &output_dir, cache, dry_run).await
                }
            }
        }
        
        Commands::Cosmos(cosmos_cmd) => {
            match cosmos_cmd {
                CosmosCommands::AnalyzeContract { msg_file, output, validate_schema } => {
                    cmd_cosmos_analyze_contract(&msg_file, output.as_deref(), validate_schema)
                }
                CosmosCommands::CompileLayout { msg_file, output, format } => {
                    cmd_cosmos_compile_layout(&msg_file, output.as_deref(), &format)
                }
                CosmosCommands::GenerateQueries { layout_file, state_keys, output, include_examples } => {
                    cmd_cosmos_generate_queries(&layout_file, &state_keys, output.as_deref(), include_examples)
                }
                CosmosCommands::ResolveQuery { query, layout, format, output } => {
                    cmd_cosmos_resolve_query(&query, &layout, &format, output.as_deref())
                }
                CosmosCommands::AutoGenerate { msg_file, rpc, contract, queries, output_dir, dry_run } => {
                    cmd_cosmos_auto_generate(&msg_file, &rpc, &contract, &queries, &output_dir, dry_run)
                }
            }
        }
        
        Commands::AutoGenerate { contract_file, rpc_ethereum, rpc_cosmos, contract_ethereum, 
                                contract_cosmos, queries_file, output_dir, dry_run } => {
            cmd_auto_generate(
                &contract_file,
                rpc_ethereum.as_deref(),
                rpc_cosmos.as_deref(),
                contract_ethereum.as_deref(),
                contract_cosmos.as_deref(),
                queries_file.as_deref(),
                &output_dir,
                dry_run,
            ).await
        }
        
        Commands::BatchGenerate { config, parallel, output_dir, dry_run } => {
            cmd_batch_generate(&config, parallel, &output_dir, dry_run).await
        }
        
        Commands::Watch { watch_dir, config, webhook } => {
            cmd_watch(&watch_dir, &config, webhook.as_deref()).await
        }
    }
} 