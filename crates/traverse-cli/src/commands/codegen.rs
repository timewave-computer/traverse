//! Code generation commands for creating minimal traverse-valence applications
//!
//! These commands generate lightweight, customized crates containing only the specific
//! functionality needed for particular storage queries and layouts.

use anyhow::Result;
use clap::{Args, Subcommand};
use std::path::PathBuf;
use traverse_valence::codegen::{CodegenOptions, LayoutInfo, QueryInfo, generate_controller_crate, generate_circuit_crate};

#[derive(Debug, Subcommand)]
pub enum CodegenCommands {
    /// Generate a minimal controller crate for witness generation
    Controller(GenerateControllerArgs),
    /// Generate a minimal circuit crate for proof verification
    Circuit(GenerateCircuitArgs),
    /// Generate both controller and circuit crates as a complete application
    App(GenerateAppArgs),
}

#[derive(Debug, Args)]
pub struct GenerateControllerArgs {
    /// Layout file containing storage layout information
    #[clap(long, short)]
    pub layout: PathBuf,
    
    /// Output directory for generated controller crate
    #[clap(long, short)]
    pub output: PathBuf,
    
    /// Name of the generated crate
    #[clap(long, default_value = "my-traverse-controller")]
    pub name: String,
    
    /// Version of the generated crate
    #[clap(long, default_value = "0.1.0")]
    pub version: String,
    
    /// Authors for the generated crate
    #[clap(long, default_value = "Generated")]
    pub authors: Vec<String>,
    
    /// Description of the generated crate
    #[clap(long)]
    pub description: Option<String>,
    
    /// Include alloy support for ABI encoding
    #[clap(long)]
    pub include_alloy: bool,
}

#[derive(Debug, Args)]
pub struct GenerateCircuitArgs {
    /// Layout file containing storage layout information
    #[clap(long, short)]
    pub layout: PathBuf,
    
    /// Output directory for generated circuit crate
    #[clap(long, short)]
    pub output: PathBuf,
    
    /// Name of the generated crate
    #[clap(long, default_value = "my-traverse-circuit")]
    pub name: String,
    
    /// Version of the generated crate
    #[clap(long, default_value = "0.1.0")]
    pub version: String,
    
    /// Authors for the generated crate
    #[clap(long, default_value = "Generated")]
    pub authors: Vec<String>,
    
    /// Description of the generated crate
    #[clap(long)]
    pub description: Option<String>,
    
    /// Include alloy support for ABI encoding
    #[clap(long)]
    pub include_alloy: bool,
}

#[derive(Debug, Args)]
pub struct GenerateAppArgs {
    /// Layout file containing storage layout information
    #[clap(long, short)]
    pub layout: PathBuf,
    
    /// Output directory for generated application workspace
    #[clap(long, short)]
    pub output: PathBuf,
    
    /// Name of the generated application
    #[clap(long, default_value = "my-traverse-app")]
    pub name: String,
    
    /// Version of the generated application
    #[clap(long, default_value = "0.1.0")]
    pub version: String,
    
    /// Authors for the generated application
    #[clap(long, default_value = "Generated")]
    pub authors: Vec<String>,
    
    /// Description of the generated application
    #[clap(long)]
    pub description: Option<String>,
    
    /// Include alloy support for ABI encoding
    #[clap(long)]
    pub include_alloy: bool,
}

/// Execute codegen commands
pub async fn cmd_codegen(command: CodegenCommands) -> Result<()> {
    match command {
        CodegenCommands::Controller(args) => cmd_generate_controller(args).await,
        CodegenCommands::Circuit(args) => cmd_generate_circuit(args).await,
        CodegenCommands::App(args) => cmd_generate_app(args).await,
    }
}

/// Generate a minimal controller crate
pub async fn cmd_generate_controller(args: GenerateControllerArgs) -> Result<()> {
    println!("Generating controller crate: {}", args.name);
    
    // Read layout file
    let layout_content = std::fs::read_to_string(&args.layout)?;
    let layout_json: serde_json::Value = serde_json::from_str(&layout_content)?;
    
    // Parse layout information
    let layout = parse_layout_from_json(&layout_json)?;
    
    // Create codegen options
    let options = CodegenOptions {
        crate_name: args.name.clone(),
        version: args.version,
        authors: args.authors,
        description: args.description.unwrap_or_else(|| {
            format!("Generated controller for {}", layout.contract_name)
        }),
        include_alloy: args.include_alloy,
        minimal: true, // Controllers are always minimal
        no_std: true, // Controllers are always no_std (but field not used in templates)
    };
    
    // Generate the crate
    generate_controller_crate(&args.output, &layout, &options)?;
    
    println!("✓ Controller crate generated at: {}", args.output.display());
    println!("  - Contract: {}", layout.contract_name);
    println!("  - Queries: {}", layout.queries.len());
    println!("  - Layout commitment: {}", layout.commitment);
    println!("  - Built for no_std environments");
    
    if args.include_alloy {
        println!("  - Includes alloy ABI support");
    }
    
    println!("\nTo use this controller in your valence app:");
    println!("  1. Add to your controller/Cargo.toml:");
    println!("     {} = {{ path = \"{}\" }}", args.name, args.output.display());
    println!("  2. Use in your controller/src/lib.rs:");
    println!("     use {}::create_witness;", args.name.replace("-", "_"));
    
    Ok(())
}

/// Generate a minimal circuit crate
pub async fn cmd_generate_circuit(args: GenerateCircuitArgs) -> Result<()> {
    println!("Generating circuit crate: {}", args.name);
    
    // Read layout file
    let layout_content = std::fs::read_to_string(&args.layout)?;
    let layout_json: serde_json::Value = serde_json::from_str(&layout_content)?;
    
    // Parse layout information
    let layout = parse_layout_from_json(&layout_json)?;
    
    // Create codegen options
    let options = CodegenOptions {
        crate_name: args.name.clone(),
        version: args.version,
        authors: args.authors,
        description: args.description.unwrap_or_else(|| {
            format!("Generated circuit for {}", layout.contract_name)
        }),
        include_alloy: args.include_alloy,
        minimal: true, // Circuits are always minimal
        no_std: true, // Circuits are always no_std
    };
    
    // Generate the crate
    generate_circuit_crate(&args.output, &layout, &options)?;
    
    println!("✓ Circuit crate generated at: {}", args.output.display());
    println!("  - Contract: {}", layout.contract_name);
    println!("  - Fields: {}", layout.field_types.len());
    println!("  - Layout commitment: {}", layout.commitment);
    println!("  - Built for minimal/constrained environments");
    
    if args.include_alloy {
        println!("  - Includes alloy ABI support");
    }
    
    println!("\nTo use this circuit in your valence app:");
    println!("  1. Add to your circuit/Cargo.toml:");
    println!("     {} = {{ path = \"{}\" }}", args.name, args.output.display());
    println!("  2. Use in your circuit/src/lib.rs:");
    println!("     use {}::circuit;", args.name.replace("-", "_"));
    
    Ok(())
}

/// Generate a complete application with both controller and circuit
pub async fn cmd_generate_app(args: GenerateAppArgs) -> Result<()> {
    println!("Generating complete valence application: {}", args.name);
    
    // Create workspace directory
    std::fs::create_dir_all(&args.output)?;
    std::fs::create_dir_all(args.output.join("crates"))?;
    
    // Read layout file
    let layout_content = std::fs::read_to_string(&args.layout)?;
    let layout_json: serde_json::Value = serde_json::from_str(&layout_content)?;
    
    // Parse layout information
    let layout = parse_layout_from_json(&layout_json)?;
    
    // Generate controller
    let controller_args = GenerateControllerArgs {
        layout: args.layout.clone(),
        output: args.output.join("crates").join("controller"),
        name: format!("{}-controller", args.name),
        version: args.version.clone(),
        authors: args.authors.clone(),
        description: Some(format!("Controller for {}", args.name)),
        include_alloy: args.include_alloy,
    };
    cmd_generate_controller(controller_args).await?;
    
    // Generate circuit
    let circuit_args = GenerateCircuitArgs {
        layout: args.layout,
        output: args.output.join("crates").join("circuit"),
        name: format!("{}-circuit", args.name),
        version: args.version.clone(),
        authors: args.authors.clone(),
        description: Some(format!("Circuit for {}", args.name)),
        include_alloy: args.include_alloy,
    };
    cmd_generate_circuit(circuit_args).await?;
    
    // Generate workspace Cargo.toml
    let workspace_cargo_toml = format!(
        r#"[workspace]
members = ["crates/controller", "crates/circuit"]
resolver = "2"

[workspace.package]
version = "{}"
edition = "2021"
authors = {:?}
description = "{}"

[workspace.dependencies]
valence-coprocessor = {{ git = "https://github.com/timewave-computer/valence-coprocessor.git", tag = "v0.1.13", default-features = false }}
traverse-valence = {{ path = "../../../traverse", default-features = false }}
"#,
        args.version,
        args.authors,
        args.description.unwrap_or_else(|| format!("Generated valence application for {}", layout.contract_name))
    );
    
    std::fs::write(args.output.join("Cargo.toml"), workspace_cargo_toml)?;
    
    // Generate README
    let readme = format!(
        r#"# {}

Generated valence coprocessor application for {} contract.

## Structure

- `crates/controller/` - Witness generation logic
- `crates/circuit/` - Proof verification logic

## Layout Information

- **Contract**: {}
- **Layout Commitment**: `{}`
- **Supported Queries**: {}
- **Field Types**: {}

## Usage

### Building

```bash
# Build all crates
cargo build

# Build specific crate
cargo build -p {}-controller
cargo build -p {}-circuit
```

### Integration

Copy the generated crates into your valence application workspace and update your dependencies.

## Generated Functionality

### Controller

The controller handles witness generation for the following queries:
{}

### Circuit

The circuit verifies storage proofs for the following field types:
{}

## Layout Commitment

This application is tied to layout commitment `{}`. Any storage proofs must match this exact layout.
"#,
        args.name,
        layout.contract_name,
        layout.contract_name,
        layout.commitment,
        layout.queries.len(),
        layout.field_types.len(),
        args.name,
        args.name,
        layout.queries.iter()
            .map(|q| format!("- `{}`", q.query))
            .collect::<Vec<_>>()
            .join("\n"),
        layout.field_types.iter()
            .enumerate()
            .map(|(i, t)| format!("- Field {}: {}", i, t))
            .collect::<Vec<_>>()
            .join("\n"),
        layout.commitment
    );
    
    std::fs::write(args.output.join("README.md"), readme)?;
    
    println!("✓ Complete application generated at: {}", args.output.display());
    println!("  - Workspace with controller and circuit crates");
    println!("  - Contract: {}", layout.contract_name);
    println!("  - {} queries, {} field types", layout.queries.len(), layout.field_types.len());
    println!("\nTo use this application:");
    println!("  1. Copy generated crates into your valence app workspace");
    println!("  2. Update your dependencies to reference the new crates");
    println!("  3. Follow the patterns shown in the generated code");
    
    Ok(())
}

/// Parse a storage slot as a 256-bit hex value
///
/// Ethereum storage slots are 32-byte (256-bit) values. This function:
/// - Accepts hex strings with or without "0x" prefix
/// - Validates hex characters and length (max 64 hex chars = 256 bits)
/// - Pads to 64 characters with leading zeros
/// - Returns formatted as "0x{64 hex chars}"
fn parse_storage_slot(slot_str: &str) -> Result<String, String> {
    // Remove 0x prefix if present
    let hex_str = slot_str.strip_prefix("0x").unwrap_or(slot_str);
    
    // Validate it's not empty
    if hex_str.is_empty() {
        return Err("Storage slot cannot be empty".to_string());
    }
    
    // Validate length (max 64 hex chars for 256 bits)
    if hex_str.len() > 64 {
        return Err(format!(
            "Storage slot too large: {} hex characters (max 64 for 256-bit value)", 
            hex_str.len()
        ));
    }
    
    // Validate all characters are hex
    if !hex_str.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(format!(
            "Storage slot contains invalid hex characters: '{}'", 
            hex_str
        ));
    }
    
    // Pad to 64 characters (32 bytes) with leading zeros
    let padded_hex = format!("{:0>64}", hex_str);
    
    Ok(format!("0x{}", padded_hex))
}

/// Parse layout information from JSON
fn parse_layout_from_json(json: &serde_json::Value) -> Result<LayoutInfo> {
    let commitment = json
        .get("commitment")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing layout commitment"))?
        .to_string();
    
    let contract_name = json
        .get("contract_name")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();
    
    let storage = json
        .get("storage_layout")
        .and_then(|v| v.get("storage"))
        .and_then(|v| v.as_array())
        .ok_or_else(|| anyhow::anyhow!("Missing storage layout"))?;
    
    let mut field_types = Vec::new();
    let mut field_semantics = Vec::new();
    let mut queries = Vec::new();
    
    for (i, entry) in storage.iter().enumerate() {
        let label = entry
            .get("label")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("field_{}", i));
        
        let type_name = entry
            .get("type_name")
            .and_then(|v| v.as_str())
            .unwrap_or("t_uint256");
        
        let slot = entry
            .get("slot")
            .and_then(|v| v.as_str())
            .unwrap_or("0");
        
        // Map Solidity types to traverse field types
        let field_type = match type_name {
            "t_bool" => "Bool",
            "t_uint8" => "Uint8",
            "t_uint16" => "Uint16", 
            "t_uint32" => "Uint32",
            "t_uint64" => "Uint64",
            "t_uint256" => "Uint256",
            "t_address" => "Address",
            "t_bytes32" => "Bytes32",
            "t_string" => "String",
            "t_bytes" => "Bytes",
            _ => "Uint256", // Default fallback
        };
        
        // Default semantics based on type
        let zero_semantics = match type_name {
            "t_address" => "NeverWritten", // Zero addresses usually indicate unset
            _ => "ValidZero", // Most fields can legitimately be zero
        };
        
        field_types.push(field_type.to_string());
        field_semantics.push(zero_semantics.to_string());
        
        // Parse storage slot as 256-bit hex value
        let expected_slot = parse_storage_slot(slot)
            .map_err(|e| anyhow::anyhow!("Invalid storage slot '{}' for field '{}': {}", slot, label, e))?;
        
        queries.push(QueryInfo {
            query: label,
            field_type: field_type.to_string(),
            zero_semantics: zero_semantics.to_string(),
            expected_slot,
        });
    }
    
    Ok(LayoutInfo {
        commitment,
        contract_name,
        field_types,
        field_semantics,
        queries,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_storage_slot() {
        // Test simple slot
        assert_eq!(
            parse_storage_slot("0").unwrap(),
            "0x0000000000000000000000000000000000000000000000000000000000000000"
        );
        
        // Test slot with 0x prefix
        assert_eq!(
            parse_storage_slot("0x1").unwrap(),
            "0x0000000000000000000000000000000000000000000000000000000000000001"
        );
        
        // Test slot without 0x prefix
        assert_eq!(
            parse_storage_slot("ff").unwrap(),
            "0x00000000000000000000000000000000000000000000000000000000000000ff"
        );
        
        // Test large 256-bit slot (keccak256 hash)
        let large_slot = "c1f51986c7e9d391993039c3c40e41ad9f26e1db9b80f8535a639eadeb1d1bd9";
        assert_eq!(
            parse_storage_slot(large_slot).unwrap(),
            format!("0x{}", large_slot)
        );
        
        // Test full 64-character slot
        let full_slot = "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff";
        assert_eq!(
            parse_storage_slot(full_slot).unwrap(),
            format!("0x{}", full_slot)
        );
        
        // Test error cases
        assert!(parse_storage_slot("").is_err()); // Empty
        assert!(parse_storage_slot("xyz").is_err()); // Invalid hex
        assert!(parse_storage_slot("1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1").is_err()); // Too long (65 chars)
        
        // Verify error messages are descriptive
        let err = parse_storage_slot("xyz").unwrap_err();
        assert!(err.contains("invalid hex characters"));
        
        let err = parse_storage_slot("1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1").unwrap_err();
        assert!(err.contains("too large"));
    }
    
    #[test]
    fn test_parse_storage_slot_edge_cases() {
        // Test maximum u64 value (should work fine)
        let max_u64 = "ffffffffffffffff"; // 18446744073709551615
        assert_eq!(
            parse_storage_slot(max_u64).unwrap(),
            "0x000000000000000000000000000000000000000000000000ffffffffffffffff"
        );
        
        // Test value larger than u64::MAX (this is why we needed the fix!)
        let larger_than_u64 = "1ffffffffffffffff"; // 9 hex chars = 36 bits > 64-bit
        assert_eq!(
            parse_storage_slot(larger_than_u64).unwrap(),
            "0x0000000000000000000000000000000000000000000000001ffffffffffffffff"
        );
        
        // Test mixed case
        assert_eq!(
            parse_storage_slot("0xAbCdEf").unwrap(),
            "0x0000000000000000000000000000000000000000000000000000000000abcdef"
        );
    }
} 