//! Constrained Environment Example
//!
//! This example demonstrates how to use traverse in constrained environments
//! such as embedded systems, WASM runtimes, and ZK circuits where standard
//! library is not available and memory is limited.
//!
//! Run with: cargo run --example constrained_environment_example --features constrained --no-default-features

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;
use alloc::{vec, vec::Vec};

#[cfg(feature = "constrained")]
use traverse_core::{
    ConstrainedLayoutInfo, ConstrainedStorageEntry, ConstrainedFieldType,
    ConstrainedKeyResolver,
};

#[cfg(feature = "constrained")]
use traverse_valence::{
    CircuitProcessor, CircuitWitness, CircuitResult,
    FieldType, ZeroSemantics,
};

/// Simulate a main function for no_std environments
#[cfg(not(feature = "std"))]
#[no_mangle]
pub extern "C" fn main() {
    constrained_example_main();
}

/// Regular main function for std environments
#[cfg(feature = "std")]
fn main() {
    constrained_example_main();
}

fn constrained_example_main() {
    #[cfg(feature = "std")]
    {
        println!("Constrained Environment Example");
        println!("==============================");
        println!();
    }

    #[cfg(feature = "constrained")]
    {
        // Example 1: Constrained Layout Info
        #[cfg(feature = "std")]
        {
            println!("Example 1: Constrained Layout Info");
            println!("----------------------------------");
        }
        
        // Note: ConstrainedMemoryPool is an internal type not exposed in the public API
        // This example shows how to work with constrained layout information instead
        
        let layout_info = create_test_layout();
        #[cfg(feature = "std")]
        {
            println!("✓ Created constrained layout with {} entries", layout_info.entry_count);
            println!("✓ Layout commitment: {:?}", &layout_info.commitment[0..4]);
            println!();
        }

        // Example 2: Constrained Layout Creation
        #[cfg(feature = "std")]
        {
            println!("Example 2: Constrained Layout Creation");
            println!("--------------------------------------");
        }
        
        let constrained_layout = create_test_layout();
        #[cfg(feature = "std")]
        {
            println!("✓ Created constrained layout with {} storage entries", 
                    constrained_layout.entry_count);
            println!("Layout commitment: 0x{}", hex::encode(&constrained_layout.commitment));
            
            for (i, entry) in constrained_layout.storage.iter().enumerate() {
                println!("  Entry {}: {:?} (size: {} bytes, semantics: {:?})", 
                        i, entry.field_type, entry.size, entry.zero_semantics);
            }
            println!();
        }

        // Example 3: Constrained Key Resolution
        #[cfg(feature = "std")]
        {
            println!("Example 3: Constrained Key Resolution");
            println!("-------------------------------------");
        }
        
        let mut resolver = ConstrainedKeyResolver::new();
        
        // Resolve storage keys for simple fields
        for field_index in 0..constrained_layout.entry_count {
            match resolver.resolve_constrained(&constrained_layout, field_index) {
                Ok(key) => {
                    #[cfg(feature = "std")]
                    println!("  Field {}: 0x{}", field_index, hex::encode(&key[..4]));
                }
                Err(e) => {
                    #[cfg(feature = "std")]
                    println!("  Field {}: Error - {:?}", field_index, e);
                }
            }
        }
        
        if let Some(usage) = resolver.memory_usage() {
            #[cfg(feature = "std")]
            println!("Resolver memory usage: {} bytes", usage.used);
        }
        #[cfg(feature = "std")]
        println!();

        // Example 4: Circuit Processing (using minimal circuit)
        #[cfg(feature = "std")]
        {
            println!("Example 4: Circuit Processing");
            println!("-----------------------------");
        }
        
        // Create processor with field types and semantics
        let field_types = vec![
            FieldType::Uint256,
            FieldType::Address,
            FieldType::Bool,
        ];
        
        let field_semantics = vec![
            ZeroSemantics::ValidZero,
            ZeroSemantics::NeverWritten,
            ZeroSemantics::ValidZero,
        ];
        
        let processor = CircuitProcessor::new(
            constrained_layout.commitment,
            field_types,
            field_semantics
        );
        
        // Create test witnesses
        let witnesses = create_test_witnesses();
        #[cfg(feature = "std")]
        println!("✓ Created {} test witnesses", witnesses.len());
        
        // Process witnesses
        let results = processor.process_batch(&witnesses);
        
        // Display results
        for (i, result) in results.iter().enumerate() {
            match result {
                CircuitResult::Valid { field_index, extracted_value: _ } => {
                    #[cfg(feature = "std")]
                    println!("  Witness {}: VALID (field {})", i, field_index);
                }
                CircuitResult::Invalid => {
                    #[cfg(feature = "std")]
                    println!("  Witness {}: INVALID", i);
                }
            }
        }
        
        #[cfg(feature = "std")]
        println!();

        // Example 5: Memory Constrained Validation
        #[cfg(feature = "std")]
        {
            println!("Example 5: Memory Constrained Validation");
            println!("----------------------------------------");
        }
        
        // Test with different memory limits
        let memory_limits = vec![100, 500, 1000, 5000];
        
        for limit in memory_limits {
            match validate_with_memory_limit(&witnesses, limit) {
                Ok(()) => {
                    #[cfg(feature = "std")]
                    println!("✓ Validation passed with {} byte memory limit", limit);
                }
                Err(e) => {
                    #[cfg(feature = "std")]
                    println!("✗ Validation failed with {} byte memory limit: {:?}", limit, e);
                }
            }
        }
        #[cfg(feature = "std")]
        println!();

        // Example 6: Stack-based Operations (no heap allocation)
        #[cfg(feature = "std")]
        {
            println!("Example 6: Stack-based Operations");
            println!("---------------------------------");
        }
        
        demonstrate_stack_operations();
        #[cfg(feature = "std")]
        println!();

        // Example 7: Error Handling in Constrained Environments
        #[cfg(feature = "std")]
        {
            println!("Example 7: Error Handling");
            println!("-------------------------");
        }
        
        demonstrate_error_handling();
        #[cfg(feature = "std")]
        println!();
    }

    #[cfg(not(feature = "constrained"))]
    {
        #[cfg(feature = "std")]
        {
            println!("Note: This example requires the 'constrained' feature to be enabled");
            println!("Run with: cargo run --example constrained_environment_example --features constrained --no-default-features");
        }
    }

    #[cfg(feature = "std")]
    {
        println!("Constrained Environment Benefits:");
        println!("• Predictable memory usage with memory pools");
        println!("• Compact data structures optimized for size");
        println!("• Stack-based operations where possible");
        println!("• Graceful degradation under memory pressure");
        println!("• no_std compatibility for embedded systems");
        println!("• WASM-friendly with minimal dependencies");
    }
}

#[cfg(feature = "constrained")]
fn create_test_layout() -> ConstrainedLayoutInfo {
    ConstrainedLayoutInfo {
        storage: vec![
            ConstrainedStorageEntry {
                slot: [0; 32],
                offset: 0,
                size: 32,
                field_type: ConstrainedFieldType::Uint256,
                zero_semantics: traverse_core::ZeroSemantics::ExplicitlyZero,
            },
            ConstrainedStorageEntry {
                slot: [1; 32],
                offset: 0,
                size: 20,
                field_type: ConstrainedFieldType::Address,
                zero_semantics: traverse_core::ZeroSemantics::NeverWritten,
            },
            ConstrainedStorageEntry {
                slot: [2; 32],
                offset: 0,
                size: 1,
                field_type: ConstrainedFieldType::Bool,
                zero_semantics: traverse_core::ZeroSemantics::ValidZero,
            },
            ConstrainedStorageEntry {
                slot: [3; 32],
                offset: 0,
                size: 8,
                field_type: ConstrainedFieldType::Uint64,
                zero_semantics: traverse_core::ZeroSemantics::ValidZero,
            },
        ],
        commitment: [0x42; 32], // Test commitment
        entry_count: 4,
    }
}

#[cfg(feature = "constrained")]
fn create_test_witnesses() -> Vec<CircuitWitness> {
    vec![
        // Uint256 field with value 1000
        create_witness_with_u256(1000, 0, ZeroSemantics::ExplicitlyZero),
        
        // Address field  
        create_witness_with_address([0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0, 0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0, 0x12, 0x34, 0x56, 0x78], 1, ZeroSemantics::NeverWritten),
        
        // Bool field (true)
        create_witness_with_bool(true, 2, ZeroSemantics::ValidZero),
        
        // Uint64 field with value 500
        create_witness_with_u64(500, 3, ZeroSemantics::ValidZero),
    ]
}

#[cfg(feature = "constrained")]
fn create_witness_with_value(value: [u8; 32], field_index: u16, semantics: ZeroSemantics) -> CircuitWitness {
    CircuitWitness {
        key: [field_index as u8; 32], // Simple key based on field index
        value,
        proof: vec![1, 2, 3, 4], // Minimal proof data
        layout_commitment: [0x42; 32], // Test commitment
        field_index,
        semantics,
        expected_slot: [field_index as u8; 32],
        block_height: 1000,
        block_hash: [0x99u8; 32],
    }
}

#[cfg(feature = "constrained")]
fn create_witness_with_address(addr: [u8; 20], field_index: u16, semantics: ZeroSemantics) -> CircuitWitness {
    let mut value = [0u8; 32];
    value[12..32].copy_from_slice(&addr);
    create_witness_with_value(value, field_index, semantics)
}

#[cfg(feature = "constrained")]
fn create_witness_with_bool(b: bool, field_index: u16, semantics: ZeroSemantics) -> CircuitWitness {
    let mut value = [0u8; 32];
    value[31] = if b { 1 } else { 0 };
    create_witness_with_value(value, field_index, semantics)
}

#[cfg(feature = "constrained")]
fn create_witness_with_u64(val: u64, field_index: u16, semantics: ZeroSemantics) -> CircuitWitness {
    let mut value = [0u8; 32];
    value[24..32].copy_from_slice(&val.to_be_bytes());
    create_witness_with_value(value, field_index, semantics)
}

#[cfg(feature = "constrained")]
fn create_witness_with_u256(val: u64, field_index: u16, semantics: ZeroSemantics) -> CircuitWitness {
    let mut value = [0u8; 32];
    value[24..32].copy_from_slice(&val.to_be_bytes());
    create_witness_with_value(value, field_index, semantics)
}

#[cfg(feature = "constrained")]
fn validate_with_memory_limit(witnesses: &[CircuitWitness], memory_limit: usize) -> Result<(), String> {
    let mut total_memory = 0;
    
    for witness in witnesses {
        // Estimate memory: 32 + 32 + proof.len() + 32 + 2 + 1 + 32 + 8 + 32
        total_memory += 32 + 32 + witness.proof.len() + 32 + 2 + 1 + 32 + 8 + 32;
        if total_memory > memory_limit {
            return Err("Out of memory".to_string());
        }
    }
    
    Ok(())
}

#[cfg(feature = "constrained")]
fn demonstrate_stack_operations() {
    // Example of stack-based hex conversion without allocation
    let bytes = [0x12, 0x34, 0xab, 0xcd];
    let mut hex_output = [0u8; 8];
    
    match traverse_core::constrained::utils::bytes_to_hex_stack(&bytes, &mut hex_output) {
        Ok(len) => {
            #[cfg(feature = "std")]
            {
                println!("✓ Stack-based hex conversion: {} bytes", len);
                println!("  Input: {:?}", bytes);
                println!("  Output: {:?}", core::str::from_utf8(&hex_output).unwrap_or("invalid"));
            }
        }
        Err(_) => {
            #[cfg(feature = "std")]
            println!("✗ Stack-based hex conversion failed");
        }
    }
    
    // Example of stack-based hex parsing
    let mut bytes_output = [0u8; 4];
    match traverse_core::constrained::utils::hex_to_bytes_stack("1234abcd", &mut bytes_output) {
        Ok(len) => {
            #[cfg(feature = "std")]
            {
                println!("✓ Stack-based hex parsing: {} bytes", len);
                println!("  Result: {:?}", bytes_output);
            }
        }
        Err(_) => {
            #[cfg(feature = "std")]
            println!("✗ Stack-based hex parsing failed");
        }
    }
}

#[cfg(feature = "constrained")]
fn demonstrate_error_handling() {
    // Test various error conditions
    let test_cases: Vec<(&str, Box<dyn Fn() -> Result<(), String>>)> = vec![
        ("Memory exhaustion", Box::new(|| -> Result<(), String> {
            let witnesses = create_test_witnesses();
            validate_with_memory_limit(&witnesses, 10) // Very small limit
        })),
        ("Invalid witness", Box::new(|| -> Result<(), String> {
            let mut witness = create_test_witnesses()[0].clone();
            witness.proof = vec![0u8; 2048]; // Large proof data
            // In a real scenario, validation would happen in the circuit processor
            Ok(())
        })),
        ("Field index out of bounds", Box::new(|| -> Result<(), String> {
            let layout = create_test_layout();
            let mut resolver = ConstrainedKeyResolver::new();
            resolver.resolve_constrained(&layout, 999)
                .map_err(|e| match e {
                    traverse_core::TraverseError::InvalidInput(_) => "Field index out of bounds".to_string(),
                    _ => "Invalid witness".to_string(),
                })?;
            Ok(())
        })),
    ];
    
    for (name, test) in test_cases {
        match test() {
            Ok(_) => {
                #[cfg(feature = "std")]
                println!("✗ Expected error for '{}' but got success", name);
            }
            Err(e) => {
                #[cfg(feature = "std")]
                println!("✓ Expected error for '{}': {:?}", name, e);
            }
        }
    }
}

// Note: println! macro is conditionally compiled
// In std environments, it uses std::println!
// In no_std environments, you would need to implement your own
// output mechanism (UART, RTT, etc.) 