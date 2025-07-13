//! Solana account proof generation and verification
//!
//! This module provides functionality to fetch and verify Solana account proofs
//! for ZK storage verification using valence-domain-clients.

use crate::{SolanaError, SolanaResult};
use serde::{Deserialize, Serialize};

// Conditional imports for client functionality
#[cfg(feature = "client")]
use valence_domain_clients::solana::SolanaClient;

#[cfg(feature = "solana")]
use solana_sdk::{pubkey::Pubkey, account::Account};

/// Solana account proof structure for ZK verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolanaAccountProof {
    /// Account address
    pub address: String,
    /// Account data
    pub data: Vec<u8>,
    /// Data length
    pub data_len: usize,
    /// Account owner program
    pub owner: String,
    /// Lamports balance
    pub lamports: u64,
    /// Rent epoch
    pub rent_epoch: u64,
    /// Slot when proof was generated
    pub slot: u64,
    /// Block hash for the slot
    pub block_hash: String,
    /// Transaction signature for proof
    pub signature: Option<String>,
}

/// Solana proof fetcher for account data verification using valence-domain-clients
pub struct SolanaProofFetcher {
    /// RPC endpoint URL
    pub rpc_url: String,
    /// Valence Solana client for RPC operations
    #[cfg(feature = "client")]
    client: SolanaClient,
}

impl SolanaProofFetcher {
    /// Create a new Solana proof fetcher using valence-domain-clients
    #[cfg(feature = "client")]
    pub async fn new(rpc_url: String) -> SolanaResult<Self> {
        // Create Solana client using valence-domain-clients
        let client = SolanaClient::new(&rpc_url, None)
            .await
            .map_err(|e| SolanaError::NetworkError(format!("Failed to create Solana client: {}", e)))?;

        Ok(Self {
            rpc_url,
            client,
        })
    }

    /// Create a new Solana proof fetcher without client (fallback)
    #[cfg(not(feature = "client"))]
    pub fn new(rpc_url: String) -> SolanaResult<Self> {
        Ok(Self {
            rpc_url,
        })
    }

    /// Fetch account proof from Solana using valence-domain-clients
    #[cfg(feature = "client")]
    pub async fn fetch_account_proof(&self, address: &str) -> SolanaResult<SolanaAccountProof> {
        // Validate address format
        self.validate_address(address)?;

        // Parse address as Pubkey
        let pubkey = address.parse::<Pubkey>()
            .map_err(|e| SolanaError::AddressParsingError(format!("Invalid Solana address: {}", e)))?;

        // Get account info using valence-domain-clients
        let account_info = self.client
            .get_account_info(&pubkey)
            .await
            .map_err(|e| SolanaError::NetworkError(format!("Failed to get account info: {}", e)))?;

        let account = account_info
            .ok_or_else(|| SolanaError::AccountNotFound(format!("Account {} not found", address)))?;

        // Get current slot and block hash
        let slot = self.get_current_slot().await?;
        let block_hash = self.get_block_hash(slot).await?;

        Ok(SolanaAccountProof {
            address: address.to_string(),
            data: account.data,
            data_len: account.data.len(),
            owner: account.owner.to_string(),
            lamports: account.lamports,
            rent_epoch: account.rent_epoch,
            slot,
            block_hash,
            signature: None,
        })
    }

    /// Fallback when client feature is not enabled
    #[cfg(not(feature = "client"))]
    pub async fn fetch_account_proof(&self, _address: &str) -> SolanaResult<SolanaAccountProof> {
        Err(SolanaError::FeatureNotEnabled("Client feature required for account proof fetching".into()))
    }

    /// Get current slot using valence-domain-clients
    #[cfg(feature = "client")]
    async fn get_current_slot(&self) -> SolanaResult<u64> {
        self.client
            .get_slot()
            .await
            .map_err(|e| SolanaError::NetworkError(format!("Failed to get current slot: {}", e)))
    }

    /// Fallback when client feature is not enabled
    #[cfg(not(feature = "client"))]
    async fn get_current_slot(&self) -> SolanaResult<u64> {
        Err(SolanaError::FeatureNotEnabled("Client feature required for slot fetching".into()))
    }

    /// Get block hash for slot using valence-domain-clients
    #[cfg(feature = "client")]
    async fn get_block_hash(&self, slot: u64) -> SolanaResult<String> {
        self.client
            .get_block_hash(slot)
            .await
            .map(|hash| hash.to_string())
            .map_err(|e| SolanaError::NetworkError(format!("Failed to get block hash: {}", e)))
    }

    /// Fallback when client feature is not enabled
    #[cfg(not(feature = "client"))]
    async fn get_block_hash(&self, _slot: u64) -> SolanaResult<String> {
        Err(SolanaError::FeatureNotEnabled("Client feature required for block hash fetching".into()))
    }

    /// Validate Solana address format
    #[cfg(feature = "solana")]
    fn validate_address(&self, address: &str) -> SolanaResult<()> {
        address.parse::<Pubkey>()
            .map_err(|e| SolanaError::AddressParsingError(format!("Invalid Solana address: {}", e)))?;
        Ok(())
    }

    /// Fallback address validation when solana feature is not enabled
    #[cfg(not(feature = "solana"))]
    fn validate_address(&self, address: &str) -> SolanaResult<()> {
        // Basic validation - should be base58 and ~44 characters
        if address.len() < 32 || address.len() > 44 {
            return Err(SolanaError::AddressParsingError("Invalid address length".into()));
        }
        
        // Check for valid base58 characters
        if !address.chars().all(|c| "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz".contains(c)) {
            return Err(SolanaError::AddressParsingError("Invalid base58 characters".into()));
        }
        
        Ok(())
    }

    /// Create a proof structure from account data
    pub fn create_proof_from_account_data(
        address: String,
        data: Vec<u8>,
        owner: String,
        lamports: u64,
        rent_epoch: u64,
        slot: u64,
        block_hash: String,
    ) -> SolanaAccountProof {
        SolanaAccountProof {
            address,
            data_len: data.len(),
            data,
            owner,
            lamports,
            rent_epoch,
            slot,
            block_hash,
            signature: None,
        }
    }

    /// Verify proof integrity (basic validation)
    pub fn verify_proof(&self, proof: &SolanaAccountProof) -> SolanaResult<bool> {
        // Basic validation checks
        if proof.data_len != proof.data.len() {
            return Ok(false);
        }

        // Validate address format
        if let Err(_) = self.validate_address(&proof.address) {
            return Ok(false);
        }

        // Validate owner format
        if let Err(_) = self.validate_address(&proof.owner) {
            return Ok(false);
        }

        Ok(true)
    }

    /// Extract specific field from account data
    pub fn extract_field(
        &self,
        proof: &SolanaAccountProof,
        offset: usize,
        size: usize,
    ) -> SolanaResult<Vec<u8>> {
        if offset + size > proof.data.len() {
            return Err(SolanaError::AccountParsingError(
                "Field offset/size exceeds account data length".into(),
            ));
        }

        Ok(proof.data[offset..offset + size].to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proof_fetcher_creation() {
        let fetcher = SolanaProofFetcher::new("https://api.mainnet-beta.solana.com".to_string());
        assert_eq!(fetcher.rpc_url, "https://api.mainnet-beta.solana.com");
    }

    #[test]
    fn test_create_proof_from_account_data() {
        let proof = SolanaProofFetcher::create_proof_from_account_data(
            "11111111111111111111111111111112".to_string(),
            vec![1, 2, 3, 4],
            "11111111111111111111111111111112".to_string(),
            1000000,
            250,
            12345,
            "AbCdEf123456".to_string(),
        );

        assert_eq!(proof.address, "11111111111111111111111111111112");
        assert_eq!(proof.data, vec![1, 2, 3, 4]);
        assert_eq!(proof.data_len, 4);
        assert_eq!(proof.lamports, 1000000);
        assert_eq!(proof.slot, 12345);
    }

    #[test]
    fn test_address_validation() {
        let fetcher = SolanaProofFetcher::new("https://api.mainnet-beta.solana.com".to_string());
        
        // Valid address
        let valid_result = fetcher.validate_address("11111111111111111111111111111112");
        assert!(valid_result.is_ok());
        
        // Invalid address (too short)
        let invalid_result = fetcher.validate_address("111");
        assert!(invalid_result.is_err());
        
        // Invalid address (contains invalid base58 chars)
        let invalid_result = fetcher.validate_address("0O111111111111111111111111111112");
        assert!(invalid_result.is_err());
    }

    #[test]
    fn test_extract_field() {
        let fetcher = SolanaProofFetcher::new("https://api.mainnet-beta.solana.com".to_string());
        let proof = SolanaProofFetcher::create_proof_from_account_data(
            "11111111111111111111111111111112".to_string(),
            vec![1, 2, 3, 4, 5, 6, 7, 8],
            "11111111111111111111111111111112".to_string(),
            1000000,
            250,
            12345,
            "AbCdEf123456".to_string(),
        );

        // Extract valid field
        let field = fetcher.extract_field(&proof, 2, 3).unwrap();
        assert_eq!(field, vec![3, 4, 5]);

        // Extract field beyond data bounds
        let result = fetcher.extract_field(&proof, 6, 5);
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_proof() {
        let fetcher = SolanaProofFetcher::new("https://api.mainnet-beta.solana.com".to_string());
        let proof = SolanaProofFetcher::create_proof_from_account_data(
            "11111111111111111111111111111112".to_string(),
            vec![1, 2, 3, 4],
            "11111111111111111111111111111112".to_string(),
            1000000,
            250,
            12345,
            "AbCdEf123456".to_string(),
        );

        let is_valid = fetcher.verify_proof(&proof).unwrap();
        assert!(is_valid);

        // Test with inconsistent data length
        let mut invalid_proof = proof.clone();
        invalid_proof.data_len = 10; // Doesn't match actual data length

        let is_valid = fetcher.verify_proof(&invalid_proof).unwrap();
        assert!(!is_valid);
    }

    #[cfg(not(feature = "client"))]
    #[tokio::test]
    async fn test_fetch_account_proof_without_client_feature() {
        let fetcher = SolanaProofFetcher::new("https://api.mainnet-beta.solana.com".to_string());
        let result = fetcher.fetch_account_proof("11111111111111111111111111111112").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Client feature required for account proof fetching"));
    }
} 