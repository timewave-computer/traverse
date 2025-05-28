//! Command-line interface definitions for the traverse CLI tool
//! 
//! This module contains all the clap-related structures for argument parsing
//! and command definitions.

use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(ValueEnum, Clone)]
pub enum OutputFormat {
    /// Standard traverse JSON format
    Traverse,
    /// Format optimized for valence coprocessor integration
    CoprocessorJson,
}

#[derive(Parser)]
#[command(name = "zkpath")]
#[command(about = "Chain-independent ZK storage path generator")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
    
    /// Enable verbose logging
    #[arg(short, long)]
    pub verbose: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Compile contract layout from ABI
    CompileLayout {
        /// Path to the contract ABI or layout file
        #[arg(value_name = "ABI_FILE")]
        abi_file: PathBuf,
        
        /// Output file (stdout if not specified)
        #[arg(short, long)]
        output: Option<PathBuf>,
        
        /// Blockchain type
        #[arg(long, default_value = "ethereum")]
        chain: String,
    },
    
    /// Resolve storage path query
    Resolve {
        /// Query string (e.g., 'withdrawals\\\[0xabc\\\].receiver')
        #[arg(value_name = "QUERY")]
        query: String,
        
        /// Layout file
        #[arg(short, long)]
        layout: PathBuf,
        
        /// Output file (stdout if not specified)
        #[arg(short, long)]
        output: Option<PathBuf>,
        
        /// Output format
        #[arg(long, default_value = "traverse", value_enum)]
        format: OutputFormat,
        
        /// Blockchain type
        #[arg(long, default_value = "ethereum")]
        chain: String,
    },
    
    /// Resolve all possible paths from layout
    ResolveAll {
        /// Layout file
        #[arg(short, long)]
        layout: PathBuf,
        
        /// Output file (stdout if not specified)
        #[arg(short, long)]
        output: Option<PathBuf>,
        
        /// Output format
        #[arg(long, default_value = "traverse", value_enum)]
        format: OutputFormat,
        
        /// Blockchain type
        #[arg(long, default_value = "ethereum")]
        chain: String,
    },
    
    /// Resolve multiple storage queries from a file
    BatchResolve {
        /// File containing queries (one per line)
        #[arg(value_name = "QUERIES_FILE")]
        queries_file: PathBuf,
        
        /// Layout file
        #[arg(short, long)]
        layout: PathBuf,
        
        /// Output file (stdout if not specified)
        #[arg(short, long)]
        output: Option<PathBuf>,
        
        /// Output format
        #[arg(long, default_value = "coprocessor-json", value_enum)]
        format: OutputFormat,
        
        /// Blockchain type
        #[arg(long, default_value = "ethereum")]
        chain: String,
    },
    
    /// Generate proof payload for ZK coprocessor
    GenerateProof {
        /// Storage slot (hex)
        #[arg(long)]
        slot: String,
        
        /// RPC endpoint URL
        #[arg(long)]
        rpc: String,
        
        /// Contract address
        #[arg(long)]
        contract: String,
        
        /// Output file (stdout if not specified)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
} 