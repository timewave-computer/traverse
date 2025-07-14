//! Shared CLI core functionality
//!
//! This crate contains common functionality that is shared across all ecosystem-specific
//! CLI binaries (Ethereum, Solana, Cosmos). This allows us to avoid code duplication while
//! keeping the CLI binaries isolated to prevent dependency conflicts.

use clap::{Parser, Subcommand};
use serde_json::Value;
use std::fs;
use std::path::Path;
use base64::Engine;

pub mod formatters;

/// Common CLI arguments shared across all ecosystems
#[derive(Parser)]
#[command(name = "traverse-cli")]
#[command(about = "Chain-independent ZK storage path generator")]
#[command(version)]
pub struct CommonArgs {
    /// Enable verbose output
    #[arg(short, long)]
    pub verbose: bool,
    
    /// Output format (json, pretty, compact)
    #[arg(short, long, default_value = "pretty")]
    pub format: OutputFormat,
    
    /// Output file (stdout if not specified)
    #[arg(short, long)]
    pub output: Option<String>,
}

/// Output format options
#[derive(Clone, Debug, clap::ValueEnum, Default)]
pub enum OutputFormat {
    /// Traverse native format
    #[value(name = "traverse")]
    #[default]
    Traverse,
    /// ZK coprocessor JSON format
    #[value(name = "coprocessor-json")]
    CoprocessorJson,
    /// TOML format
    #[value(name = "toml")]
    Toml,
    /// Binary format for performance
    #[value(name = "binary")]
    Binary,
    /// Base64 encoded binary format
    #[value(name = "base64")]
    Base64,
}

/// Common subcommands available across ecosystems
#[derive(Subcommand)]
pub enum CommonCommand {
    /// Analyze a contract/program file
    Analyze {
        /// Path to the contract/program file
        file: String,
        /// Additional analysis options
        #[arg(long)]
        deep: bool,
    },
    
    /// Compile layout information
    CompileLayout {
        /// Input file path
        input: String,
        /// Output file path
        #[arg(short, long)]
        output: Option<String>,
    },
    
    /// Generate storage queries
    GenerateQueries {
        /// Layout file path
        layout: String,
        /// Query patterns
        patterns: Vec<String>,
    },
    
    /// Resolve a specific query
    ResolveQuery {
        /// Query string to resolve
        query: String,
        /// Layout file path
        #[arg(short, long)]
        layout: String,
    },
    
    /// Auto-generate for batch processing
    AutoGenerate {
        /// Configuration file path
        config: String,
        /// Output directory
        #[arg(short, long, default_value = "output")]
        output_dir: String,
    },
}

/// Common result type for CLI operations
pub type CliResult<T> = Result<T, CliError>;

/// Common error type for CLI operations
#[derive(Debug, thiserror::Error)]
pub enum CliError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),
    
    #[error("File not found: {0}")]
    FileNotFound(String),
    
    #[error("Configuration error: {0}")]
    Configuration(String),
    
    #[error("Processing error: {0}")]
    Processing(String),
}

/// Common CLI utilities
pub struct CliUtils;

impl CliUtils {
    /// Read a file and return its contents
    pub fn read_file<P: AsRef<Path>>(path: P) -> CliResult<String> {
        let path = path.as_ref();
        if !path.exists() {
            return Err(CliError::FileNotFound(path.to_string_lossy().to_string()));
        }
        
        fs::read_to_string(path).map_err(CliError::Io)
    }
    
    /// Write content to a file or stdout
    pub fn write_output(content: &str, output_path: Option<&str>) -> CliResult<()> {
        match output_path {
            Some(path) => {
                fs::write(path, content).map_err(CliError::Io)?;
                if std::env::var("VERBOSE").is_ok() {
                    eprintln!("Output written to: {}", path);
                }
            }
            None => {
                print!("{}", content);
            }
        }
        Ok(())
    }
    
    /// Format JSON output based on format option
    pub fn format_json(value: &Value, format: &OutputFormat) -> CliResult<String> {
        match format {
            OutputFormat::Traverse => serde_json::to_string_pretty(value).map_err(CliError::Json),
            OutputFormat::CoprocessorJson => serde_json::to_string_pretty(value).map_err(CliError::Json),
            OutputFormat::Toml => {
                // For simple JSON values, convert to TOML
                let toml_value = match value {
                    Value::Object(obj) => toml::to_string_pretty(obj).map_err(|e| CliError::Processing(e.to_string()))?,
                    _ => toml::to_string_pretty(value).map_err(|e| CliError::Processing(e.to_string()))?,
                };
                Ok(toml_value)
            }
            OutputFormat::Binary => {
                // For binary output, serialize to binary and encode as base64
                let binary_data = bincode::serialize(value).map_err(|e| CliError::Processing(e.to_string()))?;
                Ok(format!("Binary data: {} bytes\nBase64: {}", binary_data.len(), base64::engine::general_purpose::STANDARD.encode(&binary_data)))
            }
            OutputFormat::Base64 => {
                // For base64 output, serialize to binary and encode as base64
                let binary_data = bincode::serialize(value).map_err(|e| CliError::Processing(e.to_string()))?;
                Ok(base64::engine::general_purpose::STANDARD.encode(&binary_data))
            }
        }
    }
    
    /// Load configuration from a JSON file
    pub fn load_config<P: AsRef<Path>>(path: P) -> CliResult<Value> {
        let content = Self::read_file(path)?;
        serde_json::from_str(&content).map_err(CliError::Json)
    }
    
    /// Create output directory if it doesn't exist
    pub fn ensure_output_dir<P: AsRef<Path>>(path: P) -> CliResult<()> {
        let path = path.as_ref();
        if !path.exists() {
            fs::create_dir_all(path).map_err(CliError::Io)?;
        }
        Ok(())
    }
    
    /// Validate file extension
    pub fn validate_file_extension<P: AsRef<Path>>(path: P, expected_exts: &[&str]) -> CliResult<()> {
        let path = path.as_ref();
        let ext = path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
            
        if !expected_exts.contains(&ext) {
            return Err(CliError::InvalidArgument(
                format!("Expected file extension: {}, got: {}", 
                    expected_exts.join(", "), ext)
            ));
        }
        
        Ok(())
    }
}

/// Macro for consistent error handling across CLI binaries
#[macro_export]
macro_rules! cli_error {
    ($($arg:tt)*) => {
        return Err($crate::CliError::Processing(format!($($arg)*)))
    };
}

/// Macro for verbose output
#[macro_export]
macro_rules! verbose_println {
    ($($arg:tt)*) => {
        if std::env::var("VERBOSE").is_ok() {
            eprintln!($($arg)*);
        }
    };
} 