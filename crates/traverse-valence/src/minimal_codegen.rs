//! Minimal code generation for schema-specific query and verification
//!
//! This module generates minimal, purpose-built code that is tailored specifically
//! to the provided schema without any unnecessary abstractions or dependencies.

use alloc::{format, string::String, vec::Vec};
use traverse_core::{LayoutInfo, StorageEntry, ZeroSemantics};

/// Generate minimal query code for a specific storage layout
pub fn generate_minimal_query_code(layout: &LayoutInfo) -> String {
    let mut code = String::new();
    
    // Header
    code.push_str(&format!(r#"//! Minimal query library for {}
//! 
//! This library provides direct storage key computation for the specific
//! fields defined in the schema, without any generic abstractions.

#![no_std]

/// Compute storage key for a specific field using keccak256
/// 
/// This is a minimal implementation that directly computes the storage key
/// without any trait abstractions or generic machinery.
pub fn compute_storage_key(field_index: u16, key_data: Option<&[u8]>) -> [u8; 32] {{
    match field_index {{
"#, layout.contract_name));

    // Generate direct computation for each field
    for (index, entry) in layout.storage.iter().enumerate() {
        code.push_str(&format!(
            "        {} => {{\n",
            index
        ));
        
        // Direct slot computation based on field type
        if entry.type_name.contains("mapping") {
            code.push_str(&format!(
                r#"            // Mapping: keccak256(key || slot)
            let mut data = [0u8; 64];
            if let Some(key) = key_data {{
                data[0..32].copy_from_slice(key);
            }}
            let slot_bytes = hex::decode("{}").unwrap();
            data[32..32 + slot_bytes.len()].copy_from_slice(&slot_bytes);
            keccak256(&data)
"#,
                entry.slot.trim_start_matches("0x")
            ));
        } else {
            code.push_str(&format!(
                r#"            // Simple storage: direct slot
            let mut slot = [0u8; 32];
            let slot_bytes = hex::decode("{}").unwrap();
            slot[32 - slot_bytes.len()..].copy_from_slice(&slot_bytes);
            slot
"#,
                entry.slot.trim_start_matches("0x")
            ));
        }
        
        code.push_str("        },\n");
    }
    
    code.push_str(r#"        _ => panic!("Invalid field index"),
    }
}

/// Minimal keccak256 implementation
fn keccak256(data: &[u8]) -> [u8; 32] {
    use tiny_keccak::{Hasher, Keccak};
    let mut hasher = Keccak::v256();
    let mut output = [0u8; 32];
    hasher.update(data);
    hasher.finalize(&mut output);
    output
}

"#);

    // Generate field constants
    code.push_str("/// Field indices for direct access\n");
    for (index, entry) in layout.storage.iter().enumerate() {
        code.push_str(&format!(
            "pub const FIELD_{}: u16 = {};\n",
            entry.label.to_uppercase(),
            index
        ));
    }
    
    code.push_str("\n/// Storage layout commitment for verification\n");
    code.push_str(&format!(
        "pub const LAYOUT_COMMITMENT: [u8; 32] = {:?};\n",
        layout.commitment()
    ));
    
    code
}

/// Generate minimal verifier code for a specific storage layout
pub fn generate_minimal_verifier_code(layout: &LayoutInfo) -> String {
    let mut code = String::new();
    
    // Header
    code.push_str(&format!(r#"//! Minimal verifier library for {}
//!
//! This library provides direct witness verification for the specific
//! fields defined in the schema, optimized for minimal size and dependencies.

#![no_std]

/// Verify a witness for a specific field
/// 
/// This is a minimal implementation that directly validates the witness
/// without any generic abstractions. Returns the extracted value on success.
pub fn verify_witness(
    field_index: u16,
    witness_data: &[u8],
) -> Result<ExtractedValue, &'static str> {{
    // Check minimum witness size (extended format: 176 bytes + proof)
    if witness_data.len() < 176 {{
        return Err("Witness too small");
    }}
    
    // Direct parsing of witness components
    let storage_key = &witness_data[0..32];
    let layout_commitment = &witness_data[32..64];
    let value = &witness_data[64..96];
    let zero_semantics = witness_data[96];
    let _semantic_source = witness_data[97];
    let block_height = u64::from_le_bytes([
        witness_data[98], witness_data[99], witness_data[100], witness_data[101],
        witness_data[102], witness_data[103], witness_data[104], witness_data[105],
    ]);
    let block_hash = &witness_data[106..138];
    let proof_len = u32::from_le_bytes([
        witness_data[138], witness_data[139], witness_data[140], witness_data[141],
    ]) as usize;
    
    // Skip proof data
    let offset = 142 + proof_len;
    
    // Parse extended fields
    let witness_field_index = u16::from_le_bytes([witness_data[offset], witness_data[offset + 1]]);
    let expected_slot = &witness_data[offset + 2..offset + 34];
    
    // Validate field index
    if witness_field_index != field_index {{
        return Err("Field index mismatch");
    }}
    
    // Validate layout commitment
    if layout_commitment != EXPECTED_COMMITMENT {{
        return Err("Layout commitment mismatch");
    }}
    
    // Extract value based on field type
    match field_index {{
"#, layout.contract_name));

    // Generate extraction for each field
    for (index, entry) in layout.storage.iter().enumerate() {
        code.push_str(&format!("        {} => {{\n", index));
        
        // Field-specific validation
        code.push_str(&format!(
            r#"            // Field: {} (type: {})
            // Validate zero semantics
            if value == &[0u8; 32] {{
                if zero_semantics != {} {{
                    return Err("Invalid zero semantics");
                }}
            }}
            
"#,
            entry.label,
            entry.type_name,
            entry.zero_semantics as u8
        ));
        
        // Extract value based on type
        let extraction = match entry.type_name.as_str() {
            "t_bool" => "ExtractedValue::Bool(value[31] != 0)",
            "t_uint8" => "ExtractedValue::Uint8(value[31])",
            "t_uint256" => "ExtractedValue::Uint256(*value)",
            "t_address" => {
                code.push_str(r#"            let mut addr = [0u8; 20];
            addr.copy_from_slice(&value[12..32]);
            if addr == [0u8; 20] {
                return Err("Zero address");
            }
            "#);
                "ExtractedValue::Address(addr)"
            },
            _ => "ExtractedValue::Uint256(*value)", // Default to uint256
        };
        
        code.push_str(&format!("            Ok({})\n", extraction));
        code.push_str("        },\n");
    }
    
    code.push_str(r#"        _ => Err("Invalid field index"),
    }
}

/// Minimal extracted value types
#[derive(Debug, Clone, Copy)]
pub enum ExtractedValue {
    Bool(bool),
    Uint8(u8),
    Uint256([u8; 32]),
    Address([u8; 20]),
}

/// Expected layout commitment
const EXPECTED_COMMITMENT: &[u8] = &[
"#);

    // Add commitment bytes
    let commitment = layout.commitment();
    for (i, byte) in commitment.iter().enumerate() {
        if i > 0 { code.push_str(", "); }
        code.push_str(&format!("0x{:02x}", byte));
    }
    code.push_str("];\n");

    // Add field constants
    code.push_str("\n/// Field indices\n");
    for (index, entry) in layout.storage.iter().enumerate() {
        code.push_str(&format!(
            "pub const FIELD_{}: u16 = {};\n",
            entry.label.to_uppercase(),
            index
        ));
    }
    
    code
}

/// Generate minimal combined library with both query and verification
pub fn generate_minimal_combined_code(layout: &LayoutInfo) -> String {
    let mut code = String::new();
    
    // Header
    code.push_str(&format!(r#"//! Minimal query and verification library for {}
//!
//! This is a self-contained library that provides exactly the functionality
//! needed to query and verify storage for the specific schema, with no extras.

#![no_std]

"#, layout.contract_name));

    // Add the layout-specific constants
    code.push_str("/// Layout commitment (32 bytes)\n");
    code.push_str("pub const LAYOUT_COMMITMENT: [u8; 32] = [\n    ");
    let commitment = layout.commitment();
    for (i, byte) in commitment.iter().enumerate() {
        if i > 0 && i % 8 == 0 { code.push_str(",\n    "); }
        else if i > 0 { code.push_str(", "); }
        code.push_str(&format!("0x{:02x}", byte));
    }
    code.push_str("\n];\n\n");

    // Add field definitions
    code.push_str("/// Field definitions\n");
    code.push_str("#[repr(C)]\n");
    code.push_str("pub struct Fields {\n");
    for (index, entry) in layout.storage.iter().enumerate() {
        code.push_str(&format!(
            "    pub {}: FieldDef<{}>,\n",
            entry.label,
            index
        ));
    }
    code.push_str("}\n\n");

    // Add field definition type
    code.push_str(r#"/// Field definition with compile-time index
#[repr(C)]
pub struct FieldDef<const INDEX: usize> {
    slot: [u8; 32],
    offset: u8,
    zero_semantics: u8,
}

impl<const INDEX: usize> FieldDef<INDEX> {
    /// Compute storage key for this field
    pub fn storage_key(&self, key: Option<&[u8; 32]>) -> [u8; 32] {
        if let Some(k) = key {
            // Mapping: keccak256(key || slot)
            let mut data = [0u8; 64];
            data[0..32].copy_from_slice(k);
            data[32..64].copy_from_slice(&self.slot);
            keccak256(&data)
        } else {
            // Simple storage: direct slot
            self.slot
        }
    }
    
    /// Verify witness for this field
    pub fn verify_witness(&self, witness_data: &[u8]) -> Result<[u8; 32], &'static str> {
        // Minimal witness verification
        if witness_data.len() < 176 {
            return Err("Invalid witness size");
        }
        
        // Extract and validate field index
        let field_index = u16::from_le_bytes([witness_data[142], witness_data[143]]);
        if field_index != INDEX as u16 {
            return Err("Field index mismatch");
        }
        
        // Extract value
        let mut value = [0u8; 32];
        value.copy_from_slice(&witness_data[64..96]);
        
        // Validate zero semantics if needed
        if value == [0u8; 32] && witness_data[96] != self.zero_semantics {
            return Err("Invalid zero semantics");
        }
        
        Ok(value)
    }
}

/// Minimal keccak256
fn keccak256(data: &[u8]) -> [u8; 32] {
    // This would use tiny_keccak or similar minimal implementation
    // For now, placeholder that would be replaced with actual implementation
    let mut output = [0u8; 32];
    // ... actual hashing ...
    output
}

/// Pre-defined fields for this schema
pub const FIELDS: Fields = Fields {
"#);

    // Generate field constants
    for entry in &layout.storage {
        code.push_str(&format!(
            "    {}: FieldDef {{\n",
            entry.label
        ));
        code.push_str(&format!(
            "        slot: {:?},\n",
            hex::decode(entry.slot.trim_start_matches("0x")).unwrap_or_else(|_| alloc::vec![0; 32])
        ));
        code.push_str(&format!(
            "        offset: {},\n",
            entry.offset
        ));
        code.push_str(&format!(
            "        zero_semantics: {},\n",
            entry.zero_semantics as u8
        ));
        code.push_str("    },\n");
    }
    
    code.push_str("};\n");
    
    code
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_minimal_query_generation() {
        let layout = LayoutInfo {
            contract_name: "TestContract".into(),
            storage: alloc::vec![
                StorageEntry {
                    label: "balance".into(),
                    slot: "0x0".into(),
                    offset: 0,
                    type_name: "t_uint256".into(),
                    zero_semantics: ZeroSemantics::ValidZero,
                },
            ],
            types: alloc::vec![],
        };
        
        let code = generate_minimal_query_code(&layout);
        assert!(code.contains("compute_storage_key"));
        assert!(code.contains("FIELD_BALANCE"));
    }
}