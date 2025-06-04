//! Live USDT storage proof example using your eth_getProof query
//! 
//! This example demonstrates the complete Traverse flow using a real eth_getProof
//! response from the USDT contract on Ethereum mainnet.

use serde_json::json;
use traverse_core::{LayoutInfo, StorageEntry, TypeInfo, CoprocessorQueryPayload};
use traverse_valence::{controller, circuit, ValenceError};
use traverse_valence::{CoprocessorStorageQuery, StorageProof};

/// Your actual eth_getProof response from USDT contract
/// Contract: 0xdAC17F958D2ee523a2206206994597C13D831ec7
/// Slot: 0x0000000000000000000000000000000000000000000000000000000000000000
/// Block: 0xf2232774da1c842a03bfaf8805b4b372fc6b6e6a508aa9b9a11c5848ade082de
fn get_live_usdt_proof_response() -> serde_json::Value {
    json!({
        "jsonrpc": "2.0",
        "id": 1,
        "result": {
            "address": "0xdac17f958d2ee523a2206206994597c13d831ec7",
            "storageProof": [{
                "key": "0x0000000000000000000000000000000000000000000000000000000000000000",
                "value": "0xc6cde7c39eb2f0f0095f41570af89efc2c1ea828",
                "proof": [
                    "0xf90211a07c3e80da05fb73f6cf77b7817b2b793831e0285950ea3318a63883e4c7cf6e31a0e730c4e0b164130e31c68d36b91cfbdd0f586c58113f09e09b27389d28e6b5a9a0f9f2e2c36090febb43fce28072a4ba05a01dbe8c16d7c143018004a02a11f0c2a0d4c2d9ed6823e749f4ae3dc41d8ca55897391f47a62bc9c38e0ece72a7effc3da0bb60065f74d3b9625679fc04376c128569b8644703e0865054cf5196ad8e5d6ca04400cc05aeeb0afcb3ac2632dde8bfd38a2bc689cc0b7bf6e01d1187be452d7da0c4457aac5d22764a301cf7ca5fddb27aaa5b6d8496d95ec80678b988477a31a3a0967fb2e152f212904df8ff2519f1a7b6479f073ddf50ef12cb32beba4babf2f2a0bd310666fe755d4fb74a3ec1fdc6572cd816facfde3e993071704da193537aafa050c333457df57c0a6c2ab68bdf1bfc1107735c7ad6fdfb06d4f292ddba1ca26aa017249ba734841584ba393a9ff055a5419f0d103bb5a43918afa1e46cfe341f94a0cb241a9051d4326dae376c939b24ea71cf57b5b65d3a211c27325f63bcbe1072a05a270456728166d5ac61a5d995e67fc1b070ea6912ca8cc79e86ca873d2ef3bca061957b42d5fc750b2c58cc1a7180b061133db5b5614cf1cc56491045fcb68daca0455af84adb1947601f88cd351f0cdf62a808d5272a9a1b8ec82b7780843736efa09c6c4fc465ec7f952d94bf24d7c2ba3919ae26dec746f242fe2d02bd90b6143180",
                    "0xf90211a0c14f11e43674494b6e5c25b1f9feef755e949f02a8d5318c32fd319e80b3f723a0e5a6632f411122b264560f8391321da64792890e2c3b7724baf176abd9bf05f3a06f3299ecfe816f317f742334b727a00739cac4d642a23a2e5e51eb8b5dc88705a01c66fd0ddcf9d59c10fbb905104b142291c2e23c42e759c973c24eee0f2a81eaa0ce56eba87c7a44958521fc832698cc0b6413c84772eaf1426e611a3e181bf707a0ca092e91fe63c8f5cafdd4ecfe0aecb85db1c9f678c3b816425e85b1a9333da014bbf0433021cdbab49b7f77e8ffae9295453c70337bf7793a2407190e2ac558a0b633e82ae70eb1804d56d80b0504402c67c5bbffc1b23cc7c508e543b8218029a035006bc3b795e778e6c0d44c4881ff3af7e06e7fddb35e3975c97629fa180592a033e2c3c82041e3e4261b3403cc94382f8c5502fc4f7516aff31a5da36dd415c9a05c22d544595472440e8f4fc9b8251b27915eb64a701ec722a864c034c5b27b2ea0e5cdf6d4303f884812d750877730ef8d8e510a1acdd8936a7ceb5ac87b530c24a00152169a2803000328ed861f95ff1231de38e368f2d374bfb1be195cd4862a22a0099dcaa283fed285a3c06ab9db95d43f35bd1a23f90931eff6de64f7a458f313a03edd66d6d8e5198c9c131ccc3729475fb9559cb88691191ad4a22a5f6e10d72aa0ea1429f06a8b0620a5511c2c1bc3fcf2501acfe37253e15c3c98cbf23a8d932d80"
                ]
            }]
        }
    })
}

/// Create USDT contract layout for storage slot 0 analysis
fn create_usdt_layout() -> LayoutInfo {
    let types = vec![
        TypeInfo {
            label: "t_address".to_string(),
            encoding: "inplace".to_string(),
            number_of_bytes: "20".to_string(),
            base: None,
            key: None,
            value: None,
        },
        TypeInfo {
            label: "t_bool".to_string(),
            encoding: "inplace".to_string(),
            number_of_bytes: "1".to_string(),
            base: None,
            key: None,
            value: None,
        },
        TypeInfo {
            label: "t_uint256".to_string(),
            encoding: "inplace".to_string(),
            number_of_bytes: "32".to_string(),
            base: None,
            key: None,
            value: None,
        },
    ];

    LayoutInfo {
        contract_name: "TetherToken".to_string(),
        storage: vec![
            StorageEntry {
                label: "slot0_data".to_string(), // USDT slot 0 contains packed administrative data
                slot: "0".to_string(),
                offset: 0,
                type_name: "t_uint256".to_string(),
            }
        ],
        types,
    }
}

/// Helper function to pad hex values to 32 bytes (64 hex chars)
fn pad_hex_to_32_bytes(hex_str: &str) -> String {
    let hex_clean = hex_str.strip_prefix("0x").unwrap_or(hex_str);
    
    if hex_clean.len() < 64 {
        // Pad with leading zeros to make it 64 characters (32 bytes)
        format!("{:0>64}", hex_clean)
    } else {
        hex_clean.to_string()
    }
}

// Convert ValenceError to Box<dyn std::error::Error>
fn convert_valence_error(err: ValenceError) -> Box<dyn std::error::Error> {
    Box::new(std::io::Error::new(std::io::ErrorKind::Other, format!("{}", err)))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("USDT Live Storage Proof Example");
    println!("=====================================");
    println!();
    
    // Step 1: Parse the live eth_getProof response
    println!("1. Processing your live eth_getProof response...");
    let proof_response = get_live_usdt_proof_response();
    let storage_proof_data = &proof_response["result"]["storageProof"][0];
    
    // Pad the storage value to 32 bytes
    let raw_value = storage_proof_data["value"].as_str().unwrap();
    let padded_value = pad_hex_to_32_bytes(raw_value);
    
    let storage_proof = StorageProof {
        key: storage_proof_data["key"].as_str().unwrap().to_string(),
        value: format!("0x{}", padded_value),
        proof: storage_proof_data["proof"].as_array().unwrap()
            .iter()
            .map(|v| v.as_str().unwrap().to_string())
            .collect(),
    };
    
    println!("   [OK] Contract: 0xdAC17F958D2ee523a2206206994597C13D831ec7 (USDT)");
    println!("   [OK] Storage Slot: {}", storage_proof.key);
    println!("   [OK] Storage Value (original): {}", raw_value);
    println!("   [OK] Storage Value (padded): {}", storage_proof.value);
    println!("   [OK] Proof Nodes: {} merkle tree nodes", storage_proof.proof.len());
    println!();
    
    // Step 2: Set up Traverse layout
    println!("2. Setting up USDT contract layout...");
    let layout = create_usdt_layout();
    let layout_commitment = layout.commitment();
    
    println!("   [OK] Layout loaded for USDT contract");
    println!("   [OK] Layout commitment: 0x{}", hex::encode(layout_commitment));
    println!();
    
    // Step 3: Create storage query for the specific slot
    println!("3. Creating coprocessor storage query...");
    let storage_query = CoprocessorStorageQuery {
        query: "slot0_data".to_string(),
        storage_key: storage_proof.key.clone(),
        layout_commitment: hex::encode(layout_commitment),
        field_size: Some(32), // Full slot
        offset: None, // Read entire slot
    };
    
    println!("   [OK] Query: {}", storage_query.query);
    println!("   [OK] Storage key: {}", storage_query.storage_key);
    println!();
    
    // Step 4: Create witness from the proof data
    println!("4. Creating storage witness from proof...");
    let json_payload = json!({
        "storage_query": {
            "query": storage_query.query,
            "storage_key": storage_query.storage_key,
            "layout_commitment": storage_query.layout_commitment,
            "field_size": storage_query.field_size,
            "offset": storage_query.offset
        },
        "storage_proof": {
            "key": storage_proof.key,
            "value": storage_proof.value,
            "proof": storage_proof.proof
        }
    });
    
    let witness = controller::create_storage_witness(&json_payload)
        .map_err(convert_valence_error)?;
    println!("   [OK] Storage witness created");
    println!();
    
    // Step 5: Verify the proof in circuit context
    println!("5. Verifying storage proof (circuit simulation)...");
    let verification_result = circuit::verify_storage_proof(
        &witness,
        &layout_commitment,
        &storage_query,
    ).map_err(convert_valence_error)?;
    
    println!("   [OK] Proof verification passed");
    println!("   [OK] Extracted value length: {} bytes", verification_result.len());
    println!("   [OK] Raw value: 0x{}", hex::encode(&verification_result));
    println!();
    
    // Step 6: Analyze the storage value
    println!("6. Analyzing USDT slot 0 data...");
    let storage_value_hex = raw_value.strip_prefix("0x").unwrap_or(raw_value);
    let storage_bytes = hex::decode(storage_value_hex)?;
    
    // USDT slot 0 typically contains administrative data (owner, paused flags, etc.)
    println!("   Storage Analysis:");
    println!("      - Full slot data: 0x{}", hex::encode(&storage_bytes));
    println!("      - Likely contains: owner address, administrative flags");
    println!("      - Last 20 bytes (address): 0x{}", hex::encode(&storage_bytes[storage_bytes.len().saturating_sub(20)..]));
    println!();
    
    // Step 7: Convert to CoprocessorQueryPayload format
    println!("7. Converting to coprocessor format...");
    let mut key_bytes = [0u8; 32];
    let key_decoded = hex::decode(storage_proof.key.strip_prefix("0x").unwrap_or(&storage_proof.key))?;
    key_bytes.copy_from_slice(&key_decoded);
    
    let mut value_bytes = [0u8; 32];
    let padded_value_decoded = hex::decode(padded_value)?;
    value_bytes.copy_from_slice(&padded_value_decoded);
    
    let proof_nodes: Vec<[u8; 32]> = storage_proof.proof
        .iter()
        .filter_map(|node_hex| {
            let clean_hex = node_hex.strip_prefix("0x").unwrap_or(node_hex);
            if clean_hex.len() >= 64 {
                // Take first 32 bytes if proof node is longer than 32 bytes
                let mut node_bytes = [0u8; 32];
                if let Ok(decoded) = hex::decode(&clean_hex[..64]) {
                    node_bytes.copy_from_slice(&decoded);
                    Some(node_bytes)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();
    
    let coprocessor_payload = CoprocessorQueryPayload {
        key: key_bytes,
        value: value_bytes,
        proof: proof_nodes,
    };
    
    println!("   [OK] Coprocessor payload created");
    println!("   [OK] Key: 0x{}", hex::encode(coprocessor_payload.key));
    println!("   [OK] Value: 0x{}", hex::encode(coprocessor_payload.value));
    println!("   [OK] Proof nodes: {} (truncated to 32-byte hashes)", coprocessor_payload.proof.len());
    println!();
    
    // Step 8: Show complete integration path
    println!("8. Complete Traverse Integration");
    println!("===================================");
    println!("Your eth_getProof query successfully flows through:");
    println!("   1. [OK] RPC Response -> StorageProof");
    println!("   2. [OK] StorageProof -> CoprocessorStorageQuery");  
    println!("   3. [OK] Layout Compilation -> LayoutInfo");
    println!("   4. [OK] Witness Generation -> MockWitness");
    println!("   5. [OK] Circuit Verification -> Verified Value");
    println!("   6. [OK] Final Output -> CoprocessorQueryPayload");
    println!();
    
    println!("Next Steps:");
    println!("   - Use this pattern for any storage slot on any contract");
    println!("   - Integrate with valence-coprocessor for ZK circuit generation");
    println!("   - Scale to batch processing for multiple storage queries");
    println!("   - Add contract-specific layout compilation from ABI");
    println!();
    
    // Save the complete payload for reference
    let output_json = serde_json::to_string_pretty(&json!({
        "traverse_flow_demo": {
            "input": {
                "contract": "0xdAC17F958D2ee523a2206206994597C13D831ec7",
                "slot": storage_proof.key,
                "block": "0xf2232774da1c842a03bfaf8805b4b372fc6b6e6a508aa9b9a11c5848ade082de",
                "rpc_method": "eth_getProof"
            },
            "traverse_processing": {
                "layout_commitment": hex::encode(layout_commitment),
                "storage_query": storage_query,
                "verification_status": "passed"
            },
            "output": {
                "coprocessor_payload": {
                    "key": hex::encode(coprocessor_payload.key),
                    "value": hex::encode(coprocessor_payload.value),
                    "proof_nodes": coprocessor_payload.proof.len()
                }
            }
        }
    }))?;
    
    std::fs::write("usdt_traverse_flow.json", output_json)?;
    println!("Complete flow data saved to: usdt_traverse_flow.json");
    
    Ok(())
} 