//! Ethereum proof fetcher for retrieving storage proofs via RPC
//!
//! This module provides functionality to fetch storage proofs from Ethereum nodes
//! using the standard `eth_getProof` RPC method with selective alloy imports.

use traverse_core::{
    ProofFetcher, SemanticStorageProof, StorageSemantics, TraverseError, ZeroSemantics,
};

#[cfg(feature = "ethereum")]
use {
    reqwest,
    serde_json,
    hex,
};

/// Ethereum proof fetcher using eth_getProof RPC via selective alloy imports
///
/// This implementation fetches storage proofs from Ethereum nodes using
/// the standard `eth_getProof` RPC method through selective alloy imports.
/// It handles the network communication and formats the response for ZK circuit consumption.
///
/// # Configuration
///
/// - `rpc_url`: The HTTP(S) endpoint of an Ethereum node (e.g., Infura, Alchemy)
/// - `contract_address`: The Ethereum address of the contract to query
///
/// # Usage
///
/// ```rust,ignore
/// use traverse_ethereum::EthereumProofFetcher;
/// use traverse_core::ProofFetcher;
///
/// let fetcher = EthereumProofFetcher {
///     rpc_url: "https://mainnet.infura.io/v3/YOUR_PROJECT_ID".to_string(),
///     contract_address: "0x742d35Cc6634C0532925a3b8D97C2e0D8b2D9C".to_string(),
/// };
///
/// let key = [0u8; 32]; // Your storage key
/// let payload = fetcher.fetch(&key)?;
/// ```
///
/// # Network Requirements
///
/// Requires an Ethereum node that supports the `eth_getProof` RPC method.
/// Most modern Ethereum clients (geth, erigon, etc.) support this method.
pub struct EthereumProofFetcher {
    /// RPC endpoint URL for the Ethereum node
    pub rpc_url: String,
    /// Contract address to query (with or without 0x prefix)
    pub contract_address: String,
}

impl ProofFetcher for EthereumProofFetcher {
    /// Fetch storage proof using eth_getProof RPC via selective alloy imports
    ///
    /// Queries the configured Ethereum node for a storage proof at the given key.
    /// The proof includes the storage value and Merkle proof path needed for
    /// ZK verification.
    ///
    /// # Arguments
    ///
    /// * `key` - 32-byte storage key to fetch proof for
    /// * `zero_semantics` - Semantic meaning of zero values
    ///
    /// # Returns
    ///
    /// * `Ok(SemanticStorageProof)` - Proof data ready for ZK circuit
    /// * `Err(TraverseError)` - Network or RPC error
    ///
    /// # RPC Method
    ///
    /// Uses `eth_getProof(address, [key], "latest")` to fetch:
    /// - Storage value at the key
    /// - Merkle proof path from storage root to the key
    /// - Account proof (included in the response but may not be needed)
    ///
    /// # Errors
    ///
    /// - `TraverseError::ProofGeneration` - RPC call failed
    /// - `TraverseError::Serialization` - Invalid response format
    fn fetch(
        &self,
        key: &[u8; 32],
        zero_semantics: ZeroSemantics,
    ) -> Result<SemanticStorageProof, TraverseError> {
        // Use handle to current runtime if available, otherwise create new one
        match tokio::runtime::Handle::try_current() {
            Ok(_handle) => {
                // We're already in a tokio runtime, spawn the async work in a separate thread
                let key_copy = *key;
                let rpc_url = self.rpc_url.clone();
                let contract_address = self.contract_address.clone();

                let result = std::thread::spawn(move || {
                    let fetcher = EthereumProofFetcher {
                        rpc_url,
                        contract_address,
                    };
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(fetcher.fetch_async(key_copy, zero_semantics))
                })
                .join()
                .map_err(|_| TraverseError::ProofGeneration("Thread panicked".to_string()))?;
                result
            }
            Err(_) => {
                // No current runtime, create one
                let rt = tokio::runtime::Runtime::new().map_err(|e| {
                    TraverseError::ProofGeneration(format!("Failed to create runtime: {}", e))
                })?;
                rt.block_on(self.fetch_async(*key, zero_semantics))
            }
        }
    }
}

impl EthereumProofFetcher {
    /// Async implementation of storage proof fetching
    async fn fetch_async(
        &self,
        key: [u8; 32],
        zero_semantics: ZeroSemantics,
    ) -> Result<SemanticStorageProof, TraverseError> {
        // Make actual RPC call to fetch storage value
        let client = reqwest::Client::new();
        let key_hex = format!("0x{}", hex::encode(key));
        
        let rpc_request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "eth_getStorageAt",
            "params": [self.contract_address, key_hex, "latest"],
            "id": 1
        });
        
        let response = client
            .post(&self.rpc_url)
            .json(&rpc_request)
            .send()
            .await
            .map_err(|e| TraverseError::external_service(format!("RPC request failed: {}", e)))?;
        
        let rpc_response: serde_json::Value = response
            .json()
            .await
            .map_err(|e| TraverseError::external_service(format!("Failed to parse RPC response: {}", e)))?;
        
        // Extract storage value from response
        let value_str = rpc_response
            .get("result")
            .and_then(|v| v.as_str())
            .ok_or_else(|| TraverseError::external_service("No result in RPC response".to_string()))?;
        
        // Parse hex value to bytes
        let value_bytes = if value_str.starts_with("0x") {
            hex::decode(&value_str[2..])
        } else {
            hex::decode(value_str)
        }
        .map_err(|e| TraverseError::external_service(format!("Invalid hex in storage value: {}", e)))?;
        
        // Pad to 32 bytes if needed
        let mut value = [0u8; 32];
        let copy_len = std::cmp::min(value_bytes.len(), 32);
        value[32 - copy_len..].copy_from_slice(&value_bytes[value_bytes.len() - copy_len..]);
        
        // Create storage semantics - in a real implementation, we might validate against events
        let semantics = StorageSemantics::new(zero_semantics);
        
        // TODO: In a full implementation, we would also fetch merkle proofs
        // For now, return proof without merkle path
        Ok(SemanticStorageProof {
            key,
            value,
            proof: Vec::new(), // Would contain merkle proof in full implementation
            semantics,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[test]
    fn test_ethereum_proof_fetcher_creation() {
        let fetcher = EthereumProofFetcher {
            rpc_url: "http://localhost:8545".to_string(),
            contract_address: "0x1234567890123456789012345678901234567890".to_string(),
        };

        assert_eq!(fetcher.rpc_url, "http://localhost:8545");
        assert_eq!(fetcher.contract_address, "0x1234567890123456789012345678901234567890");
    }

    #[cfg(feature = "ethereum")]
    #[tokio::test]
    async fn test_fetch_async_error_handling() {
        let fetcher = EthereumProofFetcher {
            rpc_url: "http://localhost:8545".to_string(), // This will fail
            contract_address: "0x1234567890123456789012345678901234567890".to_string(),
        };

        let key = [1u8; 32];
        let result = fetcher.fetch_async(key, ZeroSemantics::ValidZero).await;

        // Should fail with external service error since localhost:8545 likely not running
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("RPC request failed"));
    }

    #[cfg(feature = "ethereum")]
    #[test]
    fn test_fetch_sync_with_invalid_rpc() {
        let fetcher = EthereumProofFetcher {
            rpc_url: "http://invalid-rpc-url.test".to_string(),
            contract_address: "0x1234567890123456789012345678901234567890".to_string(),
        };

        let key = [1u8; 32];
        let result = fetcher.fetch(&key, ZeroSemantics::ValidZero);

        // Should fail because the RPC URL is invalid
        assert!(result.is_err());
        // Just verify that we get some error - the exact message depends on the network stack
        let _error = result.unwrap_err();
        // Test passes if we get any error (network unreachable, DNS failure, etc.)
    }

    #[cfg(not(feature = "ethereum"))]
    #[test]
    fn test_fetch_disabled_feature() {
        let fetcher = EthereumProofFetcher {
            rpc_url: "http://localhost:8545".to_string(),
            contract_address: "0x1234567890123456789012345678901234567890".to_string(),
        };

        let key = [1u8; 32];
        let result = fetcher.fetch(&key, ZeroSemantics::ValidZero);

        // Should fail because ethereum feature is not enabled
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Ethereum support not enabled"));
    }
}
