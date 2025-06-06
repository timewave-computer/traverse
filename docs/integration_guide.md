# Traverse Valence Coprocessor Integration Guide

This guide walks you through integrating the traverse library into your valence coprocessor application for Ethereum storage proof verification using the production-ready patterns from valence-coprocessor-app.

## Architecture Overview

Traverse provides a clean separation between storage key generation (setup phase) and proof verification (execution phase), with full Valence ecosystem integration:

### Setup Phase (External, std-compatible)
- Generate storage keys using traverse CLI tools
- Export coprocessor-compatible JSON for witness creation
- 3rd party clients fetch storage proofs via eth_getProof

### Execution Phase (Coprocessor, no_std)
- **Controller**: Parse JSON arguments and create witnesses using standard Valence patterns
- **Circuit**: Verify storage proofs and extract field values or generate ABI-encoded messages
- **Domain**: Validate Ethereum state proofs and block headers

### Valence Ecosystem Integration
- **Message Types**: Complete Valence message structures (ZkMessage, SendMsgs, ProcessorMessage, etc.)
- **ABI Encoding**: Alloy-based ABI encoding for Valence Authorization contracts
- **Standard Entry Points**: Matches valence-coprocessor-app patterns exactly

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

# For ABI encoding (optional)
[dependencies.traverse-valence-alloy]
package = "traverse-valence"
git = "https://github.com/timewave-computer/traverse"
features = ["alloy"]
optional = true

# For std-compatible components (optional)
[dependencies.traverse-valence-std]
package = "traverse-valence"
git = "https://github.com/timewave-computer/traverse"
features = ["std"]
optional = true
```

### 3. Implement Standard Valence Controller

Update your `controller/src/lib.rs` to use the standard Valence pattern:

```rust
use traverse_valence::controller;
use serde_json::Value;
use valence_coprocessor::Witness;

/// Standard Valence controller entry point - matches valence-coprocessor-app pattern
pub fn get_witnesses(args: Value) -> anyhow::Result<Vec<Witness>> {
    controller::create_storage_witnesses(&args)
        .map_err(|e| anyhow::anyhow!("Failed to create storage witnesses: {}", e))
}
```

### 4. Implement Standard Valence Circuit

Update your `circuit/src/lib.rs` to use the standard Valence pattern:

```rust
use traverse_valence::circuit;
use valence_coprocessor::Witness;

/// Standard Valence circuit entry point - matches valence-coprocessor-app pattern
pub fn circuit(witnesses: Vec<Witness>) -> Vec<u8> {
    circuit::verify_storage_proofs_and_extract(witnesses)
}

/// Custom application logic for specific value extraction
pub fn verify_storage_proofs_custom(
    witnesses: &[Witness],
) -> Result<Vec<u64>, traverse_valence::TraverseValenceError> {
    circuit::extract_multiple_u64_values(witnesses)
}

/// Extract address values from storage proofs
pub fn extract_addresses(
    witnesses: &[Witness],
) -> Result<Vec<[u8; 20]>, traverse_valence::TraverseValenceError> {
    circuit::extract_multiple_address_values(witnesses)
}

/// Generate Valence-compatible ABI-encoded message (requires alloy feature)
#[cfg(feature = "alloy")]
pub fn create_valence_message(
    witnesses: &[Witness],
    execution_id: u64,
) -> Result<Vec<u8>, traverse_valence::TraverseValenceError> {
    use traverse_valence::messages::{create_storage_validation_message, abi_encoding};
    
    // Verify all proofs and get results
    let results = circuit::verify_storage_proofs_internal(witnesses)?;
    
    // Create validation summary
    let summary_result = traverse_valence::messages::StorageProofValidationResult {
        is_valid: results.iter().all(|r| r.is_valid),
        storage_value: "batch_verification".into(),
        storage_key: "multiple_keys".into(),
        layout_commitment: "verified".into(),
        metadata: Some(format!("verified_{}_proofs", results.len())),
    };
    
    // Generate Valence message
    let zk_message = create_storage_validation_message(summary_result, execution_id);
    abi_encoding::encode_zk_message(&zk_message)
}
```

### 5. Implement Domain Integration

Update your `domain/src/lib.rs`:

```rust
use traverse_valence::domain;

/// Validate Ethereum state proofs for your application
pub fn validate_storage_proofs(
    storage_proofs: &[serde_json::Value],
    block_number: u64,
    expected_state_root: &[u8; 32],
) -> Result<Vec<domain::ValidatedStateProof>, traverse_valence::TraverseValenceError> {
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
cargo run -p traverse-cli -- batch-resolve queries.txt --layout layout.json --format coprocessor-json > batch_queries.json
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
let balances = circuit::extract_multiple_u64_values(&witnesses)?;

println!("User balances: {:?}", balances);
```

### Multi-Contract State Verification

```rust
// Verify state across multiple contracts
let contracts = ["token_a", "token_b", "pool"];
let mut all_witnesses = Vec::new();

for contract_data in coprocessor_batch_json["contracts"].as_array() {
    let contract_witnesses = controller::create_storage_witnesses(contract_data)?;
    all_witnesses.extend(contract_witnesses);
}

// Process all witnesses together
let validation_result = circuit::verify_storage_proofs_and_extract(all_witnesses);
```

## Best Practices

### Circuit Optimization

1. **Minimize Witness Size**: Use batch operations to reduce the number of individual storage proofs
2. **Layout Commitment Verification**: Always verify layout commitments to ensure circuit safety
3. **Field Size Optimization**: Use appropriate field sizes (u64 for balances, [u8; 20] for addresses)

### Error Handling

```rust
use traverse_valence::TraverseValenceError;

match controller::create_storage_witnesses(&json_args) {
    Ok(witnesses) => { /* Process witnesses */ },
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
4. **Valence Message Validation**: When using ABI encoding, ensure message structures are validated

### Valence Integration Patterns

1. **Standard Entry Points**: Always use `get_witnesses()` and `circuit()` function signatures
2. **Message Compatibility**: Use traverse-valence message types for Valence Authorization integration
3. **ABI Encoding**: Enable `alloy` feature for production Valence deployments
4. **Batch Processing**: Leverage batch operations for efficient multi-proof verification

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
cargo run -p traverse-cli -- resolve "your_query" --layout your_layout.json --format coprocessor-json
```