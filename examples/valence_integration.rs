//! Example: Valence Coprocessor Integration with Traverse
//! 
//! This example demonstrates how to integrate the traverse library with a valence
//! coprocessor application. It shows the complete workflow from storage key generation
//! to proof verification in a ZK circuit context.
//! 
//! # Architecture
//! 
//! 1. **Setup Phase** (external, std): Generate storage keys using traverse CLI
//! 2. **Controller Phase** (no_std): Parse JSON args and create witnesses
//! 3. **Circuit Phase** (no_std): Verify storage proofs and extract values
//! 4. **Domain Phase** (no_std): Validate Ethereum state proofs
//! 
//! # Usage
//! 
//! ```bash
//! # 1. Generate storage key for ERC20 balance query
//! zkpath resolve "_balances[0x742d35Cc6aB8B23c0532C65C6b555f09F9d40894]" \
//!   --layout crates/traverse-ethereum/tests/data/erc20_layout.json \
//!   --format coprocessor-json > balance_query.json
//! 
//! # 2. External client calls eth_getProof and combines with storage key
//! # 3. Coprocessor receives combined JSON and processes it
//! cargo run --example valence_integration
//! ```

use serde_json::json;
use traverse_valence::{
    controller, circuit, domain,
    CoprocessorStorageQuery, StorageProof, MockWitness, ValenceError
};

/// Example controller function that would be called by valence coprocessor
/// 
/// This simulates the `get_witnesses(Value) -> Vec<Witness>` function that
/// the valence coprocessor controller implements.
fn example_controller_get_witnesses(
    json_args: &serde_json::Value,
) -> Result<Vec<MockWitness>, ValenceError> {
    println!("Controller: Processing coprocessor JSON arguments...");
    
    // Check if this is a single query or batch
    if json_args.get("storage_batch").is_some() {
        println!("Processing batch storage queries");
        controller::create_batch_storage_witnesses(json_args)
    } else {
        println!("Processing single storage query");
        let witness = controller::create_storage_witness(json_args)?;
        Ok(vec![witness])
    }
}

/// Example circuit function that would verify storage proofs in ZK
/// 
/// This simulates the circuit logic that receives witnesses and performs
/// verification within the ZK environment.
fn example_circuit_verify_storage(
    witnesses: &[MockWitness],
    expected_layout_commitment: &[u8; 32],
    queries: &[CoprocessorStorageQuery],
) -> Result<Vec<u64>, ValenceError> {
    println!("Circuit: Verifying storage proofs in ZK context...");
    
    let mut results = Vec::new();
    
    for (witness, query) in witnesses.iter().zip(queries.iter()) {
        println!("  Verifying storage proof for query: {}", query.query);
        
        // Verify the storage proof and extract u64 value (e.g., balance)
        let balance = circuit::extract_u64_value(
            witness,
            expected_layout_commitment,
            query,
        )?;
        
        println!("  Extracted balance: {}", balance);
        results.push(balance);
    }
    
    Ok(results)
}

/// Example domain function that validates Ethereum state proofs
/// 
/// This simulates the domain logic that validates block headers and
/// state proof inclusion.
fn example_domain_validate_proofs(
    storage_proofs: &[StorageProof],
    block_number: u64,
) -> Result<Vec<domain::ValidatedStateProof>, ValenceError> {
    println!("Domain: Validating Ethereum state proofs...");
    
    let mut validated_proofs = Vec::new();
    
    for proof in storage_proofs {
        println!("  Validating storage proof for key: {}", proof.key);
        
        // Create mock block header (in real implementation, this would come from the blockchain)
        let block_header = domain::EthereumBlockHeader {
            number: block_number,
            state_root: [0u8; 32], // Would be actual state root
            hash: [0u8; 32],       // Would be actual block hash
        };
        
        let account_address = [0u8; 20]; // Would be actual contract address
        
        let validated_proof = domain::validate_ethereum_state_proof(
            proof,
            &block_header,
            &account_address,
        )?;
        
        println!("  State proof validation: {}", validated_proof.is_valid);
        validated_proofs.push(validated_proof);
    }
    
    Ok(validated_proofs)
}

/// Create example JSON payload that would come from external client
fn create_example_payload() -> serde_json::Value {
    json!({
        "storage_query": {
            "query": "_balances[0x742d35Cc6aB8B23c0532C65C6b555f09F9d40894]",
            "storage_key": "c1f51986c7e9d391993039c3c40e41ad9f26e1db9b80f8535a639eadeb1d1bd9",
            "layout_commitment": "f6dc3c4a79e95565b3cf38993f1a120c6a6b467796264e7fd9a9c8675616dd7a",
            "field_size": 32,
            "offset": null
        },
        "storage_proof": {
            "key": "c1f51986c7e9d391993039c3c40e41ad9f26e1db9b80f8535a639eadeb1d1bd9",
            "value": "00000000000000000000000000000000000000000000000000000000000003e8",
            "proof": [
                "deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef",
                "cafebabecafebabecafebabecafebabecafebabecafebabecafebabecafebabe"
            ]
        }
    })
}

/// Create example batch payload for multiple queries
fn create_example_batch_payload() -> serde_json::Value {
    json!({
        "storage_batch": [
            {
                "storage_query": {
                    "query": "_balances[0x742d35Cc6aB8B23c0532C65C6b555f09F9d40894]",
                    "storage_key": "c1f51986c7e9d391993039c3c40e41ad9f26e1db9b80f8535a639eadeb1d1bd9",
                    "layout_commitment": "f6dc3c4a79e95565b3cf38993f1a120c6a6b467796264e7fd9a9c8675616dd7a",
                    "field_size": 32,
                    "offset": null
                },
                "storage_proof": {
                    "key": "c1f51986c7e9d391993039c3c40e41ad9f26e1db9b80f8535a639eadeb1d1bd9",
                    "value": "00000000000000000000000000000000000000000000000000000000000003e8",
                    "proof": [
                        "deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef",
                        "cafebabecafebabecafebabecafebabecafebabecafebabecafebabecafebabe"
                    ]
                }
            },
            {
                "storage_query": {
                    "query": "_balances[0x8ba1f109551bD432803012645Hac136c5C1Aa000]",
                    "storage_key": "a1b2c3d4e5f67890123456789012345678901234567890123456789012345678",
                    "layout_commitment": "f6dc3c4a79e95565b3cf38993f1a120c6a6b467796264e7fd9a9c8675616dd7a",
                    "field_size": 32,
                    "offset": null
                },
                "storage_proof": {
                    "key": "a1b2c3d4e5f67890123456789012345678901234567890123456789012345678",
                    "value": "0000000000000000000000000000000000000000000000000000000000001388",
                    "proof": [
                        "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
                        "fedcba0987654321fedcba0987654321fedcba0987654321fedcba0987654321"
                    ]
                }
            }
        ]
    })
}

// Convert ValenceError to Box<dyn std::error::Error>
fn convert_valence_error(err: ValenceError) -> Box<dyn std::error::Error> {
    Box::new(std::io::Error::new(std::io::ErrorKind::Other, format!("{}", err)))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Traverse Valence Coprocessor Integration Example");
    println!("====================================================");
    
    // Example layout commitment (would be computed from actual contract layout)
    let layout_commitment = [
        0xf6, 0xdc, 0x3c, 0x4a, 0x79, 0xe9, 0x55, 0x65,
        0xb3, 0xcf, 0x38, 0x99, 0x3f, 0x1a, 0x12, 0x0c,
        0x6a, 0x6b, 0x46, 0x77, 0x96, 0x26, 0x4e, 0x7f,
        0xd9, 0xa9, 0xc8, 0x67, 0x56, 0x16, 0xdd, 0x7a,
    ];
    
    println!("\nExample 1: Single Storage Query");
    println!("-----------------------------------");
    
    // 1. Create example payload (would come from external client)
    let payload = create_example_payload();
    println!("Received coprocessor JSON payload");
    
    // 2. Controller phase: Parse JSON and create witnesses
    let witnesses = example_controller_get_witnesses(&payload)
        .map_err(convert_valence_error)?;
    println!("Controller created {} witnesses", witnesses.len());
    
    // 3. Extract query info for circuit verification
    let query: CoprocessorStorageQuery = serde_json::from_value(
        payload["storage_query"].clone()
    )?;
    
    // 4. Circuit phase: Verify storage proofs
    let balances = example_circuit_verify_storage(
        &witnesses,
        &layout_commitment,
        &[query],
    ).map_err(convert_valence_error)?;
    println!("Circuit verified {} storage proofs", balances.len());
    
    // 5. Domain phase: Validate state proofs
    let storage_proof: StorageProof = serde_json::from_value(
        payload["storage_proof"].clone()
    )?;
    let validated_proofs = example_domain_validate_proofs(&[storage_proof], 18500000)
        .map_err(convert_valence_error)?;
    println!("Domain validated {} state proofs", validated_proofs.len());
    
    println!("\nResults:");
    for (i, balance) in balances.iter().enumerate() {
        println!("  Account {}: {} tokens", i + 1, balance);
    }
    
    println!("\nExample 2: Batch Storage Queries");
    println!("------------------------------------");
    
    // 1. Create batch payload
    let batch_payload = create_example_batch_payload();
    println!("Received batch coprocessor JSON payload");
    
    // 2. Controller phase: Process batch
    let batch_witnesses = example_controller_get_witnesses(&batch_payload)
        .map_err(convert_valence_error)?;
    println!("Controller created {} witnesses from batch", batch_witnesses.len());
    
    // 3. Extract batch queries
    let batch_queries: Vec<CoprocessorStorageQuery> = batch_payload["storage_batch"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| serde_json::from_value(item["storage_query"].clone()).unwrap())
        .collect();
    
    // 4. Circuit phase: Verify batch
    let batch_balances = example_circuit_verify_storage(
        &batch_witnesses,
        &layout_commitment,
        &batch_queries,
    ).map_err(convert_valence_error)?;
    
    // 5. Domain phase: Validate batch
    let batch_storage_proofs: Vec<StorageProof> = batch_payload["storage_batch"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| serde_json::from_value(item["storage_proof"].clone()).unwrap())
        .collect();
    let _batch_validated_proofs = example_domain_validate_proofs(&batch_storage_proofs, 18500000)
        .map_err(convert_valence_error)?;
    
    println!("\nBatch Results:");
    for (i, balance) in batch_balances.iter().enumerate() {
        println!("  Account {}: {} tokens", i + 1, balance);
    }
    
    println!("\nIntegration Example Complete!");
    println!("================================");
    println!("This example demonstrates:");
    println!("• Controller: JSON parsing and witness creation");
    println!("• Circuit: Storage proof verification and value extraction");
    println!("• Domain: Ethereum state proof validation");
    println!("• Batch processing for multiple queries");
    println!("• Complete no_std compatibility for coprocessor environment");
    
    println!("\nNext Steps:");
    println!("1. Fork valence-coprocessor-app template");
    println!("2. Add traverse-valence dependency");
    println!("3. Implement controller::get_witnesses() using traverse_valence::controller");
    println!("4. Implement circuit verification using traverse_valence::circuit");
    println!("5. Implement domain validation using traverse_valence::domain");
    
    Ok(())
} 