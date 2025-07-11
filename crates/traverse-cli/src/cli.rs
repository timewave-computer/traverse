//! Command-line interface definitions for the traverse CLI tool
//!
//! This module contains all the clap-related structures for argument parsing
//! and command definitions.

use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Clone, Debug, ValueEnum)]
pub enum ZeroSemanticsArg {
    /// Slot was never written to (default state) - use for uninitialized storage
    #[value(name = "never_written")]
    NeverWritten,
    /// Slot was intentionally set to zero - use for explicit zero assignments
    #[value(name = "explicitly_zero")]
    ExplicitlyZero,
    /// Slot was previously non-zero but cleared - use for reset/revoked values
    #[value(name = "cleared")]
    Cleared,
    /// Zero is a valid operational state - use when 0 has business logic meaning
    #[value(name = "valid_zero")]
    ValidZero,
}

impl From<ZeroSemanticsArg> for traverse_core::ZeroSemantics {
    fn from(arg: ZeroSemanticsArg) -> Self {
        match arg {
            ZeroSemanticsArg::NeverWritten => traverse_core::ZeroSemantics::NeverWritten,
            ZeroSemanticsArg::ExplicitlyZero => traverse_core::ZeroSemantics::ExplicitlyZero,
            ZeroSemanticsArg::Cleared => traverse_core::ZeroSemantics::Cleared,
            ZeroSemanticsArg::ValidZero => traverse_core::ZeroSemantics::ValidZero,
        }
    }
}

#[derive(Debug, Clone, ValueEnum, Default)]
pub enum OutputFormat {
    /// Traverse native format
    #[value(name = "traverse")]
    #[default]
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

#[derive(Parser, Debug)]
#[command(name = "zkpath")]
#[command(
    about = "Chain-independent ZK storage path generator with semantic zero value disambiguation"
)]
#[command(
    long_about = "Generate and validate storage proofs for blockchain data with semantic disambiguation of zero values.\n\nThe semantic system eliminates false positives by requiring explicit declaration of what zero values mean:\n- never_written: slot has never been written to (default state)\n- explicitly_zero: slot was intentionally set to zero\n- cleared: slot was previously non-zero but cleared\n- valid_zero: zero is a valid operational state\n\nFor more information, see: https://docs.traverse.ai/semantic-proofs"
)]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Enable verbose logging
    #[arg(short, long)]
    pub verbose: bool,
}

#[derive(Subcommand, Debug)]
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

        /// Zero value semantics (required) - specifies what zero values mean to eliminate false positives
        #[arg(
            long,
            value_enum,
            help = "Semantic meaning of zero values in storage:\n  never_written: slot has never been written to\n  explicitly_zero: slot was intentionally set to zero\n  cleared: slot was previously non-zero but cleared\n  valid_zero: zero is a valid operational state"
        )]
        zero_means: ZeroSemanticsArg,

        /// Output file (stdout if not specified)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Block number (latest if not specified)
        #[arg(long)]
        block_number: Option<String>,

        /// Validate declared semantics against blockchain events using indexer services
        #[arg(
            long,
            help = "Enable semantic validation to detect conflicts between declared semantics and actual blockchain state"
        )]
        validate_semantics: bool,
    },

    /// Enhanced Ethereum commands
    #[command(subcommand)]
    Ethereum(EthereumCommands),

    /// Enhanced Cosmos commands
    #[command(subcommand)]
    Cosmos(CosmosCommands),

    /// Auto-generate everything from configuration file
    AutoGenerate {
        /// Configuration file (JSON format with contracts array)
        #[arg(value_name = "CONFIG_FILE", help = "JSON configuration file containing contracts array with chain_type, file, rpc, contract, and queries fields for each contract")]
        config_file: PathBuf,

        /// Output directory
        #[arg(long)]
        output_dir: PathBuf,

        /// Enable dry-run mode (no RPC calls)
        #[arg(long)]
        dry_run: bool,

        /// Enable caching for faster processing
        #[arg(long)]
        cache: bool,
    },

    /// Batch processing with glob pattern
    BatchGenerate {
        /// Glob pattern to match contract files (e.g., "contracts/*.json" or "**/*.abi")
        #[arg(value_name = "PATTERN", help = "Glob pattern to match contract files for batch processing")]
        pattern: String,

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

#[derive(Subcommand, Debug)]
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

        /// Zero value semantics (required) - specifies what zero values mean to eliminate false positives
        #[arg(
            long,
            value_enum,
            help = "Semantic meaning of zero values in storage:\n  never_written: slot has never been written to\n  explicitly_zero: slot was intentionally set to zero\n  cleared: slot was previously non-zero but cleared\n  valid_zero: zero is a valid operational state"
        )]
        zero_means: ZeroSemanticsArg,

        /// Output file
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Block number (latest if not specified)
        #[arg(long)]
        block_number: Option<String>,

        /// Include proof validation
        #[arg(long)]
        validate_proof: bool,

        /// Validate declared semantics against blockchain events using indexer services
        #[arg(
            long,
            help = "Enable semantic validation to detect conflicts between declared semantics and actual blockchain state"
        )]
        validate_semantics: bool,
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

#[derive(Subcommand, Debug)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn test_zero_semantics_arg_parsing() {
        // Test all four zero semantic argument variants through CLI parsing
        let args = vec![
            "zkpath",
            "generate-proof",
            "--slot",
            "0x0",
            "--rpc",
            "http://test",
            "--contract",
            "0x1234567890abcdef1234567890abcdef12345678",
            "--zero-means",
            "never_written",
        ];
        let cli = Cli::try_parse_from(args).unwrap();
        match cli.command {
            Commands::GenerateProof { zero_means, .. } => {
                assert!(matches!(zero_means, ZeroSemanticsArg::NeverWritten));
            }
            _ => panic!("Expected GenerateProof command"),
        }
    }

    #[test]
    fn test_zero_semantics_arg_invalid() {
        // Test invalid semantic arguments
        let args = vec![
            "zkpath",
            "generate-proof",
            "--slot",
            "0x0",
            "--rpc",
            "http://test",
            "--contract",
            "0x1234567890abcdef1234567890abcdef12345678",
            "--zero-means",
            "invalid_semantic",
        ];
        let result = Cli::try_parse_from(args);
        assert!(result.is_err());
    }

    #[test]
    fn test_zero_semantics_conversion() {
        // Test conversion from CLI arg to core type
        let arg = ZeroSemanticsArg::NeverWritten;
        let core_type: traverse_core::ZeroSemantics = arg.into();
        assert_eq!(core_type, traverse_core::ZeroSemantics::NeverWritten);

        let arg = ZeroSemanticsArg::ExplicitlyZero;
        let core_type: traverse_core::ZeroSemantics = arg.into();
        assert_eq!(core_type, traverse_core::ZeroSemantics::ExplicitlyZero);

        let arg = ZeroSemanticsArg::Cleared;
        let core_type: traverse_core::ZeroSemantics = arg.into();
        assert_eq!(core_type, traverse_core::ZeroSemantics::Cleared);

        let arg = ZeroSemanticsArg::ValidZero;
        let core_type: traverse_core::ZeroSemantics = arg.into();
        assert_eq!(core_type, traverse_core::ZeroSemantics::ValidZero);
    }

    #[test]
    fn test_generate_proof_command_parsing() {
        // Test successful parsing of generate-proof command with required semantics
        let args = vec![
            "zkpath",
            "generate-proof",
            "--slot",
            "0x123456",
            "--rpc",
            "https://eth-mainnet.alchemyapi.io/v2/test",
            "--contract",
            "0x1234567890abcdef1234567890abcdef12345678",
            "--zero-means",
            "never_written",
        ];

        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::GenerateProof { zero_means, .. } => {
                assert!(matches!(zero_means, ZeroSemanticsArg::NeverWritten));
            }
            _ => panic!("Expected GenerateProof command"),
        }
    }

    #[test]
    fn test_generate_proof_command_missing_semantics() {
        // Test that generate-proof command fails without required --zero-means
        let args = vec![
            "zkpath",
            "generate-proof",
            "--slot",
            "0x123456",
            "--rpc",
            "https://eth-mainnet.alchemyapi.io/v2/test",
            "--contract",
            "0x1234567890abcdef1234567890abcdef12345678",
        ];

        let result = Cli::try_parse_from(args);
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("zero-means"));
    }

    #[test]
    fn test_ethereum_generate_proof_command_parsing() {
        // Test successful parsing of ethereum generate-proof command
        let args = vec![
            "zkpath",
            "ethereum",
            "generate-proof",
            "--contract",
            "0x1234567890abcdef1234567890abcdef12345678",
            "--slot",
            "0x123456",
            "--rpc",
            "https://eth-mainnet.alchemyapi.io/v2/test",
            "--zero-means",
            "explicitly_zero",
        ];

        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Ethereum(EthereumCommands::GenerateProof { zero_means, .. }) => {
                assert!(matches!(zero_means, ZeroSemanticsArg::ExplicitlyZero));
            }
            _ => panic!("Expected Ethereum GenerateProof command"),
        }
    }

    #[test]
    fn test_ethereum_generate_proof_command_missing_semantics() {
        // Test that ethereum generate-proof command fails without required --zero-means
        let args = vec![
            "zkpath",
            "ethereum",
            "generate-proof",
            "--contract",
            "0x1234567890abcdef1234567890abcdef12345678",
            "--slot",
            "0x123456",
            "--rpc",
            "https://eth-mainnet.alchemyapi.io/v2/test",
        ];

        let result = Cli::try_parse_from(args);
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("zero-means"));
    }

    #[test]
    fn test_semantic_validation_flag() {
        // Test that --validate-semantics flag is parsed correctly
        let args = vec![
            "zkpath",
            "generate-proof",
            "--slot",
            "0x123456",
            "--rpc",
            "https://eth-mainnet.alchemyapi.io/v2/test",
            "--contract",
            "0x1234567890abcdef1234567890abcdef12345678",
            "--zero-means",
            "cleared",
            "--validate-semantics",
        ];

        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::GenerateProof {
                validate_semantics,
                zero_means,
                ..
            } => {
                assert!(validate_semantics);
                assert!(matches!(zero_means, ZeroSemanticsArg::Cleared));
            }
            _ => panic!("Expected GenerateProof command"),
        }
    }

    #[test]
    fn test_all_semantic_variants_in_commands() {
        // Test that all semantic variants work in different commands
        let semantic_variants = vec![
            ("never_written", ZeroSemanticsArg::NeverWritten),
            ("explicitly_zero", ZeroSemanticsArg::ExplicitlyZero),
            ("cleared", ZeroSemanticsArg::Cleared),
            ("valid_zero", ZeroSemanticsArg::ValidZero),
        ];

        for (semantic_str, expected_variant) in semantic_variants {
            let args = vec![
                "zkpath",
                "ethereum",
                "generate-proof",
                "--contract",
                "0x1234567890abcdef1234567890abcdef12345678",
                "--slot",
                "0x123456",
                "--rpc",
                "https://eth-mainnet.alchemyapi.io/v2/test",
                "--zero-means",
                semantic_str,
            ];

            let cli = Cli::try_parse_from(args).unwrap();

            match cli.command {
                Commands::Ethereum(EthereumCommands::GenerateProof { zero_means, .. }) => {
                    // Compare discriminants since we can't pattern match on variable
                    assert_eq!(
                        std::mem::discriminant(&zero_means),
                        std::mem::discriminant(&expected_variant),
                        "Semantic variant mismatch for: {}",
                        semantic_str
                    );
                }
                _ => panic!(
                    "Expected Ethereum GenerateProof command for semantic: {}",
                    semantic_str
                ),
            }
        }
    }
}
