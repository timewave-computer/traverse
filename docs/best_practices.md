# Traverse Best Practices

This document outlines best practices for using traverse in production valence coprocessor applications, with focus on circuit optimization, security, and performance.

## Circuit Optimization

### Minimize Witness Size

**Problem**: Large witnesses increase proof generation time and memory usage.

**Solution**: Use batch operations and optimize field sizes:

```rust
// ❌ Bad: Individual witness creation
for query in queries {
    let witness = controller::create_storage_witness(&json_for_query)?;
    witnesses.push(witness);
}

// ✅ Good: Batch witness creation
let witnesses = controller::create_batch_storage_witnesses(&batch_json)?;
```

### Layout Commitment Verification

**Note**: The current traverse-valence implementation provides basic proof verification. Layout commitment verification would be handled by external validation logic.

**Solution**: Use basic storage proof verification:

```rust
// ✅ Basic proof verification (current implementation)
let result = circuit::verify_storage_proof(&witness)?;

// Note: Full layout commitment verification would be implemented
// as part of external validation logic
```

### Field Size Optimization

**Problem**: Using wrong field sizes wastes circuit constraints.

**Solution**: Use appropriate extraction functions:

```rust
// ✅ Good: Use specific extraction functions
let balance: u64 = circuit::extract_u64_value(&witness)?;
let address: [u8; 20] = circuit::extract_address_value(&witness)?;

// ❌ Bad: Always extracting raw bytes
let field_bytes = circuit::verify_storage_proof(&witness)?;
// Manual conversion loses type safety and optimization opportunities
```

### Batch Processing Patterns

**Efficient Multi-Query Pattern**:

```rust
use traverse_valence::TraverseValenceError;

// Process multiple storage queries efficiently
pub fn verify_multiple_balances(
    batch_json: &Value,
) -> Result<Vec<u64>, TraverseValenceError> {
    let witnesses = controller::create_batch_storage_witnesses(batch_json)?;
    
    let balances = circuit::extract_multiple_u64_values(&witnesses)?;
    
    Ok(balances)
}
```

## Security Best Practices

### Layout Commitment Security

**Note**: Layout commitment security would be implemented as part of external validation workflows since the current traverse-valence implementation focuses on basic proof processing.

```rust
// Note: Layout commitment validation would be handled externally
// Current implementation provides basic proof verification
let result = circuit::verify_storage_proof(&witness)?;
```

### Storage Key Validation

**Problem**: Malicious storage keys can target arbitrary contract state.

**Solution**: Validate storage keys against expected patterns:

```rust
use traverse_valence::TraverseValenceError;

// ✅ Good: Validate storage key derivation
pub fn validate_balance_query(query: &str, expected_slot: u8) -> Result<(), TraverseValenceError> {
    // Parse query to ensure it's a balance lookup
    if !query.starts_with("_balances[") {
        return Err(TraverseValenceError::InvalidStorageKey("Not a balance query".into()));
    }
    
    // Validate address format
    let address_part = &query[10..query.len()-1]; // Extract address
    if !is_valid_ethereum_address(address_part) {
        return Err(TraverseValenceError::InvalidStorageKey("Invalid address".into()));
    }
    
    Ok(())
}

// ✅ Allowlist known contract fields
const ALLOWED_FIELDS: &[&str] = &["_balances", "_allowances", "_totalSupply"];

pub fn validate_field_access(query: &str) -> bool {
    ALLOWED_FIELDS.iter().any(|field| query.starts_with(field))
}
```

### Proof Verification Security

**Basic**: Current implementation provides basic proof verification:

```rust
// ✅ Good: Basic proof verification (current implementation)
let verified_value = circuit::verify_storage_proof(&witness)?;

// Additional validation can be implemented externally
let validated_proof = domain::validate_ethereum_state_proof(
    &storage_proof,
    &block_header,
)?;

if !validated_proof.is_valid {
    return Err(TraverseValenceError::ProofVerificationFailed("Invalid proof".into()));
}
```

## Error Handling Patterns

### Comprehensive Error Handling

```rust
use traverse_valence::TraverseValenceError;
use valence_coprocessor::Witness;

pub fn robust_witness_creation(json_args: &Value) -> Result<Witness, anyhow::Error> {
    let witness = controller::create_storage_witness(json_args)
        .map_err(|e| match e {
            TraverseValenceError::Json(msg) => {
                anyhow::anyhow!("JSON parsing failed: {}. Check input format.", msg)
            },
            TraverseValenceError::InvalidStorageKey(msg) => {
                anyhow::anyhow!("Storage key validation failed: {}. Verify query syntax.", msg)
            },
            TraverseValenceError::ProofVerificationFailed(msg) => {
                anyhow::anyhow!("Proof verification failed: {}. Check eth_getProof data.", msg)
            },
            TraverseValenceError::LayoutMismatch(msg) => {
                anyhow::anyhow!("Layout mismatch: {}. Contract version mismatch?", msg)
            },
            TraverseValenceError::InvalidWitness(msg) => {
                anyhow::anyhow!("Invalid witness: {}. Check witness format.", msg)
            },
        })?;
    
    Ok(witness)
}
```

### Graceful Batch Error Handling

```rust
pub fn process_batch_with_partial_failures(
    batch_json: &Value,
) -> (Vec<Witness>, Vec<String>) {
    let mut witnesses = Vec::new();
    let mut errors = Vec::new();
    
    if let Some(batch) = batch_json["storage_batch"].as_array() {
        for (i, item) in batch.iter().enumerate() {
            match controller::create_storage_witness(item) {
                Ok(witness) => witnesses.push(witness),
                Err(e) => errors.push(format!("Item {}: {}", i, e)),
            }
        }
    }
    
    (witnesses, errors)
}
```

## Performance Optimization

### Efficient JSON Processing

```rust
// ✅ Good: Parse once, use multiple times
let storage_query: CoprocessorStorageQuery = serde_json::from_value(
    json_args["storage_query"].clone()
)?;

// ❌ Bad: Repeated parsing
let query1 = json_args["storage_query"]["query"].as_str().unwrap();
let key1 = json_args["storage_query"]["storage_key"].as_str().unwrap();
// ... repeated access
```

### Memory-Efficient Batch Processing

```rust
// ✅ Good: Stream processing for large batches
pub fn process_large_batch<F>(
    batch_json: &Value,
    mut processor: F,
) -> Result<(), ValenceError>
where
    F: FnMut(MockWitness) -> Result<(), ValenceError>,
{
    if let Some(batch) = batch_json["storage_batch"].as_array() {
        for item in batch {
            let witness = controller::create_storage_witness(item)?;
            processor(witness)?;
        }
    }
    Ok(())
}

// ❌ Bad: Loading entire batch into memory
let all_witnesses = controller::create_batch_storage_witnesses(huge_batch)?; // OOM risk
```

## Testing Patterns

### Circuit Testing

```rust
#[cfg(test)]
mod circuit_tests {
    use super::*;
    
    const TEST_LAYOUT_COMMITMENT: [u8; 32] = [/* test commitment */];
    
    #[test]
    fn test_balance_extraction() {
        let mock_witness = create_mock_balance_witness(1000);
        let query = create_mock_balance_query();
        
        let balance = circuit::extract_u64_value(
            &mock_witness,
            &TEST_LAYOUT_COMMITMENT,
            &query,
        ).unwrap();
        
        assert_eq!(balance, 1000);
    }
    
    #[test]
    fn test_invalid_layout_commitment() {
        let mock_witness = create_mock_balance_witness(1000);
        let query = create_mock_balance_query();
        let wrong_commitment = [0u8; 32];
        
        let result = circuit::extract_u64_value(
            &mock_witness,
            &wrong_commitment,
            &query,
        );
        
        assert!(matches!(result, Err(ValenceError::LayoutMismatch(_))));
    }
}
```

### Integration Testing

```rust
#[test]
fn test_end_to_end_workflow() {
    // 1. Generate storage key
    let layout = load_test_layout();
    let resolver = EthereumKeyResolver;
    let path = resolver.resolve(&layout, "_balances[0x742d35...]").unwrap();
    
    // 2. Create coprocessor JSON
    let coprocessor_json = create_coprocessor_json(&path);
    
    // 3. Create witness
    let witness = controller::create_storage_witness(&coprocessor_json).unwrap();
    
    // 4. Verify in circuit
    let balance = circuit::extract_u64_value(
        &witness,
        &layout.commitment,
        &coprocessor_json["storage_query"],
    ).unwrap();
    
    assert_eq!(balance, 1000);
}
```

## Common Patterns

### ERC20 Token Verification

```rust
pub struct TokenVerifier {
    layout_commitment: [u8; 32],
    contract_address: [u8; 20],
}

impl TokenVerifier {
    pub fn verify_balance(
        &self,
        user_address: &str,
        witness_json: &Value,
    ) -> Result<u64, ValenceError> {
        // Validate query targets balance
        let query = format!("_balances[{}]", user_address);
        
        let witness = controller::create_storage_witness(witness_json)?;
        let query_data = CoprocessorStorageQuery {
            query,
            storage_key: witness_json["storage_query"]["storage_key"]
                .as_str().unwrap().to_string(),
            layout_commitment: hex::encode(self.layout_commitment),
            field_size: Some(32),
            offset: None,
        };
        
        circuit::extract_u64_value(&witness, &self.layout_commitment, &query_data)
    }
    
    pub fn verify_allowance(
        &self,
        owner: &str,
        spender: &str,
        witness_json: &Value,
    ) -> Result<u64, ValenceError> {
        let query = format!("_allowances[{}][{}]", owner, spender);
        // ... similar verification logic
        todo!()
    }
}
```

### Multi-Contract State Verification

```rust
pub struct MultiContractVerifier {
    contracts: Vec<ContractInfo>,
}

pub struct ContractInfo {
    address: [u8; 20],
    layout_commitment: [u8; 32],
    name: String,
}

impl MultiContractVerifier {
    pub fn verify_cross_contract_state(
        &self,
        batch_json: &Value,
    ) -> Result<Vec<ContractState>, ValenceError> {
        let mut states = Vec::new();
        
        for contract in &self.contracts {
            let contract_data = &batch_json[&contract.name];
            let witnesses = controller::create_batch_storage_witnesses(contract_data)?;
            
            // Process contract-specific state
            let state = self.process_contract_state(contract, witnesses)?;
            states.push(state);
        }
        
        Ok(states)
    }
}
```

## Debugging Tips

### Enable Detailed Logging

```rust
#[cfg(debug_assertions)]
macro_rules! debug_witness {
    ($witness:expr) => {
        match $witness {
            MockWitness::StateProof { key, value, proof } => {
                println!("Debug Witness:");
                println!("  Key: {:02x?}", key);
                println!("  Value: {:02x?}", value);
                println!("  Proof nodes: {}", proof.len());
            },
            _ => println!("Debug: Non-state-proof witness"),
        }
    };
}
```

### Validate Against Known Test Vectors

```rust
const KNOWN_BALANCE_TESTS: &[(&str, &str, u64)] = &[
    ("0x742d35Cc6aB8B23c0532C65C6b555f09F9d40894", "135d4f63f4765210c4fb1f96117747705a2369326ab4fcb6c7c368451fc84a9c", 1000),
    // Add more test vectors
];

#[test]
fn validate_against_known_vectors() {
    for (address, expected_key, expected_balance) in KNOWN_BALANCE_TESTS {
        let query = format!("_balances[{}]", address);
        let path = resolver.resolve(&layout, &query).unwrap();
        assert_eq!(hex::encode(path.storage_key()), *expected_key);
    }
}
```

---

Following these best practices will ensure your traverse integration is secure, performant, and maintainable in production valence coprocessor applications. 