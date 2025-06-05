# Traverse Valence Coprocessor Integration Guide

This guide walks you through integrating the traverse library into your valence coprocessor application for Ethereum storage proof verification.

## Architecture Overview

Traverse provides a clean separation between storage key generation (setup phase) and proof verification (execution phase):

### Setup Phase (External, std-compatible)
- Generate storage keys using traverse CLI tools
- Export coprocessor-compatible JSON for witness creation
- 3rd party clients fetch storage proofs via eth_getProof

### Execution Phase (Coprocessor, no_std)
- **Controller**: Parse JSON arguments and create witnesses
- **Circuit**: Verify storage proofs and extract field values  
- **Domain**: Validate Ethereum state proofs and block headers

## Step-by-Step Integration

### 1. Fork Valence Coprocessor Template

```bash
git clone https://github.com/timewave-computer/valence-coprocessor-app my-storage-verifier
cd my-storage-verifier
```

### 2. Add Traverse Dependencies

Add to your `Cargo.toml`:

```toml
[dependencies]
traverse-valence = { git = "https://github.com/timewave-computer/traverse", features = ["alloc"] }

# For std-compatible components (controller only if needed)
[dependencies.traverse-valence-std]
package = "traverse-valence"
git = "https://github.com/timewave-computer/traverse"
features = ["std"]
optional = true
```

### 3. Implement Controller Integration

Update your `controller/src/lib.rs`:

```rust
use traverse_valence::{controller, CoprocessorStorageQuery, StorageProof, TraverseValenceError};
use serde_json::Value;
use valence_coprocessor::Witness;

/// Controller implementation for storage proof verification
pub fn get_witnesses(json_args: Value) -> Result<Vec<Witness>, anyhow::Error> {
    // Check if this is a batch operation
    if json_args.get("storage_batch").is_some() {
        let witnesses = controller::create_batch_storage_witnesses(&json_args)
            .map_err(|e| anyhow::anyhow!("Failed to create batch witnesses: {}", e))?;
        Ok(witnesses)
    } else {
        let witness = controller::create_storage_witness(&json_args)
            .map_err(|e| anyhow::anyhow!("Failed to create witness: {}", e))?;
        Ok(vec![witness])
    }
}
```

### 4. Implement Circuit Integration

Update your `circuit/src/lib.rs`:

```rust
use traverse_valence::{circuit, CoprocessorStorageQuery, TraverseValenceError};
use valence_coprocessor::Witness;

/// Verify storage proofs and extract balance values
pub fn verify_storage_proofs(
    witnesses: &[Witness],
) -> Result<Vec<u64>, TraverseValenceError> {
    let mut balances = Vec::new();
    
    for witness in witnesses {
        let balance = circuit::extract_u64_value(witness)?;
        balances.push(balance);
    }
    
    Ok(balances)
}

/// Extract address values from storage proofs
pub fn extract_addresses(
    witnesses: &[Witness],
) -> Result<Vec<[u8; 20]>, TraverseValenceError> {
    let mut addresses = Vec::new();
    
    for witness in witnesses {
        let address = circuit::extract_address_value(witness)?;
        addresses.push(address);
    }
    
    Ok(addresses)
}
```

### 5. Implement Domain Integration

Update your `domain/src/lib.rs`:

```rust
use traverse_valence::{domain, StorageProof, TraverseValenceError};

/// Validate Ethereum state proofs for your application
pub fn validate_storage_proofs(
    storage_proofs: &[serde_json::Value],
    block_number: u64,
    expected_state_root: &[u8; 32],
) -> Result<Vec<domain::ValidatedStateProof>, TraverseValenceError> {
    let mut validated_proofs = Vec::new();
    
    for proof in storage_proofs {
        let block_header = domain::EthereumBlockHeader {
            number: block_number,
            state_root: *expected_state_root,
            hash: [0u8; 32], // Would be provided by your application
        };
        
        let validated_proof = domain::validate_ethereum_state_proof(
            proof,
            &block_header,
        )?;
        
        validated_proofs.push(validated_proof);
    }
    
    Ok(validated_proofs)
}
```

## External Setup Workflow

### 1. Generate Storage Keys

First, obtain your contract's storage layout and use traverse CLI to generate storage keys:

```bash
# Option 1: Using existing layout file
cargo run -p traverse-cli -- resolve "_balances[0x742d35Cc6aB8B23c0532C65C6b555f09F9d40894]" \
  --layout contract_layout.json \
  --format coprocessor-json > balance_query.json

# Option 2: Compile from ABI first
cargo run -p traverse-cli -- compile-layout MyContract.abi.json > layout.json
cargo run -p traverse-cli -- resolve "_balances[0x742d35...]" --layout layout.json --format coprocessor-json
```

### 2. Batch Processing

For multiple queries, use batch processing:

```bash
# Create queries file
echo "_balances[0x742d35Cc6aB8B23c0532C65C6b555f09F9d40894]" > queries.txt
echo "_balances[0x8ba1f109551bD432803012645Aac136c5C1Aa000]" >> queries.txt
echo "_totalSupply" >> queries.txt

# Generate batch storage keys
zkpath batch-resolve queries.txt --layout layout.json --format coprocessor-json > batch_queries.json
```

### 3. External Client Integration

Your external client should combine traverse output with eth_getProof:

```javascript
// 1. Get storage key from traverse
const traverseOutput = JSON.parse(fs.readFileSync('balance_query.json'));

// 2. Fetch storage proof
const proof = await web3.eth.getProof(
  contractAddress,
  [traverseOutput.storage_key],
  blockNumber
);

// 3. Create coprocessor payload
const payload = {
  storage_query: traverseOutput,
  storage_proof: {
    key: proof.storageProof[0].key,
    value: proof.storageProof[0].value,
    proof: proof.storageProof[0].proof
  }
};

// 4. Submit to coprocessor
await submitToCoprocessor(payload);
```

## Example Use Cases

### ERC20 Balance Verification

```rust
// Verify user balances against storage proofs
let balance_queries = [
    "_balances[0x742d35Cc6aB8B23c0532C65C6b555f09F9d40894]",
    "_balances[0x8ba1f109551bD432803012645Aac136c5C1Aa000]",
];

let witnesses = get_witnesses(coprocessor_json)?;
let balances = verify_storage_proofs(&witnesses)?;

println!("User balances: {:?}", balances);
```

### Multi-Contract State Verification

```rust
// Verify state across multiple contracts
let contracts = ["token_a", "token_b", "pool"];
let mut all_witnesses = Vec::new();

for contract_data in coprocessor_batch_json["contracts"].as_array() {
    let contract_witnesses = controller::create_storage_witness(contract_data)?;
    all_witnesses.push(contract_witnesses);
}
```

## Best Practices

### Circuit Optimization

1. **Minimize Witness Size**: Use batch operations to reduce the number of individual storage proofs
2. **Layout Commitment Verification**: Always verify layout commitments to ensure circuit safety
3. **Field Size Optimization**: Use appropriate field sizes (u64 for balances, [u8; 20] for addresses)

### Error Handling

```rust
use traverse_valence::TraverseValenceError;

match controller::create_storage_witness(&json_args) {
    Ok(witness) => { /* Process witness */ },
    Err(TraverseValenceError::Json(msg)) => {
        // Handle JSON parsing errors
        return Err(anyhow::anyhow!("JSON error: {}", msg));
    },
    Err(TraverseValenceError::InvalidStorageKey(msg)) => {
        // Handle invalid storage key format
        return Err(anyhow::anyhow!("Invalid storage key: {}", msg));
    },
    Err(TraverseValenceError::ProofVerificationFailed(msg)) => {
        // Handle proof verification failures
        return Err(anyhow::anyhow!("Proof verification failed: {}", msg));
    },
    Err(TraverseValenceError::LayoutMismatch(msg)) => {
        // Handle layout commitment mismatches
        return Err(anyhow::anyhow!("Layout mismatch: {}", msg));
    },
}
```

### Security Considerations

1. **Storage Key Validation**: Validate storage keys are correctly derived
2. **Proof Verification**: Use traverse-valence circuit helpers for secure proof verification  
3. **Block Validation**: Ensure block headers and state roots are validated in domain layer

## Debugging and Testing

### Enable Debug Logging

```rust
#[cfg(debug_assertions)]
{
    println!("Debug: Processing witness: {:?}", witness);
}
```

### Local Testing

```bash
# Test with example data
cargo run --example valence_integration --features std

# Run integration tests
cargo test -p traverse-valence

# Validate storage key generation
zkpath resolve "your_query" --layout your_layout.json --format coprocessor-json
```

## Troubleshooting

### Common Issues

1. **Layout Mismatch**: Ensure layout.json matches the contract version
2. **Invalid Storage Keys**: Verify query syntax matches contract field names
3. **Proof Verification Failures**: Check that eth_getProof data matches traverse storage keys
4. **no_std Compilation**: Ensure all dependencies are no_std compatible

### Getting Help

- Check the [traverse-valence examples](../examples/valence_integration.rs)
- Review the [architecture documentation](./architecture.md) for design details
- Examine the [integration tests](../crates/traverse-ethereum/tests/integration.rs)

---

This guide provides the foundation for integrating traverse into your valence coprocessor application. For more advanced use cases, refer to the API documentation and examples in the repository.