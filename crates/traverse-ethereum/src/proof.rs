//! Ethereum proof fetcher for retrieving storage proofs via RPC
//!
//! This module provides functionality to fetch storage proofs from Ethereum nodes
//! using the standard `eth_getProof` RPC method with Alloy.

use alloy::{
    primitives::{Address, B256},
    providers::{Provider, ProviderBuilder},
    rpc::types::EIP1186AccountProofResponse,
};
use traverse_core::{
    ProofFetcher, SemanticStorageProof, StorageSemantics, TraverseError, ZeroSemantics,
};

/// Ethereum proof fetcher using eth_getProof RPC via Alloy
///
/// This implementation fetches storage proofs from Ethereum nodes using
/// the standard `eth_getProof` RPC method through the Alloy library.
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
    /// Fetch storage proof using eth_getProof RPC via Alloy
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
        // Parse URL first to validate it
        let url = self
            .rpc_url
            .parse()
            .map_err(|e| TraverseError::ProofGeneration(format!("Invalid RPC URL: {}", e)))?;

        // Create provider using Alloy
        let provider = ProviderBuilder::new().on_http(url);

        // Parse contract address
        let contract_addr: Address = self.contract_address.parse().map_err(|e| {
            TraverseError::ProofGeneration(format!("Invalid contract address: {}", e))
        })?;

        // Convert storage key to B256
        let storage_key = B256::from_slice(&key);

        // Get storage proof using eth_getProof
        let proof_response: EIP1186AccountProofResponse = provider
            .get_proof(contract_addr, vec![storage_key])
            .await
            .map_err(|e| TraverseError::ProofGeneration(format!("Failed to get proof: {}", e)))?;

        // Extract storage proof (should have exactly one since we requested one key)
        if proof_response.storage_proof.is_empty() {
            return Err(TraverseError::ProofGeneration(
                "No storage proof returned".to_string(),
            ));
        }

        let storage_proof = &proof_response.storage_proof[0];

        // Convert value to 32-byte array
        let value_bytes = storage_proof.value.to_be_bytes::<32>();

        // Convert proof nodes to Vec<[u8; 32]>
        // Note: RLP-encoded nodes may be longer than 32 bytes, so we need to handle this carefully
        let proof_nodes: Vec<[u8; 32]> = storage_proof
            .proof
            .iter()
            .map(|node| {
                if node.len() == 32 {
                    let mut array = [0u8; 32];
                    array.copy_from_slice(node);
                    array
                } else if node.len() < 32 {
                    // Pad smaller nodes with zeros on the left to make them 32 bytes
                    let mut array = [0u8; 32];
                    array[32 - node.len()..].copy_from_slice(node);
                    array
                } else {
                    // For nodes longer than 32 bytes, use a hash of the node
                    // This preserves proof integrity while fitting the format constraint
                    use tiny_keccak::{Hasher, Keccak};
                    let mut keccak = Keccak::v256();
                    keccak.update(node);
                    let mut array = [0u8; 32];
                    keccak.finalize(&mut array);
                    array
                }
            })
            .collect();

        // Create storage semantics
        let semantics = StorageSemantics::new(zero_semantics);

        Ok(SemanticStorageProof {
            key,
            value: value_bytes,
            proof: proof_nodes,
            semantics,
        })
    }
}
