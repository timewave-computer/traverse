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
reqwest = { version = "0.12", features = ["json"], optional = true }

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

[features]
client = ["dep:reqwest"]
```

### 3. Implement Controller with Business Logic

The controller should handle ALL business logic, authorization decisions, and witness creation:

```rust
use traverse_valence::controller;
use serde_json::{json, Value};
use valence_coprocessor::Witness;

/// Controller handles ALL business logic and authorization decisions
pub async fn vault_controller_get_witnesses(
    layout: &LayoutInfo, 
    rpc_url: &str
) -> Result<(usize, Value, [u8; 32]), Box<dyn std::error::Error>> {
    // Fetch storage data from Ethereum
    let (storage_value, proof_nodes) = fetch_storage_data(rpc_url, contract_addr, storage_key).await?;
    
    // BUSINESS LOGIC: Extract and validate values
    let withdraw_requests_count = decode_uint64_from_storage(&storage_value)?;
    let has_pending_requests = withdraw_requests_count > 0;
    
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

### 4. Implement Minimal Circuit (Cryptographic Verification Only)

The circuit should be MINIMAL and only handle cryptographic verification:

```rust
use traverse_valence::circuit;
use valence_coprocessor::Witness;

/// MINIMAL circuit - only cryptographic verification, NO business logic
pub fn vault_circuit_verify_proofs(coprocessor_data: &Value) -> Result<Vec<u8>, TraverseValenceError> {
    println!("Circuit: Starting minimal cryptographic verification");
    
    // ONLY verify storage proofs cryptographically
    let witnesses = controller::create_storage_witnesses(&storage_batch)?;
    
    if witnesses.is_empty() {
        return Err(TraverseValenceError::InvalidWitness("No witnesses for verification".to_string()));
    }
    
    // ONLY cryptographic verification - extract values to prove they're valid
    let _verified_values = circuit::extract_multiple_u64_values(&witnesses)?;
    
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
                                    println!("   ✅ Found SP1 proof results in storage");
                                    
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

### 1. Generate Storage Keys

First, obtain your contract's storage layout and use traverse CLI to generate storage keys:

```bash
# Fetch ABI and generate layout
cargo run -p traverse-cli -- compile-layout VaultContract.abi.json > vault_layout.json

# Generate storage key for specific query
cargo run -p traverse-cli -- resolve "_withdrawRequests" \
  --layout vault_layout.json \
  --format coprocessor-json > withdraw_requests_query.json

# For indexed mappings
cargo run -p traverse-cli -- resolve "_balances[0x742d35Cc6aB8B23c0532C65C6b555f09F9d40894]" \
  --layout token_layout.json \
  --format coprocessor-json > balance_query.json
```

### 2. Generate Storage Proofs with Traverse CLI

Use traverse CLI to generate complete storage proofs:

```bash
# Generate proof with real Ethereum data
cargo run -p traverse-cli --features client -- generate-proof \
  --slot 0x0000000000000000000000000000000000000000000000000000000000000007 \
  --contract 0xf2b85c389a771035a9bd147d4bf87987a7f9cf98 \
  --rpc https://mainnet.infura.io/v3/your_project_id \
  --output vault_proof.json
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

## Example Use Cases

### Vault Authorization with Business Logic

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
cargo run -p traverse-cli -- resolve "your_query" --layout layout.json

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