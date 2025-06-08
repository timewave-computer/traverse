//! Command-line interface definitions for the traverse CLI tool
//! 
//! This module contains all the clap-related structures for argument parsing
//! and command definitions.

use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Clone, Debug, ValueEnum)]
pub enum OutputFormat {
    /// Traverse native format
    #[value(name = "traverse")]
    Traverse,
    /// ZK coprocessor JSON format
    #[value(name = "coprocessor-json")]
    CoprocessorJson,
    /// TOML format for configuration
    #[value(name = "toml")]
    Toml,
    /// Binary format for performance
    #[value(name = "binary")]
    Binary,
    /// Base64 encoded binary format
    #[value(name = "base64")]
    Base64,
}

impl Default for OutputFormat {
    fn default() -> Self {
        OutputFormat::Traverse
    }
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
        
        /// Block number (latest if not specified)
        #[arg(long)]
        block_number: Option<String>,
    },
    
    /// Enhanced Ethereum commands
    #[command(subcommand)]
    Ethereum(EthereumCommands),
    
    /// Enhanced Cosmos commands
    #[command(subcommand)]
    Cosmos(CosmosCommands),
    
    /// Auto-generate everything from contract files
    AutoGenerate {
        /// Contract file (ABI JSON or layout)
        #[arg(value_name = "CONTRACT_FILE")]
        contract_file: PathBuf,
        
        /// RPC endpoint URL for Ethereum
        #[arg(long)]
        rpc_ethereum: Option<String>,
        
        /// RPC endpoint URL for Cosmos
        #[arg(long)]
        rpc_cosmos: Option<String>,
        
        /// Ethereum contract address
        #[arg(long)]
        contract_ethereum: Option<String>,
        
        /// Cosmos contract address
        #[arg(long)]
        contract_cosmos: Option<String>,
        
        /// Queries file (YAML format)
        #[arg(long)]
        queries_file: Option<PathBuf>,
        
        /// Output directory
        #[arg(long)]
        output_dir: PathBuf,
        
        /// Enable dry-run mode (no RPC calls)
        #[arg(long)]
        dry_run: bool,
    },
    
    /// Batch processing with configuration file
    BatchGenerate {
        /// Configuration file (YAML)
        #[arg(value_name = "CONFIG_FILE")]
        config: PathBuf,
        
        /// Number of parallel workers
        #[arg(long, default_value = "1")]
        parallel: usize,
        
        /// Output directory
        #[arg(long)]
        output_dir: PathBuf,
        
        /// Enable dry-run mode
        #[arg(long)]
        dry_run: bool,
    },
    
    /// Watch mode for continuous generation
    Watch {
        /// Directory to watch for changes
        #[arg(value_name = "WATCH_DIR")]
        watch_dir: PathBuf,
        
        /// Configuration file
        #[arg(long)]
        config: PathBuf,
        
        /// Webhook URL for notifications
        #[arg(long)]
        webhook: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum EthereumCommands {
    /// Analyze Ethereum contract from ABI
    AnalyzeContract {
        /// Contract ABI JSON file
        #[arg(value_name = "ABI_FILE")]
        abi_file: PathBuf,
        
        /// Output file
        #[arg(short, long)]
        output: Option<PathBuf>,
        
        /// Validate storage layout
        #[arg(long)]
        validate_storage: bool,
        
        /// Contract address for enhanced analysis
        #[arg(long)]
        contract_address: Option<String>,
        
        /// RPC endpoint for live analysis
        #[arg(long)]
        rpc: Option<String>,
    },
    
    /// Compile Ethereum storage layout
    CompileLayout {
        /// Contract ABI JSON file
        #[arg(value_name = "ABI_FILE")]
        abi_file: PathBuf,
        
        /// Output file
        #[arg(short, long)]
        output: Option<PathBuf>,
        
        /// Output format
        #[arg(long, default_value = "traverse", value_enum)]
        format: OutputFormat,
        
        /// Validate layout for conflicts
        #[arg(long)]
        validate: bool,
    },
    
    /// Generate storage queries for specific fields
    GenerateQueries {
        /// Compiled layout file
        #[arg(value_name = "LAYOUT_FILE")]
        layout_file: PathBuf,
        
        /// Comma-separated list of field names
        #[arg(long)]
        fields: String,
        
        /// Output file
        #[arg(short, long)]
        output: Option<PathBuf>,
        
        /// Include example keys for mappings
        #[arg(long)]
        include_examples: bool,
    },
    
    /// Resolve specific storage query
    ResolveQuery {
        /// Query string
        #[arg(value_name = "QUERY")]
        query: String,
        
        /// Layout file
        #[arg(long)]
        layout: PathBuf,
        
        /// Output format
        #[arg(long, default_value = "coprocessor-json", value_enum)]
        format: OutputFormat,
        
        /// Output file
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    
    /// Generate storage proof
    GenerateProof {
        /// Contract address
        #[arg(long)]
        contract: String,
        
        /// Storage slot (hex)
        #[arg(long)]
        slot: String,
        
        /// RPC endpoint URL
        #[arg(long)]
        rpc: String,
        
        /// Output file
        #[arg(short, long)]
        output: Option<PathBuf>,
        
        /// Block number (latest if not specified)
        #[arg(long)]
        block_number: Option<String>,
        
        /// Include proof validation
        #[arg(long)]
        validate_proof: bool,
    },
    
    /// Verify storage layout correctness
    VerifyLayout {
        /// Layout file
        #[arg(value_name = "LAYOUT_FILE")]
        layout_file: PathBuf,
        
        /// Contract address for verification
        #[arg(long)]
        contract: Option<String>,
        
        /// RPC endpoint for live verification
        #[arg(long)]
        rpc: Option<String>,
        
        /// Run comprehensive tests
        #[arg(long)]
        comprehensive: bool,
    },
    
    /// End-to-end automation for Ethereum
    AutoGenerate {
        /// Contract ABI JSON file
        #[arg(value_name = "ABI_FILE")]
        abi_file: PathBuf,
        
        /// RPC endpoint URL
        #[arg(long)]
        rpc: String,
        
        /// Contract address
        #[arg(long)]
        contract: String,
        
        /// Comma-separated list of queries
        #[arg(long)]
        queries: String,
        
        /// Output directory
        #[arg(long)]
        output_dir: PathBuf,
        
        /// Enable caching
        #[arg(long)]
        cache: bool,
        
        /// Enable dry-run mode
        #[arg(long)]
        dry_run: bool,
    },
}

#[derive(Subcommand)]
pub enum CosmosCommands {
    /// Analyze CosmWasm contract from message schema
    AnalyzeContract {
        /// Contract message JSON file
        #[arg(value_name = "MSG_FILE")]
        msg_file: PathBuf,
        
        /// Output file
        #[arg(short, long)]
        output: Option<PathBuf>,
        
        /// Validate schema
        #[arg(long)]
        validate_schema: bool,
    },
    
    /// Compile CosmWasm storage layout
    CompileLayout {
        /// Contract message JSON file
        #[arg(value_name = "MSG_FILE")]
        msg_file: PathBuf,
        
        /// Output file
        #[arg(short, long)]
        output: Option<PathBuf>,
        
        /// Output format
        #[arg(long, default_value = "traverse", value_enum)]
        format: OutputFormat,
    },
    
    /// Generate storage queries for CosmWasm state
    GenerateQueries {
        /// Compiled layout file
        #[arg(value_name = "LAYOUT_FILE")]
        layout_file: PathBuf,
        
        /// Comma-separated list of state keys
        #[arg(long)]
        state_keys: String,
        
        /// Output file
        #[arg(short, long)]
        output: Option<PathBuf>,
        
        /// Include example keys for maps
        #[arg(long)]
        include_examples: bool,
    },
    
    /// Resolve specific CosmWasm storage query
    ResolveQuery {
        /// Query string
        #[arg(value_name = "QUERY")]
        query: String,
        
        /// Layout file
        #[arg(long)]
        layout: PathBuf,
        
        /// Output format
        #[arg(long, default_value = "coprocessor-json", value_enum)]
        format: OutputFormat,
        
        /// Output file
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    
    /// End-to-end automation for CosmWasm
    AutoGenerate {
        /// Contract message JSON file
        #[arg(value_name = "MSG_FILE")]
        msg_file: PathBuf,
        
        /// RPC endpoint URL
        #[arg(long)]
        rpc: String,
        
        /// Contract address
        #[arg(long)]
        contract: String,
        
        /// Comma-separated list of queries
        #[arg(long)]
        queries: String,
        
        /// Output directory
        #[arg(long)]
        output_dir: PathBuf,
        
        /// Enable dry-run mode
        #[arg(long)]
        dry_run: bool,
    },
} 