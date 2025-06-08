//! IAVL proof fetching and verification for Cosmos chains
//!
//! This module provides functionality to fetch and verify IAVL tree proofs
//! from Cosmos SDK-based chains using the ICS23 standard merkle proof format.
//!
//! # Features
//!
//! - Fetch storage proofs via Cosmos RPC endpoints
//! - Verify IAVL proofs using ICS23 verification
//! - Support for both existence and non-existence proofs
//! - Integration with CosmWasm contract storage
//!
//! # IAVL Overview
//!
//! IAVL (Immutable AVL) trees are the default storage backend for Cosmos SDK.
//! They provide authenticated storage with cryptographic proofs that can be
//! verified against block headers. The ICS23 standard defines a common format
//! for these proofs across different merkle tree implementations.

use crate::CosmosError;
use anyhow::Result;
use ics23::{verify_membership, verify_non_membership, iavl_spec, CommitmentProof, ProofSpec, HostFunctionsManager};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use base64::{engine::general_purpose::STANDARD, Engine};

/// IAVL proof data from Cosmos RPC
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IavlProof {
    /// Storage key that was queried
    pub key: Vec<u8>,
    /// Storage value (if exists)
    pub value: Option<Vec<u8>>,
    /// ICS23 commitment proof
    pub proof: CommitmentProof,
    /// Block height for the proof
    pub height: u64,
    /// State root hash
    pub root: Vec<u8>,
}

/// IAVL proof fetcher for Cosmos chains
pub struct CosmosProofFetcher {
    /// RPC endpoint URL
    pub rpc_url: String,
    /// Chain-specific configuration
    pub config: CosmosChainConfig,
}

/// Chain-specific configuration for proof fetching
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CosmosChainConfig {
    /// Chain ID
    pub chain_id: String,
    /// Whether to use gRPC or REST API
    pub use_grpc: bool,
    /// Custom proof spec (if different from standard IAVL)
    pub proof_spec: Option<ProofSpec>,
    /// Store prefix for state queries
    pub store_prefix: String,
}

impl Default for CosmosChainConfig {
    fn default() -> Self {
        Self {
            chain_id: "cosmoshub-4".to_string(),
            use_grpc: false,
            proof_spec: None,
            store_prefix: "store".to_string(),
        }
    }
}

impl CosmosProofFetcher {
    /// Create a new IAVL proof fetcher
    pub fn new(rpc_url: String, config: CosmosChainConfig) -> Self {
        Self { rpc_url, config }
    }

    /// Create a proof fetcher with default configuration
    pub fn with_defaults(rpc_url: String) -> Self {
        Self::new(rpc_url, CosmosChainConfig::default())
    }

    /// Fetch an IAVL storage proof for the given key
    ///
    /// # Arguments
    ///
    /// * `store_key` - The storage key to query (e.g., "wasm" for CosmWasm)
    /// * `key` - The specific key within the store
    /// * `height` - Block height to query (None for latest)
    ///
    /// # Returns
    ///
    /// IAVL proof that can be verified against the state root
    pub async fn fetch_proof(
        &self,
        store_key: &str,
        key: &[u8],
        height: Option<u64>,
    ) -> Result<IavlProof, CosmosError> {
        let client = reqwest::Client::new();
        
        // Construct the query path
        let query_path = if let Some(h) = height {
            format!("/store/{}/key?key={}&height={}", 
                    store_key, 
                    hex::encode(key), 
                    h)
        } else {
            format!("/store/{}/key?key={}", 
                    store_key, 
                    hex::encode(key))
        };
        
        let url = format!("{}/abci_query?path=\"{}\"", self.rpc_url, query_path);
        
        // Fetch the proof from RPC
        let response = client.get(&url).send().await?;
        let rpc_response: serde_json::Value = response.json().await?;
        
        // Parse the response (this is a simplified version - real implementation would
        // need to handle the full Cosmos RPC response format)
        let result = rpc_response.get("result")
            .and_then(|r| r.get("response"))
            .ok_or_else(|| CosmosError::InvalidSchema(
                "No response data in RPC result".to_string()
            ))?;
        
        let proof_data = result.get("proofOps")
            .ok_or_else(|| CosmosError::InvalidSchema(
                "No proof data in response".to_string()
            ))?;
        
        // Convert to ICS23 proof format
        let proof = self.parse_cosmos_proof(proof_data)?;
        
        let value = result.get("value")
            .and_then(|v| v.as_str())
            .map(|s| STANDARD.decode(s).unwrap_or_default());
        
        let height = result.get("height")
            .and_then(|h| h.as_str())
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        
        // Get the state root (this would come from the block header)
        let root = self.fetch_state_root(height).await?;
        
        Ok(IavlProof {
            key: key.to_vec(),
            value,
            proof,
            height,
            root,
        })
    }

    /// Verify an IAVL proof against a known state root
    ///
    /// # Arguments
    ///
    /// * `proof` - The IAVL proof to verify
    /// * `expected_value` - Expected value (for existence proofs)
    ///
    /// # Returns
    ///
    /// True if the proof is valid, false otherwise
    pub fn verify_proof(
        &self,
        proof: &IavlProof,
        expected_value: Option<&[u8]>,
    ) -> Result<bool, CosmosError> {
        let default_spec = iavl_spec();
        let spec = self.config.proof_spec
            .as_ref()
            .unwrap_or(&default_spec);
        
        match expected_value {
            Some(value) => {
                // Verify existence proof
                Ok(verify_membership::<HostFunctionsManager>(
                    &proof.proof,
                    spec,
                    &proof.root,
                    &proof.key,
                    value,
                ))
            }
            None => {
                // Verify non-existence proof
                Ok(verify_non_membership::<HostFunctionsManager>(
                    &proof.proof,
                    spec,
                    &proof.root,
                    &proof.key,
                ))
            }
        }
    }

    /// Fetch storage proofs for multiple keys in batch
    ///
    /// # Arguments
    ///
    /// * `store_key` - The storage key to query
    /// * `keys` - List of keys to fetch proofs for
    /// * `height` - Block height to query (None for latest)
    ///
    /// # Returns
    ///
    /// Map of keys to their corresponding proofs
    pub async fn fetch_batch_proofs(
        &self,
        store_key: &str,
        keys: &[Vec<u8>],
        height: Option<u64>,
    ) -> Result<HashMap<Vec<u8>, IavlProof>, CosmosError> {
        let mut proofs = HashMap::new();
        
        // For now, fetch proofs sequentially
        // A more efficient implementation could use batch RPC calls
        for key in keys {
            let proof = self.fetch_proof(store_key, key, height).await?;
            proofs.insert(key.clone(), proof);
        }
        
        Ok(proofs)
    }

    /// Parse Cosmos RPC proof response into ICS23 format
    fn parse_cosmos_proof(&self, _proof_data: &serde_json::Value) -> Result<CommitmentProof, CosmosError> {
        // This is a simplified parser - real implementation would need to handle
        // the full Cosmos proof operation format and convert to ICS23
        
        // For now, return a placeholder proof structure
        // In a full implementation, this would parse the proof operations
        // and construct a proper ICS23 CommitmentProof
        
        Err(CosmosError::UnsupportedPattern(
            "Full IAVL proof parsing not yet implemented".to_string()
        ))
    }

    /// Fetch the state root for a given block height
    async fn fetch_state_root(&self, height: u64) -> Result<Vec<u8>, CosmosError> {
        let client = reqwest::Client::new();
        let url = format!("{}/block?height={}", self.rpc_url, height);
        
        let response = client.get(&url).send().await?;
        let block_response: serde_json::Value = response.json().await?;
        
        let app_hash = block_response
            .get("result")
            .and_then(|r| r.get("block"))
            .and_then(|b| b.get("header"))
            .and_then(|h| h.get("app_hash"))
            .and_then(|ah| ah.as_str())
            .ok_or_else(|| CosmosError::InvalidSchema(
                "No app_hash in block response".to_string()
            ))?;
        
        hex::decode(app_hash).map_err(|e| CosmosError::InvalidSchema(
            format!("Invalid app_hash format: {}", e)
        ))
    }
}

/// Create IAVL-specific proof spec
pub fn cosmos_iavl_spec() -> ProofSpec {
    iavl_spec()
}

/// Verify a single IAVL proof
pub fn verify_iavl_proof(
    proof: &IavlProof,
    expected_value: Option<&[u8]>,
    spec: Option<&ProofSpec>,
) -> Result<bool, CosmosError> {
    let default_spec = iavl_spec();
    let proof_spec = spec.unwrap_or(&default_spec);
    
    match expected_value {
        Some(value) => {
            Ok(verify_membership::<HostFunctionsManager>(
                &proof.proof,
                proof_spec,
                &proof.root,
                &proof.key,
                value,
            ))
        }
        None => {
            Ok(verify_non_membership::<HostFunctionsManager>(
                &proof.proof,
                proof_spec,
                &proof.root,
                &proof.key,
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosmos_chain_config_default() {
        let config = CosmosChainConfig::default();
        assert_eq!(config.chain_id, "cosmoshub-4");
        assert!(!config.use_grpc);
        assert_eq!(config.store_prefix, "store");
    }

    #[test]
    fn test_iavl_spec_generation() {
        let spec = cosmos_iavl_spec();
        // Verify it's a valid IAVL spec
        assert!(spec.leaf_spec.is_some());
        assert!(spec.inner_spec.is_some());
    }

    #[tokio::test]
    async fn test_proof_fetcher_creation() {
        let fetcher = CosmosProofFetcher::with_defaults("http://localhost:26657".to_string());
        assert_eq!(fetcher.rpc_url, "http://localhost:26657");
        assert_eq!(fetcher.config.chain_id, "cosmoshub-4");
    }
} 