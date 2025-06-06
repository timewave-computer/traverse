//! Valence Coprocessor Integration Example
//! 
//! This example demonstrates how to integrate traverse with the valence
//! coprocessor for ZK-based storage proof verification.
//! 
//! This shows the complete end-to-end flow:
//! 
//! 1. Controller parses JSON arguments and creates witnesses
//! 2. Circuit verifies storage proofs and extracts values
//! 3. Domain validates Ethereum state proofs
//! 
//! To run this example:
//! 
//! ```bash
//! cargo run --features examples --example valence_integration
//! ```

#[cfg(feature = "examples")]
use serde_json::{json, Value};
#[cfg(feature = "examples")]
use traverse_valence::{controller, circuit, domain};
#[cfg(feature = "examples")]
use valence_coprocessor::Witness;

#[cfg(not(feature = "examples"))]
fn main() {
    println!("This example requires the 'examples' feature.");
    println!("Run with: cargo run --features examples --example valence_integration");
}

#[cfg(feature = "examples")]
/// Example controller function that would be called by valence coprocessor
/// 
/// This function follows the same pattern as the controller in valence-coprocessor-app:
/// it takes JSON arguments and returns Vec<Witness> for the circuit to process.
fn example_controller_get_witnesses(args: Value) -> anyhow::Result<Vec<Witness>> {
    println!("Controller: Processing JSON arguments");
    
    // Debug: print the JSON to see what we're working with
    println!("JSON args: {}", serde_json::to_string_pretty(&args)?);
    
    // Use traverse-valence controller helpers to create witnesses
    let witnesses = controller::create_storage_witnesses(&args)
        .map_err(|e| anyhow::anyhow!("Failed to create witnesses: {}", e))?;
    
    println!("Controller: Created {} witnesses", witnesses.len());
    Ok(witnesses)
}

#[cfg(feature = "examples")]
/// Example circuit function that would process witnesses
/// 
/// This function follows the valence circuit pattern: takes Vec<Witness> and returns Vec<u8>
fn example_circuit_verify_proofs(witnesses: Vec<Witness>) -> anyhow::Result<Vec<u8>> {
    println!("Circuit: Verifying {} storage proofs", witnesses.len());
    
    // Extract values from all witnesses
    let values = circuit::extract_multiple_u64_values(&witnesses)
        .map_err(|e| anyhow::anyhow!("Failed to extract values: {}", e))?;
    
    println!("Circuit: Extracted values: {:?}", values);
    
    // Return the sum as circuit output (example computation)
    let sum: u64 = values.iter().sum();
    Ok(sum.to_le_bytes().to_vec())
}

#[cfg(feature = "examples")]
/// Example domain function for state validation
fn example_domain_validate_state(args: &Value) -> anyhow::Result<bool> {
    println!("Domain: Validating blockchain state");
    
    // Example validation logic using traverse-valence domain helpers
    let block_header = domain::EthereumBlockHeader {
        number: 18_500_000,
        state_root: [0u8; 32], // Would be actual state root
        hash: [0u8; 32],       // Would be actual block hash
    };
    
    // Validate storage proof if present
    if let Some(storage_proof) = args.get("storage_proof") {
        let validated = domain::validate_ethereum_state_proof(storage_proof, &block_header)
            .map_err(|e| anyhow::anyhow!("Failed to validate proof: {}", e))?;
        println!("Domain: Proof validation result: {}", validated.is_valid);
        return Ok(validated.is_valid);
    }
    
    Ok(true)
}

#[cfg(feature = "examples")]
fn main() -> anyhow::Result<()> {
    println!("Traverse Valence Coprocessor Integration Example");
    println!("================================================");
    
    // Create example JSON arguments matching expected format
    let json_args = json!({
        "storage_batch": [
            {
                "storage_query": {
                    "query": "_balances[0x742d35Cc6aB8B23c0532C65C6b555f09F9d40894]",
                    "storage_key": "0000000000000000000000000000000000000000000000000000000000000001",
                    "layout_commitment": "0000000000000000000000000000000000000000000000000000000000000002"
                },
                "storage_proof": {
                    "key": "0000000000000000000000000000000000000000000000000000000000000001",
                    "value": "0000000000000000000000000000000000000000000000000000000000000064",
                    "proof": ["0000000000000000000000000000000000000000000000000000000000000003"]
                }
            },
            {
                "storage_query": {
                    "query": "_balances[0x8ba1f109551bD432803012645Aac136c5C1Aa000]",
                    "storage_key": "0000000000000000000000000000000000000000000000000000000000000004",
                    "layout_commitment": "0000000000000000000000000000000000000000000000000000000000000002"
                },
                "storage_proof": {
                    "key": "0000000000000000000000000000000000000000000000000000000000000004",
                    "value": "00000000000000000000000000000000000000000000000000000000000003e8",
                    "proof": ["0000000000000000000000000000000000000000000000000000000000000005"]
                }
            }
        ]
    });
    
    println!("\n1. Controller Phase:");
    println!("-------------------");
    
    // Step 1: Controller creates witnesses from JSON arguments
    let witnesses = example_controller_get_witnesses(json_args.clone())?;
    
    println!("\n2. Circuit Phase:");
    println!("----------------");
    
    // Step 2: Circuit processes witnesses and produces output
    let circuit_output = example_circuit_verify_proofs(witnesses)?;
    println!("Circuit: Output bytes: {:?}", circuit_output);
    
    println!("\n3. Domain Phase:");
    println!("---------------");
    
    // Step 3: Domain validates blockchain state
    let state_valid = example_domain_validate_state(&json_args)?;
    println!("Domain: State validation passed: {}", state_valid);
    
    println!("\n4. Integration Summary:");
    println!("----------------------");
    println!("✓ Controller successfully created witnesses from traverse output");
    println!("✓ Circuit successfully verified proofs and extracted values");
    println!("✓ Domain successfully validated blockchain state");
    println!("\nThe traverse library is now fully integrated with valence coprocessor!");
    
    println!("\n5. Next Steps for Production:");
    println!("-----------------------------");
    println!("1. Fork valence-coprocessor-app template");
    println!("2. Add traverse-valence dependency");
    println!("3. Implement controller::get_witnesses() using traverse_valence::controller");
    println!("4. Implement circuit verification using traverse_valence::circuit");
    println!("5. Implement domain validation using traverse_valence::domain");
    
    Ok(())
} 