//! Comprehensive ABI Type Support Example
//!
//! This example demonstrates the complete ABI type support available through
//! selective alloy imports in the traverse-valence crate.

#[cfg(any(feature = "lightweight-alloy", feature = "full-alloy"))]
use traverse_valence::abi::{AlloyAbiTypes, AbiValue, AbiType};

#[cfg(feature = "lightweight-alloy")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Comprehensive ABI Type Support Example ===");
    println!();

    // Test all basic ABI types
    println!("🔧 Testing Basic ABI Types:");
    
    let basic_types = vec![
        AbiValue::Bool(true),
        AbiValue::Uint8(255),
        AbiValue::Uint16(65535),
        AbiValue::Uint32(4294967295),
        AbiValue::Uint64(18446744073709551615),
        AbiValue::Uint128(340282366920938463463374607431768211455),
        AbiValue::Address("0x742d35Cc6634C0532925a3b8D97C2e0D8b2D9C".to_string()),
        AbiValue::Bytes(vec![1, 2, 3, 4, 5]),
        AbiValue::FixedBytes([42u8; 32]),
        AbiValue::String("Hello, ABI!".to_string()),
    ];

    for (i, value) in basic_types.iter().enumerate() {
        println!("  {}: {:?}", i + 1, value.abi_type());
        let encoded = AlloyAbiTypes::encode_abi_value(value)?;
        println!("     Encoded size: {} bytes", encoded.len());
        println!("     First 4 bytes: {:02x?}", &encoded[..4.min(encoded.len())]);
    }

    // Test complex types
    println!("\n🔧 Testing Complex ABI Types:");
    
    let array_value = AbiValue::Array(vec![
        AbiValue::Uint8(1),
        AbiValue::Uint8(2),
        AbiValue::Uint8(3),
    ]);
    println!("  Array: {:?}", array_value.abi_type());
    
    let tuple_value = AbiValue::Tuple(vec![
        AbiValue::Bool(true),
        AbiValue::Address("0x742d35Cc6634C0532925a3b8D97C2e0D8b2D9C".to_string()),
        AbiValue::Uint256([1u8; 32]),
    ]);
    println!("  Tuple: {:?}", tuple_value.abi_type());

    // Test function selector generation
    println!("\n🔧 Testing Function Selector Generation:");
    
    let signatures = vec![
        "transfer(address,uint256)",
        "approve(address,uint256)",
        "balanceOf(address)",
        "totalSupply()",
        "mint(address,uint256)",
        "burn(uint256)",
    ];

    for signature in signatures {
        let selector = AlloyAbiTypes::function_selector(signature);
        println!("  {}: 0x{:02x}{:02x}{:02x}{:02x}", 
                 signature, selector[0], selector[1], selector[2], selector[3]);
    }

    // Test function call encoding
    println!("\n🔧 Testing Function Call Encoding:");
    
    let transfer_params = vec![
        AbiValue::Address("0x742d35Cc6634C0532925a3b8D97C2e0D8b2D9C".to_string()),
        AbiValue::Uint256([0u8; 32]), // Amount (0 for example)
    ];
    
    let encoded_call = AlloyAbiTypes::encode_function_call(
        "transfer(address,uint256)", 
        &transfer_params
    )?;
    
    println!("  Function call encoded size: {} bytes", encoded_call.len());
    println!("  Function selector: 0x{:02x}{:02x}{:02x}{:02x}", 
             encoded_call[0], encoded_call[1], encoded_call[2], encoded_call[3]);

    // Test parsing utilities
    println!("\n🔧 Testing Parsing Utilities:");
    
    let address = AlloyAbiTypes::parse_address("0x742d35Cc6634C0532925a3b8D97C2e0D8b2D9C")?;
    println!("  Parsed address: {:?}", address);
    
    let u256_val = AlloyAbiTypes::parse_u256("12345")?;
    println!("  Parsed U256: {:?}", u256_val);
    
    let b256_val = AlloyAbiTypes::parse_b256("0x1234567890123456789012345678901234567890123456789012345678901234")?;
    println!("  Parsed B256: {:?}", b256_val);

    // Test availability check
    println!("\n🔧 Feature Availability:");
    println!("  Alloy features available: {}", AlloyAbiTypes::alloy_features_available());

    // Test all integer types
    println!("\n🔧 Testing All Integer Types:");
    
    let int_types = vec![
        AbiValue::Int8(-128),
        AbiValue::Int16(-32768),
        AbiValue::Int32(-2147483648),
        AbiValue::Int64(-9223372036854775808),
        AbiValue::Int128(-170141183460469231731687303715884105728),
    ];

    for (i, value) in int_types.iter().enumerate() {
        println!("  Signed int {}: {:?}", i + 1, value.abi_type());
        let encoded = AlloyAbiTypes::encode_abi_value(value)?;
        println!("     Encoded size: {} bytes", encoded.len());
    }

    // Test comprehensive encoding comparison
    println!("\n🔧 Testing Comprehensive Encoding vs Fallback:");
    
    let test_value = AbiValue::Uint256([255u8; 32]);
    
    // Encode with alloy
    let alloy_encoded = AlloyAbiTypes::encode_abi_value(&test_value)?;
    println!("  Alloy encoding size: {} bytes", alloy_encoded.len());
    
    // Encode with fallback method
    let fallback_encoded = test_value.encode()?;
    println!("  Fallback encoding size: {} bytes", fallback_encoded.len());
    
    if alloy_encoded == fallback_encoded {
        println!("  Alloy and fallback encodings match!");
    } else {
        println!("  Alloy and fallback encodings differ");
    }

    println!("\nAll comprehensive ABI type tests completed successfully!");
    
    Ok(())
}

#[cfg(not(feature = "lightweight-alloy"))]
fn main() {
    println!("This example requires the 'lightweight-alloy' feature to be enabled.");
    println!("Run with: cargo run --example comprehensive_abi_example --features lightweight-alloy");
} 