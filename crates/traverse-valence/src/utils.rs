//! Utility functions for traverse-valence
//! 
//! This module contains helper functions used across the crate,
//! particularly for hex parsing and data conversion.

use alloc::format;
use crate::ValenceError;

/// Helper function to parse hex string to 32-byte array
pub fn parse_hex_32(hex_str: &str) -> Result<[u8; 32], ValenceError> {
    let hex_clean = hex_str.strip_prefix("0x").unwrap_or(hex_str);
    
    if hex_clean.len() != 64 {
        return Err(ValenceError::InvalidStorageKey(
            format!("Expected 64 hex chars, got {}", hex_clean.len())
        ));
    }
    
    let bytes = hex::decode(hex_clean)
        .map_err(|e| ValenceError::InvalidStorageKey(format!("{:?}", e)))?;
    
    let mut result = [0u8; 32];
    result.copy_from_slice(&bytes);
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_hex_32() {
        let hex_str = "c1f51986c7e9d391993039c3c40e41ad9f26e1db9b80f8535a639eadeb1d1bd9";
        let result = parse_hex_32(hex_str);
        assert!(result.is_ok());
        
        let with_prefix = "0xc1f51986c7e9d391993039c3c40e41ad9f26e1db9b80f8535a639eadeb1d1bd9";
        let result2 = parse_hex_32(with_prefix);
        assert!(result2.is_ok());
        assert_eq!(result.unwrap(), result2.unwrap());
    }
    
    #[test]
    fn test_parse_hex_32_invalid_length() {
        let short_hex = "c1f51986";
        let result = parse_hex_32(short_hex);
        assert!(result.is_err());
        
        match result {
            Err(ValenceError::InvalidStorageKey(msg)) => {
                assert!(msg.contains("Expected 64 hex chars"));
            }
            _ => panic!("Expected InvalidStorageKey error"),
        }
    }
    
    #[test]
    fn test_parse_hex_32_invalid_chars() {
        let invalid_hex = "g1f51986c7e9d391993039c3c40e41ad9f26e1db9b80f8535a639eadeb1d1bd9";
        let result = parse_hex_32(invalid_hex);
        assert!(result.is_err());
        
        match result {
            Err(ValenceError::InvalidStorageKey(_)) => {
                // Expected - invalid hex character
            }
            _ => panic!("Expected InvalidStorageKey error"),
        }
    }
} 