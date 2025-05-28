//! Comprehensive RLP encoding and decoding tests using validated test vectors
//! 
//! This module contains extensive tests for RLP functionality using test vectors
//! from the official Ethereum test suite and additional edge cases to ensure
//! robust handling of different RLP encoding scenarios.

use rlp::{encode, decode, DecoderError, Rlp, RlpStream};

/// Test basic string encoding from official Ethereum test vectors
#[cfg(test)]
mod string_encoding_tests {
    use super::*;

    #[test]
    fn test_empty_string() {
        // Test vector: emptystring
        let input = "";
        let expected = hex::decode("80").unwrap();
        let encoded = encode(&input);
        assert_eq!(encoded.to_vec(), expected, "Empty string should encode to 0x80");
        
        // Test decoding
        let decoded: String = decode(&expected).unwrap();
        assert_eq!(decoded, input);
    }

    #[test]
    fn test_single_byte_strings() {
        // Test vector: bytestring01
        let input = b"\x01";
        let expected = hex::decode("01").unwrap();
        let encoded = encode(&input.to_vec());
        assert_eq!(encoded.to_vec(), expected, "Byte 0x01 should encode to 0x01");

        // Test vector: bytestring7F
        let input = b"\x7f";
        let expected = hex::decode("7f").unwrap();
        let encoded = encode(&input.to_vec());
        assert_eq!(encoded.to_vec(), expected, "Byte 0x7F should encode to 0x7F");
    }

    #[test]
    fn test_short_string() {
        // Test vector: shortstring
        let input = "dog";
        let expected = hex::decode("83646f67").unwrap();
        let encoded = encode(&input);
        assert_eq!(encoded.to_vec(), expected, "Short string 'dog' encoding mismatch");
        
        // Test decoding
        let decoded: String = decode(&expected).unwrap();
        assert_eq!(decoded, input);
    }

    #[test]
    fn test_medium_string() {
        // Test vector: shortstring2 (55 bytes - boundary case)
        let input = "Lorem ipsum dolor sit amet, consectetur adipisicing eli";
        let expected = hex::decode("b74c6f72656d20697073756d20646f6c6f722073697420616d65742c20636f6e7365637465747572206164697069736963696e6720656c69").unwrap();
        let encoded = encode(&input);
        assert_eq!(encoded.to_vec(), expected, "Medium string (55 bytes) encoding mismatch");
        
        // Test decoding
        let decoded: String = decode(&expected).unwrap();
        assert_eq!(decoded, input);
    }

    #[test]
    fn test_long_string() {
        // Test vector: longstring (55 bytes - near the boundary)
        let input = "Lorem ipsum dolor sit amet, consectetur adipiscing elit";
        let encoded = encode(&input);
        
        // Verify the string is exactly 55 bytes
        assert_eq!(input.len(), 55, "Test string should be 55 bytes");
        
        // Verify RLP encoding structure for 55-byte string
        assert_eq!(encoded[0], 0xb7, "55-byte string should start with 0xb7 (183)");
        
        // Test decoding round-trip
        let decoded: String = decode(&encoded).unwrap();
        assert_eq!(decoded, input, "String should round-trip correctly");
    }

    #[test]
    fn test_very_long_string() {
        // Test vector: longstring2 (1024 bytes)
        let input = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Curabitur mauris magna, suscipit sed vehicula non, iaculis faucibus tortor. Proin suscipit ultricies malesuada. Duis tortor elit, dictum quis tristique eu, ultrices at risus. Morbi a est imperdiet mi ullamcorper aliquet suscipit nec lorem. Aenean quis leo mollis, vulputate elit varius, consequat enim. Nulla ultrices turpis justo, et posuere urna consectetur nec. Proin non convallis metus. Donec tempor ipsum in mauris congue sollicitudin. Vestibulum ante ipsum primis in faucibus orci luctus et ultrices posuere cubilia Curae; Suspendisse convallis sem vel massa faucibus, eget lacinia lacus tempor. Nulla quis ultricies purus. Proin auctor rhoncus nibh condimentum mollis. Aliquam consequat enim at metus luctus, a eleifend purus egestas. Curabitur at nibh metus. Nam bibendum, neque at auctor tristique, lorem libero aliquet arcu, non interdum tellus lectus sit amet eros. Cras rhoncus, metus ac ornare cursus, dolor justo ultrices metus, at ullamcorper volutpat";
        
        // This is a very long encoding, so we'll test the structure rather than exact bytes
        let encoded = encode(&input);
        
        // Should start with 0xb9 (0xb7 + 2) indicating 2 bytes for length
        assert_eq!(encoded[0], 0xb9, "Very long string should start with 0xb9");
        
        // Next 2 bytes should be the length (1024 = 0x0400)
        assert_eq!(encoded[1], 0x04, "Length high byte should be 0x04");
        assert_eq!(encoded[2], 0x00, "Length low byte should be 0x00");
        
        // Test decoding
        let decoded: String = decode(&encoded).unwrap();
        assert_eq!(decoded, input);
        assert_eq!(decoded.len(), 1024, "Decoded string should be 1024 bytes");
    }
}

/// Test integer encoding from official Ethereum test vectors
#[cfg(test)]
mod integer_encoding_tests {
    use super::*;

    #[test]
    fn test_zero() {
        // Test vector: zero - encoded as empty byte array
        let input = 0u32;
        let expected = hex::decode("80").unwrap();
        let encoded = encode(&input);
        assert_eq!(encoded.to_vec(), expected, "Zero should encode to 0x80");
        
        // Test decoding
        let decoded: u32 = decode(&expected).unwrap();
        assert_eq!(decoded, input);
    }

    #[test]
    fn test_small_integers() {
        // Test vector: smallint
        let input = 1u32;
        let expected = hex::decode("01").unwrap();
        let encoded = encode(&input);
        assert_eq!(encoded.to_vec(), expected, "Integer 1 should encode to 0x01");

        // Test vector: smallint2
        let input = 16u32;
        let expected = hex::decode("10").unwrap();
        let encoded = encode(&input);
        assert_eq!(encoded.to_vec(), expected, "Integer 16 should encode to 0x10");

        // Test vector: smallint3
        let input = 79u32;
        let expected = hex::decode("4f").unwrap();
        let encoded = encode(&input);
        assert_eq!(encoded.to_vec(), expected, "Integer 79 should encode to 0x4f");
    }

    #[test]
    fn test_medium_integers() {
        // Test vector: mediumint1
        let input = 255u32;
        let expected = hex::decode("81ff").unwrap();
        let encoded = encode(&input);
        assert_eq!(encoded.to_vec(), expected, "Integer 255 should encode to 0x81ff");

        // Test vector: mediumint2
        let input = 1024u32;
        let expected = hex::decode("820400").unwrap();
        let encoded = encode(&input);
        assert_eq!(encoded.to_vec(), expected, "Integer 1024 should encode to 0x820400");

        // Test vector: mediumint3
        let input = 0xFFFFFFu32;
        let expected = hex::decode("83ffffff").unwrap();
        let encoded = encode(&input);
        assert_eq!(encoded.to_vec(), expected, "Integer 0xFFFFFF should encode to 0x83ffffff");
    }

    #[test]
    fn test_large_integers() {
        // Test vector: mediumint4
        let input = 0xFFFFFFFFu32;
        let expected = hex::decode("84ffffffff").unwrap();
        let encoded = encode(&input);
        assert_eq!(encoded.to_vec(), expected, "Integer 0xFFFFFFFF should encode to 0x84ffffffff");

        // Test vector: mediumint5
        let input = 0x0102030405060708u64;
        let expected = hex::decode("880102030405060708").unwrap();
        let encoded = encode(&input);
        assert_eq!(encoded.to_vec(), expected, "Large integer encoding mismatch");
    }

    #[test]
    fn test_big_integer() {
        // Test vector: bigint (using byte array for very large numbers)
        // #115792089237316195423570985008687907853269984665640564039457584007913129639936
        let big_int_bytes = hex::decode("010000000000000000000000000000000000000000000000000000000000000000").unwrap();
        let expected = hex::decode("a1010000000000000000000000000000000000000000000000000000000000000000").unwrap();
        let encoded = encode(&big_int_bytes);
        assert_eq!(encoded.to_vec(), expected, "Big integer encoding mismatch");
    }
}

/// Test list encoding from official Ethereum test vectors
#[cfg(test)]
mod list_encoding_tests {
    use super::*;

    #[test]
    fn test_empty_list() {
        // Test vector: emptylist
        let stream = RlpStream::new_list(0);
        let expected = hex::decode("c0").unwrap();
        let encoded = stream.out();
        assert_eq!(encoded.to_vec(), expected, "Empty list should encode to 0xc0");
    }

    #[test]
    fn test_simple_list() {
        // Test vector: multilist
        let expected = hex::decode("c6827a77c10401").unwrap();
        
        // We need to encode this as a heterogeneous list: ["zw", [4], 1]
        let mut stream = RlpStream::new_list(3);
        stream.append(&"zw");
        
        // Create inner list [4]
        let mut inner_stream = RlpStream::new_list(1);
        inner_stream.append(&4u32);
        stream.append_raw(&inner_stream.out(), 1);
        
        stream.append(&1u32);
        let encoded = stream.out();
        
        assert_eq!(encoded.to_vec(), expected, "Multilist encoding mismatch");
    }

    #[test]
    fn test_nested_lists() {
        // Test vector: listsoflists - structure: [[[],[]],[]]
        let expected = hex::decode("c4c2c0c0c0").unwrap();
        
        let mut stream = RlpStream::new_list(2);
        
        // First element: [[],[]]
        let mut inner_stream = RlpStream::new_list(2);
        inner_stream.begin_list(0); // Empty list []
        inner_stream.begin_list(0); // Empty list []
        stream.append_raw(&inner_stream.out(), 1);
        
        // Second element: []
        stream.begin_list(0);
        
        let encoded = stream.out();
        assert_eq!(encoded.to_vec(), expected, "Nested lists encoding mismatch");
    }

    #[test]
    fn test_dictionary_like_list() {
        // Test vector: dictTest1
        let expected = hex::decode("ecca846b6579318476616c31ca846b6579328476616c32ca846b6579338476616c33ca846b6579348476616c34").unwrap();
        
        // Create structure: [["key1","val1"],["key2","val2"],["key3","val3"],["key4","val4"]]
        let mut stream = RlpStream::new_list(4);
        
        for i in 1..=4 {
            let mut pair_stream = RlpStream::new_list(2);
            pair_stream.append(&format!("key{}", i));
            pair_stream.append(&format!("val{}", i));
            stream.append_raw(&pair_stream.out(), 1);
        }
        
        let encoded = stream.out();
        assert_eq!(encoded.to_vec(), expected, "Dictionary-like list encoding mismatch");
    }

    #[test]
    fn test_string_list() {
        // Test encoding a list of strings using RlpStream
        let mut stream = RlpStream::new_list(3);
        stream.append(&"cat");
        stream.append(&"dog");
        stream.append(&"mouse");
        
        let encoded = stream.out();
        
        // Should start with list header
        assert!(encoded[0] >= 0xc0, "List should start with byte >= 0xc0");
        
        // Test decoding by parsing the RLP structure
        let rlp = Rlp::new(&encoded);
        assert!(rlp.is_list(), "Should be a list");
        assert_eq!(rlp.item_count().unwrap(), 3, "Should have 3 items");
        
        let item0: String = rlp.at(0).unwrap().as_val().unwrap();
        let item1: String = rlp.at(1).unwrap().as_val().unwrap();
        let item2: String = rlp.at(2).unwrap().as_val().unwrap();
        
        assert_eq!(item0, "cat");
        assert_eq!(item1, "dog");
        assert_eq!(item2, "mouse");
    }

    #[test]
    fn test_long_list() {
        // Test vector: longList1 - list with total payload > 55 bytes
        let mut stream = RlpStream::new_list(20);
        for i in 0..20 {
            stream.append(&format!("item{:02}", i));
        }
        let encoded = stream.out();
        
        // Should start with 0xf8 or higher (indicating long list)
        assert!(encoded[0] >= 0xf8, "Long list should start with byte >= 0xf8");
        
        // Test that we can parse it back
        let rlp = Rlp::new(&encoded);
        assert!(rlp.is_list());
        assert_eq!(rlp.item_count().unwrap(), 20);
    }

    #[test]
    fn test_very_long_list() {
        // Test vector: longList2 - list with payload requiring 2+ bytes for length
        let mut stream = RlpStream::new_list(100);
        for i in 0..100 {
            stream.append(&format!("longer_item_{:03}", i));
        }
        let encoded = stream.out();
        
        // Should start with 0xf9 or higher (indicating very long list)
        assert!(encoded[0] >= 0xf9, "Very long list should start with byte >= 0xf9");
        
        // Test that we can parse it back
        let rlp = Rlp::new(&encoded);
        assert!(rlp.is_list());
        assert_eq!(rlp.item_count().unwrap(), 100);
    }
}

/// Test edge cases and boundary conditions
#[cfg(test)]
mod edge_case_tests {
    use super::*;

    #[test]
    fn test_boundary_string_lengths() {
        // Test strings at exact boundary lengths
        
        // 55 bytes (boundary between short and long string)
        let str_55 = "a".repeat(55);
        let encoded_55 = encode(&str_55);
        assert_eq!(encoded_55[0], 0xb7, "55-byte string should start with 0xb7");
        
        // 56 bytes (first long string)
        let str_56 = "a".repeat(56);
        let encoded_56 = encode(&str_56);
        assert_eq!(encoded_56[0], 0xb8, "56-byte string should start with 0xb8");
        assert_eq!(encoded_56[1], 56, "Length byte should be 56");
        
        // Test decoding
        let decoded_55: String = decode(&encoded_55).unwrap();
        let decoded_56: String = decode(&encoded_56).unwrap();
        assert_eq!(decoded_55, str_55);
        assert_eq!(decoded_56, str_56);
    }

    #[test]
    fn test_boundary_list_lengths() {
        // Test lists at exact boundary lengths
        
        // Create a list with payload exactly 55 bytes using single-char strings
        let mut stream = RlpStream::new_list(55);
        for _ in 0..55 {
            stream.append(&"a");
        }
        let encoded = stream.out();
        assert_eq!(encoded[0], 0xf7, "55-byte payload list should start with 0xf7");
        
        // Create a list with payload 56+ bytes
        let mut stream_56 = RlpStream::new_list(30);
        for _ in 0..30 {
            stream_56.append(&"ab"); // 30 two-char strings
        }
        let encoded_56 = stream_56.out();
        assert_eq!(encoded_56[0], 0xf8, "Long payload list should start with 0xf8");
    }

    #[test]
    fn test_leading_zeros_in_byte_arrays() {
        // Test that byte arrays with leading zeros are handled correctly
        let data_with_leading_zeros = hex::decode("0001").unwrap();
        let data_without_leading_zeros = hex::decode("01").unwrap();
        
        // Both should encode as byte arrays preserving their original form
        let rlp_with_zeros = encode(&data_with_leading_zeros);
        let rlp_without_zeros = encode(&data_without_leading_zeros);
        
        // The RLP encoding should be different (preserving leading zeros for byte arrays)
        assert_ne!(rlp_with_zeros.to_vec(), rlp_without_zeros.to_vec());
        
        // When decoded as byte arrays, they should maintain their original form
        let decoded_with: Vec<u8> = decode(&rlp_with_zeros).unwrap();
        let decoded_without: Vec<u8> = decode(&rlp_without_zeros).unwrap();
        assert_eq!(decoded_with, data_with_leading_zeros);
        assert_eq!(decoded_without, data_without_leading_zeros);
    }

    #[test]
    fn test_unicode_strings() {
        // Test non-ASCII Unicode strings
        let unicode_str = "Hello 世界 🌍";
        let encoded = encode(&unicode_str);
        let decoded: String = decode(&encoded).unwrap();
        assert_eq!(decoded, unicode_str, "Unicode string should round-trip correctly");
    }

    #[test]
    fn test_binary_data() {
        // Test encoding raw binary data
        let binary_data: Vec<u8> = (0..=255).collect();
        let encoded = encode(&binary_data);
        let decoded: Vec<u8> = decode(&encoded).unwrap();
        assert_eq!(decoded, binary_data, "Binary data should round-trip correctly");
    }

    #[test]
    fn test_deeply_nested_structures() {
        // Test deeply nested list structures
        let mut nested = RlpStream::new_list(1);
        nested.append(&"innermost");
        
        for _ in 0..10 {
            let mut outer = RlpStream::new_list(1);
            outer.append_raw(&nested.out(), 1);
            nested = outer;
        }
        
        let encoded = nested.out();
        
        // Should be able to parse the nested structure
        let rlp = Rlp::new(&encoded);
        assert!(rlp.is_list(), "Should be a list");
        assert_eq!(rlp.item_count().unwrap(), 1, "Should have one item");
        
        // Navigate to the innermost string - need to go through 11 levels total
        let mut current = rlp;
        for _ in 0..11 {  
            current = current.at(0).unwrap();
        }
        
        // Check that the innermost element is data, not a list
        assert!(!current.is_list(), "Innermost element should be data, not a list");
        let innermost: String = current.as_val().unwrap();
        assert_eq!(innermost, "innermost");
    }
}

/// Test error conditions and invalid RLP data
#[cfg(test)]
mod error_handling_tests {
    use super::*;

    #[test]
    fn test_truncated_data() {
        // Test various truncated RLP data scenarios
        
        // Truncated string length
        let truncated_string = hex::decode("b8").unwrap(); // Says 1-byte length follows, but no length
        let _result: Result<String, DecoderError> = decode(&truncated_string);
        assert!(_result.is_err(), "Truncated string length should fail");
        
        // Truncated string data
        let truncated_data = hex::decode("8548656c6c").unwrap(); // Says 5 bytes, but only 4 follow
        let _result: Result<String, DecoderError> = decode(&truncated_data);
        assert!(_result.is_err(), "Truncated string data should fail");
    }

    #[test]
    fn test_invalid_length_encoding() {
        // Test invalid length encodings
        
        // Length longer than available data
        let invalid_length = hex::decode("b90100").unwrap(); // Says 256 bytes follow, but none do
        let _result: Result<String, DecoderError> = decode(&invalid_length);
        assert!(_result.is_err(), "Invalid length should fail");
    }

    #[test]
    fn test_empty_input() {
        // Test completely empty input
        let empty = Vec::new();
        let _result: Result<String, DecoderError> = decode(&empty);
        assert!(_result.is_err(), "Empty input should fail");
    }

    #[test]
    fn test_malformed_list() {
        // Test malformed list structures
        
        // List claims more items than available data
        let malformed_list = hex::decode("c3010203").unwrap(); // Says 3 bytes, provides exactly 3
        let rlp = Rlp::new(&malformed_list);
        
        // This should parse as a list with byte values, not cause an error
        assert!(rlp.is_list());
        
        // But trying to get too many items should fail
        let result = rlp.at(10); // Ask for 11th item
        assert!(result.is_err(), "Accessing non-existent list item should fail");
    }
}

/// Test RLP encoding/decoding performance with large datasets
#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_large_string_performance() {
        // Test encoding/decoding a very large string
        let large_string = "x".repeat(1_000_000); // 1MB string
        
        let start = Instant::now();
        let encoded = encode(&large_string);
        let encode_time = start.elapsed();
        
        let start = Instant::now();
        let decoded: String = decode(&encoded).unwrap();
        let decode_time = start.elapsed();
        
        assert_eq!(decoded, large_string);
        println!("Large string (1MB) - Encode: {:?}, Decode: {:?}", encode_time, decode_time);
        
        // Should complete in reasonable time
        assert!(encode_time.as_millis() < 1000, "Encoding should be fast");
        assert!(decode_time.as_millis() < 1000, "Decoding should be fast");
    }

    #[test]
    fn test_large_list_performance() {
        // Test encoding/decoding a list with many items
        let mut stream = RlpStream::new_list(10_000);
        for i in 0..10_000 {
            stream.append(&format!("item_{:06}", i));
        }
        
        let start = Instant::now();
        let encoded = stream.out();
        let encode_time = start.elapsed();
        
        let start = Instant::now();
        let rlp = Rlp::new(&encoded);
        let item_count = rlp.item_count().unwrap();
        let decode_time = start.elapsed();
        
        assert_eq!(item_count, 10_000);
        println!("Large list (10k items) - Encode: {:?}, Parse: {:?}", encode_time, decode_time);
        
        // Should complete in reasonable time
        assert!(encode_time.as_millis() < 5000, "List encoding should be reasonably fast");
        assert!(decode_time.as_millis() < 1000, "List parsing should be fast");
    }

    #[test]
    fn test_repeated_operations() {
        // Test repeated encode/decode operations for consistency
        let test_data = vec!["short string", "medium length string for testing", "12345"];
        
        let start = Instant::now();
        
        for _ in 0..1000 {
            let mut stream = RlpStream::new_list(test_data.len());
            for item in &test_data {
                stream.append(item);
            }
            let encoded = stream.out();
            
            let rlp = Rlp::new(&encoded);
            assert_eq!(rlp.item_count().unwrap(), test_data.len());
            
            for (i, expected) in test_data.iter().enumerate() {
                let decoded: String = rlp.at(i).unwrap().as_val().unwrap();
                assert_eq!(&decoded, expected);
            }
        }
        
        let total_time = start.elapsed();
        println!("1000 encode/decode cycles: {:?}", total_time);
        
        // Should complete in reasonable time
        assert!(total_time.as_millis() < 1000, "Repeated operations should be fast");
    }
}

/// Test compatibility with Ethereum transaction and block structures
#[cfg(test)]
mod ethereum_compatibility_tests {
    use super::*;

    #[test]
    fn test_ethereum_address_encoding() {
        // Test encoding Ethereum addresses (20 bytes)
        let address = hex::decode("742d35Cc6634C0532925a3b8D97C2e0D8b2D9C53").unwrap();
        let encoded = encode(&address);
        
        // Should encode as 20-byte string
        assert_eq!(encoded[0], 0x80 + 20, "Address should encode with 0x94 prefix");
        
        let decoded: Vec<u8> = decode(&encoded).unwrap();
        assert_eq!(decoded, address);
    }

    #[test]
    fn test_ethereum_hash_encoding() {
        // Test encoding Ethereum hashes (32 bytes)
        let hash = hex::decode("1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef").unwrap();
        let encoded = encode(&hash);
        
        // Should encode as 32-byte string
        assert_eq!(encoded[0], 0x80 + 32, "Hash should encode with 0xa0 prefix");
        
        let decoded: Vec<u8> = decode(&encoded).unwrap();
        assert_eq!(decoded, hash);
    }

    #[test]
    fn test_transaction_like_structure() {
        // Test encoding a structure similar to an Ethereum transaction
        let mut tx_stream = RlpStream::new_list(9);
        
        // nonce
        tx_stream.append(&12u64);
        
        // gasPrice
        tx_stream.append(&20_000_000_000u64);
        
        // gasLimit  
        tx_stream.append(&21_000u64);
        
        // to address
        let to_addr = hex::decode("742d35Cc6634C0532925a3b8D97C2e0D8b2D9C53").unwrap();
        tx_stream.append(&to_addr);
        
        // value
        tx_stream.append(&1_000_000_000_000_000_000u64); // 1 ETH in wei
        
        // data
        tx_stream.append(&Vec::<u8>::new()); // empty data
        
        // v, r, s (signature components)
        tx_stream.append(&27u8);
        let r = hex::decode("1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef").unwrap();
        let s = hex::decode("fedcba0987654321fedcba0987654321fedcba0987654321fedcba0987654321").unwrap();
        tx_stream.append(&r);
        tx_stream.append(&s);
        
        let encoded = tx_stream.out();
        
        // Should be able to parse back
        let rlp = Rlp::new(&encoded);
        assert!(rlp.is_list());
        assert_eq!(rlp.item_count().unwrap(), 9);
        
        // Verify individual fields
        let nonce: u64 = rlp.at(0).unwrap().as_val().unwrap();
        assert_eq!(nonce, 12);
        
        let to: Vec<u8> = rlp.at(3).unwrap().as_val().unwrap();
        assert_eq!(to, to_addr);
    }

    #[test]
    fn test_storage_proof_structure() {
        // Test encoding a structure similar to Ethereum storage proofs
        let mut proof_stream = RlpStream::new_list(3);
        
        // Storage key (32 bytes)
        let key = hex::decode("0000000000000000000000000000000000000000000000000000000000000001").unwrap();
        proof_stream.append(&key);
        
        // Storage value (32 bytes)
        let value = hex::decode("000000000000000000000000000000000000000000000000000000000000007b").unwrap();
        proof_stream.append(&value);
        
        // Merkle proof (array of hashes)
        let mut proof_nodes = RlpStream::new_list(3);
        for i in 0..3 {
            let node = hex::decode(format!("{:064x}", i)).unwrap();
            proof_nodes.append(&node);
        }
        proof_stream.append_raw(&proof_nodes.out(), 1);
        
        let encoded = proof_stream.out();
        
        // Should be able to parse back
        let rlp = Rlp::new(&encoded);
        assert!(rlp.is_list());
        assert_eq!(rlp.item_count().unwrap(), 3);
        
        let decoded_key: Vec<u8> = rlp.at(0).unwrap().as_val().unwrap();
        assert_eq!(decoded_key, key);
        
        let proof_list = rlp.at(2).unwrap();
        assert!(proof_list.is_list());
        assert_eq!(proof_list.item_count().unwrap(), 3);
    }
} 