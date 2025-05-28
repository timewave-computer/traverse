//! Proof command implementation
//! 
//! Handles storage proof generation for ZK coprocessor integration.

use std::path::Path;
use anyhow::Result;
use tracing::info;
use traverse_ethereum::EthereumProofFetcher;
use traverse_core::CoprocessorQueryPayload;

#[cfg(feature = "valence")]
use valence_domain_clients::evm::{GenericClient, chains::ethereum::Ethereum};

/// Execute generate-proof command
pub fn cmd_generate_proof(
    slot: &str,
    rpc: &str,
    contract: &str,
    output: Option<&Path>,
) -> Result<()> {
    info!("Generating proof for slot {} from contract {} via {}", slot, contract, rpc);
    
    // Parse the slot as hex
    let slot_hex = slot.strip_prefix("0x").unwrap_or(slot);
    let slot_bytes = hex::decode(slot_hex)
        .map_err(|e| anyhow::anyhow!("Invalid hex slot: {}", e))?;
    
    if slot_bytes.len() != 32 {
        return Err(anyhow::anyhow!("Slot must be exactly 32 bytes (64 hex chars)"));
    }
    
    let mut slot_array = [0u8; 32];
    slot_array.copy_from_slice(&slot_bytes);
    
    #[cfg(feature = "valence")]
    {
        // Use valence-domain-clients for actual proof fetching
        info!("Using valence-domain-clients for proof generation");
        
        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(async {
            // Create an Ethereum client using valence-domain-clients
            let client = GenericClient::<Ethereum>::new(rpc)?;
            
            // Get storage proof using eth_getProof
            let proof_response = client.get_proof(
                contract.parse()?,
                vec![format!("0x{}", hex::encode(slot_array))],
                None, // Use latest block
            ).await?;
            
            // Convert to our CoprocessorQueryPayload format
            let storage_proof = &proof_response.storage_proof[0];
            
            let value_bytes = if storage_proof.value == "0x0" || storage_proof.value == "0x" {
                [0u8; 32]
            } else {
                let value_hex = storage_proof.value.strip_prefix("0x").unwrap_or(&storage_proof.value);
                let mut value_array = [0u8; 32];
                let value_vec = hex::decode(value_hex)?;
                let copy_len = std::cmp::min(value_vec.len(), 32);
                value_array[32 - copy_len..].copy_from_slice(&value_vec[value_vec.len() - copy_len..]);
                value_array
            };
            
            let proof_nodes: Vec<[u8; 32]> = storage_proof.proof.iter()
                .filter_map(|node| {
                    let node_hex = node.strip_prefix("0x").unwrap_or(node);
                    if node_hex.len() == 64 {
                        let mut array = [0u8; 32];
                        if let Ok(bytes) = hex::decode(node_hex) {
                            if bytes.len() == 32 {
                                array.copy_from_slice(&bytes);
                                return Some(array);
                            }
                        }
                    }
                    None
                })
                .collect();
            
            let payload = CoprocessorQueryPayload {
                key: slot_array,
                value: value_bytes,
                proof: proof_nodes,
            };
            
            let json = serde_json::to_string_pretty(&payload)?;
            
            if let Some(output_path) = output {
                std::fs::write(output_path, &json)?;
                println!("Storage proof written to {}", output_path.display());
            } else {
                println!("Storage proof generated:");
                println!("{}", json);
            }
            
            println!();
            println!("Proof generation completed using valence-domain-clients");
            println!("  Contract: {}", contract);
            println!("  Slot: 0x{}", hex::encode(slot_array));
            println!("  Value: 0x{}", hex::encode(value_bytes));
            println!("  Proof nodes: {}", proof_nodes.len());
            
            Ok::<(), anyhow::Error>(())
        })?;
    }
    
    #[cfg(not(feature = "valence"))]
    {
        // Fallback to mock implementation
        info!("Using mock implementation (valence feature not enabled)");
        
        let _proof_fetcher = EthereumProofFetcher {
            rpc_url: rpc.to_string(),
            contract_address: contract.to_string(),
        };
        
        println!("Generate-proof command structure (MOCK MODE):");
        println!("  Contract: {}", contract);
        println!("  RPC: {}", rpc);
        println!("  Slot: 0x{}", hex::encode(slot_array));
        println!();
        println!("To enable live proof generation, rebuild with:");
        println!("  cargo build --features valence");
        println!();
        
        // Create a mock CoprocessorQueryPayload to show the expected output format
        let mock_payload = CoprocessorQueryPayload {
            key: slot_array,
            value: [0u8; 32], // Would be actual storage value from RPC
            proof: vec![], // Would be actual proof nodes from RPC
        };
        
        let json = serde_json::to_string_pretty(&mock_payload)?;
        
        if let Some(output_path) = output {
            std::fs::write(output_path, &json)?;
            println!("Mock payload written to {}", output_path.display());
        } else {
            println!("Mock CoprocessorQueryPayload structure:");
            println!("{}", json);
        }
        
        println!();
        println!("Note: This is a mock implementation. For live proof generation,");
        println!("  rebuild with --features valence flag.");
    }
    
    Ok(())
} 