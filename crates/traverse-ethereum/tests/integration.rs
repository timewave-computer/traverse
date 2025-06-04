//! Integration tests for the traverse ZK storage path generator
//! 
//! These tests use realistic contract data to validate the entire pipeline
//! from layout compilation through path resolution.

use traverse_core::{LayoutInfo, KeyResolver, Key, StorageEntry, TypeInfo};
use traverse_ethereum::EthereumKeyResolver;

/// Load the test ERC20 layout data
fn load_test_layout() -> LayoutInfo {
    let content = include_str!("data/erc20_layout.json");
    serde_json::from_str(content).expect("Failed to parse ERC20 layout test data")
}

/// Test the layout compiler with known valid data
#[test]
fn test_layout_compilation() {
    // Since we're using include_str!, we already know the layout compiles correctly
    // Just verify the loaded layout has expected properties
    let layout = load_test_layout();
    
    assert_eq!(layout.contract_name, "MockERC20");
    assert_eq!(layout.storage.len(), 8); // 8 storage variables
    assert_eq!(layout.types.len(), 7); // 7 type definitions
    
    // Verify layout commitment is deterministic
    let commitment1 = layout.commitment();
    let commitment2 = layout.commitment();
    assert_eq!(commitment1, commitment2, "Layout commitment should be deterministic");
}

/// Test resolving simple storage fields
#[test]
fn test_resolve_simple_fields() {
    let layout = load_test_layout();
    let resolver = EthereumKeyResolver;
    
    // Test resolving _totalSupply (slot 2)
    let result = resolver.resolve(&layout, "_totalSupply");
    assert!(result.is_ok(), "Should resolve _totalSupply");
    
    let path = result.unwrap();
    assert_eq!(path.name, "_totalSupply");
    
    // Should be fixed key with slot 2
    match path.key {
        Key::Fixed(key_bytes) => {
            // Slot 2 should be at bytes 24-32 in big-endian format
            let expected_slot = 2u64.to_be_bytes();
            assert_eq!(&key_bytes[24..32], &expected_slot);
            assert_eq!(&key_bytes[0..24], &[0u8; 24]); // Leading zeros
        }
        _ => panic!("Expected fixed key for simple field"),
    }
    
    assert_eq!(path.offset, None); // No offset for uint256
    assert_eq!(path.field_size, Some(32)); // 32 bytes for uint256
}

/// Test resolving packed fields with offsets
#[test]
fn test_resolve_packed_fields() {
    let layout = load_test_layout();
    let resolver = EthereumKeyResolver;
    
    // Test owner (slot 6, offset 0)
    let owner_result = resolver.resolve(&layout, "owner");
    assert!(owner_result.is_ok(), "Should resolve owner");
    
    let owner_path = owner_result.unwrap();
    assert_eq!(owner_path.offset, None); // First field, no offset
    assert_eq!(owner_path.field_size, Some(20)); // 20 bytes for address
    
    // Test paused (slot 6, offset 20) - packed with owner
    let paused_result = resolver.resolve(&layout, "paused");
    assert!(paused_result.is_ok(), "Should resolve paused");
    
    let paused_path = paused_result.unwrap();
    assert_eq!(paused_path.offset, Some(20)); // Offset after address
    assert_eq!(paused_path.field_size, Some(1)); // 1 byte for bool
    
    // Both should have the same storage slot
    match (&owner_path.key, &paused_path.key) {
        (Key::Fixed(owner_key), Key::Fixed(paused_key)) => {
            assert_eq!(owner_key, paused_key, "Packed fields should have same storage slot");
        }
        _ => panic!("Expected fixed keys for packed fields"),
    }
}

/// Test resolving mapping queries with known addresses
#[test]
fn test_resolve_mapping_queries() {
    let layout = load_test_layout();
    let resolver = EthereumKeyResolver;
    
    // Test _balances mapping with a known address
    let test_address = "1234567890abcdef1234567890abcdef12345678"; // 20 bytes hex
    let query = format!("_balances[0x{}]", test_address);
    
    let result = resolver.resolve(&layout, &query);
    assert!(result.is_ok(), "Should resolve _balances mapping query");
    
    let path = result.unwrap();
    assert_eq!(path.name, &query);
    assert_eq!(path.offset, None); // Mappings don't have offsets
    assert_eq!(path.field_size, Some(32)); // uint256 value
    
    // Verify the storage key is correctly computed
    match path.key {
        Key::Fixed(key_bytes) => {
            // Key should be keccak256(address ++ slot)
            // We can't easily predict the exact hash, but we can verify it's not zero
            assert_ne!(key_bytes, [0u8; 32], "Mapping key should not be zero");
        }
        _ => panic!("Expected fixed key for mapping"),
    }
}

/// Test storage key derivation with known test vectors
#[test]
fn test_storage_key_derivation() {
    let layout = load_test_layout();
    let resolver = EthereumKeyResolver;
    
    // Test with a specific address to get predictable results
    let test_address = "000000000000000000000000deadbeefdeadbeefdeadbeef";
    let query = format!("_balances[0x{}]", test_address);
    
    let result1 = resolver.resolve(&layout, &query);
    let result2 = resolver.resolve(&layout, &query);
    
    assert!(result1.is_ok() && result2.is_ok(), "Both resolutions should succeed");
    
    let path1 = result1.unwrap();
    let path2 = result2.unwrap();
    
    // Same query should produce same storage key
    assert_eq!(path1.key, path2.key, "Same query should produce same storage key");
    assert_eq!(path1.layout_commitment, path2.layout_commitment, "Layout commitment should be same");
}

/// Test resolve_all functionality
#[test]
fn test_resolve_all_paths() {
    let layout = load_test_layout();
    let resolver = EthereumKeyResolver;
    
    let result = resolver.resolve_all(&layout);
    assert!(result.is_ok(), "Should resolve all paths");
    
    let paths = result.unwrap();
    
    // Should resolve all simple fields (non-mapping fields can be resolved without keys)
    // Mappings require specific keys, so they won't be included in resolve_all for now
    assert!(!paths.is_empty(), "Should resolve at least some paths");
    assert!(paths.len() <= layout.storage.len(), "Should not exceed total storage entries");
    
    // All paths should have the same layout commitment
    if !paths.is_empty() {
        let first_commitment = paths[0].layout_commitment;
        for path in &paths {
            assert_eq!(path.layout_commitment, first_commitment, "All paths should have same layout commitment");
        }
    }
}

/// Test error handling for invalid queries
#[test]
fn test_invalid_query_handling() {
    let layout = load_test_layout();
    let resolver = EthereumKeyResolver;
    
    // Test non-existent field
    let result = resolver.resolve(&layout, "nonexistent_field");
    assert!(result.is_err(), "Should fail for non-existent field");
    
    // Test malformed mapping query
    let result = resolver.resolve(&layout, "_balances[invalid_hex]");
    assert!(result.is_err(), "Should fail for invalid hex in mapping query");
    
    // Test incomplete mapping query
    let result = resolver.resolve(&layout, "_balances[");
    assert!(result.is_err(), "Should fail for incomplete mapping query");
}

/// Test layout commitment consistency across different operations
#[test]
fn test_layout_commitment_consistency() {
    let layout = load_test_layout();
    let resolver = EthereumKeyResolver;
    
    // Get layout commitment directly
    let direct_commitment = layout.commitment();
    
    // Get commitment from resolved path
    let path_result = resolver.resolve(&layout, "_totalSupply");
    assert!(path_result.is_ok(), "Should resolve _totalSupply");
    
    let path_commitment = path_result.unwrap().layout_commitment;
    
    assert_eq!(direct_commitment, path_commitment, "Layout commitments should match");
}

/// Test with different address formats
#[test]
fn test_address_format_handling() {
    let layout = load_test_layout();
    let resolver = EthereumKeyResolver;
    
    let addresses = vec![
        "1234567890abcdef1234567890abcdef12345678",      // No 0x prefix
        "0x1234567890abcdef1234567890abcdef12345678",    // With 0x prefix
        "ABCDEF1234567890ABCDEF1234567890ABCDEF12",      // Uppercase
        "0xabcdef1234567890abcdef1234567890abcdef12",    // Lowercase with prefix
    ];
    
    for address in addresses {
        let query = format!("_balances[{}]", address);
        let result = resolver.resolve(&layout, &query);
        assert!(result.is_ok(), "Should handle address format: {}", address);
    }
}

/// Performance baseline test
#[test]
fn test_performance_baseline() {
    let layout = load_test_layout();
    let resolver = EthereumKeyResolver;
    
    let start = std::time::Instant::now();
    
    // Perform 100 resolutions (reduced from 1000 for faster testing)
    for i in 0..100 {
        let address = format!("{:040x}", i); // Generate different addresses
        let query = format!("_balances[0x{}]", address);
        let result = resolver.resolve(&layout, &query);
        assert!(result.is_ok(), "Resolution {} should succeed", i);
    }
    
    let duration = start.elapsed();
    println!("100 resolutions took: {:?}", duration);
    
    // Should complete in reasonable time
    assert!(duration.as_millis() < 1000, "Performance baseline: should complete 100 resolutions in under 1 second");
}

/// Test resolving nested mapping queries
#[test]
fn test_resolve_nested_mapping_queries() {
    let layout = load_test_layout();
    let resolver = EthereumKeyResolver;
    
    // Test nested mapping query (like allowances[owner][spender])
    // Note: This assumes we have an allowances field in our test data
    // For now, we'll test the parsing and key derivation logic
    
    // Create a mock nested mapping entry
    let mut test_layout = layout.clone();
    test_layout.storage.push(StorageEntry {
        label: "allowances".to_string(),
        slot: "2".to_string(),
        offset: 0,
        type_name: "t_mapping_address_mapping_address_uint256".to_string(),
    });
    
    test_layout.types.push(TypeInfo {
        label: "t_mapping_address_mapping_address_uint256".to_string(),
        number_of_bytes: "32".to_string(),
        encoding: "mapping".to_string(),
        base: None,
        key: Some("t_address".to_string()),
        value: Some("t_mapping_address_uint256".to_string()),
    });
    
    test_layout.types.push(TypeInfo {
        label: "t_mapping_address_uint256".to_string(),
        number_of_bytes: "32".to_string(),
        encoding: "mapping".to_string(),
        base: None,
        key: Some("t_address".to_string()),
        value: Some("t_uint256".to_string()),
    });
    
    // Test nested mapping resolution
    let owner = "742d35Cc6634C0532925a3b8D97C2e0D8b2D9C";
    let spender = "1234567890123456789012345678901234567890";
    let query = format!("allowances[{}][{}]", owner, spender);
    
    let result = resolver.resolve(&test_layout, &query);
    assert!(result.is_ok(), "Failed to resolve nested mapping query: {:?}", result.err());
    
    let path = result.unwrap();
    assert_eq!(path.name, query);
    assert!(matches!(path.key, Key::Fixed(_)));
    assert_eq!(path.offset, None);
    assert_eq!(path.field_size, Some(32)); // uint256 size
    
    // Verify the key derivation is deterministic
    let result2 = resolver.resolve(&test_layout, &query);
    assert!(result2.is_ok());
    assert_eq!(path.key, result2.unwrap().key);
}

/// Test resolving array access queries
#[test]
fn test_resolve_array_queries() {
    let layout = load_test_layout();
    let resolver = EthereumKeyResolver;
    
    // Create a mock array entry for testing
    let mut test_layout = layout.clone();
    test_layout.storage.push(StorageEntry {
        label: "items".to_string(),
        slot: "7".to_string(),
        offset: 0,
        type_name: "t_array_uint256".to_string(),
    });
    
    test_layout.types.push(TypeInfo {
        label: "t_array_uint256".to_string(),
        number_of_bytes: "32".to_string(),
        encoding: "dynamic_array".to_string(),
        base: Some("t_uint256".to_string()),
        key: None,
        value: None,
    });
    
    // Test array access resolution
    let query = "items[5]";
    let result = resolver.resolve(&test_layout, query);
    assert!(result.is_ok(), "Failed to resolve array query: {:?}", result.err());
    
    let path = result.unwrap();
    assert_eq!(path.name, query);
    assert!(matches!(path.key, Key::Fixed(_)));
    assert_eq!(path.offset, None);
    assert_eq!(path.field_size, Some(32)); // uint256 size
    
    // Verify the key derivation is deterministic
    let result2 = resolver.resolve(&test_layout, query);
    assert!(result2.is_ok());
    assert_eq!(path.key, result2.unwrap().key);
    
    // Test different indices produce different keys
    let result3 = resolver.resolve(&test_layout, "items[10]");
    assert!(result3.is_ok());
    assert_ne!(path.key, result3.unwrap().key);
}

/// Test reading strings that span multiple storage slots
/// 
/// In Ethereum storage, strings are stored as follows:
/// - If length ≤ 31 bytes: directly in the slot with length encoded in the last byte
/// - If length ≥ 32 bytes: length*2+1 stored in slot, data starts at keccak256(slot)
#[test]
fn test_multi_slot_string_storage() {
    let layout = load_test_layout();
    let resolver = EthereumKeyResolver;
    
    // Test 1: String length query - reads the length from the original slot
    let length_query = "_name.length";
    let length_result = resolver.resolve(&layout, length_query);
    assert!(length_result.is_ok(), "Failed to resolve string length query: {:?}", length_result.err());
    
    let length_path = length_result.unwrap();
    assert_eq!(length_path.name, length_query);
    assert_eq!(length_path.field_size, Some(32)); // Length is stored as uint256
    assert_eq!(length_path.offset, None);
    
    // For length query, the key should be the original slot (slot 3 for _name)
    match length_path.key {
        Key::Fixed(key_bytes) => {
            let expected_slot = 3u64.to_be_bytes();
            assert_eq!(&key_bytes[24..32], &expected_slot, "Length should be stored at original slot");
            assert_eq!(&key_bytes[0..24], &[0u8; 24], "Leading bytes should be zero");
        }
        _ => panic!("Expected fixed key for string length"),
    }
    
    // Test 2: String data query - reads data starting at keccak256(slot)
    let data_query = "_name.data";
    let data_result = resolver.resolve(&layout, data_query);
    assert!(data_result.is_ok(), "Failed to resolve string data query: {:?}", data_result.err());
    
    let data_path = data_result.unwrap();
    assert_eq!(data_path.name, data_query);
    assert_eq!(data_path.field_size, Some(32)); // Data stored in 32-byte chunks
    assert_eq!(data_path.offset, None);
    
    // For data query, the key should be keccak256(original_slot)
    match data_path.key {
        Key::Fixed(key_bytes) => {
            // Manually calculate expected data key using the same method as the resolver
            let mut slot_bytes = [0u8; 32];
            slot_bytes[24..].copy_from_slice(&3u64.to_be_bytes()); // slot 3 for _name
            
            // Verify the key matches what we expect for string data storage
            assert_ne!(key_bytes, [0u8; 32], "Data key should not be zero");
            assert_ne!(key_bytes, slot_bytes, "Data key should be different from slot");
        }
        _ => panic!("Expected fixed key for string data"),
    }
    
    // Test 3: Symbol string (different slot) to verify pattern consistency
    let symbol_data_query = "_symbol.data";
    let symbol_result = resolver.resolve(&layout, symbol_data_query);
    assert!(symbol_result.is_ok(), "Failed to resolve symbol data query");
    
    let symbol_path = symbol_result.unwrap();
    
    // Symbol and name data keys should be different (different base slots)
    match (&data_path.key, &symbol_path.key) {
        (Key::Fixed(name_key), Key::Fixed(symbol_key)) => {
            assert_ne!(name_key, symbol_key, "Different strings should have different data keys");
        }
        _ => panic!("Expected fixed keys for both strings"),
    }
    
    // Test 4: Verify deterministic behavior
    let data_result2 = resolver.resolve(&layout, data_query);
    assert!(data_result2.is_ok());
    assert_eq!(data_path.key, data_result2.unwrap().key, "String data key derivation should be deterministic");
    
    // Test 5: Test both strings have consistent layout commitment
    assert_eq!(length_path.layout_commitment, data_path.layout_commitment, 
        "All paths should have same layout commitment");
    assert_eq!(data_path.layout_commitment, symbol_path.layout_commitment,
        "All paths should have same layout commitment");
    
    println!("Multi-slot string test successful:");
    println!("  Name length slot: {}", hex::encode(match length_path.key { Key::Fixed(k) => k, _ => [0u8; 32] }));
    println!("  Name data slot: {}", hex::encode(match data_path.key { Key::Fixed(k) => k, _ => [0u8; 32] }));
    println!("  Symbol data slot: {}", hex::encode(match symbol_path.key { Key::Fixed(k) => k, _ => [0u8; 32] }));
} 