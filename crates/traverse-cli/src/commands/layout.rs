//! Layout command implementation
//! 
//! Handles compilation of contract ABIs to canonical layout format.

use std::path::Path;
use anyhow::Result;
use tracing::{info, error};
use traverse_core::LayoutCompiler;
use traverse_ethereum::EthereumLayoutCompiler;
use crate::formatters::write_output;

/// Execute compile-layout command
pub fn cmd_compile_layout(abi_file: &Path, output: Option<&Path>, chain: &str) -> Result<()> {
    info!("Compiling layout from {}", abi_file.display());
    
    match chain {
        "ethereum" => {
            let compiler = EthereumLayoutCompiler;
            let layout = compiler.compile_layout(abi_file)?;
            let json = serde_json::to_string_pretty(&layout)?;
            write_output(&json, output)?;
        }
        _ => {
            error!("Unsupported chain: {}", chain);
            std::process::exit(1);
        }
    }
    Ok(())
} 