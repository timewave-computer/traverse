//! Proof command implementation
//! 
//! Handles storage proof generation for ZK coprocessor integration.

use std::path::Path;
use anyhow::Result;
use tracing::info;

#[cfg(feature = "client")]
use valence_domain_clients::clients::ethereum::EthereumClient;

#[cfg(feature = "client")]
use valence_domain_clients::evm::request_provider_client::RequestProviderClient;

#[cfg(feature = "client")]
use alloy::providers::Provider;

#[cfg(not(feature = "client"))]
use traverse_ethereum::EthereumProofFetcher;

#[cfg(not(feature = "client"))]
use traverse_core::ProofFetcher;

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
    
    #[cfg(feature = "client")]
    {
        // Use valence-domain-clients for actual proof fetching
        info!("Using valence-domain-clients for proof generation");
        
        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(async {
            // For now, use a dummy mnemonic since we only need to make RPC calls
            // In a real implementation, this could be configurable or optional
            let dummy_mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
            
            // Create an Ethereum client using valence-domain-clients  
            let client = EthereumClient::new(rpc, dummy_mnemonic, None)
                .map_err(|e| anyhow::anyhow!("Failed to create Ethereum client: {}", e))?;
            
            // Get the provider to make raw RPC calls
            let provider = client.get_request_provider().await
                .map_err(|e| anyhow::anyhow!("Failed to get provider: {}", e))?;
            
            // Parse contract address
            let contract_addr: alloy::primitives::Address = contract.parse()
                .map_err(|e| anyhow::anyhow!("Invalid contract address: {}", e))?;
            
            // Convert slot to alloy format
            let slot_b256 = alloy::primitives::B256::from_slice(&slot_array);
            
            // Get storage proof using eth_getProof
            let proof_response = provider.get_proof(contract_addr, vec![slot_b256])
                .await.map_err(|e| anyhow::anyhow!("Failed to get proof: {}", e))?;
            
            // Convert to our CoprocessorQueryPayload format
            if proof_response.storage_proof.is_empty() {
                return Err(anyhow::anyhow!("No storage proof returned"));
            }
            
            let storage_proof = &proof_response.storage_proof[0];
            
            // Convert value to bytes
            let value_bytes = storage_proof.value.to_be_bytes::<32>();
            
            // Convert proof nodes - note that RLP-encoded nodes may be longer than 32 bytes
            // For now, we'll just store the raw proof data as hex strings in a Vec<u8>
            let proof_data: Vec<u8> = storage_proof.proof.iter()
                .flat_map(|node| node.as_ref().iter().copied())
                .collect();
            
            // For the CoprocessorQueryPayload, we need Vec<[u8; 32]> but RLP nodes can be variable length
            // This is a limitation of the current format - in practice, you'd need a more flexible format
            let proof_nodes: Vec<[u8; 32]> = storage_proof.proof.iter()
                .filter_map(|node| {
                    if node.len() == 32 {
                        let mut array = [0u8; 32];
                        array.copy_from_slice(node);
                        Some(array)
                    } else {
                        None // Skip non-32-byte nodes for now
                    }
                })
                .collect();
            
            let payload = traverse_core::CoprocessorQueryPayload {
                key: slot_array,
                value: value_bytes,
                proof: proof_nodes.clone(),
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
            println!("  Proof nodes: {} (filtered to 32-byte nodes)", proof_nodes.len());
            println!("  Raw proof data: {} bytes", proof_data.len());
            
            if proof_nodes.len() != storage_proof.proof.len() {
                println!("  Note: Some proof nodes were filtered out due to length != 32 bytes");
                println!("        Total proof nodes from RPC: {}", storage_proof.proof.len());
            }
            
            Ok::<(), anyhow::Error>(())
        })?;
    }
    
    #[cfg(not(feature = "client"))]
    {
        // Fallback using traverse-ethereum's proof fetcher
        info!("Using traverse-ethereum proof fetcher (client feature not enabled)");
        
        let proof_fetcher = EthereumProofFetcher {
            rpc_url: rpc.to_string(),
            contract_address: contract.to_string(),
        };
        
        // Use the improved proof fetcher from traverse-ethereum
        let payload = proof_fetcher.fetch(&slot_array)
            .map_err(|e| anyhow::anyhow!("Failed to fetch proof: {}", e))?;
        
        let json = serde_json::to_string_pretty(&payload)?;
        
        if let Some(output_path) = output {
            std::fs::write(output_path, &json)?;
            println!("Storage proof written to {}", output_path.display());
        } else {
            println!("Storage proof generated:");
            println!("{}", json);
        }
        
        println!();
        println!("Proof generation completed using traverse-ethereum");
        println!("  Contract: {}", contract);
        println!("  RPC: {}", rpc);
        println!("  Slot: 0x{}", hex::encode(slot_array));
        println!();
        println!("For enhanced features, rebuild with:");
        println!("  cargo build --features client");
    }
    
    Ok(())
} 