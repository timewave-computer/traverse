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
        
        queries.push(QueryInfo {
            query: label,
            field_type: field_type.to_string(),
            zero_semantics: zero_semantics.to_string(),
            expected_slot: format!("0x{:064x}", slot.parse::<u64>().unwrap_or(0)),
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