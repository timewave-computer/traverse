//! Resolve command implementations
//!
//! Handles storage query resolution including single queries, bulk resolution,
//! and batch processing from files.

use crate::cli::OutputFormat;
use crate::formatters::{
    format_storage_path, format_storage_paths, load_layout, path_to_coprocessor_query, write_output,
};
use anyhow::Result;
use std::path::Path;
use tracing::{error, info};
use traverse_core::KeyResolver;
use traverse_cosmos::CosmosKeyResolver;
use traverse_ethereum::EthereumKeyResolver;

/// Execute resolve command
pub fn cmd_resolve(
    query: &str,
    layout_path: &Path,
    output: Option<&Path>,
    format: &OutputFormat,
    chain: &str,
) -> Result<()> {
    info!(
        "Resolving query '{}' with layout {}",
        query,
        layout_path.display()
    );

    let layout = load_layout(layout_path)?;

    let path = match chain {
        "ethereum" => {
            let resolver = EthereumKeyResolver;
            resolver.resolve(&layout, query)?
        }
        "cosmos" => {
            let resolver = CosmosKeyResolver;
            resolver.resolve(&layout, query)?
        }
        _ => {
            error!("Unsupported chain: {}", chain);
            return Err(anyhow::anyhow!("Unsupported chain: {}", chain));
        }
    };

    let output_content = format_storage_path(&path, query, format)?;
    write_output(&output_content, output)?;

    println!("Storage query resolved successfully:");
    println!("  • Query: {}", query);
    println!(
        "  • Storage key: {}",
        match &path.key {
            traverse_core::Key::Fixed(key) => hex::encode(key),
            _ => "dynamic".to_string(),
        }
    );

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

    let paths = match chain {
        "ethereum" => {
            let resolver = EthereumKeyResolver;
            resolver.resolve_all(&layout)?
        }
        "cosmos" => {
            let resolver = CosmosKeyResolver;
            resolver.resolve_all(&layout)?
        }
        _ => {
            error!("Unsupported chain: {}", chain);
            return Err(anyhow::anyhow!("Unsupported chain: {}", chain));
        }
    };

    let output_content = format_storage_paths(&paths, format)?;
    write_output(&output_content, output)?;

    println!("Resolved all storage paths successfully:");
    println!("  • Total paths: {}", paths.len());

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
    info!(
        "Resolving multiple queries from file {} with layout {}",
        queries_file.display(),
        layout_path.display()
    );

    let queries = std::fs::read_to_string(queries_file)?;
    let query_lines: Vec<&str> = queries
        .lines()
        .filter(|line| !line.trim().is_empty())
        .collect();

    let layout = load_layout(layout_path)?;

    let resolver: Box<dyn KeyResolver> = match chain {
        "ethereum" => Box::new(EthereumKeyResolver),
        "cosmos" => Box::new(CosmosKeyResolver),
        _ => {
            error!("Unsupported chain: {}", chain);
            return Err(anyhow::anyhow!("Unsupported chain: {}", chain));
        }
    };

    let mut results = Vec::new();
    let mut errors = Vec::new();

    for query in query_lines {
        match resolver.resolve(&layout, query) {
            Ok(path) => {
                let result = match format {
                    OutputFormat::Traverse => serde_json::to_value(&path)?,
                    OutputFormat::CoprocessorJson => {
                        let coprocessor_payload = path_to_coprocessor_query(&path, query);
                        serde_json::to_value(&coprocessor_payload)?
                    }
                    OutputFormat::Toml => {
                        let coprocessor_payload = path_to_coprocessor_query(&path, query);
                        serde_json::to_value(&coprocessor_payload)?
                    }
                    OutputFormat::Binary => {
                        let coprocessor_payload = path_to_coprocessor_query(&path, query);
                        serde_json::to_value(&coprocessor_payload)?
                    }
                    OutputFormat::Base64 => {
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

    println!("Batch resolved {} queries successfully", results.len());
    if !errors.is_empty() {
        println!("Errors encountered:");
        for error in errors {
            println!("  {}", error);
        }
    }

    Ok(())
}
