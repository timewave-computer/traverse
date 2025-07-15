# Valence Integration Guide

This guide shows how to integrate Traverse with Valence coprocessor applications for ZK storage proofs.

## Overview

Traverse provides ZK-compatible storage proof generation that can be used in Valence coprocessor circuits. The integration involves:

1. **Controller**: Generates witnesses from storage proofs
2. **Circuit**: Processes witnesses to verify storage state
3. **Domain**: Optional validation of results

## Quick Start

### 1. Add Dependencies

Add to your valence app's `controller/Cargo.toml`:

```toml
[dependencies]
traverse-valence = { version = "0.1", default-features = false, features = ["controller"] }
valence-coprocessor = { git = "https://github.com/timewave-computer/valence-coprocessor.git", tag = "v0.1.13" }
```

Add to your valence app's `circuit/Cargo.toml`:

```toml
[dependencies]
traverse-valence = { version = "0.1", default-features = false, features = ["circuit", "constrained"] }
valence-coprocessor = { git = "https://github.com/timewave-computer/valence-coprocessor.git", tag = "v0.1.13" }
```

### 2. Controller Implementation

In your `controller/src/lib.rs`:

```rust
use traverse_valence::{StorageVerificationRequest, create_witness_from_request};
use valence_coprocessor::Witness;

pub fn get_witnesses(request: StorageVerificationRequest) -> anyhow::Result<Vec<Witness>> {
    let witness = create_witness_from_request(&request)?;
    Ok(vec![witness])
}
```

### 3. Circuit Implementation

In your `circuit/src/lib.rs`:

```rust
use traverse_valence::circuit::{CircuitProcessor, CircuitWitness};
use valence_coprocessor::Witness;

pub fn circuit(witnesses: Vec<Witness>) -> Vec<u8> {
    // Create processor with your layout parameters
    let processor = CircuitProcessor::new(
        layout_commitment,
        field_types,
        field_semantics,
    );
    
    // Parse witnesses
    let circuit_witnesses: Vec<CircuitWitness> = witnesses
        .into_iter()
        .map(|w| CircuitProcessor::parse_witness_from_bytes(
            w.as_data().expect("Expected witness data")
        ))
        .collect::<Result<Vec<_>, _>>()
        .expect("Failed to parse witnesses");
    
    // Process and return results
    let results = processor.process_batch(&circuit_witnesses);
    results.abi_encode() // Or your custom encoding
}
```

## Storage Verification Request

The `StorageVerificationRequest` contains all information needed to generate a witness:

```rust
pub struct StorageVerificationRequest {
    pub storage_query: StorageQuery,
    pub storage_proof: StorageProof,
    pub block_header: BlockHeader,
}

pub struct StorageQuery {
    pub layout_commitment: String,
    pub query: String,
    pub parameters: serde_json::Value,
}
```

## Example: ERC20 Balance Verification

### Controller

```rust
use traverse_valence::{StorageVerificationRequest, create_witness_from_request};
use valence_coprocessor::Witness;

pub fn get_witnesses(request: StorageVerificationRequest) -> anyhow::Result<Vec<Witness>> {
    // Validate the request is for ERC20 balance
    if request.storage_query.query != "balanceOf" {
        return Err(anyhow::anyhow!("Only balanceOf queries supported"));
    }
    
    // Generate witness
    let witness = create_witness_from_request(&request)?;
    Ok(vec![witness])
}
```

### Circuit

```rust
use traverse_valence::circuit::{CircuitProcessor, CircuitWitness, FieldType, ZeroSemantics};
use valence_coprocessor::Witness;
use alloy_sol_types::{sol, SolValue};

// Define output structure
sol! {
    struct BalanceOutput {
        address token;
        address holder;
        uint256 balance;
        bool verified;
    }
}

pub fn circuit(witnesses: Vec<Witness>) -> Vec<u8> {
    // ERC20 balance field configuration
    let field_types = vec![
        FieldType::Address,  // mapping key (holder address)
        FieldType::Uint256,  // balance value
    ];
    
    let field_semantics = vec![
        ZeroSemantics::NeverWritten,  // address keys
        ZeroSemantics::ValidZero,     // balance can be zero
    ];
    
    let processor = CircuitProcessor::new(
        [0u8; 32], // Your layout commitment
        field_types,
        field_semantics,
    );
    
    // Process witness
    let circuit_witness = CircuitProcessor::parse_witness_from_bytes(
        witnesses[0].as_data().expect("Expected witness data")
    ).expect("Failed to parse witness");
    
    let results = processor.process(&circuit_witness);
    
    // Create output
    let output = BalanceOutput {
        token: /* extract from witness */,
        holder: /* extract from witness */,
        balance: /* extract from results */,
        verified: true,
    };
    
    output.abi_encode()
}
```

## Feature Flags

### Controller Features

- `controller` - Basic controller functionality (no-std compatible)
- `ethereum` - Ethereum-specific support
- `cosmos` - Cosmos-specific support

### Circuit Features

- `circuit` - Basic circuit functionality  
- `constrained` - Memory-optimized for ZK circuits
- `no-std` - No standard library (recommended for circuits)

## Best Practices

### 1. Layout Management

Store your layout commitments and field configurations:

```rust
pub mod layouts {
    use traverse_valence::circuit::{FieldType, ZeroSemantics};
    
    pub const ERC20_COMMITMENT: [u8; 32] = [/* ... */];
    
    pub const ERC20_BALANCE_TYPES: &[FieldType] = &[
        FieldType::Address,
        FieldType::Uint256,
    ];
    
    pub const ERC20_BALANCE_SEMANTICS: &[ZeroSemantics] = &[
        ZeroSemantics::NeverWritten,
        ZeroSemantics::ValidZero,
    ];
}
```

### 2. Error Handling

Always validate inputs in the controller:

```rust
pub fn get_witnesses(request: StorageVerificationRequest) -> anyhow::Result<Vec<Witness>> {
    // Validate layout commitment
    if request.storage_query.layout_commitment != expected_commitment {
        return Err(anyhow::anyhow!("Invalid layout commitment"));
    }
    
    // Validate query type
    if !SUPPORTED_QUERIES.contains(&request.storage_query.query.as_str()) {
        return Err(anyhow::anyhow!("Unsupported query type"));
    }
    
    create_witness_from_request(&request)
}
```

### 3. Circuit Optimization

For memory-constrained environments:

```rust
#![no_std]
extern crate alloc;

use alloc::vec::Vec;
use traverse_valence::circuit::{CircuitProcessor, CircuitWitness};

// Use fixed-size arrays where possible
const MAX_WITNESSES: usize = 10;

pub fn circuit(witnesses: Vec<Witness>) -> Vec<u8> {
    assert!(witnesses.len() <= MAX_WITNESSES, "Too many witnesses");
    
    // Process with constrained memory
    // ...
}
```

## Advanced Usage

### Custom Field Types

Implement custom processing for domain-specific types:

```rust
use traverse_valence::circuit::{FieldProcessor, FieldType};

pub struct CustomFieldProcessor;

impl FieldProcessor for CustomFieldProcessor {
    fn process_field(&self, field_type: &FieldType, data: &[u8]) -> Vec<u8> {
        match field_type {
            FieldType::Custom(name) if name == "MyCustomType" => {
                // Custom processing logic
            }
            _ => {
                // Default processing
            }
        }
    }
}
```

### Multi-Chain Support

Handle different blockchain types:

```rust
use traverse_valence::{StorageVerificationRequest, ChainType};

pub fn get_witnesses(request: StorageVerificationRequest) -> anyhow::Result<Vec<Witness>> {
    match request.block_header.chain_type() {
        ChainType::Ethereum => handle_ethereum_request(request),
        ChainType::Cosmos => handle_cosmos_request(request),
        _ => Err(anyhow::anyhow!("Unsupported chain type")),
    }
}
```

## Future Features

The following features are planned for future releases:

### Code Generation (Future Feature)

Generate minimal, optimized crates for specific layouts:

```bash
# Generate minimal controller and circuit crates
traverse-cli-ethereum codegen app \
  --layout my-layout.json \
  --output ./my-valence-app \
  --name my-storage-app
```

This will generate custom crates with:
- Hardcoded layout commitments
- Only required field types
- Optimized processing logic
- Minimal dependencies

### Domain Integration (Future Feature)

Automatic domain validation:

```rust
use traverse_valence::domain::DomainValidator;

pub fn validate_results(results: Vec<u8>) -> Result<(), String> {
    let validator = DomainValidator::new(expected_schema);
    validator.validate(&results)
}
```

## Troubleshooting

### Common Issues

1. **"Layout mismatch" errors**
   - Ensure layout commitment matches between controller and circuit
   - Verify the storage layout was compiled correctly

2. **"Invalid witness data" errors**
   - Check that witness format matches expected structure
   - Ensure proper serialization/deserialization

3. **Memory constraints in circuits**
   - Use `constrained` feature flag
   - Minimize allocations
   - Use fixed-size data structures

### Getting Help

- Check the [examples](../examples/valence/) directory
- Review test cases in `crates/traverse-valence/src/tests/`
- Open an issue with your use case

## Additional Resources

- [Valence Coprocessor Documentation](https://github.com/timewave-computer/valence-coprocessor)
- [Semantic Storage Proofs](semantic_storage_proofs.md)
- [Architecture Overview](architecture.md)