//! Minimal code generation command for creating schema-specific libraries
//!
//! This command generates minimal, purpose-built code that is tailored specifically
//! to the provided schema without any unnecessary abstractions or dependencies.

use anyhow::Result;
use clap::Args;
use std::path::PathBuf;
use std::fs;

#[derive(Debug, Args)]
pub struct GenerateMinimalArgs {
    /// Layout file containing storage layout information
    #[clap(long, short)]
    pub layout: PathBuf,
    
    /// Output file for generated code
    #[clap(long, short)]
    pub output: PathBuf,
    
    /// Type of minimal code to generate
    #[clap(long, value_enum, default_value = "combined")]
    pub code_type: MinimalCodeType,
}

#[derive(Debug, Clone, clap::ValueEnum)]
pub enum MinimalCodeType {
    /// Generate query code only
    Query,
    /// Generate verifier code only
    Verifier,
    /// Generate combined query and verifier code (default)
    Combined,
}

/// Generate minimal code from a layout file
pub async fn cmd_generate_minimal(args: GenerateMinimalArgs) -> Result<()> {
    println!("Generating minimal {} code...", match args.code_type {
        MinimalCodeType::Query => "query",
        MinimalCodeType::Verifier => "verifier",
        MinimalCodeType::Combined => "combined",
    });
    
    // Read and parse layout file
    let layout_content = fs::read_to_string(&args.layout)?;
    let layout: traverse_core::LayoutInfo = serde_json::from_str(&layout_content)?;
    
    // Generate code based on type
    let code = match args.code_type {
        MinimalCodeType::Query => {
            traverse_valence::minimal_codegen::generate_minimal_query_code(&layout)
        },
        MinimalCodeType::Verifier => {
            traverse_valence::minimal_codegen::generate_minimal_verifier_code(&layout)
        },
        MinimalCodeType::Combined => {
            traverse_valence::minimal_codegen::generate_minimal_combined_code(&layout)
        },
    };
    
    // Write output file
    fs::write(&args.output, code)?;
    
    println!("✓ Generated minimal code at: {}", args.output.display());
    println!("  - Contract: {}", layout.contract_name);
    println!("  - Fields: {}", layout.storage.len());
    println!("  - Layout commitment: 0x{}", hex::encode(layout.commitment()));
    
    // Show usage instructions
    println!("\nUsage:");
    match args.code_type {
        MinimalCodeType::Query => {
            println!("  - Use `compute_storage_key(field_index, key_data)` to get storage keys");
            println!("  - Field constants: {}", layout.storage.iter()
                .enumerate()
                .map(|(i, e)| format!("FIELD_{} = {}", e.label.to_uppercase(), i))
                .collect::<Vec<_>>()
                .join(", "));
        },
        MinimalCodeType::Verifier => {
            println!("  - Use `verify_witness(field_index, witness_data)` to verify and extract values");
            println!("  - Returns `ExtractedValue` enum with the typed value");
        },
        MinimalCodeType::Combined => {
            println!("  - Access fields directly: `FIELDS.{}.storage_key(key)`", 
                layout.storage.first().map(|e| &e.label).unwrap_or(&"field_name".to_string()));
            println!("  - Verify witnesses: `FIELDS.{}.verify_witness(witness_data)`",
                layout.storage.first().map(|e| &e.label).unwrap_or(&"field_name".to_string()));
            println!("  - Everything is compile-time validated with zero overhead");
        },
    }
    
    Ok(())
}

/// Generate minimal code for all storage fields in a layout
pub fn generate_field_specific_code(layout: &traverse_core::LayoutInfo) -> Result<String> {
    let mut code = String::new();
    
    // Generate header
    code.push_str(&format!(r#"//! Minimal field-specific code for {}
//!
//! This code is generated specifically for your schema with zero overhead.
//! Each field has its own optimized storage key computation and verification.

#![no_std]

"#, layout.contract_name));

    // Generate code for each field
    for (index, entry) in layout.storage.iter().enumerate() {
        code.push_str(&format!(r#"
/// Storage operations for field: {}
pub mod {} {{
    /// Compute storage key for {}
    pub const fn storage_key() -> [u8; 32] {{
        // Direct computation - no runtime overhead
        const SLOT: [u8; 32] = hex_literal::hex!("{}");
        SLOT
    }}
    
    /// Verify witness for {} (field index {})
    pub fn verify_witness(witness_data: &[u8]) -> Result<{}, &'static str> {{
        // Minimal validation - exactly what's needed
        if witness_data.len() < 176 {{
            return Err("Invalid witness");
        }}
        
        // Check field index
        let field_index = u16::from_le_bytes([witness_data[142], witness_data[143]]);
        if field_index != {} {{
            return Err("Wrong field");
        }}
        
        // Extract value at offset 64-96
        {}
    }}
}}
"#,
            entry.label,
            entry.label.to_lowercase(),
            entry.label,
            entry.slot.trim_start_matches("0x").chars()
                .collect::<String>()
                .chars()
                .collect::<Vec<_>>()
                .chunks(2)
                .map(|c| c.iter().collect::<String>())
                .collect::<Vec<_>>()
                .join("")
                .chars()
                .take(64)
                .collect::<String>(),
            entry.label,
            index,
            type_for_entry(&entry.type_name),
            index,
            extraction_code_for_type(&entry.type_name)
        ));
    }
    
    Ok(code)
}

fn type_for_entry(type_name: &str) -> &'static str {
    match type_name {
        "t_bool" => "bool",
        "t_uint8" => "u8",
        "t_uint16" => "u16",
        "t_uint32" => "u32",
        "t_uint64" => "u64",
        "t_uint256" => "[u8; 32]",
        "t_address" => "[u8; 20]",
        "t_bytes32" => "[u8; 32]",
        _ => "[u8; 32]",
    }
}

fn extraction_code_for_type(type_name: &str) -> &'static str {
    match type_name {
        "t_bool" => "Ok(witness_data[95] != 0)",
        "t_uint8" => "Ok(witness_data[95])",
        "t_uint16" => "Ok(u16::from_be_bytes([witness_data[94], witness_data[95]]))",
        "t_uint32" => "Ok(u32::from_be_bytes([witness_data[92], witness_data[93], witness_data[94], witness_data[95]]))",
        "t_uint64" => {
            "Ok(u64::from_be_bytes([
                witness_data[88], witness_data[89], witness_data[90], witness_data[91],
                witness_data[92], witness_data[93], witness_data[94], witness_data[95]
            ]))"
        },
        "t_address" => {
            "let mut addr = [0u8; 20];
            addr.copy_from_slice(&witness_data[76..96]);
            Ok(addr)"
        },
        _ => {
            "let mut value = [0u8; 32];
            value.copy_from_slice(&witness_data[64..96]);
            Ok(value)"
        }
    }
}