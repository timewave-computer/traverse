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
valence-coprocessor = {{ "{{" }} git = "https://github.com/timewave-computer/valence-coprocessor.git", tag = "v0.1.13", default-features = false {{ "}}" }}
traverse-valence = {{ "{{" }} path = "../../../traverse", default-features = false, features = ["controller"] {{ "}}" }}
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
        assert!(result.is_ok(), "Valid layout should generate code");
        
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

    #[test]
    fn test_security_layout_commitment_injection() {
        // Security Test: Layout commitment injection prevention
        let malicious_commitments = [
            "'; DROP TABLE users; --", // SQL injection attempt
            "<script>alert(1)</script>", // XSS attempt  
            "../../etc/passwd", // Path traversal
            "\n\r\t\0", // Control characters
            // Buffer overflow attempt - use format! to create the long string
            &format!("0x{}", "41".repeat(1000)), // Buffer overflow attempt
            "", // Empty string
            "not_hex_at_all", // Invalid hex
            "0x", // Just prefix
            "0xzzzz", // Invalid hex characters
        ];
        
        for (i, malicious_input) in malicious_commitments.iter().enumerate() {
            let result = parse_commitment_to_byte_literals(malicious_input);
            
            // Should either parse correctly or return error - never panic or execute code
            match result {
                Ok(bytes) => {
                    assert_eq!(bytes.len(), 32, "Valid parsing should produce 32 bytes");
                    // Verify the output is safe for use in generated code
                    for byte_literal in bytes {
                        assert!(byte_literal.starts_with("0x"), "Byte literal should start with 0x");
                        assert!(byte_literal.len() == 4, "Byte literal should be 4 chars (0x + 2 hex)");
                    }
                }
                Err(error) => {
                    // Error should be descriptive but not leak internal details
                    assert!(!error.is_empty(), "Error message should not be empty");
                    assert!(!error.contains("panic"), "Error should not mention panic");
                    assert!(!error.contains("unwrap"), "Error should not mention unwrap");
                    // Malicious input correctly rejected
                }
            }
        }
    }

    #[test]
    fn test_security_template_injection_prevention() {
        // Security Test: Template injection prevention in generated code
        let malicious_layout = LayoutInfo {
            commitment: "f6dc3c4a79e95565b3cf38993f1a120c6a6b467796264e7fd9a9c8675616dd7a".to_string(),
            contract_name: "'; rm -rf /; echo 'pwned".to_string(), // Command injection attempt
            field_types: vec!["<script>alert(1)</script>".to_string()], // XSS attempt
            field_semantics: vec!["{{7*7}}".to_string()], // Template injection
            queries: vec![QueryInfo {
                query: "{{constructor.constructor('return process')().exit()}}".to_string(), // JS injection
                field_type: "Uint256".to_string(),
                zero_semantics: "ValidZero".to_string(),
                expected_slot: "0x0000000000000000000000000000000000000000000000000000000000000000".to_string(),
            }],
        };
        
        let options = CodegenOptions::default();
        
        // Test that template generation handles malicious input safely
        let result = generate_circuit_template(&malicious_layout, &options);
        
        match result {
            Ok((cargo_toml, lib_rs)) => {
                // Verify malicious input is escaped/sanitized in generated code
                assert!(!cargo_toml.contains("rm -rf"), "Generated Cargo.toml should not contain shell commands");
                assert!(!cargo_toml.contains("<script>"), "Generated Cargo.toml should not contain script tags");
                
                assert!(!lib_rs.contains("rm -rf"), "Generated lib.rs should not contain shell commands");
                assert!(!lib_rs.contains("<script>"), "Generated lib.rs should not contain script tags");
                assert!(!lib_rs.contains("constructor"), "Generated lib.rs should not contain JS injection");
                
                // Generated code should still be syntactically valid Rust
                assert!(lib_rs.contains("pub const LAYOUT_COMMITMENT"), "Should contain layout commitment");
                assert!(lib_rs.contains("#![no_std]"), "Should be no_std compatible");
            }
            Err(_) => {
                // If rejected, should be for security reasons
                // Malicious layout appropriately rejected
            }
        }
    }

    #[test]
    fn test_security_generated_code_compilation_safety() {
        // Security Test: Generated code should be safe to compile
        let layout = LayoutInfo {
            commitment: "f6dc3c4a79e95565b3cf38993f1a120c6a6b467796264e7fd9a9c8675616dd7a".to_string(),
            contract_name: "TestContract".to_string(),
            field_types: vec!["Uint256".to_string(), "Address".to_string()],
            field_semantics: vec!["ValidZero".to_string(), "NeverWritten".to_string()],
            queries: vec![
                QueryInfo {
                    query: "balance".to_string(),
                    field_type: "Uint256".to_string(),
                    zero_semantics: "ValidZero".to_string(),
                    expected_slot: "0x0000000000000000000000000000000000000000000000000000000000000000".to_string(),
                },
                QueryInfo {
                    query: "owner".to_string(),
                    field_type: "Address".to_string(),
                    zero_semantics: "NeverWritten".to_string(),
                    expected_slot: "0x0000000000000000000000000000000000000000000000000000000000000001".to_string(),
                },
            ],
        };
        
        let options = CodegenOptions::default();
        
        let result = generate_circuit_template(&layout, &options);
        assert!(result.is_ok(), "Valid layout should generate code");
        
        let (cargo_toml, lib_rs) = result.unwrap();
        
        // Verify generated code has security properties
        
        // 1. No unsafe blocks in generated code
        assert!(!lib_rs.contains("unsafe"), "Generated code should not contain unsafe blocks");
        
        // 2. No unwrap() calls that could panic
        let unsafe_patterns = ["unwrap()", "expect(", "panic!(", "unreachable!()"];
        for pattern in unsafe_patterns {
            assert!(!lib_rs.contains(pattern), "Generated code should not contain {}", pattern);
        }
        
        // 3. Proper error handling patterns
        assert!(lib_rs.contains("Result<"), "Generated code should use Result for error handling");
        assert!(lib_rs.contains("match "), "Generated code should use pattern matching");
        
        // 4. No hardcoded credentials or secrets
        let secret_patterns = ["password", "secret", "key", "token", "auth"];
        for pattern in secret_patterns {
            // Should not contain these as variable names (could be in comments)
            let lines_with_pattern: Vec<_> = lib_rs.lines()
                .filter(|line| !line.trim().starts_with("//") && !line.trim().starts_with("*"))
                .filter(|line| line.to_lowercase().contains(pattern))
                .collect();
            
            if !lines_with_pattern.is_empty() {
                // Allow certain legitimate uses
                for line in lines_with_pattern {
                    assert!(
                        line.contains("storage_key") || 
                        line.contains("layout_key") ||
                        line.contains("field_key"),
                        "Unexpected secret-like pattern in generated code: {}", line
                    );
                }
            }
        }
        
        // 5. Verify commitment bytes are properly escaped
        assert!(lib_rs.contains("0xf6, 0xdc"), "Layout commitment should be properly formatted");
    }

    #[test]
    fn test_security_field_type_validation() {
        // Security Test: Field type validation to prevent code injection
        let malicious_field_types = [
            "(); system(\"rm -rf /\"); //", // Code injection
            "'; DROP TABLE users; --", // SQL injection style
            "<script>alert(1)</script>", // XSS style
            "{{7*7}}", // Template injection
            "Uint256; panic!(\"pwned\")", // Rust code injection
            "\n\r\t\0", // Control characters
            &"A".repeat(1000), // Very long field type
        ];
        
        for (i, malicious_type) in malicious_field_types.iter().enumerate() {
            let layout = LayoutInfo {
                commitment: "f6dc3c4a79e95565b3cf38993f1a120c6a6b467796264e7fd9a9c8675616dd7a".to_string(),
                contract_name: "TestContract".to_string(),
                field_types: vec![malicious_type.to_string()],
                field_semantics: vec!["ValidZero".to_string()],
                queries: vec![QueryInfo {
                    query: "test_field".to_string(),
                    field_type: malicious_type.to_string(),
                    zero_semantics: "ValidZero".to_string(),
                    expected_slot: "0x0000000000000000000000000000000000000000000000000000000000000000".to_string(),
                }],
            };
            
            let options = CodegenOptions::default();
            let result = generate_circuit_template(&layout, &options);
            
            match result {
                Ok((_, lib_rs)) => {
                    // If accepted, verify the malicious content is safely escaped
                    assert!(!lib_rs.contains("system("), "Generated code should not contain system calls");
                    assert!(!lib_rs.contains("DROP TABLE"), "Generated code should not contain SQL");
                    assert!(!lib_rs.contains("panic!(\"pwned\")"), "Generated code should not contain injected panics");
                    // Malicious field type was sanitized
                }
                Err(_) => {
                    // If rejected, that's also acceptable
                    // Malicious field type was rejected
                }
            }
        }
    }

    #[test]
    fn test_security_path_traversal_prevention() {
        // Security Test: Prevent path traversal in generated file paths
        // This test ensures that if paths were generated, they would be safe
        
        let layout = LayoutInfo {
            commitment: "f6dc3c4a79e95565b3cf38993f1a120c6a6b467796264e7fd9a9c8675616dd7a".to_string(),
            contract_name: "../../../etc/passwd".to_string(), // Path traversal attempt
            field_types: vec!["Uint256".to_string()],
            field_semantics: vec!["ValidZero".to_string()],
            queries: vec![QueryInfo {
                query: "../../root/.ssh/id_rsa".to_string(), // Path traversal in query
                field_type: "Uint256".to_string(),
                zero_semantics: "ValidZero".to_string(),
                expected_slot: "0x0000000000000000000000000000000000000000000000000000000000000000".to_string(),
            }],
        };
        
        let options = CodegenOptions {
            crate_name: "../../etc/passwd".to_string(), // Path traversal in crate name
            ..Default::default()
        };
        
        let result = generate_circuit_template(&layout, &options);
        
        match result {
            Ok((cargo_toml, lib_rs)) => {
                // Verify path traversal sequences are not present in generated content
                assert!(!cargo_toml.contains("../"), "Generated Cargo.toml should not contain path traversal");
                assert!(!lib_rs.contains("../"), "Generated lib.rs should not contain path traversal");
                assert!(!lib_rs.contains("/etc/passwd"), "Generated lib.rs should not reference system files");
                assert!(!lib_rs.contains("/root/"), "Generated lib.rs should not reference root directory");
            }
            Err(_) => {
                // Path traversal attempt appropriately rejected
            }
        }
    }

    #[test]
    fn test_security_resource_exhaustion_protection() {
        // Security Test: Resource exhaustion protection during code generation
        
        // Test with extremely large layout
        let large_field_types = vec!["Uint256".to_string(); 10000];
        let large_field_semantics = vec!["ValidZero".to_string(); 10000];
        let large_queries: Vec<QueryInfo> = (0..10000).map(|i| QueryInfo {
            query: format!("field_{}", i),
            field_type: "Uint256".to_string(),
            zero_semantics: "ValidZero".to_string(),
            expected_slot: format!("0x{:064x}", i),
        }).collect();
        
        let large_layout = LayoutInfo {
            commitment: "f6dc3c4a79e95565b3cf38993f1a120c6a6b467796264e7fd9a9c8675616dd7a".to_string(),
            contract_name: "LargeContract".to_string(),
            field_types: large_field_types,
            field_semantics: large_field_semantics,
            queries: large_queries,
        };
        
        let options = CodegenOptions::default();
        
        // Should handle large inputs gracefully (either succeed or fail with reasonable error)
        let result = generate_circuit_template(&large_layout, &options);
        
        match result {
            Ok((cargo_toml, lib_rs)) => {
                // If successful, verify reasonable size limits
                assert!(cargo_toml.len() < 1_000_000, "Generated Cargo.toml should not be excessively large");
                assert!(lib_rs.len() < 10_000_000, "Generated lib.rs should not be excessively large");
                // Large layout handled successfully
            }
            Err(_) => {
                // Large layout appropriately rejected to prevent resource exhaustion
            }
        }
        
        // Test with extremely long strings
        let long_string = "A".repeat(100_000);
        let long_layout = LayoutInfo {
            commitment: "f6dc3c4a79e95565b3cf38993f1a120c6a6b467796264e7fd9a9c8675616dd7a".to_string(),
            contract_name: long_string.clone(),
            field_types: vec!["Uint256".to_string()],
            field_semantics: vec!["ValidZero".to_string()],
            queries: vec![QueryInfo {
                query: long_string,
                field_type: "Uint256".to_string(),
                zero_semantics: "ValidZero".to_string(),
                expected_slot: "0x0000000000000000000000000000000000000000000000000000000000000000".to_string(),
            }],
        };
        
        let result = generate_circuit_template(&long_layout, &options);
        
        // Should handle long strings without crashing
        match result {
            Ok(_) => {
                // Long strings handled successfully
            }
            Err(_) => {
                // Long strings appropriately rejected
            }
        }
    }

    #[test]
    fn test_security_numeric_overflow_protection() {
        // Security Test: Numeric overflow protection in generated code
        
        let layout = LayoutInfo {
            commitment: "f6dc3c4a79e95565b3cf38993f1a120c6a6b467796264e7fd9a9c8675616dd7a".to_string(),
            contract_name: "TestContract".to_string(),
            field_types: vec!["Uint256".to_string()],
            field_semantics: vec!["ValidZero".to_string()],
            queries: vec![QueryInfo {
                query: "balance".to_string(),
                field_type: "Uint256".to_string(),
                zero_semantics: "ValidZero".to_string(),
                expected_slot: "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff".to_string(), // Max slot
            }],
        };
        
        let options = CodegenOptions::default();
        
        let result = generate_circuit_template(&layout, &options);
        assert!(result.is_ok(), "Maximum valid slot should be handled");
        
        let (_, lib_rs) = result.unwrap();
        
        // Verify generated code handles maximum values safely
        assert!(lib_rs.contains("0xff"), "Should handle maximum hex values");
        
        // Test with overflow-inducing slot values
        let overflow_layout = LayoutInfo {
            commitment: "f6dc3c4a79e95565b3cf38993f1a120c6a6b467796264e7fd9a9c8675616dd7a".to_string(),
            contract_name: "TestContract".to_string(),
            field_types: vec!["Uint256".to_string()],
            field_semantics: vec!["ValidZero".to_string()],
            queries: vec![QueryInfo {
                query: "balance".to_string(),
                field_type: "Uint256".to_string(),
                zero_semantics: "ValidZero".to_string(),
                expected_slot: format!("0x{}", "f".repeat(100)), // Too long
            }],
        };
        
        let result = generate_circuit_template(&overflow_layout, &options);
        
        // Should handle overflow gracefully
        match result {
            Ok(_) => {
                // Overflow value handled successfully
            }
            Err(_) => {
                // Overflow value appropriately rejected
            }
        }
    }

    #[test]
    fn test_security_concurrent_code_generation() {
        // Security Test: Thread safety during concurrent code generation
        #[cfg(feature = "std")]
        {
            use std::sync::Arc;
            use std::thread;
            
            let layout = Arc::new(LayoutInfo {
                commitment: "f6dc3c4a79e95565b3cf38993f1a120c6a6b467796264e7fd9a9c8675616dd7a".to_string(),
                contract_name: "ConcurrentTest".to_string(),
                field_types: vec!["Uint256".to_string()],
                field_semantics: vec!["ValidZero".to_string()],
                queries: vec![QueryInfo {
                    query: "test_field".to_string(),
                    field_type: "Uint256".to_string(),
                    zero_semantics: "ValidZero".to_string(),
                    expected_slot: "0x0000000000000000000000000000000000000000000000000000000000000000".to_string(),
                }],
            });
            
            let options = Arc::new(CodegenOptions::default());
            
            let handles: Vec<_> = (0..10).map(|i| {
                let layout = Arc::clone(&layout);
                let options = Arc::clone(&options);
                thread::spawn(move || {
                    let result = generate_circuit_template(&layout, &options);
                    assert!(result.is_ok(), "Concurrent generation {} should succeed", i);
                    result.unwrap()
                })
            }).collect();
            
            // All threads should complete successfully with identical results
            let mut results = Vec::new();
            for handle in handles {
                let result = handle.join().expect("Thread should not panic");
                results.push(result);
            }
            
            // All results should be identical (deterministic generation)
            for i in 1..results.len() {
                assert_eq!(results[0].0, results[i].0, "Cargo.toml should be identical across threads");
                assert_eq!(results[0].1, results[i].1, "lib.rs should be identical across threads");
            }
        }
    }
} 

/// Format field types as string literals for code generation
fn format_field_types(field_types: &[String]) -> String {
    if field_types.is_empty() {
        return String::new();
    }
    
    field_types.iter()
        .map(|t| format!("\"{}\"", t))
        .collect::<Vec<_>>()
        .join(", ")
}

/// Format field semantics as byte literals for code generation
fn format_field_semantics(field_semantics: &[String]) -> String {
    if field_semantics.is_empty() {
        return String::new();
    }
    
    field_semantics.iter()
        .map(|s| {
            // Convert semantic strings to byte values
            match s.as_str() {
                "ValidZero" => "0u8",
                "InvalidZero" => "1u8",
                "RequiredNonZero" => "2u8",
                _ => "0u8" // Default
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
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
valence-coprocessor = {{ "{{" }} git = "https://github.com/timewave-computer/valence-coprocessor.git", tag = "v0.1.13", default-features = false {{ "}}" }}
traverse-valence = {{ "{{" }} path = "../../../traverse", default-features = false, features = ["circuit"] {{ "}}" }}
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

pub fn circuit(witnesses: Vec<Witness>) -> Vec<u8> {{{{
    let processor = CircuitProcessor::new(
        LAYOUT_COMMITMENT,
        generate_field_types_from_layout(layout),
        generate_field_semantics_from_layout(layout),
    );
    
    // Process witnesses through the circuit
    let mut result = alloc::vec![0x01]; // Success indicator
    
    for witness in witnesses {{{{
        // Basic witness validation
        if witness.len() < 32 {{{{
            return alloc::vec![0x00]; // Error indicator
        }}}}
        
        // Process witness data (simplified)
        let witness_hash = simple_hash(&witness);
        result.extend_from_slice(&witness_hash[..8]); // First 8 bytes as proof
    }}}}
    
    result
}}}}

/// Generate field types from layout
fn generate_field_types_from_layout(layout: &LayoutInfo) -> alloc::vec::Vec<&'static str> {{{{
    alloc::vec![{}]
}}}}

/// Generate field semantics from layout
fn generate_field_semantics_from_layout(layout: &LayoutInfo) -> alloc::vec::Vec<u8> {{{{
    alloc::vec![{}]
}}}}

/// Simple hash function for witness processing
fn simple_hash(data: &[u8]) -> [u8; 32] {{{{
    let mut result = [0u8; 32];
    let mut state = 0x9e3779b9u32; // Golden ratio constant
    
    for (i, &byte) in data.iter().enumerate() {{{{
        state = state.wrapping_mul(0x85ebca6b);
        state = state.wrapping_add(byte as u32);
        state = state.wrapping_add(i as u32);
        result[i % 32] ^= (state >> 24) as u8;
    }}}}
    
    result
}}}}
"#,
        options.crate_name,
        layout.commitment,
        layout.commitment,
        commitment_array,
        format_field_types(&layout.field_types),
        format_field_semantics(&layout.field_semantics)
    );

    Ok((cargo_toml, lib_rs))
}

#[cfg(test)]
mod new_functionality_tests {
    use super::*;

    #[test]
    fn test_format_field_types_empty() {
        let field_types = vec![];
        let result = format_field_types(&field_types);
        assert_eq!(result, "");
    }

    #[test]
    fn test_format_field_types_single() {
        let field_types = vec!["uint256".to_string()];
        let result = format_field_types(&field_types);
        assert_eq!(result, "\"uint256\"");
    }

    #[test]
    fn test_format_field_types_multiple() {
        let field_types = vec![
            "uint256".to_string(),
            "address".to_string(),
            "bool".to_string(),
        ];
        let result = format_field_types(&field_types);
        assert_eq!(result, "\"uint256\", \"address\", \"bool\"");
    }

    #[test]
    fn test_format_field_semantics_empty() {
        let field_semantics = vec![];
        let result = format_field_semantics(&field_semantics);
        assert_eq!(result, "");
    }

    #[test]
    fn test_format_field_semantics_valid_zero() {
        let field_semantics = vec!["ValidZero".to_string()];
        let result = format_field_semantics(&field_semantics);
        assert_eq!(result, "0u8");
    }

    #[test]
    fn test_format_field_semantics_invalid_zero() {
        let field_semantics = vec!["InvalidZero".to_string()];
        let result = format_field_semantics(&field_semantics);
        assert_eq!(result, "1u8");
    }

    #[test]
    fn test_format_field_semantics_required_non_zero() {
        let field_semantics = vec!["RequiredNonZero".to_string()];
        let result = format_field_semantics(&field_semantics);
        assert_eq!(result, "2u8");
    }

    #[test]
    fn test_format_field_semantics_unknown() {
        let field_semantics = vec!["UnknownSemantic".to_string()];
        let result = format_field_semantics(&field_semantics);
        assert_eq!(result, "0u8"); // Default fallback
    }

    #[test]
    fn test_format_field_semantics_multiple() {
        let field_semantics = vec![
            "ValidZero".to_string(),
            "InvalidZero".to_string(),
            "RequiredNonZero".to_string(),
            "Unknown".to_string(),
        ];
        let result = format_field_semantics(&field_semantics);
        assert_eq!(result, "0u8, 1u8, 2u8, 0u8");
    }

    #[test]
    fn test_layout_info_with_field_data() {
        let layout = LayoutInfo {
            commitment: "abc123".to_string(),
            contract_name: "TestContract".to_string(),
            field_types: vec!["uint256".to_string(), "address".to_string()],
            field_semantics: vec!["ValidZero".to_string(), "InvalidZero".to_string()],
            queries: vec![],
        };

        assert_eq!(layout.field_types.len(), 2);
        assert_eq!(layout.field_semantics.len(), 2);
        assert_eq!(format_field_types(&layout.field_types), "\"uint256\", \"address\"");
        assert_eq!(format_field_semantics(&layout.field_semantics), "0u8, 1u8");
    }
} 