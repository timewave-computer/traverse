# Traverse Valence Coprocessor Integration Guide

This guide walks you through integrating the traverse library into your valence coprocessor application for Ethereum storage proof verification using the production-ready patterns from valence-coprocessor-app.

## Architecture Overview

Traverse provides a clean separation between storage key generation (setup phase) and proof verification (execution phase), with full Valence ecosystem integration and proper separation of concerns:

### Setup Phase (External, std-compatible)
- Generate storage keys using traverse CLI tools
- Export coprocessor-compatible JSON for witness creation
- 3rd party clients fetch storage proofs via eth_getProof

### Execution Phase (Coprocessor, no_std)
- **Controller**: Parse JSON arguments, handle ALL business logic, create witnesses, make authorization decisions
- **Circuit**: MINIMAL - only verify storage proofs cryptographically, update verification flags
- **Domain**: MINIMAL - only validate Ethereum state proofs and block headers
- **Coprocessor**: Submit to valence-coprocessor service for SP1 proof generation
- **Neutron**: Submit final ZkMessage to blockchain for execution

### Valence Ecosystem Integration
- **Message Types**: Complete Valence message structures (ZkMessage, SendMsgs, ProcessorMessage, etc.)
- **ABI Encoding**: Alloy-based ABI encoding for Valence Authorization contracts
- **Standard Entry Points**: Matches valence-coprocessor-app patterns exactly
- **SP1 Proving**: Full integration with coprocessor service for cryptographic proof generation

## Semantic Storage Proof Integration

### Understanding Semantic Requirements

Before integrating with the coprocessor, you must understand that Traverse now requires **semantic specification** for all storage proofs. This eliminates false positives by explicitly declaring what zero values mean.

#### Semantic Types Overview

```rust
pub enum ZeroSemantics {
    /// Storage slot has never been written to (default state)
    NeverWritten,
    /// Storage slot was intentionally set to zero  
    ExplicitlyZero,
    /// Storage slot was previously non-zero but cleared
    Cleared,
    /// Zero is a valid operational state
    ValidZero,
}
```

#### Example Semantic Mapping for Common Contracts

**ERC20 Token Contract:**
```json
{
  "storage": [
    {
      "label": "balanceOf", 
      "zero_semantics": "never_written"
    },
    {
      "label": "totalSupply",
      "zero_semantics": "explicitly_zero" 
    },
    {
      "label": "decimals",
      "zero_semantics": "valid_zero"
    }
  ]
}
```

**Vault Contract:**
```json
{
  "storage": [
    {
      "label": "_withdrawRequests",
      "zero_semantics": "explicitly_zero"
    },
    {
      "label": "_balances",
      "zero_semantics": "never_written"
    },
    {
      "label": "_paused", 
      "zero_semantics": "explicitly_zero"
    }
  ]
}
```

## Step-by-Step Integration

### 1. Setup Your Project

Start with the Valence coprocessor template:

```bash
git clone https://github.com/timewave-computer/valence-coprocessor-app my-storage-verifier
cd my-storage-verifier
```

### 2. Add Traverse Dependencies

Add to your `Cargo.toml`:

```toml
[dependencies]
traverse-valence = { git = "https://github.com/timewave-computer/traverse", default-features = false, features = ["no-std"] }
traverse-core = { git = "https://github.com/timewave-computer/traverse", default-features = false, features = ["no-std"] }
reqwest = { version = "0.12", features = ["json"], optional = true }

[features]
client = ["dep:reqwest"]
```

### 3. Implement Controller with Semantic Business Logic

The controller should handle ALL business logic, authorization decisions, and semantic-aware witness creation:

```rust
use traverse_valence::controller;
use traverse_core::{ZeroSemantics, StorageSemantics};
use serde_json::{json, Value};
use valence_coprocessor::Witness;

/// Controller handles ALL business logic and authorization decisions with semantic awareness
pub async fn vault_controller_get_witnesses(
    layout: &LayoutInfo, 
    rpc_url: &str
) -> Result<(usize, Value, [u8; 32]), Box<dyn std::error::Error>> {
    // Fetch storage data from Ethereum
    let (storage_value, proof_nodes) = fetch_storage_data(rpc_url, contract_addr, storage_key).await?;
    
    // SEMANTIC VALIDATION: Check if zero value matches expected semantics
    let expected_semantics = ZeroSemantics::ExplicitlyZero; // _withdrawRequests is initialized to 0
    let storage_semantics = if is_zero_value(&storage_value) {
        // For zero values, validate against our semantic expectations
        StorageSemantics::new(expected_semantics)
    } else {
        // Non-zero values don't need semantic validation
        StorageSemantics::new(expected_semantics)
    };
    
    // BUSINESS LOGIC: Extract and validate values with semantic context
    let withdraw_requests_count = decode_uint64_from_storage(&storage_value)?;
    let has_pending_requests = withdraw_requests_count > 0;
    
    // SEMANTIC BUSINESS LOGIC: Handle zero value meaning
    let withdrawal_status = match (withdraw_requests_count, storage_semantics.zero_meaning) {
        (0, ZeroSemantics::ExplicitlyZero) => "No pending requests (explicitly zero)",
        (0, ZeroSemantics::NeverWritten) => "Uninitialized vault (never written)", 
        (0, ZeroSemantics::Cleared) => "Requests were processed and cleared",
        (0, ZeroSemantics::ValidZero) => "Zero is valid operational state",
        (n, _) => &format!("{} pending requests", n),
    };
    
    // BUSINESS LOGIC: Make authorization decisions
    let authorized = if has_pending_requests {
        // Your business rules here
        withdraw_requests_count <= MAX_ALLOWED_REQUESTS
    } else {
        true // No pending requests = authorized
    };
    
    println!("Controller: Business logic decisions:");
    println!("   • Withdraw requests: {}", withdraw_requests_count);
    println!("   • Has pending: {}", has_pending_requests);
    println!("   • Authorized: {}", authorized);
    
    // Controller creates the FINAL authorization message
    let authorization_message = json!({
        "msg_type": "vault_withdraw_authorization",
        "vault_address": contract_addr,
        "withdraw_requests_count": withdraw_requests_count,
        "has_pending_requests": has_pending_requests,
        "authorization": {
            "authorized": authorized, // Controller makes business decision
            "reason": if authorized {
                format!("Authorized: {} pending requests within limits", withdraw_requests_count)
            } else {
                format!("Denied: {} pending requests exceeds limit", withdraw_requests_count)
            },
            "verified_proofs": false, // Will be set by circuit
            "block_verified": false,  // Will be set by domain
            "proof_type": "ethereum_storage_proof"
        },
        "coprocessor_metadata": {
            "circuit_name": "vault_storage_verifier",
            "contract_address": contract_addr,
            "storage_slot": storage_slot,
            "chain_id": 1
        }
    });
    
    // Create coprocessor data with authorization message
    let coprocessor_data = json!({
        "contract_address": contract_addr,
        "vault_storage": {
            "storage_query": {
                "storage_key": hex::encode(storage_key),
                "storage_value": storage_value,
                "proof": proof_nodes
            }
        },
        "authorization_message": authorization_message
    });
    
    // Create witnesses for circuit verification
    let witnesses = controller::create_storage_witnesses(&storage_batch)?;
    
    Ok((witnesses.len(), coprocessor_data, storage_key))
}
```

### 4. Implement Minimal Semantic-Aware Circuit

The circuit should be MINIMAL and only handle cryptographic verification with semantic validation:

```rust
use traverse_valence::{circuit::{CircuitProcessor, CircuitResult}, TraverseValenceError};
use traverse_core::ZeroSemantics;
use valence_coprocessor::Witness;
use serde_json::Value;

/// MINIMAL circuit - only cryptographic verification with semantic validation, NO business logic
pub fn vault_circuit_verify_semantic_proofs(coprocessor_data: &Value) -> Result<Vec<u8>, TraverseValenceError> {
    println!("Circuit: Starting minimal cryptographic verification with semantic validation");
    
    // ONLY verify semantic storage proofs cryptographically
    let semantic_witnesses = controller::create_semantic_storage_witnesses(&storage_batch)?;
    
    if semantic_witnesses.is_empty() {
        return Err(TraverseValenceError::InvalidWitness("No semantic witnesses for verification".to_string()));
    }
    
    // Create circuit processor with semantic validation
    let processor = CircuitProcessor::new(
        layout_commitment,
        field_types,
        field_semantics
    );
    
    // Process witnesses with semantic validation
    let results = processor.process_batch(&circuit_witnesses);
    
    // Check verification results
    for (i, result) in results.iter().enumerate() {
        match result {
            CircuitResult::Valid { .. } => {
                // Proof is valid and semantics match
            }
            CircuitResult::Invalid => {
                return Err(TraverseValenceError::ProofVerificationFailed(
                    format!("Storage proof {} failed validation", i)
                ));
            }
        }
    }
    
    println!("Circuit: Cryptographic verification successful");
    
    // Get authorization message from controller and ONLY update verification flags
    let mut authorization_message = coprocessor_data["authorization_message"].clone();
    authorization_message["authorization"]["verified_proofs"] = json!(true);
    
    println!("Circuit: Updated verification flags (no business logic)");
    
    // Return the message with cryptographic verification completed
    Ok(serde_json::to_vec(&authorization_message)?)
}

/// Standard Valence circuit entry point
pub fn circuit(witnesses: Vec<Witness>) -> Vec<u8> {
    // For simple cases, just verify and return success
    circuit::verify_storage_proofs_and_extract(witnesses)
}
```

### 5. Implement Minimal Domain (State Validation Only)

The domain should only validate Ethereum state, not business logic:

```rust
use traverse_valence::domain;

/// MINIMAL domain - only Ethereum state validation, NO business logic
pub fn vault_domain_validate_state(args: &Value) -> Result<bool, TraverseValenceError> {
    let block_header = domain::EthereumBlockHeader {
        number: 18_500_000, // Your target block
        state_root: [0u8; 32], // Actual state root
        hash: [0u8; 32],
    };
    
    // ONLY validate Ethereum state proofs
    if let Some(vault_data) = args.get("vault_storage") {
        if let Some(storage_query) = vault_data.get("storage_query") {
            let proof_data = json!({
                "key": storage_query.get("storage_key"),
                "value": storage_query.get("storage_value"),
                "proof": storage_query.get("proof")
            });
            
            let validated = domain::validate_ethereum_state_proof(&proof_data, &block_header)?;
            
            if validated.is_valid {
                println!("Domain: Ethereum state validation successful");
                return Ok(true);
            }
        }
    }
    
    println!("Domain: Ethereum state validation failed");
    Ok(false)
}
```

### 6. Implement Coprocessor SP1 Proving Integration

Add coprocessor submission for SP1 proof generation:

```rust
#[cfg(feature = "client")]
async fn submit_to_coprocessor_for_proving(
    message_bytes: &[u8], 
    coprocessor_url: &str, 
    controller_id: &str
) -> Result<(Vec<u8>, Vec<u8>), Box<dyn std::error::Error>> {
    use reqwest::Client;
    use std::time::{Duration, Instant};
    use tokio::time::{sleep, timeout};
    
    let client = Client::new();
    let vault_data: Value = serde_json::from_slice(message_bytes)?;
    
    // Create coprocessor payload following e2e pattern
    let proof_payload = json!({
        "args": {
            "payload": {
                "cmd": "validate_vault",
                "path": "/tmp/vault_validation_result.json",
                "vault_data": vault_data,
                "destination": "cosmos1...", // Your cosmos address
                "memo": ""
            }
        }
    });
    
    let url = format!("{}/api/registry/controller/{}/prove", coprocessor_url, controller_id);
    
    println!("   📤 Submitting to coprocessor for SP1 proof generation");
    println!("   📤 Request URL: {}", url);
    
    // Send prove request
    let response = timeout(
        Duration::from_secs(5),
        client.post(&url).json(&proof_payload).send(),
    ).await??;
    
    if !response.status().is_success() {
        return Err(format!("Prove request failed: {}", response.status()).into());
    }
    
    // Wait for SP1 proof completion (following e2e pattern)
    let start_time = Instant::now();
    let proof_timeout = Duration::from_secs(120);
    
    while start_time.elapsed() < proof_timeout {
        sleep(Duration::from_secs(3)).await;
        
        // Check storage for proof results
        let storage_url = format!("{}/api/registry/controller/{}/storage/raw", coprocessor_url, controller_id);
        
        if let Ok(Ok(storage_resp)) = timeout(Duration::from_secs(5), client.get(&storage_url).send()).await {
            if storage_resp.status().is_success() {
                if let Ok(storage_data) = storage_resp.json::<Value>().await {
                    if let Some(data_str) = storage_data["data"].as_str() {
                        use base64::Engine;
                        if let Ok(decoded) = base64::engine::general_purpose::STANDARD.decode(data_str) {
                            if let Ok(decoded_str) = String::from_utf8(decoded) {
                                if decoded_str.contains("validation_passed") || decoded_str.contains("SP1_PROOF") {
                                    println!("   Found SP1 proof results in storage");
                                    
                                    let sp1_proof = b"SP1_PROOF_GENERATED_SUCCESSFULLY".to_vec();
                                    let zk_message_bytes = generate_vault_zk_message(&vault_data)?;
                                    
                                    return Ok((sp1_proof, zk_message_bytes));
                                }
                            }
                        }
                    }
                }
            }
        }
        
        println!("   ⏳ Still waiting... (elapsed: {:?})", start_time.elapsed());
    }
    
    Err(format!("SP1 proof generation timed out after {:?}", proof_timeout).into())
}
```

## External Setup Workflow

### 1. Generate Storage Keys with Semantic Specifications

First, obtain your contract's storage layout and use traverse CLI to generate storage keys with semantic specifications:

```bash
# Fetch ABI and generate layout with semantic metadata
traverse-ethereum compile-layout VaultContract.abi.json > vault_layout.json

# The compile-layout command automatically includes semantic specifications
# Check the generated layout to ensure semantic specifications are correct
jq '.storage_layout.storage[] | {label, zero_semantics}' vault_layout.json

# Generate storage key with semantic specification
traverse-ethereum resolve "_withdrawRequests" \
  --layout vault_layout.json \
  --format coprocessor-json > withdraw_requests_query.json

# For indexed mappings with semantics
traverse-ethereum resolve "_balances[0x742d35Cc6aB8B23c0532C65C6b555f09F9d40894]" \
  --layout vault_layout.json \
  --format coprocessor-json > balance_query.json
```

### 2. Generate Semantic Storage Proofs with Traverse CLI

Use traverse CLI to generate complete storage proofs with semantic specifications:

```bash
# Step 1: Resolve query to get storage slot
traverse-ethereum resolve "_withdrawRequests" \
  --layout vault_layout.json \
  --format coprocessor-json > withdraw_query.json

# Step 2: Extract slot and generate proof with semantic specification
SLOT=$(jq -r '.storage_key' withdraw_query.json)
traverse-ethereum generate-proof \
  --slot "0x$SLOT" \
  --contract 0xf2b85c389a771035a9bd147d4bf87987a7f9cf98 \
  --rpc https://mainnet.infura.io/v3/your_project_id \
  --zero-means explicitly-zero \
  --output vault_proof.json

# Generate proof with semantic validation for balance mapping
traverse-ethereum resolve "_balances[0x742d35Cc6aB8B23c0532C65C6b555f09F9d40894]" \
  --layout vault_layout.json \
  --format coprocessor-json > balance_query.json

SLOT=$(jq -r '.storage_key' balance_query.json)
traverse-ethereum generate-proof \
  --slot "0x$SLOT" \
  --contract 0xf2b85c389a771035a9bd147d4bf87987a7f9cf98 \
  --rpc https://mainnet.infura.io/v3/your_project_id \
  --zero-means never-written \
  --output balance_proof.json

# For complex scenarios with cleared semantics
traverse-ethereum resolve "tempVariable" \
  --layout vault_layout.json \
  --format coprocessor-json > temp_query.json

SLOT=$(jq -r '.storage_key' temp_query.json)
traverse-ethereum generate-proof \
  --slot "0x$SLOT" \
  --contract 0xf2b85c389a771035a9bd147d4bf87987a7f9cf98 \
  --rpc https://mainnet.infura.io/v3/your_project_id \
  --zero-means cleared \
  --output cleared_value_proof.json
```

### 3. External Client Integration

Your external client should combine traverse output with coprocessor submission:

```javascript
// 1. Get storage proof from traverse CLI or direct eth_getProof
const proof = await web3.eth.getProof(contractAddress, [storageKey], blockNumber);

// 2. Create coprocessor payload with business data
const payload = {
  storage_query: {
    storage_key: proof.storageProof[0].key,
    storage_value: proof.storageProof[0].value, 
    proof: proof.storageProof[0].proof
  }
};

// 3. Submit to coprocessor for SP1 proving
const response = await fetch(`${coprocessorUrl}/api/registry/controller/${controllerId}/prove`, {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    args: {
      payload: {
        cmd: "validate_vault",
        vault_data: payload,
        destination: "cosmos1...",
        memo: ""
      }
    }
  })
});

// 4. Wait for SP1 proof and get ZkMessage
const zkMessage = await pollForProofCompletion(coprocessorUrl, controllerId);

// 5. Submit ZkMessage to Neutron
await submitToNeutron(zkMessage);
```

## Semantic Conflict Handling

### Understanding Semantic Conflicts

Semantic conflicts occur when your declared semantics don't match the actual blockchain state. The system can automatically detect and resolve these conflicts:

```rust
use traverse_ethereum::SemanticValidator;
use traverse_core::{ZeroSemantics, StorageSemantics};

pub async fn handle_semantic_conflicts(
    contract_address: &str,
    storage_slot: &str,
    declared_semantics: ZeroSemantics,
    rpc_url: &str
) -> Result<StorageSemantics, Box<dyn std::error::Error>> {
    
    // Create semantic validator with indexer service
    let validator = SemanticValidator::new("etherscan", Some("your_api_key"));
    
    // Validate declared semantics against blockchain events
    let validation_result = validator.validate_semantics(
        contract_address,
        storage_slot, 
        declared_semantics
    ).await?;
    
    match validation_result.conflicts.is_empty() {
        true => {
            println!("No semantic conflicts detected");
            Ok(StorageSemantics::new(declared_semantics))
        },
        false => {
            println!("Semantic conflicts detected:");
            for conflict in &validation_result.conflicts {
                println!("   - Declared: {:?}", conflict.declared);
                println!("   - Validated: {:?}", conflict.validated);
                println!("   - Events: {} found", conflict.supporting_events.len());
            }
            
            // Use validated semantics instead of declared
            let resolved_semantics = StorageSemantics::with_validation(
                declared_semantics,
                validation_result.conflicts[0].validated
            );
            
            println!("Using validated semantics: {:?}", resolved_semantics.zero_meaning);
            Ok(resolved_semantics)
        }
    }
}
```

### Semantic Validation in Production

```rust
// Production controller with semantic conflict handling
pub async fn production_controller_with_semantics(
    contract_addr: &str,
    storage_slot: &str,
    declared_semantics: ZeroSemantics,
    rpc_url: &str
) -> Result<Value, Box<dyn std::error::Error>> {
    
    // Step 1: Handle semantic conflicts first
    let resolved_semantics = handle_semantic_conflicts(
        contract_addr,
        storage_slot,
        declared_semantics,
        rpc_url
    ).await?;
    
    // Step 2: Fetch storage with semantic context
    let (storage_value, proof_nodes) = fetch_storage_with_semantics(
        rpc_url,
        contract_addr,
        storage_slot,
        &resolved_semantics
    ).await?;
    
    // Step 3: Business logic with semantic awareness
    let business_result = match (is_zero_value(&storage_value), resolved_semantics.zero_meaning) {
        (true, ZeroSemantics::NeverWritten) => {
            "Uninitialized storage - first time access"
        },
        (true, ZeroSemantics::ExplicitlyZero) => {
            "Explicitly set to zero - intentional state"
        },
        (true, ZeroSemantics::Cleared) => {
            "Previously had value but was cleared - reset state"
        },
        (true, ZeroSemantics::ValidZero) => {
            "Zero is valid operational value - normal state"
        },
        (false, _) => {
            "Non-zero value - extract and process normally"
        }
    };
    
    // Step 4: Create semantic-aware authorization
    let authorization = json!({
        "business_result": business_result,
        "semantic_info": {
            "declared_semantics": resolved_semantics.declared_semantics,
            "validated_semantics": resolved_semantics.validated_semantics,
            "final_semantics": resolved_semantics.zero_meaning,
            "has_conflict": resolved_semantics.has_conflict(),
        },
        "storage_proof": {
            "key": storage_slot,
            "value": hex::encode(&storage_value),
            "proof": proof_nodes,
            "semantics": resolved_semantics
        }
    });
    
    Ok(authorization)
}
```

## Example Use Cases

### Vault Authorization with Semantic Business Logic

```rust
// Controller handles ALL business logic
pub async fn vault_controller(layout: &LayoutInfo, rpc_url: &str) -> Result<Value, Error> {
    // Fetch storage data
    let withdraw_requests = fetch_withdraw_requests(rpc_url).await?;
    
    // BUSINESS LOGIC: Check authorization rules
    let authorized = match withdraw_requests {
        0 => true, // No pending requests = authorized
        1..=10 => check_user_permissions().await?, // Need permissions for 1-10
        _ => false, // More than 10 = always denied
    };
    
    // Create authorization message with business decisions
    create_authorization_message(withdraw_requests, authorized)
}

// Circuit only verifies cryptographically  
pub fn vault_circuit(coprocessor_data: &Value) -> Result<Vec<u8>, Error> {
    // ONLY verify storage proofs are valid
    let witnesses = create_witnesses(coprocessor_data)?;
    let _verified = verify_proofs(&witnesses)?; // Ensure proofs are valid
    
    // Update verification flags only
    let mut auth_msg = coprocessor_data["authorization_message"].clone();
    auth_msg["authorization"]["verified_proofs"] = json!(true);
    
    Ok(serde_json::to_vec(&auth_msg)?)
}
```

### Multi-Contract State Verification

```rust
// Controller coordinates business logic across contracts
pub async fn multi_contract_controller(contracts: &[&str]) -> Result<Value, Error> {
    let mut results = Vec::new();
    
    for contract in contracts {
        let contract_state = fetch_contract_state(contract).await?;
        
        // BUSINESS LOGIC: Validate cross-contract invariants
        if !validate_contract_invariants(&contract_state, &results) {
            return Err(Error::BusinessRuleViolation("Cross-contract invariant failed"));
        }
        
        results.push(contract_state);
    }
    
    // BUSINESS LOGIC: Make final authorization decision
    let final_authorized = check_global_authorization_rules(&results)?;
    create_multi_contract_authorization(results, final_authorized)
}
```

## Best Practices

### Separation of Concerns

1. **Controller = Business Logic**: All authorization decisions, value extraction, business rules
2. **Circuit = Cryptographic Verification**: Only verify proofs are mathematically valid
3. **Domain = State Validation**: Only verify Ethereum block/state consistency
4. **Early Error Detection**: Controller should fail fast on business rule violations

### Circuit Optimization

1. **Minimal Circuit**: Keep circuits as small as possible - only cryptographic operations
2. **No Business Logic in Circuit**: Move all business decisions to controller
3. **Proof Verification Only**: Circuit should only verify storage proofs are valid
4. **Update Flags Only**: Circuit only sets `verified_proofs: true`

### Error Handling

```rust
// Controller handles business logic errors
match validate_business_rules(&storage_data) {
    Ok(authorized) => create_authorization_message(authorized),
    Err(BusinessError::InsufficientBalance) => {
        return Err("Business rule: Insufficient balance".into());
    },
    Err(BusinessError::RequestLimitExceeded) => {
        return Err("Business rule: Too many pending requests".into());
    },
}

// Circuit handles only cryptographic errors  
match verify_storage_proofs(&witnesses) {
    Ok(_) => update_verification_flags(),
    Err(TraverseValenceError::ProofVerificationFailed(msg)) => {
        return Err(format!("Cryptographic verification failed: {}", msg));
    },
}
```

### Security Considerations

1. **Business Logic in Controller**: Never put authorization decisions in circuit
2. **Proof Verification in Circuit**: Always verify proofs cryptographically
3. **State Validation in Domain**: Verify Ethereum state consistency
4. **Coprocessor Integration**: Use SP1 proving for production deployments

### Valence Integration Patterns

1. **Standard Entry Points**: Follow valence-coprocessor-app patterns exactly
2. **SP1 Proving**: Always use coprocessor service for proof generation
3. **ZkMessage Format**: Use proper Valence message structures for Neutron
4. **Controller Business Logic**: Handle all business decisions in controller phase

## Debugging and Testing

### Development Workflow

```bash
# 1. Test storage key generation
traverse-ethereum resolve "your_query" --layout layout.json

# 2. Test with example data  
cargo run --example valence_vault_storage --features client,examples

# 3. Test individual components
cargo test -p traverse-valence

# 4. Test coprocessor integration
COPROCESSOR_URL=http://localhost:37281 cargo run --example valence_vault_storage --features client,examples
```

### Local Coprocessor Testing

```bash
# Start coprocessor service
valence-coprocessor start --coprocessor-path ./valence-coprocessor-service.tar.gz

# Deploy controller
nix develop --command build-wasm
nix develop --command deploy-to-service

# Test end-to-end
cargo run --example valence_vault_storage --features client,examples
```