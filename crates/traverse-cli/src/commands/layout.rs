//! Layout command implementation
//!
//! Handles compilation of contract ABIs to canonical layout format.

use crate::formatters::write_output;
use anyhow::Result;
use serde_json::json;
use std::path::Path;
use tracing::{error, info};
use traverse_core::LayoutCompiler;
use traverse_cosmos::CosmosLayoutCompiler;
use traverse_ethereum::EthereumLayoutCompiler;

/// Execute compile-layout command
pub fn cmd_compile_layout(abi_file: &Path, output: Option<&Path>, chain: &str) -> Result<()> {
    info!("Compiling layout from {}", abi_file.display());

    let layout = match chain {
        "ethereum" => {
            let compiler = EthereumLayoutCompiler;
            compiler.compile_layout(abi_file)?
        }
        "cosmos" => {
            let compiler = CosmosLayoutCompiler;
            compiler.compile_layout(abi_file)?
        }
        _ => {
            error!("Unsupported chain: {}", chain);
            return Err(anyhow::anyhow!("Unsupported chain: {}", chain));
        }
    };

    // Create output structure with metadata
    let output_structure = json!({
        "contract_name": layout.contract_name,
        "storage_layout": {
            "storage": layout.storage,
            "types": layout.types
        },
        "commitment": hex::encode(layout.commitment()),
        "metadata": {
            "generated_at": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            "storage_entries": layout.storage.len(),
            "type_definitions": layout.types.len(),
            "compiler": format!("traverse-{}", chain),
            "chain": chain
        }
    });

    let json = serde_json::to_string_pretty(&output_structure)?;
    write_output(&json, output)?;

    println!("Compiling storage layout: {}_layout detected", chain);
    println!("Storage layout compiled successfully:");
    println!("  • Contract: {}", layout.contract_name);
    println!("  • Storage entries: {}", layout.storage.len());
    println!("  • Type definitions: {}", layout.types.len());
    println!(
        "  • Layout commitment: {}",
        hex::encode(layout.commitment())
    );

    Ok(())
}
