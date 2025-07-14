//! Solana key resolution for PDA and ATA derivation
//!
//! This module provides functionality to derive Program Derived Accounts (PDA)
//! and Associated Token Accounts (ATA) for Solana programs.

use crate::{SolanaError, SolanaResult};
use std::{format, string::String, vec::Vec};

// Conditional Solana SDK imports
#[cfg(feature = "solana")]
use solana_sdk::{
    pubkey::Pubkey,
    // program_pack::Pack,
};

#[cfg(all(feature = "solana", feature = "spl-token"))]
use spl_token::state::Account as TokenAccount;

/// Query patterns for Solana account resolution
#[derive(Debug, Clone)]
pub enum SolanaQuery {
    /// Direct account access: "program_data"
    Direct {
        /// Account name
        account_name: String,
    },
    /// PDA with seeds: "user_account[{pubkey}]"
    PDA {
        /// Account name
        account_name: String,
        /// Seed values
        seeds: Vec<String>,
    },
    /// Associated Token Account: "token_balance[{mint},{owner}]"
    ATA {
        /// Token mint address
        mint: String,
        /// Token owner address
        owner: String,
    },
    /// Field access: "user_account.balance"
    FieldAccess {
        /// Account name
        account_name: String,
        /// Field path
        field_path: String,
    },
}

/// Solana-specific key resolver for PDA and ATA derivation
pub struct SolanaKeyResolver {
    /// Default program ID for PDA derivation
    pub default_program_id: Option<String>,
}

impl SolanaKeyResolver {
    /// Create a new Solana key resolver
    pub fn new() -> Self {
        Self {
            default_program_id: None,
        }
    }

    /// Create resolver with default program ID
    pub fn with_program_id(program_id: String) -> Self {
        Self {
            default_program_id: Some(program_id),
        }
    }

    /// Parse query string into SolanaQuery
    pub fn parse_query(query: &str) -> SolanaResult<SolanaQuery> {
        // Handle field access: "account.field"
        if query.contains('.') && !query.contains('[') {
            let parts: Vec<&str> = query.splitn(2, '.').collect();
            if parts.len() == 2 {
                return Ok(SolanaQuery::FieldAccess {
                    account_name: parts[0].to_string(),
                    field_path: parts[1].to_string(),
                });
            }
        }

        // Handle bracket notation: "account[args]"
        if let Some(bracket_start) = query.find('[') {
            let account_name = query[..bracket_start].to_string();
            let bracket_end = query.rfind(']')
                .ok_or_else(|| SolanaError::InvalidQuery("Missing closing bracket".to_string()))?;
            
            let args_str = &query[bracket_start + 1..bracket_end];
            let args: Vec<String> = args_str.split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();

            // Check if this looks like an ATA query (has exactly 2 args)
            if args.len() == 2 && account_name.contains("token") {
                return Ok(SolanaQuery::ATA {
                    mint: args[0].clone(),
                    owner: args[1].clone(),
                });
            }

            // Otherwise, treat as PDA
            return Ok(SolanaQuery::PDA {
                account_name,
                seeds: args,
            });
        }

        // Simple direct access
        Ok(SolanaQuery::Direct {
            account_name: query.to_string(),
        })
    }

    /// Resolve query to account address
    pub fn resolve_account_address(&self, query: &SolanaQuery) -> SolanaResult<String> {
        match query {
            SolanaQuery::Direct { account_name: _ } => {
                // For direct access, we need the program's account address
                // This would typically come from program deployment or configuration
                Err(SolanaError::InvalidQuery(
                    "Direct account access requires specific account address".to_string()
                ))
            }
            SolanaQuery::PDA { account_name: _, seeds } => {
                self.derive_pda_address(seeds)
            }
            SolanaQuery::ATA { mint, owner } => {
                self.derive_ata_address(mint, owner)
            }
            SolanaQuery::FieldAccess { account_name: _, field_path: _ } => {
                // Field access resolves to the account address, not a specific field
                Err(SolanaError::InvalidQuery(
                    "Field access requires account address resolution first".to_string()
                ))
            }
        }
    }

    /// Derive Program Derived Account (PDA) address
    #[cfg(feature = "solana")]
    pub fn derive_pda_address(&self, seeds: &[String]) -> SolanaResult<String> {
        let program_id_str = self.default_program_id.as_ref()
            .ok_or_else(|| SolanaError::InvalidProgramId("No program ID configured".to_string()))?;

        let program_id = program_id_str.parse::<Pubkey>()
            .map_err(|e| SolanaError::InvalidProgramId(format!("Invalid program ID: {}", e)))?;

        // Convert string seeds to bytes
        let mut seed_bytes = Vec::new();
        for seed in seeds {
            if seed.starts_with("0x") {
                // Hex seed
                let hex_bytes = hex::decode(&seed[2..])
                    .map_err(|e| SolanaError::InvalidQuery(format!("Invalid hex seed: {}", e)))?;
                seed_bytes.push(hex_bytes);
            } else if let Ok(pubkey) = seed.parse::<Pubkey>() {
                // Pubkey seed
                seed_bytes.push(pubkey.to_bytes().to_vec());
            } else {
                // String seed
                seed_bytes.push(seed.as_bytes().to_vec());
            }
        }

        // Convert to byte slices
        let seed_slices: Vec<&[u8]> = seed_bytes.iter().map(|v| v.as_slice()).collect();

        // Derive PDA
        let (pda, _bump) = Pubkey::find_program_address(&seed_slices, &program_id);
        
        Ok(pda.to_string())
    }

    /// Derive Program Derived Account (PDA) address (fallback without solana feature)
    #[cfg(not(feature = "solana"))]
    pub fn derive_pda_address(&self, _seeds: &[String]) -> SolanaResult<String> {
        Err(SolanaError::NetworkError(
            "PDA derivation requires 'solana' feature".to_string()
        ))
    }

    /// Derive Associated Token Account (ATA) address
    #[cfg(all(feature = "solana", feature = "spl-token"))]
    pub fn derive_ata_address(&self, mint: &str, owner: &str) -> SolanaResult<String> {
        let mint_pubkey = mint.parse::<Pubkey>()
            .map_err(|e| SolanaError::InvalidQuery(format!("Invalid mint address: {}", e)))?;
        
        let owner_pubkey = owner.parse::<Pubkey>()
            .map_err(|e| SolanaError::InvalidQuery(format!("Invalid owner address: {}", e)))?;

        let ata_address = spl_associated_token_account::get_associated_token_address(
            &owner_pubkey,
            &mint_pubkey,
        );

        Ok(ata_address.to_string())
    }

    /// Derive Associated Token Account (ATA) address (fallback)
    #[cfg(not(all(feature = "solana", feature = "spl-token")))]
    pub fn derive_ata_address(&self, _mint: &str, _owner: &str) -> SolanaResult<String> {
        Err(SolanaError::NetworkError(
            "ATA derivation requires 'solana' and 'spl-token' features".to_string()
        ))
    }

    /// Resolve storage key for field access
    pub fn resolve_field_key(
        &self,
        account_address: &str,
        field_path: &str,
        field_offset: usize,
    ) -> SolanaResult<String> {
        // For Solana, the "storage key" is really the account address + field offset
        // We encode this as a deterministic key for consistency with other chains
        let combined = format!("{}:{}", account_address, field_path);
        let mut key_bytes = [0u8; 32];
        
        // Use SHA256 to create a deterministic key
        #[cfg(feature = "std")]
        {
            use sha2::{Sha256, Digest};
            let hash = Sha256::digest(combined.as_bytes());
            key_bytes.copy_from_slice(&hash[..32]);
        }
        
        #[cfg(not(feature = "std"))]
        {
            // Simple fallback for no_std
            let bytes = combined.as_bytes();
            for (i, &b) in bytes.iter().enumerate().take(32) {
                key_bytes[i] = b;
            }
            // Include field offset in key
            key_bytes[28..32].copy_from_slice(&(field_offset as u32).to_le_bytes());
        }

        Ok(hex::encode(key_bytes))
    }

    /// Generate multiple account addresses for batch operations
    pub fn resolve_batch_queries(&self, queries: &[SolanaQuery]) -> Vec<SolanaResult<String>> {
        queries.iter()
            .map(|query| self.resolve_account_address(query))
            .collect()
    }

    /// Validate Solana address format
    pub fn validate_address(address: &str) -> bool {
        #[cfg(feature = "solana")]
        {
            address.parse::<Pubkey>().is_ok()
        }
        #[cfg(not(feature = "solana"))]
        {
            // Basic validation - Solana addresses are base58 encoded 32-byte values
            address.len() >= 32 && address.len() <= 44 && 
            address.chars().all(|c| "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz".contains(c))
        }
    }

    /// Extract program ID from account query if present
    pub fn extract_program_id(&self, query: &str) -> Option<String> {
        // Look for program ID hints in query
        if query.contains("program_id:") {
            if let Some(start) = query.find("program_id:") {
                let rest = &query[start + 11..];
                if let Some(end) = rest.find(|c: char| c.is_whitespace() || c == ',') {
                    return Some(rest[..end].to_string());
                } else {
                    return Some(rest.to_string());
                }
            }
        }
        self.default_program_id.clone()
    }
}

impl Default for SolanaKeyResolver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_direct_query() {
        let query = SolanaKeyResolver::parse_query("user_account").unwrap();
        assert!(matches!(query, SolanaQuery::Direct { .. }));
        if let SolanaQuery::Direct { account_name } = query {
            assert_eq!(account_name, "user_account");
        }
    }

    #[test]
    fn test_parse_pda_query() {
        let query = SolanaKeyResolver::parse_query("user_account[0x123,user1]").unwrap();
        assert!(matches!(query, SolanaQuery::PDA { .. }));
        if let SolanaQuery::PDA { account_name, seeds } = query {
            assert_eq!(account_name, "user_account");
            assert_eq!(seeds.len(), 2);
            assert_eq!(seeds[0], "0x123");
            assert_eq!(seeds[1], "user1");
        }
    }

    #[test]
    fn test_parse_ata_query() {
        let query = SolanaKeyResolver::parse_query("token_balance[mint123,owner456]").unwrap();
        assert!(matches!(query, SolanaQuery::ATA { .. }));
        if let SolanaQuery::ATA { mint, owner } = query {
            assert_eq!(mint, "mint123");
            assert_eq!(owner, "owner456");
        }
    }

    #[test]
    fn test_parse_field_access_query() {
        let query = SolanaKeyResolver::parse_query("user_account.balance").unwrap();
        assert!(matches!(query, SolanaQuery::FieldAccess { .. }));
        if let SolanaQuery::FieldAccess { account_name, field_path } = query {
            assert_eq!(account_name, "user_account");
            assert_eq!(field_path, "balance");
        }
    }

    #[test]
    fn test_parse_invalid_query() {
        let result = SolanaKeyResolver::parse_query("user_account[missing_bracket");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_address() {
        // Valid base58 string (example Solana address format)
        assert!(SolanaKeyResolver::validate_address("9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM"));
        
        // Invalid characters
        assert!(!SolanaKeyResolver::validate_address("0x1234567890"));
        
        // Too short
        assert!(!SolanaKeyResolver::validate_address("short"));
        
        // Too long  
        assert!(!SolanaKeyResolver::validate_address(&"a".repeat(50)));
    }

    #[test]
    fn test_extract_program_id() {
        let resolver = SolanaKeyResolver::new();
        
        // No program ID in query
        assert_eq!(resolver.extract_program_id("user_account"), None);
        
        // Program ID in query
        let program_id = resolver.extract_program_id("user_account program_id:9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM");
        assert_eq!(program_id, Some("9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM".to_string()));
    }

    #[test]
    fn test_resolve_field_key() {
        let resolver = SolanaKeyResolver::new();
        let account = "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM";
        let field = "balance";
        
        let key1 = resolver.resolve_field_key(account, field, 8).unwrap();
        let key2 = resolver.resolve_field_key(account, field, 8).unwrap();
        let key3 = resolver.resolve_field_key(account, field, 16).unwrap();
        
        // Same inputs should produce same key
        assert_eq!(key1, key2);
        
        // Different offsets should produce different keys
        assert_ne!(key1, key3);
        
        // Key should be 64 hex characters (32 bytes)
        assert_eq!(key1.len(), 64);
        assert!(key1.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_batch_query_resolution() {
        let resolver = SolanaKeyResolver::new();
        let queries = vec![
            SolanaQuery::Direct { account_name: "test1".to_string() },
            SolanaQuery::Direct { account_name: "test2".to_string() },
        ];
        
        let results = resolver.resolve_batch_queries(&queries);
        assert_eq!(results.len(), 2);
        
        // Both should fail since we don't have addresses for direct access
        assert!(results[0].is_err());
        assert!(results[1].is_err());
    }

    #[cfg(feature = "solana")]
    #[test]
    fn test_pda_derivation() {
        let resolver = SolanaKeyResolver::with_program_id(
            "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM".to_string()
        );
        
        let seeds = vec!["user".to_string(), "123".to_string()];
        let result = resolver.derive_pda_address(&seeds);
        
        assert!(result.is_ok());
        let address = result.unwrap();
        assert!(SolanaKeyResolver::validate_address(&address));
    }
} 