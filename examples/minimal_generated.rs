//! Example of minimal generated code for a specific schema
//!
//! This shows what the `traverse-cli minimal` command would generate for a
//! simple ERC20-like contract with balance and allowance mappings.

// Note: This example uses std for println! demonstrations
// Generated minimal code would typically be #![no_std] compatible

/// Layout commitment (32 bytes)
pub const LAYOUT_COMMITMENT: [u8; 32] = [
    0xf6, 0xdc, 0x3c, 0x4a, 0x79, 0xe9, 0x55, 0x65,
    0xb3, 0xcf, 0x38, 0x99, 0x3f, 0x1a, 0x12, 0x0c,
    0x6a, 0x6b, 0x46, 0x77, 0x96, 0x26, 0x4e, 0x7f,
    0xd9, 0xa9, 0xc8, 0x67, 0x56, 0x16, 0xdd, 0x7a,
];

/// Field definitions
#[repr(C)]
pub struct Fields {
    pub total_supply: FieldDef<0>,
    pub balances: FieldDef<1>,
    pub allowances: FieldDef<2>,
}

/// Field definition with compile-time index
#[repr(C)]
pub struct FieldDef<const INDEX: usize> {
    slot: [u8; 32],
    offset: u8,
    zero_semantics: u8,
}

impl<const INDEX: usize> FieldDef<INDEX> {
    /// Compute storage key for this field
    #[inline]
    pub const fn storage_key(&self, key: Option<&[u8; 32]>) -> [u8; 32] {
        if let Some(k) = key {
            // Mapping: keccak256(key || slot)
            keccak256_concat(k, &self.slot)
        } else {
            // Simple storage: direct slot
            self.slot
        }
    }
    
    /// Verify witness for this field (minimal validation)
    #[inline]
    pub fn verify_witness(&self, witness_data: &[u8]) -> Result<[u8; 32], &'static str> {
        // Check minimum size (176 bytes for extended format)
        if witness_data.len() < 176 {
            return Err("Invalid witness size");
        }
        
        // Extract and validate field index at offset 142-144 (after proof)
        let proof_len = u32::from_le_bytes([
            witness_data[138], witness_data[139], 
            witness_data[140], witness_data[141]
        ]) as usize;
        let field_index_offset = 142 + proof_len;
        
        if witness_data.len() < field_index_offset + 2 {
            return Err("Missing field index");
        }
        
        let field_index = u16::from_le_bytes([
            witness_data[field_index_offset], 
            witness_data[field_index_offset + 1]
        ]);
        
        if field_index != INDEX as u16 {
            return Err("Field index mismatch");
        }
        
        // Extract value at offset 64-96
        let mut value = [0u8; 32];
        value.copy_from_slice(&witness_data[64..96]);
        
        // Validate zero semantics if value is zero
        if value == [0u8; 32] && witness_data[96] != self.zero_semantics {
            return Err("Invalid zero semantics");
        }
        
        Ok(value)
    }
}

/// Minimal keccak256 for concatenated data
const fn keccak256_concat(a: &[u8; 32], b: &[u8; 32]) -> [u8; 32] {
    // In real implementation, this would use tiny_keccak or similar
    // For demonstration, returning a placeholder
    let mut result = [0u8; 32];
    // ... actual hashing implementation ...
    result
}

/// Pre-defined fields for ERC20-like schema
pub const FIELDS: Fields = Fields {
    total_supply: FieldDef {
        slot: [0u8; 32], // Slot 0
        offset: 0,
        zero_semantics: 3, // ValidZero
    },
    balances: FieldDef {
        slot: [
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1
        ], // Slot 1
        offset: 0,
        zero_semantics: 3, // ValidZero
    },
    allowances: FieldDef {
        slot: [
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2
        ], // Slot 2
        offset: 0,
        zero_semantics: 3, // ValidZero
    },
};

/// Example usage of the generated code
fn main() {
    println!("Minimal Generated Code Example");
    println!("==============================");
    
    // Show layout commitment
    println!("Layout commitment: {:02x?}", LAYOUT_COMMITMENT);
    
    // Query total supply (simple storage)
    let total_supply_key = FIELDS.total_supply.storage_key(None);
    println!("Total supply storage key: {:02x?}", total_supply_key);
    
    // Query balance for a specific address (mapping)
    let address = [0x42u8; 32]; // Example address padded to 32 bytes
    let balance_key = FIELDS.balances.storage_key(Some(&address));
    println!("Balance storage key for address: {:02x?}", balance_key);
    
    // Query allowances for owner->spender mapping
    let allowance_key = FIELDS.allowances.storage_key(Some(&address));
    println!("Allowance storage key: {:02x?}", allowance_key);
    
    // Demonstrate witness verification (would fail with dummy data)
    let witness_data = [0u8; 180]; // Example witness data
    println!("Attempting witness verification...");
    match FIELDS.balances.verify_witness(&witness_data) {
        Ok(value) => {
            println!("Witness verification successful! Extracted value: {:02x?}", value);
        },
        Err(e) => {
            println!("Witness verification failed: {}", e);
        }
    }
    
    println!("Example completed successfully!");
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_field_definitions() {
        // Test that field definitions are properly configured
        assert_eq!(FIELDS.total_supply.slot, [0u8; 32]);
        assert_eq!(FIELDS.balances.slot[31], 1);
        assert_eq!(FIELDS.allowances.slot[31], 2);
    }
    
    #[test]
    fn test_storage_key_generation() {
        // Test storage key generation for simple storage
        let total_supply_key = FIELDS.total_supply.storage_key(None);
        assert_eq!(total_supply_key, [0u8; 32]);
        
        // Test storage key generation for mapping
        let address = [0x42u8; 32];
        let balance_key = FIELDS.balances.storage_key(Some(&address));
        // Should return the computed keccak256 hash (placeholder in this example)
        assert_eq!(balance_key, [0u8; 32]); // This is just the placeholder result
    }
}