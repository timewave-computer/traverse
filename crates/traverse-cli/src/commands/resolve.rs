//! Resolve command implementations
//! 
//! Handles storage query resolution including single queries, bulk resolution,
//! and batch processing from files.

use std::path::Path;
use anyhow::Result;
use tracing::{info, error};
use traverse_core::KeyResolver;
use traverse_ethereum::EthereumKeyResolver;
use crate::cli::OutputFormat;
use crate::formatters::{
    write_output, load_layout, format_storage_path, format_storage_paths, path_to_coprocessor_query
};

/// Execute resolve command
pub fn cmd_resolve(
    query: &str,
    layout_path: &Path,
    output: Option<&Path>,
    format: &OutputFormat,
    chain: &str,
) -> Result<()> {
    info!("Resolving query '{}' with layout {}", query, layout_path.display());
    
    let layout = load_layout(layout_path)?;
    
    match chain {
        "ethereum" => {
            let resolver = EthereumKeyResolver;
            let path = resolver.resolve(&layout, query)?;
            let output_content = format_storage_path(&path, query, format)?;
            write_output(&output_content, output)?;
        }
        _ => {
            error!("Unsupported chain: {}", chain);
            std::process::exit(1);
        }
    }
    Ok(())
}

/// Execute resolve-all command
pub fn cmd_resolve_all(
    layout_path: &Path,
    output: Option<&Path>,
    format: &OutputFormat,
    chain: &str,
) -> Result<()> {
    info!("Resolving all paths from layout {}", layout_path.display());
    
    let layout = load_layout(layout_path)?;
    
    match chain {
        "ethereum" => {
            let resolver = EthereumKeyResolver;
            let paths = resolver.resolve_all(&layout)?;
            let output_content = format_storage_paths(&paths, format)?;
            write_output(&output_content, output)?;
        }
        _ => {
            error!("Unsupported chain: {}", chain);
            std::process::exit(1);
        }
    }
    Ok(())
}

/// Execute batch-resolve command
pub fn cmd_batch_resolve(
    queries_file: &Path,
    layout_path: &Path,
    output: Option<&Path>,
    format: &OutputFormat,
    chain: &str,
) -> Result<()> {
    info!("Resolving multiple queries from file {} with layout {}", 
          queries_file.display(), layout_path.display());
    
    let queries = std::fs::read_to_string(queries_file)?;
    let query_lines: Vec<&str> = queries.lines().filter(|line| !line.trim().is_empty()).collect();
    
    let layout = load_layout(layout_path)?;
    
    match chain {
        "ethereum" => {
            let resolver = EthereumKeyResolver;
            let mut results = Vec::new();
            let mut errors = Vec::new();
            
            for query in query_lines {
                match resolver.resolve(&layout, query) {
                    Ok(path) => {
                        let result = match format {
                            OutputFormat::Traverse => {
                                serde_json::to_value(&path)?
                            }
                            OutputFormat::CoprocessorJson => {
                                let coprocessor_payload = path_to_coprocessor_query(&path, query);
                                serde_json::to_value(&coprocessor_payload)?
                            }
                        };
                        results.push(result);
                    }
                    Err(e) => {
                        error!("Failed to resolve query '{}': {}", query, e);
                        errors.push(format!("Query '{}': {}", query, e));
                    }
                }
            }
            
            let output_content = serde_json::to_string_pretty(&results)?;
            write_output(&output_content, output)?;
            
            if !errors.is_empty() {
                eprintln!("\nErrors encountered:");
                for error in errors {
                    eprintln!("  {}", error);
                }
            }
        }
        _ => {
            error!("Unsupported chain: {}", chain);
            std::process::exit(1);
        }
    }
    Ok(())
} 