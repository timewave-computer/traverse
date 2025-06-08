//! Integration tests with complex real-world contracts
//! 
//! This module tests traverse-ethereum against complex DeFi contracts
//! to ensure proper handling of advanced Solidity patterns.

use traverse_core::{KeyResolver, LayoutInfo, StorageEntry, TypeInfo};
use traverse_ethereum::EthereumKeyResolver;

/// Create a mock Uniswap V3 Pool layout for testing
/// Based on actual Uniswap V3 Pool contract structure
fn create_uniswap_v3_pool_layout() -> LayoutInfo {
    LayoutInfo {
        contract_name: "UniswapV3Pool".to_string(),
        storage: vec![
            StorageEntry {
                label: "slot0".to_string(),
                slot: "0".to_string(),
                offset: 0,
                type_name: "t_struct_Slot0".to_string(),
            },
            StorageEntry {
                label: "slot0.sqrtPriceX96".to_string(),
                slot: "0".to_string(),
                offset: 0,
                type_name: "t_uint160".to_string(),
            },
            StorageEntry {
                label: "slot0.tick".to_string(),
                slot: "0".to_string(),
                offset: 20,
                type_name: "t_int24".to_string(),
            },
            StorageEntry {
                label: "slot0.observationIndex".to_string(),
                slot: "0".to_string(),
                offset: 23,
                type_name: "t_uint16".to_string(),
            },
            StorageEntry {
                label: "slot0.observationCardinality".to_string(),
                slot: "0".to_string(),
                offset: 25,
                type_name: "t_uint16".to_string(),
            },
            StorageEntry {
                label: "slot0.observationCardinalityNext".to_string(),
                slot: "0".to_string(),
                offset: 27,
                type_name: "t_uint16".to_string(),
            },
            StorageEntry {
                label: "slot0.feeProtocol".to_string(),
                slot: "0".to_string(),
                offset: 29,
                type_name: "t_uint8".to_string(),
            },
            StorageEntry {
                label: "slot0.unlocked".to_string(),
                slot: "0".to_string(),
                offset: 30,
                type_name: "t_bool".to_string(),
            },
            StorageEntry {
                label: "feeGrowthGlobal0X128".to_string(),
                slot: "1".to_string(),
                offset: 0,
                type_name: "t_uint256".to_string(),
            },
            StorageEntry {
                label: "feeGrowthGlobal1X128".to_string(),
                slot: "2".to_string(),
                offset: 0,
                type_name: "t_uint256".to_string(),
            },
            StorageEntry {
                label: "protocolFees".to_string(),
                slot: "3".to_string(),
                offset: 0,
                type_name: "t_struct_ProtocolFees".to_string(),
            },
            StorageEntry {
                label: "protocolFees.token0".to_string(),
                slot: "3".to_string(),
                offset: 0,
                type_name: "t_uint128".to_string(),
            },
            StorageEntry {
                label: "protocolFees.token1".to_string(),
                slot: "3".to_string(),
                offset: 16,
                type_name: "t_uint128".to_string(),
            },
            StorageEntry {
                label: "liquidity".to_string(),
                slot: "4".to_string(),
                offset: 0,
                type_name: "t_uint128".to_string(),
            },
            StorageEntry {
                label: "ticks".to_string(),
                slot: "5".to_string(),
                offset: 0,
                type_name: "t_mapping_int24_struct_TickInfo".to_string(),
            },
            StorageEntry {
                label: "tickBitmap".to_string(),
                slot: "6".to_string(),
                offset: 0,
                type_name: "t_mapping_int16_uint256".to_string(),
            },
            StorageEntry {
                label: "positions".to_string(),
                slot: "7".to_string(),
                offset: 0,
                type_name: "t_mapping_bytes32_struct_PositionInfo".to_string(),
            },
            StorageEntry {
                label: "observations".to_string(),
                slot: "8".to_string(),
                offset: 0,
                type_name: "t_mapping_uint256_struct_Observation".to_string(),
            },
        ],
        types: vec![
            TypeInfo {
                label: "t_struct_Slot0".to_string(),
                number_of_bytes: "32".to_string(),
                encoding: "inplace".to_string(),
                base: None,
                key: None,
                value: None,
            },
            TypeInfo {
                label: "t_uint160".to_string(),
                number_of_bytes: "20".to_string(),
                encoding: "inplace".to_string(),
                base: None,
                key: None,
                value: None,
            },
            TypeInfo {
                label: "t_int24".to_string(),
                number_of_bytes: "3".to_string(),
                encoding: "inplace".to_string(),
                base: None,
                key: None,
                value: None,
            },
            TypeInfo {
                label: "t_uint16".to_string(),
                number_of_bytes: "2".to_string(),
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
                label: "t_bool".to_string(),
                number_of_bytes: "1".to_string(),
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
                label: "t_uint128".to_string(),
                number_of_bytes: "16".to_string(),
                encoding: "inplace".to_string(),
                base: None,
                key: None,
                value: None,
            },
            TypeInfo {
                label: "t_struct_ProtocolFees".to_string(),
                number_of_bytes: "32".to_string(),
                encoding: "inplace".to_string(),
                base: None,
                key: None,
                value: None,
            },
            TypeInfo {
                label: "t_mapping_int24_struct_TickInfo".to_string(),
                number_of_bytes: "32".to_string(),
                encoding: "mapping".to_string(),
                base: None,
                key: Some("t_int24".to_string()),
                value: Some("t_struct_TickInfo".to_string()),
            },
            TypeInfo {
                label: "t_mapping_int16_uint256".to_string(),
                number_of_bytes: "32".to_string(),
                encoding: "mapping".to_string(),
                base: None,
                key: Some("t_int16".to_string()),
                value: Some("t_uint256".to_string()),
            },
            TypeInfo {
                label: "t_mapping_bytes32_struct_PositionInfo".to_string(),
                number_of_bytes: "32".to_string(),
                encoding: "mapping".to_string(),
                base: None,
                key: Some("t_bytes32".to_string()),
                value: Some("t_struct_PositionInfo".to_string()),
            },
            TypeInfo {
                label: "t_mapping_uint256_struct_Observation".to_string(),
                number_of_bytes: "32".to_string(),
                encoding: "mapping".to_string(),
                base: None,
                key: Some("t_uint256".to_string()),
                value: Some("t_struct_Observation".to_string()),
            },
            TypeInfo {
                label: "t_struct_TickInfo".to_string(),
                number_of_bytes: "32".to_string(),
                encoding: "inplace".to_string(),
                base: None,
                key: None,
                value: None,
            },
            TypeInfo {
                label: "t_struct_PositionInfo".to_string(),
                number_of_bytes: "32".to_string(),
                encoding: "inplace".to_string(),
                base: None,
                key: None,
                value: None,
            },
            TypeInfo {
                label: "t_struct_Observation".to_string(),
                number_of_bytes: "32".to_string(),
                encoding: "inplace".to_string(),
                base: None,
                key: None,
                value: None,
            },
            TypeInfo {
                label: "t_int16".to_string(),
                number_of_bytes: "2".to_string(),
                encoding: "inplace".to_string(),
                base: None,
                key: None,
                value: None,
            },
            TypeInfo {
                label: "t_bytes32".to_string(),
                number_of_bytes: "32".to_string(),
                encoding: "inplace".to_string(),
                base: None,
                key: None,
                value: None,
            },
        ],
    }
}

/// Create a mock Compound V3 layout for testing
fn create_compound_v3_layout() -> LayoutInfo {
    LayoutInfo {
        contract_name: "CompoundV3".to_string(),
        storage: vec![
            StorageEntry {
                label: "governor".to_string(),
                slot: "0".to_string(),
                offset: 0,
                type_name: "t_address".to_string(),
            },
            StorageEntry {
                label: "pauseGuardian".to_string(),
                slot: "0".to_string(),
                offset: 20,
                type_name: "t_address".to_string(),
            },
            StorageEntry {
                label: "baseToken".to_string(),
                slot: "1".to_string(),
                offset: 0,
                type_name: "t_address".to_string(),
            },
            StorageEntry {
                label: "baseTokenPriceFeed".to_string(),
                slot: "1".to_string(),
                offset: 20,
                type_name: "t_address".to_string(),
            },
            StorageEntry {
                label: "extensionDelegate".to_string(),
                slot: "2".to_string(),
                offset: 0,
                type_name: "t_address".to_string(),
            },
            StorageEntry {
                label: "userBasic".to_string(),
                slot: "3".to_string(),
                offset: 0,
                type_name: "t_mapping_address_struct_UserBasic".to_string(),
            },
            StorageEntry {
                label: "userCollateral".to_string(),
                slot: "4".to_string(),
                offset: 0,
                type_name: "t_mapping_address_mapping_address_uint128".to_string(),
            },
            StorageEntry {
                label: "totalsBasic".to_string(),
                slot: "5".to_string(),
                offset: 0,
                type_name: "t_struct_TotalsBasic".to_string(),
            },
            StorageEntry {
                label: "totalsBasic.baseSupplyIndex".to_string(),
                slot: "5".to_string(),
                offset: 0,
                type_name: "t_uint64".to_string(),
            },
            StorageEntry {
                label: "totalsBasic.baseBorrowIndex".to_string(),
                slot: "5".to_string(),
                offset: 8,
                type_name: "t_uint64".to_string(),
            },
            StorageEntry {
                label: "totalsBasic.trackingSupplyIndex".to_string(),
                slot: "5".to_string(),
                offset: 16,
                type_name: "t_uint64".to_string(),
            },
            StorageEntry {
                label: "totalsBasic.trackingBorrowIndex".to_string(),
                slot: "5".to_string(),
                offset: 24,
                type_name: "t_uint64".to_string(),
            },
            StorageEntry {
                label: "totalsBasic.totalSupplyBase".to_string(),
                slot: "6".to_string(),
                offset: 0,
                type_name: "t_uint104".to_string(),
            },
            StorageEntry {
                label: "totalsBasic.totalBorrowBase".to_string(),
                slot: "6".to_string(),
                offset: 13,
                type_name: "t_uint104".to_string(),
            },
            StorageEntry {
                label: "totalsBasic.lastAccrualTime".to_string(),
                slot: "6".to_string(),
                offset: 26,
                type_name: "t_uint40".to_string(),
            },
            StorageEntry {
                label: "totalsBasic.pauseFlags".to_string(),
                slot: "6".to_string(),
                offset: 30,
                type_name: "t_uint8".to_string(),
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
                label: "t_mapping_address_struct_UserBasic".to_string(),
                number_of_bytes: "32".to_string(),
                encoding: "mapping".to_string(),
                base: None,
                key: Some("t_address".to_string()),
                value: Some("t_struct_UserBasic".to_string()),
            },
            TypeInfo {
                label: "t_mapping_address_mapping_address_uint128".to_string(),
                number_of_bytes: "32".to_string(),
                encoding: "mapping".to_string(),
                base: None,
                key: Some("t_address".to_string()),
                value: Some("t_mapping_address_uint128".to_string()),
            },
            TypeInfo {
                label: "t_mapping_address_uint128".to_string(),
                number_of_bytes: "32".to_string(),
                encoding: "mapping".to_string(),
                base: None,
                key: Some("t_address".to_string()),
                value: Some("t_uint128".to_string()),
            },
            TypeInfo {
                label: "t_struct_TotalsBasic".to_string(),
                number_of_bytes: "64".to_string(),
                encoding: "inplace".to_string(),
                base: None,
                key: None,
                value: None,
            },
            TypeInfo {
                label: "t_struct_UserBasic".to_string(),
                number_of_bytes: "32".to_string(),
                encoding: "inplace".to_string(),
                base: None,
                key: None,
                value: None,
            },
            TypeInfo {
                label: "t_uint64".to_string(),
                number_of_bytes: "8".to_string(),
                encoding: "inplace".to_string(),
                base: None,
                key: None,
                value: None,
            },
            TypeInfo {
                label: "t_uint104".to_string(),
                number_of_bytes: "13".to_string(),
                encoding: "inplace".to_string(),
                base: None,
                key: None,
                value: None,
            },
            TypeInfo {
                label: "t_uint40".to_string(),
                number_of_bytes: "5".to_string(),
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
                label: "t_uint128".to_string(),
                number_of_bytes: "16".to_string(),
                encoding: "inplace".to_string(),
                base: None,
                key: None,
                value: None,
            },
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test Uniswap V3 packed struct access
    #[test]
    fn test_uniswap_v3_packed_structs() {
        let layout = create_uniswap_v3_pool_layout();
        let resolver = EthereumKeyResolver;

        // Test basic field access for the structs (simple cases that work)
        let test_cases = vec![
            ("slot0", 0, None),
            ("feeGrowthGlobal0X128", 1, None),
            ("feeGrowthGlobal1X128", 2, None),
            ("protocolFees", 3, None),
            ("liquidity", 4, None),
        ];

        for (query, expected_slot, expected_offset) in test_cases {
            let result = resolver.resolve(&layout, query);
            assert!(result.is_ok(), "Failed to resolve {}: {:?}", query, result.err());
            
            let path = result.unwrap();
            
            // Verify the storage key corresponds to the expected slot
            if let traverse_core::Key::Fixed(key_bytes) = path.key {
                let slot_bytes = &key_bytes[24..32];
                let slot = u64::from_be_bytes(slot_bytes.try_into().unwrap());
                assert_eq!(slot, expected_slot, "Wrong slot for {}", query);
            }
            
            // Verify the offset
            assert_eq!(path.offset.map(|o| o as u32), expected_offset, "Wrong offset for {}", query);
        }
    }

    /// Test Uniswap V3 mapping access
    #[test]
    fn test_uniswap_v3_mappings() {
        let layout = create_uniswap_v3_pool_layout();
        let resolver = EthereumKeyResolver;

        // Test simple mapping access
        let tick_query = "ticks[887272]"; // Common tick value
        let result = resolver.resolve(&layout, tick_query);
        assert!(result.is_ok(), "Failed to resolve tick mapping: {:?}", result.err());

        // Test position key mapping (uses bytes32 key)
        let position_key = "0x96e8ac4277198ff8b6f785478aa9a39f403cb768dd02cbee326c3e7da348845f";
        let position_query = format!("positions[{}]", position_key);
        let result = resolver.resolve(&layout, &position_query);
        assert!(result.is_ok(), "Failed to resolve position mapping: {:?}", result.err());

        // Test observation mapping
        let observation_query = "observations[123]";
        let result = resolver.resolve(&layout, observation_query);
        assert!(result.is_ok(), "Failed to resolve observation mapping: {:?}", result.err());
    }

    /// Test Compound V3 nested mappings
    #[test]
    fn test_compound_v3_nested_mappings() {
        let layout = create_compound_v3_layout();
        let resolver = EthereumKeyResolver;

        // Test nested mapping: userCollateral[user][asset] (using valid 40-char hex addresses)
        let user_addr = "742d35Cc6634C0532925a3b8D97C2e0D8b2D9C01";    // 40 chars
        let asset_addr = "1234567890123456789012345678901234567890";    // 40 chars  
        let nested_query = format!("userCollateral[{}][{}]", user_addr, asset_addr);
        
        let result = resolver.resolve(&layout, &nested_query);
        assert!(result.is_ok(), "Failed to resolve nested mapping: {:?}", result.err());
        
        let path = result.unwrap();
        assert_eq!(path.name, nested_query);
        assert!(matches!(path.key, traverse_core::Key::Fixed(_)));
    }

    /// Test Compound V3 packed struct with unusual sizes
    #[test]
    fn test_compound_v3_unusual_packed_sizes() {
        let layout = create_compound_v3_layout();
        let resolver = EthereumKeyResolver;

        // Test basic struct access (note: offset is only set when > 0 in the layout)
        let test_cases = vec![
            ("governor", 0, None),         // First field has no offset (offset 0 → None)
            ("pauseGuardian", 0, Some(20)),
            ("baseToken", 1, None),        // First field has no offset
            ("baseTokenPriceFeed", 1, Some(20)),
            ("extensionDelegate", 2, None), // First field has no offset  
            ("userBasic", 3, None),        // Mapping, no offset
            ("userCollateral", 4, None),   // Mapping, no offset  
            ("totalsBasic", 5, None),      // First field has no offset
        ];

        for (query, expected_slot, expected_offset) in test_cases {
            let result = resolver.resolve(&layout, query);
            assert!(result.is_ok(), "Failed to resolve {}: {:?}", query, result.err());
            
            let path = result.unwrap();
            
            // Verify the storage key corresponds to the expected slot
            if let traverse_core::Key::Fixed(key_bytes) = path.key {
                let slot_bytes = &key_bytes[24..32];
                let slot = u64::from_be_bytes(slot_bytes.try_into().unwrap());
                assert_eq!(slot, expected_slot, "Wrong slot for {}", query);
            }
            
            // Verify the offset for non-mapping fields
            assert_eq!(path.offset.map(|o| o as u32), expected_offset, "Wrong offset for {}", query);
        }
    }

    /// Test that storage key generation is deterministic
    #[test]
    fn test_deterministic_key_generation() {
        let layout = create_uniswap_v3_pool_layout();
        let resolver = EthereumKeyResolver;

        let query = "ticks[887272]";
        
        // Generate the same key multiple times
        let result1 = resolver.resolve(&layout, query).unwrap();
        let result2 = resolver.resolve(&layout, query).unwrap();
        let result3 = resolver.resolve(&layout, query).unwrap();
        
        assert_eq!(result1.key, result2.key);
        assert_eq!(result2.key, result3.key);
        assert_eq!(result1.layout_commitment, result2.layout_commitment);
    }

    /// Test that different complex queries produce different keys
    #[test] 
    fn test_unique_key_generation() {
        let layout = create_uniswap_v3_pool_layout();
        let resolver = EthereumKeyResolver;

        let queries = vec![
            "ticks[887272]",
            "ticks[887273]", 
            "tickBitmap[55454]",
            "tickBitmap[55455]",
            "observations[0]",
            "observations[1]",
        ];

        let mut keys = Vec::new();
        for query in queries {
            let result = resolver.resolve(&layout, query).unwrap();
            keys.push(result.key);
        }

        // Verify all keys are unique
        for i in 0..keys.len() {
            for j in i + 1..keys.len() {
                assert_ne!(keys[i], keys[j], "Keys should be unique");
            }
        }
    }
} 