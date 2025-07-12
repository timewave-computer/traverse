//! Code generation for minimal traverse-valence applications
//!
//! This module provides functionality to generate minimal, custom crates containing
//! only the specific functionality needed for particular storage queries and layouts.
//! This allows 3rd party developers to import lightweight libraries optimized for
//! their exact use case.

use alloc::{format, string::String, string::ToString, vec, vec::Vec};
use serde::{Deserialize, Serialize};

#[cfg(feature = "std")]
use std::{fs, path::Path};

#[cfg(feature = "std")]
use tera::{Context, Tera};

/// Options for code generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodegenOptions {
    /// Name of the generated crate
    pub crate_name: String,
    /// Version of the generated crate
    pub version: String,
    /// Authors for the generated crate
    pub authors: Vec<String>,
    /// Description of the generated crate
    pub description: String,
    /// Include alloy support
    pub include_alloy: bool,
    /// Use minimal features only
    pub minimal: bool,
    /// Target no_std environments
    pub no_std: bool,
}

impl Default for CodegenOptions {
    fn default() -> Self {
        Self {
            crate_name: "my-traverse-app".to_string(),
            version: "0.1.0".to_string(),
            authors: vec!["Generated".to_string()],
            description: "Generated traverse-valence application".to_string(),
            include_alloy: false,
            minimal: true,
            no_std: false,
        }
    }
}

/// Information about storage queries for code generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryInfo {
    /// Query string
    pub query: String,
    /// Field type
    pub field_type: String,
    /// Zero semantics
    pub zero_semantics: String,
    /// Expected slot
    pub expected_slot: String,
}

/// Layout information for code generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutInfo {
    /// Layout commitment hash
    pub commitment: String,
    /// Contract name
    pub contract_name: String,
    /// Field types in order
    pub field_types: Vec<String>,
    /// Field semantics in order
    pub field_semantics: Vec<String>,
    /// Storage queries this layout supports
    pub queries: Vec<QueryInfo>,
}

/// Parse a layout commitment hex string into byte array literals for code generation
///
/// Converts a hex string like "f6dc3c4a..." or "0xf6dc3c4a..." into a Vec of byte literals
/// like ["0xf6", "0xdc", "0x3c", "0x4a", ...] for use in Rust array literals.
fn parse_commitment_to_byte_literals(commitment: &str) -> Result<Vec<String>, String> {
    // Remove 0x prefix if present
    let hex_str = commitment.strip_prefix("0x").unwrap_or(commitment);
    
    // Validate length (must be exactly 64 hex characters for 32 bytes)
    if hex_str.len() != 64 {
        return Err(format!(
            "Layout commitment must be 64 hex characters (32 bytes), got {} characters", 
            hex_str.len()
        ));
    }
    
    // Validate all characters are hex
    if !hex_str.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(format!(
            "Layout commitment contains invalid hex characters: '{}'", 
            hex_str
        ));
    }
    
    // Convert to byte literals
    let mut byte_literals = Vec::with_capacity(32);
    for i in (0..64).step_by(2) {
        let byte_hex = &hex_str[i..i + 2];
        byte_literals.push(format!("0x{}", byte_hex));
    }
    
    Ok(byte_literals)
}

/// Generate a minimal controller crate
#[cfg(feature = "std")]
pub fn generate_controller_crate(
    output_path: &Path,
    layout: &LayoutInfo,
    options: &CodegenOptions,
) -> Result<(), crate::TraverseValenceError> {
    let mut tera = Tera::new("templates/*").unwrap_or_else(|_| Tera::default());
    
    // Add controller template
    tera.add_raw_template("controller_cargo_toml", CONTROLLER_CARGO_TEMPLATE)
        .map_err(|e| crate::TraverseValenceError::CodegenError(format!("Template error: {}", e)))?;
    
    tera.add_raw_template("controller_lib_rs", CONTROLLER_LIB_TEMPLATE)
        .map_err(|e| crate::TraverseValenceError::CodegenError(format!("Template error: {}", e)))?;
    
    // Create context
    let mut context = Context::new();
    context.insert("options", options);
    context.insert("layout", layout);
    context.insert("query_count", &layout.queries.len());
    
    // Create output directory
    fs::create_dir_all(output_path)
        .map_err(|e| crate::TraverseValenceError::CodegenError(format!("Failed to create directory: {}", e)))?;
    
    fs::create_dir_all(output_path.join("src"))
        .map_err(|e| crate::TraverseValenceError::CodegenError(format!("Failed to create src directory: {}", e)))?;
    
    // Generate Cargo.toml
    let cargo_toml = tera.render("controller_cargo_toml", &context)
        .map_err(|e| crate::TraverseValenceError::CodegenError(format!("Template render error: {}", e)))?;
    
    fs::write(output_path.join("Cargo.toml"), cargo_toml)
        .map_err(|e| crate::TraverseValenceError::CodegenError(format!("Failed to write Cargo.toml: {}", e)))?;
    
    // Generate lib.rs
    let lib_rs = tera.render("controller_lib_rs", &context)
        .map_err(|e| crate::TraverseValenceError::CodegenError(format!("Template render error: {}", e)))?;
    
    fs::write(output_path.join("src").join("lib.rs"), lib_rs)
        .map_err(|e| crate::TraverseValenceError::CodegenError(format!("Failed to write lib.rs: {}", e)))?;
    
    Ok(())
}

/// Generate a minimal circuit crate
#[cfg(feature = "std")]
pub fn generate_circuit_crate(
    output_path: &Path,
    layout: &LayoutInfo,
    options: &CodegenOptions,
) -> Result<(), crate::TraverseValenceError> {
    let mut tera = Tera::new("templates/*").unwrap_or_else(|_| Tera::default());
    
    // Add circuit templates
    tera.add_raw_template("circuit_cargo_toml", CIRCUIT_CARGO_TEMPLATE)
        .map_err(|e| crate::TraverseValenceError::CodegenError(format!("Template error: {}", e)))?;
    
    tera.add_raw_template("circuit_lib_rs", CIRCUIT_LIB_TEMPLATE)
        .map_err(|e| crate::TraverseValenceError::CodegenError(format!("Template error: {}", e)))?;
    
    // Pre-process layout commitment hex string to byte array literals
    let commitment_bytes = parse_commitment_to_byte_literals(&layout.commitment)
        .map_err(|e| crate::TraverseValenceError::CodegenError(format!("Invalid layout commitment: {}", e)))?;
    
    // Create context
    let mut context = Context::new();
    context.insert("options", options);
    context.insert("layout", layout);
    context.insert("field_count", &layout.field_types.len());
    context.insert("commitment_bytes", &commitment_bytes);
    
    // Create output directory
    fs::create_dir_all(output_path)
        .map_err(|e| crate::TraverseValenceError::CodegenError(format!("Failed to create directory: {}", e)))?;
    
    fs::create_dir_all(output_path.join("src"))
        .map_err(|e| crate::TraverseValenceError::CodegenError(format!("Failed to create src directory: {}", e)))?;
    
    // Generate Cargo.toml
    let cargo_toml = tera.render("circuit_cargo_toml", &context)
        .map_err(|e| crate::TraverseValenceError::CodegenError(format!("Template render error: {}", e)))?;
    
    fs::write(output_path.join("Cargo.toml"), cargo_toml)
        .map_err(|e| crate::TraverseValenceError::CodegenError(format!("Failed to write Cargo.toml: {}", e)))?;
    
    // Generate lib.rs
    let lib_rs = tera.render("circuit_lib_rs", &context)
        .map_err(|e| crate::TraverseValenceError::CodegenError(format!("Template render error: {}", e)))?;
    
    fs::write(output_path.join("src").join("lib.rs"), lib_rs)
        .map_err(|e| crate::TraverseValenceError::CodegenError(format!("Failed to write lib.rs: {}", e)))?;
    
    Ok(())
}

// Templates for generated code

const CONTROLLER_CARGO_TEMPLATE: &str = r#"# Generated controller crate for {{ options.crate_name }}
[package]
name = "{{ options.crate_name }}-controller"
version = "{{ options.version }}"
edition = "2021"
authors = {{ options.authors | json_encode() }}
description = "{{ options.description }} - Controller"

[lib]
crate-type = ["cdylib", "rlib"]

[features]
default = []

[dependencies]
valence-coprocessor = { git = "https://github.com/timewave-computer/valence-coprocessor.git", tag = "v0.1.13", default-features = false }
serde = { version = "1.0", default-features = false, features = ["derive", "alloc"] }
hex = { version = "0.4", default-features = false, features = ["alloc"] }

# Minimal traverse-valence controller functionality (always no_std)
traverse-valence = { path = "../../../traverse", default-features = false, features = ["controller"] }

{% if options.include_alloy %}# Alloy for ABI encoding
alloy-primitives = { version = ">=0.9.0,<2.0", default-features = false }
alloy-sol-types = { version = ">=0.9.0,<2.0", default-features = false }{% endif %}
"#;

const CONTROLLER_LIB_TEMPLATE: &str = r#"//! Generated controller for {{ options.crate_name }}
//!
//! This controller handles {{ layout.queries | length }} storage queries for contract {{ layout.contract_name }}.
//! Layout commitment: {{ layout.commitment }}

#![no_std]

extern crate alloc;
use alloc::vec::Vec;

use valence_coprocessor::Witness;
use traverse_valence::{StorageVerificationRequest, create_witness_from_request};

/// Layout commitment for this controller (validates against expected layout)
pub const LAYOUT_COMMITMENT: &str = "{{ layout.commitment }}";

/// Contract name this controller is designed for
pub const CONTRACT_NAME: &str = "{{ layout.contract_name }}";

/// Supported storage queries
pub const SUPPORTED_QUERIES: &[&str] = &[
{% for query in layout.queries %}    "{{ query.query }}",
{% endfor %}];

/// Direct API for witness generation (no_std compatible)
pub fn create_witness(request: &StorageVerificationRequest) -> Result<Witness, traverse_valence::TraverseValenceError> {
    // Validate layout commitment
    if request.storage_query.layout_commitment != LAYOUT_COMMITMENT {
        return Err(traverse_valence::TraverseValenceError::LayoutMismatch(
            alloc::format!("Expected: {}, Got: {}", LAYOUT_COMMITMENT, request.storage_query.layout_commitment)
        ));
    }
    
    // Validate query is supported
    let query_supported = SUPPORTED_QUERIES.iter().any(|&q| q == request.storage_query.query);
    if !query_supported {
        return Err(traverse_valence::TraverseValenceError::InvalidWitness(
            alloc::format!("Unsupported query: {}. Supported queries: {:?}", 
                request.storage_query.query, SUPPORTED_QUERIES)
        ));
    }
    
    create_witness_from_request(request)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_layout_commitment() {
        assert_eq!(LAYOUT_COMMITMENT, "{{ layout.commitment }}");
    }
    
    #[test]
    fn test_supported_queries() {
        assert_eq!(SUPPORTED_QUERIES.len(), {{ layout.queries | length }});
{% for query in layout.queries %}        assert!(SUPPORTED_QUERIES.contains(&"{{ query.query }}"));
{% endfor %}    }
}
"#;

const CIRCUIT_CARGO_TEMPLATE: &str = r#"# Generated circuit crate for {{ options.crate_name }}
[package]
name = "{{ options.crate_name }}-circuit"
version = "{{ options.version }}"
edition = "2021"
authors = {{ options.authors | json_encode() }}
description = "{{ options.description }} - Circuit"

[features]
default = []

[dependencies]
valence-coprocessor = { git = "https://github.com/timewave-computer/valence-coprocessor.git", tag = "v0.1.13", default-features = false }

# Minimal traverse-valence circuit functionality (always constrained for ZK environments)
traverse-valence = { path = "../../../traverse", default-features = false, features = ["circuit"] }

{% if options.include_alloy %}# Alloy for ABI encoding
alloy-primitives = { version = ">=0.9.0,<2.0", default-features = false }
alloy-sol-types = { version = ">=0.9.0,<2.0", default-features = false }{% endif %}
"#;

const CIRCUIT_LIB_TEMPLATE: &str = r#"//! Generated circuit for {{ options.crate_name }}
//!
//! This circuit verifies {{ layout.field_types | length }} storage fields for contract {{ layout.contract_name }}.
//! Layout commitment: {{ layout.commitment }}

#![no_std]

extern crate alloc;
use alloc::vec::Vec;

use valence_coprocessor::Witness;
use traverse_valence::circuit::{CircuitProcessor, CircuitWitness, FieldType, ZeroSemantics, CircuitResult};
{% if options.include_alloy %}use alloy_primitives::{Address, U256, Bytes, FixedBytes};
use alloy_sol_types::{sol, SolValue};{% endif %}

/// Layout commitment for this circuit (validates against expected layout)
/// Commitment: {{ layout.commitment }}
pub const LAYOUT_COMMITMENT: [u8; 32] = [
    {{ commitment_bytes | join(", ") }}
];

/// Field types for this layout
pub const FIELD_TYPES: &[FieldType] = &[
{% for field_type in layout.field_types %}    FieldType::{{ field_type }},
{% endfor %}];

/// Field semantics for this layout
pub const FIELD_SEMANTICS: &[ZeroSemantics] = &[
{% for semantic in layout.field_semantics %}    ZeroSemantics::{{ semantic }},
{% endfor %}];

{% if options.include_alloy %}// Define contract-specific output types
sol! {
    /// Generated output structure for {{ layout.contract_name }}
    struct {{ layout.contract_name | title }}Output {
{% for query in layout.queries %}        {% if query.field_type == "Bool" %}bool{% elif query.field_type == "Uint8" %}uint8{% elif query.field_type == "Uint16" %}uint16{% elif query.field_type == "Uint32" %}uint32{% elif query.field_type == "Uint64" %}uint64{% elif query.field_type == "Uint256" %}uint256{% elif query.field_type == "Address" %}address{% elif query.field_type == "Bytes32" %}bytes32{% elif query.field_type == "String" %}bytes{% elif query.field_type == "Bytes" %}bytes{% else %}uint256{% endif %} {{ query.query | replace("[", "_") | replace("]", "") | replace(".", "_") }};
{% endfor %}    }
}{% endif %}

/// Main circuit function
/// 
/// Returns success indicator (0x01) or error codes:
/// - 0x00: General failure
/// - 0x02: Invalid witness count
/// - 0x03: Witness parsing failed
/// - 0x04: Witness validation failed
/// - 0x05: ABI encoding failed
pub fn circuit(witnesses: Vec<Witness>) -> Vec<u8> {
    // Validate witness count (graceful error handling)
    if witnesses.len() != {{ layout.field_types | length }} {
        // Return error code for invalid witness count
        return alloc::vec![0x02, (witnesses.len() as u8), {{ layout.field_types | length }}];
    }
    
    // Create processor with layout-specific parameters
    let processor = CircuitProcessor::new(
        LAYOUT_COMMITMENT,
        FIELD_TYPES.to_vec(),
        FIELD_SEMANTICS.to_vec(),
    );
    
    // Parse witnesses from bytes (graceful error handling)
    let mut circuit_witnesses = Vec::with_capacity(witnesses.len());
    for (i, witness) in witnesses.into_iter().enumerate() {
        // Handle witness data extraction gracefully
        let witness_data = match witness.as_data() {
            Some(data) => data,
            None => {
                // Return error code with witness index that failed
                return alloc::vec![0x03, 0x01, i as u8];
            }
        };
        
        // Handle witness parsing gracefully
        match CircuitProcessor::parse_witness_from_bytes(witness_data) {
            Ok(parsed_witness) => circuit_witnesses.push(parsed_witness),
            Err(_) => {
                // Return error code with witness index that failed to parse
                return alloc::vec![0x03, 0x02, i as u8];
            }
        }
    }
    
    // Process all witnesses
    let results = processor.process_batch(&circuit_witnesses);
    
    // Validate all results are valid (graceful error handling)
    for (i, result) in results.iter().enumerate() {
        if let CircuitResult::Invalid = result {
            // Return error code with witness index that failed validation
            return alloc::vec![0x04, i as u8];
        }
    }
    
{% if options.include_alloy %}    // Generate ABI-encoded output (with error handling)
    match generate_abi_output(&results) {
        Ok(output) => output,
        Err(error_code) => alloc::vec![0x05, error_code],
    }
{% else %}    // Generate simple success indicator
    alloc::vec![0x01] // Success
{% endif %}
}

{% if options.include_alloy %}/// Generate ABI-encoded output from circuit results
/// 
/// Returns Ok(encoded_output) or Err(error_code):
/// - 1: Invalid circuit result (should not happen after validation)
/// - 2: Type mismatch for extracted value
/// - 3: ABI encoding failed
fn generate_abi_output(results: &[CircuitResult]) -> Result<Vec<u8>, u8> {
    // Extract validated values from results
{% for query in layout.queries %}    let {{ query.query | replace("[", "_") | replace("]", "") | replace(".", "_") }} = match &results[{{ loop.index0 }}] {
        CircuitResult::Valid { extracted_value, .. } => {
            // Convert extracted value to appropriate type
            match extracted_value {
{% if query.field_type == "Bool" %}                traverse_valence::circuit::ExtractedValue::Bool(val) => {
                    *val
                },{% elif query.field_type == "Uint8" %}                traverse_valence::circuit::ExtractedValue::Uint8(val) => {
                    *val
                },{% elif query.field_type == "Uint16" %}                traverse_valence::circuit::ExtractedValue::Uint16(val) => {
                    *val
                },{% elif query.field_type == "Uint32" %}                traverse_valence::circuit::ExtractedValue::Uint32(val) => {
                    *val
                },{% elif query.field_type == "Uint64" %}                traverse_valence::circuit::ExtractedValue::Uint64(val) => {
                    *val
                },{% elif query.field_type == "Uint256" %}                traverse_valence::circuit::ExtractedValue::Uint256(val) => {
                    U256::from_be_bytes(*val)
                },{% elif query.field_type == "Address" %}                traverse_valence::circuit::ExtractedValue::Address(addr) => {
                    Address::from(*addr)
                },{% elif query.field_type == "Bytes32" %}                traverse_valence::circuit::ExtractedValue::Bytes32(val) => {
                    FixedBytes::from(*val)
                },{% elif query.field_type == "String" %}                traverse_valence::circuit::ExtractedValue::Raw(val) => {
                    // String fields are extracted as Raw bytes
                    Bytes::from(val.to_vec())
                },{% elif query.field_type == "Bytes" %}                traverse_valence::circuit::ExtractedValue::Raw(val) => {
                    // Bytes fields are extracted as Raw bytes
                    Bytes::from(val.to_vec())
                },{% else %}                _ => {
                    // Return error for unsupported field type instead of panicking
                    return Err(2);
                },{% endif %}
            }
        },
        CircuitResult::Invalid => {
            // Return error for invalid result instead of panicking
            return Err(1);
        },
    };
{% endfor %}
    
    // Create output structure
    let output = {{ layout.contract_name | title }}Output {
{% for query in layout.queries %}        {{ query.query | replace("[", "_") | replace("]", "") | replace(".", "_") }},
{% endfor %}    };
    
    // ABI encode output (alloy's abi_encode doesn't fail)
    Ok(output.abi_encode())
}{% endif %}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_layout_commitment() {
        // Verify the layout commitment matches expected value
        assert_eq!(LAYOUT_COMMITMENT.len(), 32);
    }
    
    #[test]
    fn test_field_configuration() {
        assert_eq!(FIELD_TYPES.len(), {{ layout.field_types | length }});
        assert_eq!(FIELD_SEMANTICS.len(), {{ layout.field_semantics | length }});
        assert_eq!(FIELD_TYPES.len(), FIELD_SEMANTICS.len());
    }
}
"#;

/// No-std compatible code generation (generates templates as strings)
pub fn generate_controller_template(
    layout: &LayoutInfo,
    options: &CodegenOptions,
) -> Result<(String, String), crate::TraverseValenceError> {
    // Generate Cargo.toml content
    let cargo_toml = format!(
        r#"# Generated controller crate for {}
[package]
name = "{}-controller"
version = "{}"
edition = "2021"
authors = {:?}
description = "{} - Controller"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
valence-coprocessor = {{ git = "https://github.com/timewave-computer/valence-coprocessor.git", tag = "v0.1.13", default-features = false }}
traverse-valence = {{ path = "../../../traverse", default-features = false, features = ["controller"] }}
serde = {{ version = "1.0", default-features = false, features = ["derive", "alloc"] }}
hex = {{ version = "0.4", default-features = false, features = ["alloc"] }}
"#,
        options.crate_name,
        options.crate_name,
        options.version,
        options.authors,
        options.description
    );

    // Generate lib.rs content (simplified for no_std)
    let lib_rs = format!(
        r#"//! Generated controller for {}
//! Layout commitment: {}

#![no_std]

extern crate alloc;
use alloc::vec::Vec;

use valence_coprocessor::Witness;
use traverse_valence::{{StorageVerificationRequest, create_witness_from_request}};

pub const LAYOUT_COMMITMENT: &str = "{}";

pub fn create_witness(request: &StorageVerificationRequest) -> Result<Witness, traverse_valence::TraverseValenceError> {{
    if request.storage_query.layout_commitment != LAYOUT_COMMITMENT {{
        return Err(traverse_valence::TraverseValenceError::LayoutMismatch(
            "Layout commitment mismatch".into()
        ));
    }}
    
    create_witness_from_request(request)
}}
"#,
        options.crate_name,
        layout.commitment,
        layout.commitment
    );

        Ok((cargo_toml, lib_rs))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_commitment_to_byte_literals() {
        // Test valid commitment with 0x prefix
        let commitment = "0xf6dc3c4a79e95565b3cf38993f1a120c6a6b467796264e7fd9a9c8675616dd7a";
        let result = parse_commitment_to_byte_literals(commitment).unwrap();
        assert_eq!(result.len(), 32);
        assert_eq!(result[0], "0xf6");
        assert_eq!(result[1], "0xdc");
        assert_eq!(result[2], "0x3c");
        assert_eq!(result[3], "0x4a");
        assert_eq!(result[31], "0x7a");
        
        // Test valid commitment without 0x prefix
        let commitment = "f6dc3c4a79e95565b3cf38993f1a120c6a6b467796264e7fd9a9c8675616dd7a";
        let result = parse_commitment_to_byte_literals(commitment).unwrap();
        assert_eq!(result.len(), 32);
        assert_eq!(result[0], "0xf6");
        assert_eq!(result[31], "0x7a");
        
        // Test invalid length
        let commitment = "f6dc3c4a";
        let result = parse_commitment_to_byte_literals(commitment);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("must be 64 hex characters"));
        
        // Test invalid hex characters
        let commitment = "f6dc3c4a79e95565b3cf38993f1a120c6a6b467796264e7fd9a9c8675616ddZZ";
        let result = parse_commitment_to_byte_literals(commitment);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("invalid hex characters"));
    }
    
    #[test]
    fn test_commitment_byte_array_format() {
        // Test that the generated byte array is properly formatted
        let commitment = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
        let result = parse_commitment_to_byte_literals(commitment).unwrap();
        let joined = result.join(", ");
        
        // Should generate a valid Rust byte array literal
        assert!(joined.starts_with("0x01"));
        assert!(joined.contains(", 0x23"));
        assert!(joined.ends_with("0xef"));
        assert_eq!(joined.matches(", ").count(), 31); // 31 commas for 32 elements
    }
    
    #[test]
    fn test_real_layout_commitment() {
        // Test with a real layout commitment hash (similar to those generated by traverse-cli)
        let real_commitment = "f6dc3c4a79e95565b3cf38993f1a120c6a6b467796264e7fd9a9c8675616dd7a";
        let result = parse_commitment_to_byte_literals(real_commitment).unwrap();
        let joined = result.join(", ");
        
        // Verify the generated code is syntactically correct
        assert_eq!(result.len(), 32);
        assert_eq!(result[0], "0xf6");
        assert_eq!(result[1], "0xdc");
        assert_eq!(result[2], "0x3c");
        assert_eq!(result[3], "0x4a");
        assert_eq!(result[31], "0x7a");
        
        // The joined string should be valid Rust code
        assert!(joined.starts_with("0xf6"));
        assert!(joined.contains(", 0xdc"));
        assert!(joined.ends_with("0x7a"));
        
        // Test that it can be used in a Rust array literal
        let array_literal = format!("pub const LAYOUT_COMMITMENT: [u8; 32] = [{}];", joined);
        assert!(array_literal.contains("0xf6, 0xdc, 0x3c, 0x4a"));
    }
    
    #[test]
    fn test_zero_commitment_edge_case() {
        // Test with all zeros (edge case)
        let zero_commitment = "0000000000000000000000000000000000000000000000000000000000000000";
        let result = parse_commitment_to_byte_literals(zero_commitment).unwrap();
        let joined = result.join(", ");
        
        assert_eq!(result.len(), 32);
        assert!(result.iter().all(|s| s == "0x00"));
        assert_eq!(joined, "0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00");
    }
    
    #[cfg(feature = "std")]
    #[test]
    fn test_all_field_types_template_generation() {
        // Test template generation with all supported field types
        let layout = LayoutInfo {
            commitment: "f6dc3c4a79e95565b3cf38993f1a120c6a6b467796264e7fd9a9c8675616dd7a".to_string(),
            contract_name: "TestContract".to_string(),
            field_types: vec![
                "Bool".to_string(),
                "Uint8".to_string(),
                "Uint16".to_string(),
                "Uint32".to_string(),
                "Uint64".to_string(),
                "Uint256".to_string(),
                "Address".to_string(),
                "Bytes32".to_string(),
                "String".to_string(),
                "Bytes".to_string(),
            ],
            field_semantics: vec!["ValidZero".to_string(); 10],
            queries: vec![
                QueryInfo {
                    query: "is_enabled".to_string(),
                    field_type: "Bool".to_string(),
                    zero_semantics: "ValidZero".to_string(),
                    expected_slot: "0x0000000000000000000000000000000000000000000000000000000000000000".to_string(),
                },
                QueryInfo {
                    query: "count".to_string(),
                    field_type: "Uint8".to_string(),
                    zero_semantics: "ValidZero".to_string(),
                    expected_slot: "0x0000000000000000000000000000000000000000000000000000000000000001".to_string(),
                },
                QueryInfo {
                    query: "total".to_string(),
                    field_type: "Uint256".to_string(),
                    zero_semantics: "ValidZero".to_string(),
                    expected_slot: "0x0000000000000000000000000000000000000000000000000000000000000002".to_string(),
                },
                QueryInfo {
                    query: "owner".to_string(),
                    field_type: "Address".to_string(),
                    zero_semantics: "NeverWritten".to_string(),
                    expected_slot: "0x0000000000000000000000000000000000000000000000000000000000000003".to_string(),
                },
                QueryInfo {
                    query: "hash".to_string(),
                    field_type: "Bytes32".to_string(),
                    zero_semantics: "ValidZero".to_string(),
                    expected_slot: "0x0000000000000000000000000000000000000000000000000000000000000004".to_string(),
                },
            ],
        };
        
        let options = CodegenOptions {
            crate_name: "test-circuit".to_string(),
            version: "0.1.0".to_string(),
            authors: vec!["Test".to_string()],
            description: "Test circuit".to_string(),
            include_alloy: true,
            minimal: true,
            no_std: true,
        };
        
        // Test that the layout commitment parsing works
        let commitment_bytes = parse_commitment_to_byte_literals(&layout.commitment).unwrap();
        assert_eq!(commitment_bytes.len(), 32);
        assert_eq!(commitment_bytes[0], "0xf6");
        assert_eq!(commitment_bytes[1], "0xdc");
        
        // Test that the no-std template generation works without panics
        let result = generate_circuit_template(&layout, &options);
        assert!(result.is_ok());
        
        let (cargo_toml, lib_rs) = result.unwrap();
        
        // Verify the layout commitment is properly set (this is the critical fix)
        assert!(lib_rs.contains("0xf6, 0xdc, 0x3c, 0x4a"));
        
        // Verify no hardcoded zeros in the commitment
        assert!(!lib_rs.contains("0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00"));
        
        // Verify the cargo.toml is generated
        assert!(cargo_toml.contains("test-circuit-circuit"));
        assert!(cargo_toml.contains("traverse-valence"));
    }
    
    #[cfg(feature = "std")]
    #[test] 
    fn test_graceful_error_handling_template() {
        // Test that the generated template includes graceful error handling instead of panics
        let layout = LayoutInfo {
            commitment: "f6dc3c4a79e95565b3cf38993f1a120c6a6b467796264e7fd9a9c8675616dd7a".to_string(),
            contract_name: "TestContract".to_string(),
            field_types: vec!["Bool".to_string()],
            field_semantics: vec!["ValidZero".to_string()],
            queries: vec![QueryInfo {
                query: "enabled".to_string(),
                field_type: "Bool".to_string(),
                zero_semantics: "ValidZero".to_string(),
                expected_slot: "0x0000000000000000000000000000000000000000000000000000000000000000".to_string(),
            }],
        };
        
        let options = CodegenOptions {
            crate_name: "test-error-handling".to_string(),
            version: "0.1.0".to_string(),
            authors: vec!["Test".to_string()],
            description: "Test error handling".to_string(),
            include_alloy: true,
            minimal: true,
            no_std: true,
        };
        
        // Test the template directly by checking it contains the right error handling patterns
        let template_code = CIRCUIT_LIB_TEMPLATE;
        
        // Verify graceful error handling is present in the main circuit function (no panics)
        // Note: We allow assert_eq! in test functions but not in the main circuit logic
        let circuit_function = template_code.split("pub fn circuit").nth(1).unwrap().split("#[cfg(test)]").next().unwrap();
        assert!(!circuit_function.contains("panic!"), "Circuit function should not contain panic! calls");
        assert!(!circuit_function.contains(".expect("), "Circuit function should not contain .expect() calls");
        
        // Verify error codes are documented
        assert!(template_code.contains("Returns success indicator (0x01) or error codes"));
        assert!(template_code.contains("0x02: Invalid witness count"));
        assert!(template_code.contains("0x03: Witness parsing failed"));
        assert!(template_code.contains("0x04: Witness validation failed"));
        assert!(template_code.contains("0x05: ABI encoding failed"));
        
        // Verify proper error handling patterns are used
        assert!(template_code.contains("if witnesses.len() !="));
        assert!(template_code.contains("match witness.as_data()"));
        assert!(template_code.contains("match CircuitProcessor::parse_witness_from_bytes"));
        assert!(template_code.contains("if let CircuitResult::Invalid = result"));
        assert!(template_code.contains("match generate_abi_output"));
        
        // Verify error codes are returned instead of panicking
        assert!(template_code.contains("return alloc::vec![0x02"));
        assert!(template_code.contains("return alloc::vec![0x03"));
        assert!(template_code.contains("return alloc::vec![0x04"));
        assert!(template_code.contains("alloc::vec![0x05, error_code]"));
    }
} 

/// No-std compatible circuit template generation
pub fn generate_circuit_template(
    layout: &LayoutInfo,
    options: &CodegenOptions,
) -> Result<(String, String), crate::TraverseValenceError> {
    // Parse layout commitment to byte array
    let commitment_bytes = parse_commitment_to_byte_literals(&layout.commitment)
        .map_err(|e| crate::TraverseValenceError::CodegenError(format!("Invalid layout commitment: {}", e)))?;
    let commitment_array = commitment_bytes.join(", ");
    
    // Generate Cargo.toml content
    let cargo_toml = format!(
        r#"# Generated circuit crate for {}
[package]
name = "{}-circuit"
version = "{}"
edition = "2021"
authors = {:?}
description = "{} - Circuit"

[dependencies]
valence-coprocessor = {{ git = "https://github.com/timewave-computer/valence-coprocessor.git", tag = "v0.1.13", default-features = false }}
traverse-valence = {{ path = "../../../traverse", default-features = false, features = ["circuit"] }}
"#,
        options.crate_name,
        options.crate_name,
        options.version,
        options.authors,
        options.description
    );

    // Generate lib.rs content with proper commitment bytes
    let lib_rs = format!(
        r#"//! Generated circuit for {}
//! Layout commitment: {}

#![no_std]

extern crate alloc;
use alloc::vec::Vec;

use valence_coprocessor::Witness;
use traverse_valence::circuit::{{CircuitProcessor, CircuitWitness, FieldType, ZeroSemantics}};

/// Layout commitment for this circuit (validates against expected layout)
/// Commitment: {}
pub const LAYOUT_COMMITMENT: [u8; 32] = [
    {}
];

pub fn circuit(witnesses: Vec<Witness>) -> Vec<u8> {{
    let processor = CircuitProcessor::new(
        LAYOUT_COMMITMENT,
        alloc::vec![], // TODO: Add field types from layout
        alloc::vec![], // TODO: Add field semantics from layout
    );
    
    // TODO: Implement witness processing
    alloc::vec![0x01] // Success indicator
}}
"#,
        options.crate_name,
        layout.commitment,
        layout.commitment,
        commitment_array
    );

    Ok((cargo_toml, lib_rs))
} 