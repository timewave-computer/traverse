//! CLI tool for chain-independent ZK storage path generation
//!
//! This binary provides the `zkpath` command-line interface for compiling layouts,
//! resolving storage paths, and generating proofs.

use anyhow::Result;
use clap::Parser;

mod cli;
mod commands;
mod formatters;

use cli::{Cli, Commands, CosmosCommands, EthereumCommands, SolanaCommands};
use commands::cosmos::{
    cmd_cosmos_analyze_contract, cmd_cosmos_auto_generate, cmd_cosmos_compile_layout,
    cmd_cosmos_generate_queries, cmd_cosmos_resolve_query,
};
use commands::ethereum::{
    cmd_ethereum_analyze_contract, cmd_ethereum_auto_generate, cmd_ethereum_compile_layout,
    cmd_ethereum_generate_queries, cmd_ethereum_resolve_query, cmd_ethereum_verify_layout,
};
use commands::solana::{
    cmd_solana_analyze_program, cmd_solana_auto_generate, cmd_solana_compile_layout,
    cmd_solana_generate_queries, cmd_solana_resolve_query,
};
use commands::{cmd_auto_generate, cmd_batch_generate, cmd_watch};
use commands::{
    cmd_batch_resolve, cmd_compile_layout, cmd_generate_proof, cmd_resolve, cmd_resolve_all,
    cmd_codegen,
};
use commands::minimal::cmd_generate_minimal;

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
        Commands::CompileLayout {
            abi_file,
            output,
            chain,
        } => async move { cmd_compile_layout(&abi_file, output.as_deref(), &chain) }.await,

        Commands::Resolve {
            query,
            layout,
            output,
            format,
            chain,
        } => cmd_resolve(&query, &layout, output.as_deref(), &format, &chain),

        Commands::ResolveAll {
            layout,
            output,
            format,
            chain,
        } => cmd_resolve_all(&layout, output.as_deref(), &format, &chain),

        Commands::BatchResolve {
            queries_file,
            layout,
            output,
            format,
            chain,
        } => cmd_batch_resolve(&queries_file, &layout, output.as_deref(), &format, &chain),

        Commands::GenerateProof {
            slot,
            rpc,
            contract,
            zero_means,
            output,
            block_number: _,
            validate_semantics: _,
        } => cmd_generate_proof(&slot, &rpc, &contract, zero_means, output.as_deref()).await,

        Commands::Ethereum(ethereum_cmd) => {
            match ethereum_cmd {
                EthereumCommands::AnalyzeContract {
                    abi_file,
                    output,
                    validate_storage,
                    contract_address,
                    rpc,
                } => {
                    cmd_ethereum_analyze_contract(
                        &abi_file,
                        output.as_deref(),
                        validate_storage,
                        contract_address.as_deref(),
                        rpc.as_deref(),
                    )
                    .await
                }
                EthereumCommands::CompileLayout {
                    abi_file,
                    output,
                    format,
                    validate,
                } => cmd_ethereum_compile_layout(&abi_file, output.as_deref(), &format, validate),
                EthereumCommands::GenerateQueries {
                    layout_file,
                    fields,
                    output,
                    include_examples,
                } => cmd_ethereum_generate_queries(
                    &layout_file,
                    &fields,
                    output.as_deref(),
                    include_examples,
                ),
                EthereumCommands::ResolveQuery {
                    query,
                    layout,
                    format,
                    output,
                } => cmd_ethereum_resolve_query(&query, &layout, &format, output.as_deref()),
                EthereumCommands::VerifyLayout {
                    layout_file,
                    contract,
                    rpc,
                    comprehensive,
                } => {
                    cmd_ethereum_verify_layout(
                        &layout_file,
                        contract.as_deref(),
                        rpc.as_deref(),
                        comprehensive,
                    )
                    .await
                }
                EthereumCommands::GenerateProof {
                    contract,
                    slot,
                    rpc,
                    zero_means,
                    output,
                    block_number: _,
                    validate_proof: _,
                    validate_semantics: _,
                } => {
                    // Use the existing proof generation logic but with Ethereum-specific enhancements
                    cmd_generate_proof(&slot, &rpc, &contract, zero_means, output.as_deref()).await
                }
                EthereumCommands::AutoGenerate {
                    abi_file,
                    rpc,
                    contract,
                    queries,
                    output_dir,
                    cache,
                    dry_run,
                } => {
                    cmd_ethereum_auto_generate(
                        &abi_file,
                        &rpc,
                        &contract,
                        &queries,
                        &output_dir,
                        cache,
                        dry_run,
                    )
                    .await
                }
            }
        }

        Commands::Cosmos(cosmos_cmd) => match cosmos_cmd {
            CosmosCommands::AnalyzeContract {
                msg_file,
                output,
                validate_schema,
            } => {
                cmd_cosmos_analyze_contract(
                    &msg_file,
                    output.as_deref(),
                    validate_schema,
                    None,
                    None,
                )
                .await
            }
            CosmosCommands::CompileLayout {
                msg_file,
                output,
                format,
            } => {
                async move { cmd_cosmos_compile_layout(&msg_file, output.as_deref(), &format) }
                    .await
            }
            CosmosCommands::GenerateQueries {
                layout_file,
                state_keys,
                output,
                include_examples,
            } => {
                async move {
                    cmd_cosmos_generate_queries(
                        &layout_file,
                        &state_keys,
                        output.as_deref(),
                        include_examples,
                    )
                }
                .await
            }
            CosmosCommands::ResolveQuery {
                query,
                layout,
                format,
                output,
            } => {
                async move { cmd_cosmos_resolve_query(&query, &layout, &format, output.as_deref()) }
                    .await
            }
            CosmosCommands::AutoGenerate {
                msg_file,
                rpc,
                contract,
                queries,
                output_dir,
                dry_run,
            } => {
                cmd_cosmos_auto_generate(
                    &msg_file,
                    &rpc,
                    &contract,
                    &queries,
                    &output_dir,
                    false,
                    dry_run,
                )
                .await
            }
        },

        Commands::Solana(solana_cmd) => match solana_cmd {
            SolanaCommands::AnalyzeProgram {
                idl_file,
                output,
                validate_schema,
            } => {
                cmd_solana_analyze_program(
                    &idl_file,
                    output.as_deref(),
                    validate_schema,
                )
                .await
            }
            SolanaCommands::CompileLayout {
                idl_file,
                output,
                format,
            } => {
                cmd_solana_compile_layout(&idl_file, output.as_deref(), &format).await
            }
            SolanaCommands::GenerateQueries {
                layout_file,
                state_keys,
                output,
                include_examples,
            } => {
                cmd_solana_generate_queries(
                    &layout_file,
                    &state_keys,
                    output.as_deref(),
                    include_examples,
                )
                .await
            }
            SolanaCommands::ResolveQuery {
                query,
                layout,
                format,
                output,
            } => {
                cmd_solana_resolve_query(&query, &layout, &format, output.as_deref()).await
            }
            SolanaCommands::AutoGenerate {
                idl_file,
                rpc,
                program_address,
                queries,
                output_dir,
                dry_run,
            } => {
                cmd_solana_auto_generate(
                    &idl_file,
                    &rpc,
                    &program_address,
                    &queries,
                    &output_dir,
                    dry_run,
                )
                .await
            }
        },

        Commands::Codegen(codegen_cmd) => {
            cmd_codegen(codegen_cmd).await
        }
        
        Commands::Minimal(args) => {
            cmd_generate_minimal(args).await
        }

        Commands::AutoGenerate {
            config_file,
            output_dir,
            dry_run,
            cache,
        } => {
            cmd_auto_generate(
                &config_file,
                &output_dir,
                dry_run,
                cache,
            )
            .await
        }

        Commands::BatchGenerate {
            pattern,
            parallel,
            output_dir,
            dry_run,
        } => {
            cmd_batch_generate(
                &pattern,
                &output_dir,
                parallel,
                dry_run,
            )
            .await
        }

        Commands::Watch {
            watch_dir,
            config,
            webhook: _,
        } => {
            cmd_watch(&config, &watch_dir, 60).await // 60 second interval
        }
    }
}
