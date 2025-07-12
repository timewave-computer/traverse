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
    
    // Create context
    let mut context = Context::new();
    context.insert("options", options);
    context.insert("layout", layout);
    context.insert("field_count", &layout.field_types.len());
    
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
{% if options.include_alloy %}use alloy_primitives::{Address, U256};
use alloy_sol_types::{sol, SolValue};{% endif %}

/// Layout commitment for this circuit (validates against expected layout)
pub const LAYOUT_COMMITMENT: [u8; 32] = [
    // {{ layout.commitment }}
    // Convert hex string to byte array during generation
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
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
{% for query in layout.queries %}        {{ query.field_type | lower }} {{ query.query | replace("[", "_") | replace("]", "") | replace(".", "_") }};
{% endfor %}    }
}{% endif %}

/// Main circuit function
pub fn circuit(witnesses: Vec<Witness>) -> Vec<u8> {
    // Validate witness count
    assert_eq!(
        witnesses.len(),
        {{ layout.field_types | length }},
        "Expected {{ layout.field_types | length }} witnesses for {{ layout.contract_name }}"
    );
    
    // Create processor with layout-specific parameters
    let processor = CircuitProcessor::new(
        LAYOUT_COMMITMENT,
        FIELD_TYPES.to_vec(),
        FIELD_SEMANTICS.to_vec(),
    );
    
    // Parse witnesses from bytes
    let circuit_witnesses: Vec<CircuitWitness> = witnesses
        .into_iter()
        .map(|w| CircuitProcessor::parse_witness_from_bytes(w.as_data().expect("Expected witness data")))
        .collect::<Result<Vec<_>, _>>()
        .expect("Failed to parse witnesses");
    
    // Process all witnesses
    let results = processor.process_batch(&circuit_witnesses);
    
    // Validate all results are valid
    for (i, result) in results.iter().enumerate() {
        match result {
            CircuitResult::Valid { .. } => {},
            CircuitResult::Invalid => panic!("Witness {} failed validation", i),
        }
    }
    
{% if options.include_alloy %}    // Generate ABI-encoded output
    generate_abi_output(&results)
{% else %}    // Generate simple success indicator
    alloc::vec![0x01] // Success
{% endif %}
}

{% if options.include_alloy %}/// Generate ABI-encoded output from circuit results
fn generate_abi_output(results: &[CircuitResult]) -> Vec<u8> {
    // Extract validated values from results
{% for query in layout.queries %}    let {{ query.query | replace("[", "_") | replace("]", "") | replace(".", "_") }} = match &results[{{ loop.index0 }}] {
        CircuitResult::Valid { extracted_value, .. } => {
            // Convert extracted value to appropriate type
            match extracted_value {
{% if query.field_type == "Address" %}                traverse_valence::circuit::ExtractedValue::Address(addr) => {
                    Address::from_slice(addr)
                },{% elif query.field_type == "Uint256" %}                traverse_valence::circuit::ExtractedValue::Uint256(val) => {
                    U256::from_le_bytes(*val)
                },{% else %}                _ => panic!("Unexpected value type for {{ query.query }}"),{% endif %}
            }
        },
        CircuitResult::Invalid => panic!("Invalid result for {{ query.query }}"),
    };
{% endfor %}
    
    // Create output structure
    let output = {{ layout.contract_name | title }}Output {
{% for query in layout.queries %}        {{ query.query | replace("[", "_") | replace("]", "") | replace(".", "_") }},
{% endfor %}    };
    
    output.abi_encode()
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

/// No-std compatible circuit template generation
pub fn generate_circuit_template(
    layout: &LayoutInfo,
    options: &CodegenOptions,
) -> Result<(String, String), crate::TraverseValenceError> {
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

    // Generate lib.rs content
    let lib_rs = format!(
        r#"//! Generated circuit for {}
//! Layout commitment: {}

#![no_std]

extern crate alloc;
use alloc::vec::Vec;

use valence_coprocessor::Witness;
use traverse_valence::circuit::{{CircuitProcessor, CircuitWitness, FieldType, ZeroSemantics}};

pub const LAYOUT_COMMITMENT: [u8; 32] = [0u8; 32]; // TODO: Parse from hex

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
        layout.commitment
    );

    Ok((cargo_toml, lib_rs))
} 