# Valence Integration Guide: Minimal Traverse Applications

This guide shows how 3rd party valence application developers can generate minimal, optimized libraries containing only the specific functionality needed for their storage queries.

## Overview

Instead of importing the full traverse-valence crate, you can generate custom, minimal crates that contain only the code needed for your specific contract layouts and queries. This results in:

- **Smaller binary sizes** - Only code for your specific fields is included
- **Faster compilation** - No unused dependencies or features
- **Cleaner APIs** - Generated code is tailored to your exact use case
- **Better performance** - Optimized for your specific data structures

## Quick Start

### 1. Generate Layout from Your Contract

First, compile your contract's storage layout:

```bash
# For Ethereum contracts
traverse-cli ethereum compile-layout my-contract.abi.json --output my-layout.json

# For CosmWasm contracts  
traverse-cli cosmos compile-layout my-contract-msgs.json --output my-layout.json
```

### 2. Generate Minimal Applications

#### Option A: Generate Individual Crates

```bash
# Generate a minimal controller crate
traverse-cli codegen controller \
  --layout my-layout.json \
  --output ./my-controller \
  --name my-storage-controller \
  --include-alloy

# Generate a minimal circuit crate  
traverse-cli codegen circuit \
  --layout my-layout.json \
  --output ./my-circuit \
  --name my-storage-circuit \
  --include-alloy
```

#### Option B: Generate Complete Application

```bash
# Generate both controller and circuit as a workspace
traverse-cli codegen app \
  --layout my-layout.json \
  --output ./my-valence-app \
  --name my-storage-app \
  --include-alloy \
  --minimal
```

### 3. Integration Examples

#### Controller Integration

Add to your valence app's `controller/Cargo.toml`:

```toml
[dependencies]
my-storage-controller = { path = "../generated/my-controller" }
valence-coprocessor = { git = "https://github.com/timewave-computer/valence-coprocessor.git", tag = "v0.1.13" }
```

Use in your `controller/src/lib.rs`:

```rust
use my_storage_controller::create_witness;
use traverse_valence::StorageVerificationRequest;
use valence_coprocessor::Witness;

pub fn get_witnesses(request: StorageVerificationRequest) -> anyhow::Result<Vec<Witness>> {
    let witness = create_witness(&request)?;
    Ok(vec![witness])
}
```

#### Circuit Integration

Add to your valence app's `circuit/Cargo.toml`:

```toml
[dependencies]
my-storage-circuit = { path = "../generated/my-circuit" }
valence-coprocessor = { git = "https://github.com/timewave-computer/valence-coprocessor.git", tag = "v0.1.13" }
```

Use in your `circuit/src/lib.rs`:

```rust
use my_storage_circuit::circuit;
use valence_coprocessor::Witness;

pub fn circuit(witnesses: Vec<Witness>) -> Vec<u8> {
    my_storage_circuit::circuit(witnesses)
}
```

## Feature Comparison

### Traditional Approach (Full Import)

```toml
[dependencies]
traverse-valence = { version = "0.1", features = ["controller", "circuit", "ethereum", "cosmos", "alloy"] }
```

**Result**: Large binary with all blockchain support, unused field types, and generic processing.

### Modular Approach (Generated Crates)

```toml
[dependencies]
my-storage-controller = { path = "../generated/my-controller" }
```

**Result**: Minimal binary with only your specific field types, hardcoded layout commitment, and optimized processing.

## Code Generation Options

### Controller Options

| Flag | Description | Use Case |
|------|-------------|----------|
| `--include-alloy` | Include Alloy ABI encoding support | When you need ABI-compatible output |

**Note**: All controllers are generated as `no_std` builds for maximum compatibility across environments.

### Circuit Options

| Flag | Description | Use Case |
|------|-------------|----------|
| `--include-alloy` | Include Alloy ABI encoding support | When circuit needs to generate ABI output |

**Note**: All circuits are generated as minimal/constrained builds optimized for ZK environments.

### App Options

| Flag | Description | Use Case |
|------|-------------|----------|
| `--include-alloy` | Both crates include Alloy support | Full ABI compatibility in both components |

**Note**: 
- Controllers are always generated as `no_std` builds for maximum compatibility
- Circuits are always generated as minimal/constrained builds optimized for ZK environments

## Example: ERC20 Token Balance Verification

### 1. Start with ERC20 Layout

```bash
# Generate layout from ERC20 ABI
traverse-cli ethereum compile-layout erc20.abi.json --output erc20-layout.json
```

### 2. Generate Minimal Application

```bash
traverse-cli codegen app \
  --layout erc20-layout.json \
  --output ./token-balance-app \
  --name token-balance \
  --include-alloy \
  --description "Minimal ERC20 balance verification for valence"
```

### 3. Generated Structure

```
token-balance-app/
├── Cargo.toml                    # Workspace configuration
├── README.md                     # Usage instructions
├── crates/
│   ├── controller/
│   │   ├── Cargo.toml           # Controller dependencies
│   │   └── src/
│   │       └── lib.rs           # Controller implementation
│   └── circuit/
│       ├── Cargo.toml           # Circuit dependencies  
│       └── src/
│           └── lib.rs           # Circuit implementation
```

### 4. Generated Controller Code

The generated controller (`crates/controller/src/lib.rs`) contains:

```rust
//! Generated controller for token-balance
//! Layout commitment: 0xabc123...

#![no_std]

extern crate alloc;
use alloc::vec::Vec;

use valence_coprocessor::Witness;
use traverse_valence::{StorageVerificationRequest, create_witness_from_request};

/// Layout commitment for this controller (validates against expected layout)
pub const LAYOUT_COMMITMENT: &str = "0xabc123...";

/// Supported storage queries
pub const SUPPORTED_QUERIES: &[&str] = &[
    "totalSupply",
    "balanceOf",
    "allowance",
    "name",
    "symbol",
    "decimals",
];

pub fn create_witness(request: &StorageVerificationRequest) -> Result<Witness, traverse_valence::TraverseValenceError> {
    // Validate layout commitment matches
    if request.storage_query.layout_commitment != LAYOUT_COMMITMENT {
        return Err(traverse_valence::TraverseValenceError::LayoutMismatch(
            format!("Expected: {}, Got: {}", LAYOUT_COMMITMENT, request.storage_query.layout_commitment)
        ));
    }
    
    // Validate query is supported
    let query_supported = SUPPORTED_QUERIES.iter().any(|&q| q == request.storage_query.query);
    if !query_supported {
        return Err(traverse_valence::TraverseValenceError::InvalidWitness(
            format!("Unsupported query: {}", request.storage_query.query)
        ));
    }
    
    create_witness_from_request(request)
}
```

### 5. Generated Circuit Code

The generated circuit (`crates/circuit/src/lib.rs`) contains:

```rust
//! Generated circuit for token-balance
//! Layout commitment: 0xabc123...

#![no_std]

extern crate alloc;
use alloc::vec::Vec;

use valence_coprocessor::Witness;
use traverse_valence::circuit::{CircuitProcessor, CircuitWitness, FieldType, ZeroSemantics};
use alloy_primitives::{Address, U256};
use alloy_sol_types::{sol, SolValue};

/// Layout commitment for this circuit
pub const LAYOUT_COMMITMENT: [u8; 32] = [/* parsed from layout */];

/// Field types for ERC20 layout
pub const FIELD_TYPES: &[FieldType] = &[
    FieldType::Uint256,  // totalSupply
    FieldType::Address,  // owner (mapping key)
    FieldType::Uint256,  // balanceOf value
    // ... etc for all fields
];

/// Field semantics for ERC20 layout
pub const FIELD_SEMANTICS: &[ZeroSemantics] = &[
    ZeroSemantics::ExplicitlyZero,  // totalSupply can be explicitly zero
    ZeroSemantics::NeverWritten,    // address keys default to never written
    ZeroSemantics::ValidZero,       // balance values can legitimately be zero
    // ... etc
];

// ABI output structure specific to this contract
sol! {
    struct TokenBalanceOutput {
        uint256 totalSupply;
        uint256 userBalance;
        address tokenAddress;
        bool isValidBalance;
    }
}

pub fn circuit(witnesses: Vec<Witness>) -> Vec<u8> {
    // Create processor with layout-specific parameters
    let processor = CircuitProcessor::new(
        LAYOUT_COMMITMENT,
        FIELD_TYPES.to_vec(),
        FIELD_SEMANTICS.to_vec(),
    );
    
    // Process witnesses with validation
    let circuit_witnesses: Vec<CircuitWitness> = witnesses
        .into_iter()
        .map(|w| CircuitProcessor::parse_witness_from_bytes(w.as_data().expect("Expected witness data")))
        .collect::<Result<Vec<_>, _>>()
        .expect("Failed to parse witnesses");
    
    let results = processor.process_batch(&circuit_witnesses);
    
    // Generate ABI-encoded output specific to this contract
    generate_token_balance_output(&results)
}

fn generate_token_balance_output(results: &[CircuitResult]) -> Vec<u8> {
    // Extract validated values and create typed output
    let output = TokenBalanceOutput {
        totalSupply: /* extract from results[0] */,
        userBalance: /* extract from results[2] */,
        tokenAddress: /* extract from results[1] */,
        isValidBalance: true, // All validations passed
    };
    
    output.abi_encode()
}
```

## Benefits for 3rd Party Developers

### 1. Minimal Binary Size

**Before** (full traverse-valence import):
- ✗ All blockchain support (Ethereum + Cosmos)
- ✗ All field types (even unused ones)  
- ✗ Generic processing logic
- ✗ All feature flags and optional dependencies

**After** (generated crate):
- ✅ Only your specific blockchain  
- ✅ Only your specific field types
- ✅ Hardcoded layout commitment
- ✅ Optimized processing for your exact use case

### 2. Compile-Time Guarantees

Generated code provides compile-time guarantees that your application:
- Only accepts witnesses matching your exact layout commitment
- Only processes queries you've explicitly defined
- Uses the correct field types and semantics for your contract
- Generates correctly-typed ABI output

### 3. No Runtime Configuration

Traditional approach requires runtime layout validation:

```rust
// Runtime validation needed
let processor = CircuitProcessor::new(layout_from_config, types_from_config, semantics_from_config);
```

Generated approach has everything compile-time validated:

```rust
// Everything is compile-time validated
let processor = CircuitProcessor::new(LAYOUT_COMMITMENT, FIELD_TYPES, FIELD_SEMANTICS);
```

### 4. Optimized Dependencies

Generated crates only include the exact dependencies needed:

```toml
# Generated controller - minimal dependencies
[dependencies]
valence-coprocessor = { git = "...", default-features = false }
traverse-valence = { path = "...", default-features = false, features = ["controller"] }
serde = { version = "1.0", default-features = false, features = ["alloc"] }

# vs full import - many unused dependencies
[dependencies]
traverse-valence = { version = "0.1", features = ["controller", "circuit", "ethereum", "cosmos", "alloy"] }
# ... many transitive dependencies for unused features
```

## Advanced Usage

### Custom Field Types

If your contract uses custom types, you can extend the generated code:

```rust
// In your generated circuit
impl CustomFieldProcessor for MyCircuit {
    fn process_custom_field(&self, data: &[u8]) -> CustomResult {
        // Your custom processing logic
    }
}
```

### Multiple Contracts

Generate separate applications for different contracts:

```bash
# Generate for DeFi protocol
traverse-cli codegen app --layout defi-vault.json --output ./defi-app --name defi-vault

# Generate for governance
traverse-cli codegen app --layout governance.json --output ./governance-app --name governance  

# Generate for token factory
traverse-cli codegen app --layout factory.json --output ./factory-app --name token-factory
```

### Workspace Integration

Create a master workspace for multiple generated applications:

```toml
# Cargo.toml
[workspace]
members = [
    "generated/defi-app/crates/controller",
    "generated/defi-app/crates/circuit", 
    "generated/governance-app/crates/controller",
    "generated/governance-app/crates/circuit",
    "generated/factory-app/crates/controller",
    "generated/factory-app/crates/circuit",
]
```

## Best Practices

### 1. Version Management

Pin generated crates to specific layout commitments:

```rust
// Generated code includes layout validation
pub const LAYOUT_COMMITMENT: &str = "0xabc123...";
```

When your contract is upgraded, regenerate your crates with the new layout.

### 2. Testing

Generated crates include basic tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_layout_commitment() {
        assert_eq!(LAYOUT_COMMITMENT, "0xabc123...");
    }
    
    #[test] 
    fn test_supported_queries() {
        assert!(SUPPORTED_QUERIES.contains(&"balanceOf"));
    }
}
```

Add your own integration tests that verify the generated code works with your specific use cases.

### 3. Documentation

Generated crates include comprehensive documentation:

```rust
//! Generated controller for my-token
//!
//! This controller handles 6 storage queries for contract MyToken.
//! Layout commitment: 0xabc123...
//!
//! ## Supported Queries
//! - `totalSupply`: Total token supply
//! - `balanceOf`: User token balances
//! - `allowance`: Token allowances
//! - `name`: Token name
//! - `symbol`: Token symbol  
//! - `decimals`: Token decimals
```

### 4. CI/CD Integration

Automate crate generation in your CI/CD:

```yaml
# .github/workflows/generate-crates.yml
name: Generate Traverse Crates
on:
  push:
    paths: ['contracts/*.abi.json']

jobs:
  generate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - name: Install traverse-cli
        run: cargo install traverse-cli
      - name: Generate crates
        run: |
          for abi in contracts/*.abi.json; do
            name=$(basename "$abi" .abi.json)
            traverse-cli ethereum compile-layout "$abi" --output "layouts/$name.json"
            traverse-cli codegen app --layout "layouts/$name.json" --output "generated/$name-app" --name "$name"
          done
      - name: Commit generated crates
        run: |
          git add generated/
          git commit -m "Update generated traverse crates"
          git push
```

## Troubleshooting

### Common Issues

**Issue**: Generated code doesn't compile
- **Solution**: Ensure the layout file is valid JSON and contains all required fields

**Issue**: Layout commitment mismatch at runtime  
- **Solution**: Regenerate crates when your contract layout changes

**Issue**: Unsupported query error
- **Solution**: Check that your query names match exactly what's in the layout file

**Issue**: Large binary size despite using generated crates
- **Solution**: Use `--minimal` flag and avoid `--include-alloy` unless you need ABI encoding

### Getting Help

1. Check the generated `README.md` in your output directory
2. Look at the generated test cases for usage examples
3. Verify your layout file with `traverse-cli ethereum verify-layout`
4. Use `--dry-run` flag to test generation without creating files

---

This modular approach allows 3rd party valence application developers to create highly optimized, minimal libraries that contain only the exact functionality needed for their specific use cases, resulting in smaller binaries, faster compilation, and better performance. 