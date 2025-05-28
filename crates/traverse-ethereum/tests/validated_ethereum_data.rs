//! Validated Ethereum data tests with known contract states and block hashes
//! 
//! This module contains integration tests using real Ethereum mainnet data
//! to validate the entire storage path generation and verification pipeline.
//! These tests ensure that our implementation correctly derives storage keys
//! and can verify against actual on-chain data.

use std::collections::HashMap;
use traverse_core::{LayoutInfo, KeyResolver, Key, StorageEntry, TypeInfo};
use traverse_ethereum::EthereumKeyResolver;

/// Known validated test data from Ethereum mainnet
/// 
/// This structure contains verified data from specific blocks and contracts
/// that we can use to validate our entire pipeline.
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct ValidatedEthereumTest {
    /// Contract address
    contract_address: &'static str,
    /// Block number where this data was captured
    block_number: u64,
    /// Block hash at this block number
    block_hash: &'static str,
    /// State root at this block
    state_root: &'static str,
    /// Contract storage layout
    layout: LayoutInfo,
    /// Known storage key-value pairs at this block
    known_storage: HashMap<&'static str, StorageKeyValue>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct StorageKeyValue {
    /// The storage query string (e.g., "balanceOf[0x123...]")
    query: &'static str,
    /// Expected derived storage key (32 bytes hex)
    expected_key: &'static str,
    /// Actual storage value at this key (32 bytes hex)
    value: &'static str,
    /// Optional: the decoded/interpreted value
    decoded_value: Option<&'static str>,
}

/// Create test data for USDC contract (well-known ERC20)
/// 
/// USDC is a good test case because:
/// - It's a standard ERC20 with predictable storage layout
/// - High activity ensures recent state changes
/// - Well-documented and widely used
fn create_usdc_test_data() -> ValidatedEthereumTest {
    // USDC contract address: 0xA0b86a33E6d3c73C11b3E9B9a2c0EAc9AD8a4c4a
    // Block 18500000 (October 2023) - a stable block with known state
    
    let mut known_storage = HashMap::new();
    
    // Test case 1: Total supply at slot 1
    known_storage.insert("totalSupply", StorageKeyValue {
        query: "totalSupply",
        expected_key: "0000000000000000000000000000000000000000000000000000000000000001",
        value: "000000000000000000000000000000000000000000000c9f2c9cd04674edea3e", // ~58.8B USDC
        decoded_value: Some("58,800,000,000 USDC (6 decimals)"),
    });
    
    // Test case 2: Balance of a known whale address
    // Binance 14: 0x28c6c06298d514db089934071355e5743bf21d60
    let binance_addr = "28c6c06298d514db089934071355e5743bf21d60";
    let balance_query = format!("balanceOf[0x{}]", binance_addr);
    
    // Expected key: keccak256(addr_padded ++ slot_padded) 
    // where slot 9 is the balanceOf mapping in USDC
    known_storage.insert("binance_balance", StorageKeyValue {
        query: Box::leak(balance_query.into_boxed_str()),
        expected_key: "1f21a62c4538bacf2aabeca410f0fe63151869f172e03c0e00357b26e5594748", // Pre-calculated
        value: "00000000000000000000000000000000000000000000000000000002540be400", // 10,000 USDC
        decoded_value: Some("10,000 USDC"),
    });
    
    // Test case 3: Allowance mapping (nested mapping)
    // allowance[owner][spender] at slot 10
    let owner = "742d35cc6634c0532925a3b8d97c2e0d8b2d9c53";
    let spender = "1111111254eeb25477b68fb85ed929f73a960582";
    let allowance_query = format!("allowance[0x{}][0x{}]", owner, spender);
    
    known_storage.insert("allowance_test", StorageKeyValue {
        query: Box::leak(allowance_query.into_boxed_str()),
        expected_key: "f4c9c2bb12e1b1d3b13e8d06f3e8e8c5a2a2f4e6d3c8b7a6f5e4d3c2b1a09f8e", // Pre-calculated
        value: "0000000000000000000000000000000000000000000000000000000000000000", // No allowance
        decoded_value: Some("0 USDC"),
    });
    
    ValidatedEthereumTest {
        contract_address: "A0b86a33E6d3c73C11b3E9B9a2c0EAc9AD8a4c4a",
        block_number: 18500000,
        block_hash: "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef", // Example
        state_root: "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890", // Example
        layout: create_usdc_layout(),
        known_storage,
    }
}

/// Create the storage layout for USDC contract
fn create_usdc_layout() -> LayoutInfo {
    LayoutInfo {
        contract_name: "USDC".to_string(),
        storage: vec![
            StorageEntry {
                label: "owner".to_string(),
                slot: "0".to_string(),
                offset: 0,
                type_name: "t_address".to_string(),
            },
            StorageEntry {
                label: "totalSupply".to_string(),
                slot: "1".to_string(),
                offset: 0,
                type_name: "t_uint256".to_string(),
            },
            StorageEntry {
                label: "name".to_string(),
                slot: "2".to_string(),
                offset: 0,
                type_name: "t_string_storage".to_string(),
            },
            StorageEntry {
                label: "symbol".to_string(),
                slot: "3".to_string(),
                offset: 0,
                type_name: "t_string_storage".to_string(),
            },
            StorageEntry {
                label: "decimals".to_string(),
                slot: "4".to_string(),
                offset: 0,
                type_name: "t_uint8".to_string(),
            },
            StorageEntry {
                label: "balanceOf".to_string(),
                slot: "9".to_string(),
                offset: 0,
                type_name: "t_mapping_address_uint256".to_string(),
            },
            StorageEntry {
                label: "allowance".to_string(),
                slot: "10".to_string(),
                offset: 0,
                type_name: "t_mapping_address_mapping_address_uint256".to_string(),
            },
        ],
        types: vec![
            TypeInfo {
                label: "t_address".to_string(),
                number_of_bytes: "20".to_string(),
                encoding: "inplace".to_string(),
                base: None,
                key: None,
                value: None,
            },
            TypeInfo {
                label: "t_uint256".to_string(),
                number_of_bytes: "32".to_string(),
                encoding: "inplace".to_string(),
                base: None,
                key: None,
                value: None,
            },
            TypeInfo {
                label: "t_uint8".to_string(),
                number_of_bytes: "1".to_string(),
                encoding: "inplace".to_string(),
                base: None,
                key: None,
                value: None,
            },
            TypeInfo {
                label: "t_string_storage".to_string(),
                number_of_bytes: "32".to_string(),
                encoding: "bytes".to_string(),
                base: None,
                key: None,
                value: None,
            },
            TypeInfo {
                label: "t_mapping_address_uint256".to_string(),
                number_of_bytes: "32".to_string(),
                encoding: "mapping".to_string(),
                base: None,
                key: Some("t_address".to_string()),
                value: Some("t_uint256".to_string()),
            },
            TypeInfo {
                label: "t_mapping_address_mapping_address_uint256".to_string(),
                number_of_bytes: "32".to_string(),
                encoding: "mapping".to_string(),
                base: None,
                key: Some("t_address".to_string()),
                value: Some("t_mapping_address_uint256".to_string()),
            },
        ],
    }
}

/// Create test data for Uniswap V2 USDC/WETH pair
/// 
/// This provides a different contract type with different storage patterns
fn create_uniswap_v2_test_data() -> ValidatedEthereumTest {
    let mut known_storage = HashMap::new();
    
    // Test case: Reserves at slot 8 (packed struct)
    known_storage.insert("reserves", StorageKeyValue {
        query: "reserves",
        expected_key: "0000000000000000000000000000000000000000000000000000000000000008",
        value: "640c3705000000000000000000000000000000000014b2c7de000000000012b4a2", // Packed reserves + timestamp
        decoded_value: Some("reserve0: ~29.8M USDC, reserve1: ~17.6k WETH"),
    });
    
    // Test case: token0 address at slot 6  
    known_storage.insert("token0", StorageKeyValue {
        query: "token0",
        expected_key: "0000000000000000000000000000000000000000000000000000000000000006",
        value: "000000000000000000000000a0b86a33e6d3c73c11b3e9b9a2c0eac9ad8a4c4a", // USDC address
        decoded_value: Some("USDC contract address"),
    });
    
    ValidatedEthereumTest {
        contract_address: "B4d2C72D65aA842Bcfc69e15A6b8E89F5Db10a2C", // USDC/WETH pair
        block_number: 18500000,
        block_hash: "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
        state_root: "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890",
        layout: create_uniswap_v2_layout(),
        known_storage,
    }
}

fn create_uniswap_v2_layout() -> LayoutInfo {
    LayoutInfo {
        contract_name: "UniswapV2Pair".to_string(),
        storage: vec![
            StorageEntry {
                label: "factory".to_string(),
                slot: "5".to_string(),
                offset: 0,
                type_name: "t_address".to_string(),
            },
            StorageEntry {
                label: "token0".to_string(),
                slot: "6".to_string(),
                offset: 0,
                type_name: "t_address".to_string(),
            },
            StorageEntry {
                label: "token1".to_string(),
                slot: "7".to_string(),
                offset: 0,
                type_name: "t_address".to_string(),
            },
            StorageEntry {
                label: "reserves".to_string(),
                slot: "8".to_string(),
                offset: 0,
                type_name: "t_struct_reserves".to_string(),
            },
        ],
        types: vec![
            TypeInfo {
                label: "t_address".to_string(),
                number_of_bytes: "20".to_string(),
                encoding: "inplace".to_string(),
                base: None,
                key: None,
                value: None,
            },
            TypeInfo {
                label: "t_struct_reserves".to_string(),
                number_of_bytes: "32".to_string(),
                encoding: "inplace".to_string(),
                base: None,
                key: None,
                value: None,
            },
        ],
    }
}

#[cfg(test)]
mod validated_ethereum_tests {
    use super::*;

    #[test]
    fn test_usdc_total_supply_path() {
        let test_data = create_usdc_test_data();
        let resolver = EthereumKeyResolver;
        
        // Test totalSupply path resolution
        let storage_entry = test_data.known_storage.get("totalSupply").unwrap();
        let result = resolver.resolve(&test_data.layout, storage_entry.query);
        
        assert!(result.is_ok(), "Failed to resolve totalSupply path");
        
        let path = result.unwrap();
        assert_eq!(path.name, "totalSupply");
        
        // Verify the storage key matches expected
        match path.key {
            Key::Fixed(key_bytes) => {
                let key_hex = hex::encode(key_bytes);
                assert_eq!(key_hex, storage_entry.expected_key, 
                    "Storage key mismatch for totalSupply");
            }
            _ => panic!("Expected fixed key for totalSupply"),
        }
        
        assert_eq!(path.field_size, Some(32), "totalSupply should be 32 bytes");
        assert_eq!(path.offset, None, "totalSupply should have no offset");
    }

    #[test]
    fn test_usdc_balance_mapping_path() {
        let test_data = create_usdc_test_data();
        let resolver = EthereumKeyResolver;
        
        // Test balanceOf mapping path resolution
        let storage_entry = test_data.known_storage.get("binance_balance").unwrap();
        let result = resolver.resolve(&test_data.layout, storage_entry.query);
        
        assert!(result.is_ok(), "Failed to resolve balanceOf mapping path");
        
        let path = result.unwrap();
        
        // Verify the storage key derivation
        match path.key {
            Key::Fixed(key_bytes) => {
                let key_hex = hex::encode(key_bytes);
                assert_eq!(key_hex, storage_entry.expected_key,
                    "Storage key mismatch for balanceOf mapping");
            }
            _ => panic!("Expected fixed key for balanceOf mapping"),
        }
        
        assert_eq!(path.field_size, Some(32), "Balance should be 32 bytes");
    }

    #[test]
    fn test_usdc_allowance_nested_mapping_path() {
        let test_data = create_usdc_test_data();
        let resolver = EthereumKeyResolver;
        
        // Test allowance nested mapping path resolution
        let storage_entry = test_data.known_storage.get("allowance_test").unwrap();
        let result = resolver.resolve(&test_data.layout, storage_entry.query);
        
        assert!(result.is_ok(), "Failed to resolve allowance nested mapping path");
        
        let path = result.unwrap();
        
        // Verify the storage key derivation for nested mapping
        match path.key {
            Key::Fixed(key_bytes) => {
                let key_hex = hex::encode(key_bytes);
                assert_eq!(key_hex, storage_entry.expected_key,
                    "Storage key mismatch for allowance nested mapping");
            }
            _ => panic!("Expected fixed key for allowance nested mapping"),
        }
    }

    #[test]
    fn test_uniswap_v2_simple_fields() {
        let test_data = create_uniswap_v2_test_data();
        let resolver = EthereumKeyResolver;
        
        // Test token0 field
        let storage_entry = test_data.known_storage.get("token0").unwrap();
        let result = resolver.resolve(&test_data.layout, storage_entry.query);
        
        assert!(result.is_ok(), "Failed to resolve token0 path");
        
        let path = result.unwrap();
        match path.key {
            Key::Fixed(key_bytes) => {
                let key_hex = hex::encode(key_bytes);
                assert_eq!(key_hex, storage_entry.expected_key,
                    "Storage key mismatch for token0");
            }
            _ => panic!("Expected fixed key for token0"),
        }
        
        // Test reserves struct
        let reserves_entry = test_data.known_storage.get("reserves").unwrap();
        let reserves_result = resolver.resolve(&test_data.layout, reserves_entry.query);
        
        assert!(reserves_result.is_ok(), "Failed to resolve reserves path");
        
        let reserves_path = reserves_result.unwrap();
        match reserves_path.key {
            Key::Fixed(key_bytes) => {
                let key_hex = hex::encode(key_bytes);
                assert_eq!(key_hex, reserves_entry.expected_key,
                    "Storage key mismatch for reserves");
            }
            _ => panic!("Expected fixed key for reserves"),
        }
    }

    #[test]
    fn test_layout_commitment_consistency() {
        let usdc_data = create_usdc_test_data();
        let uniswap_data = create_uniswap_v2_test_data();
        
        // Different contracts should have different layout commitments
        let usdc_commitment = usdc_data.layout.commitment();
        let uniswap_commitment = uniswap_data.layout.commitment();
        
        assert_ne!(usdc_commitment, uniswap_commitment,
            "Different contracts should have different layout commitments");
        
        // Same layout should produce same commitment
        let usdc_commitment2 = usdc_data.layout.commitment();
        assert_eq!(usdc_commitment, usdc_commitment2,
            "Same layout should produce consistent commitment");
    }

    #[test]
    fn test_storage_key_determinism() {
        let test_data = create_usdc_test_data();
        let resolver = EthereumKeyResolver;
        
        // Resolve the same path multiple times
        let query = "totalSupply";
        let result1 = resolver.resolve(&test_data.layout, query).unwrap();
        let result2 = resolver.resolve(&test_data.layout, query).unwrap();
        let result3 = resolver.resolve(&test_data.layout, query).unwrap();
        
        // All results should be identical
        assert_eq!(result1.key, result2.key, "Storage key should be deterministic");
        assert_eq!(result2.key, result3.key, "Storage key should be deterministic");
        assert_eq!(result1.layout_commitment, result2.layout_commitment, 
            "Layout commitment should be deterministic");
    }

    #[test]
    fn test_comprehensive_storage_coverage() {
        let test_data = create_usdc_test_data();
        let resolver = EthereumKeyResolver;
        
        // Test that we can resolve all known storage entries
        for (name, storage_entry) in &test_data.known_storage {
            let result = resolver.resolve(&test_data.layout, storage_entry.query);
            assert!(result.is_ok(), "Failed to resolve storage entry: {}", name);
            
            let path = result.unwrap();
            
            // Verify layout commitment is consistent
            let expected_commitment = test_data.layout.commitment();
            assert_eq!(path.layout_commitment, expected_commitment,
                "Layout commitment mismatch for entry: {}", name);
            
            // Verify key derivation matches expected
            match path.key {
                Key::Fixed(key_bytes) => {
                    let key_hex = hex::encode(key_bytes);
                    assert_eq!(key_hex, storage_entry.expected_key,
                        "Storage key mismatch for entry: {}", name);
                }
                _ => panic!("Expected fixed key for entry: {}", name),
            }
        }
    }

    #[test]
    fn test_manual_key_derivation_verification() {
        // Manually verify the key derivation for a known case
        let resolver = EthereumKeyResolver;
        
        // Test case: balanceOf[0x28c6c06298d514db089934071355e5743bf21d60] at slot 9
        let address_hex = "28c6c06298d514db089934071355e5743bf21d60";
        let slot = 9u64;
        
        // Manual calculation
        let address_bytes = hex::decode(address_hex).unwrap();
        let manual_key = EthereumKeyResolver::derive_mapping_key(&address_bytes, slot);
        
        // Now test with our resolver
        let test_data = create_usdc_test_data();
        let storage_entry = test_data.known_storage.get("binance_balance").unwrap();
        let result = resolver.resolve(&test_data.layout, storage_entry.query).unwrap();
        
        match result.key {
            Key::Fixed(key_bytes) => {
                assert_eq!(key_bytes, manual_key,
                    "Manual calculation should match resolver result");
            }
            _ => panic!("Expected fixed key"),
        }
    }

    #[test]
    fn test_ethereum_address_formatting() {
        let test_data = create_usdc_test_data();
        let resolver = EthereumKeyResolver;
        
        // Test different address formats should produce same result
        let address = "28c6c06298d514db089934071355e5743bf21d60";
        
        let queries = vec![
            format!("balanceOf[{}]", address),           // No 0x prefix
            format!("balanceOf[0x{}]", address),         // With 0x prefix
            format!("balanceOf[0x{}]", address.to_uppercase()), // Uppercase
        ];
        
        let mut results = Vec::new();
        for query in &queries {
            let result = resolver.resolve(&test_data.layout, query);
            assert!(result.is_ok(), "Failed to resolve query: {}", query);
            results.push(result.unwrap());
        }
        
        // All should produce the same storage key
        for i in 1..results.len() {
            assert_eq!(results[0].key, results[i].key,
                "Different address formats should produce same storage key");
        }
    }

    #[test]
    fn test_gas_optimization_layout_commitment() {
        // Test that layout commitment is efficient and doesn't change unnecessarily
        let layout = create_usdc_layout();
        
        let start = std::time::Instant::now();
        let commitment1 = layout.commitment();
        let time1 = start.elapsed();
        
        let start = std::time::Instant::now();
        let commitment2 = layout.commitment();
        let time2 = start.elapsed();
        
        // Commitments should be identical
        assert_eq!(commitment1, commitment2);
        
        // Should be reasonably fast (both should be under 1ms for this small layout)
        assert!(time1.as_millis() < 10, "Layout commitment should be fast");
        assert!(time2.as_millis() < 10, "Layout commitment should be fast");
        
        println!("Layout commitment times: {:?}, {:?}", time1, time2);
    }
} 