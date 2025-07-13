//! Solana account data structures and types
//!
//! This module defines the core data structures for representing Solana accounts,
//! their types, and their storage layouts for traverse analysis.

use serde::{Deserialize, Serialize};
use crate::{Result, SolanaError};

// #[cfg(feature = "solana")]
// use solana_sdk::{pubkey::Pubkey, account::Account};

/// Different types of Solana accounts that can be analyzed
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AccountType {
    /// Program Derived Account (PDA) - deterministically derived addresses
    PDA {
        /// Program ID that owns this PDA
        program_id: String,
        /// Seeds used for derivation
        seeds: Vec<Vec<u8>>,
        /// Bump seed for finding valid PDA
        bump: u8,
    },
    
    /// Associated Token Account (ATA) - standardized token accounts
    ATA {
        /// Mint address for the token
        mint: String,
        /// Owner of the token account
        owner: String,
    },
    
    /// System-owned account (regular wallet, etc.)
    System {
        /// Owner program (usually system program)
        owner: String,
    },
    
    /// SPL Token account
    Token {
        /// Token mint address
        mint: String,
        /// Account owner
        owner: String,
        /// Token program ID (Token or Token-2022)
        token_program: String,
    },
    
    /// Custom program account
    Program {
        /// Program ID that owns this account
        program_id: String,
        /// Account discriminator (first 8 bytes for Anchor)
        discriminator: Option<[u8; 8]>,
    },
}

impl AccountType {
    /// Get the expected owner program for this account type
    pub fn expected_owner(&self) -> String {
        match self {
            AccountType::PDA { program_id, .. } => program_id.clone(),
            AccountType::ATA { .. } => "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA".to_string(), // spl_token::id()
            AccountType::System { owner } => owner.clone(),
            AccountType::Token { token_program, .. } => token_program.clone(),
            AccountType::Program { program_id, .. } => program_id.clone(),
        }
    }
    
    /// Check if this account type supports semantic zero analysis
    pub fn supports_semantic_analysis(&self) -> bool {
        match self {
            AccountType::PDA { .. } | AccountType::Program { .. } => true,
            AccountType::ATA { .. } | AccountType::Token { .. } => true,
            AccountType::System { .. } => false, // Limited semantic meaning
        }
    }
}

/// Solana account layout information for traverse analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountLayout {
    /// Account type classification
    pub account_type: AccountType,
    
    /// Account address
    pub address: String,
    
    /// Data layout structure
    pub data_layout: Vec<FieldLayout>,
    
    /// Total account size in bytes
    pub size: u64,
    
    /// Whether the account is initialized
    pub initialized: bool,
    
    /// Account discriminator for program accounts
    pub discriminator: Option<[u8; 8]>,
}

impl AccountLayout {
    /// Create a new account layout
    pub fn new(
        account_type: AccountType,
        address: String,
        data_layout: Vec<FieldLayout>,
        size: u64,
    ) -> Self {
        let discriminator = match &account_type {
            AccountType::Program { discriminator, .. } => *discriminator,
            _ => None,
        };
        
        Self {
            account_type,
            address,
            data_layout,
            size,
            initialized: true, // Assume initialized by default
            discriminator,
        }
    }
    
    /// Get field by name
    pub fn get_field(&self, name: &str) -> Option<&FieldLayout> {
        self.data_layout.iter().find(|field| field.name == name)
    }
    
    /// Get the storage key for a specific field
    pub fn get_field_key(&self, field_name: &str) -> Result<Vec<u8>> {
        let field = self.get_field(field_name)
            .ok_or_else(|| SolanaError::InvalidAccountData(format!("Field '{}' not found", field_name)))?;
        
        // For Solana, the "storage key" is the account address + field offset
        let mut key = self.address.as_bytes().to_vec();
        key.extend_from_slice(&field.offset.to_le_bytes());
        Ok(key)
    }
}

/// Field layout within a Solana account
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldLayout {
    /// Field name
    pub name: String,
    
    /// Field type
    pub field_type: FieldType,
    
    /// Offset within the account data
    pub offset: u32,
    
    /// Size of the field in bytes
    pub size: u32,
    
    /// Zero semantics for this field
    pub zero_semantics: ZeroSemantics,
}

/// Solana-specific field types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FieldType {
    /// Boolean (1 byte)
    Bool,
    
    /// Unsigned integers
    U8,
    U16,
    U32,
    U64,
    U128,
    
    /// Signed integers  
    I8,
    I16,
    I32,
    I64,
    I128,
    
    /// Solana Pubkey (32 bytes)
    Pubkey,
    
    /// Fixed-size byte arrays
    Bytes(u32),
    
    /// Variable-length strings
    String,
    
    /// Vec of elements
    Vec(Box<FieldType>),
    
    /// Option type
    Option(Box<FieldType>),
    
    /// Custom struct/enum types
    Custom(String),
}

impl FieldType {
    /// Get the size in bytes for fixed-size types
    pub fn fixed_size(&self) -> Option<u32> {
        match self {
            FieldType::Bool => Some(1),
            FieldType::U8 | FieldType::I8 => Some(1),
            FieldType::U16 | FieldType::I16 => Some(2),
            FieldType::U32 | FieldType::I32 => Some(4),
            FieldType::U64 | FieldType::I64 => Some(8),
            FieldType::U128 | FieldType::I128 => Some(16),
            FieldType::Pubkey => Some(32),
            FieldType::Bytes(size) => Some(*size),
            _ => None, // Variable-size types
        }
    }
    
    /// Check if this type can legitimately be zero
    pub fn can_be_zero(&self) -> bool {
        match self {
            FieldType::Bool => true, // false = 0
            FieldType::U8 | FieldType::U16 | FieldType::U32 | FieldType::U64 | FieldType::U128 => true,
            FieldType::I8 | FieldType::I16 | FieldType::I32 | FieldType::I64 | FieldType::I128 => true,
            FieldType::Pubkey => false, // Zero pubkey is suspicious
            FieldType::Bytes(_) => true,
            FieldType::String => true,
            FieldType::Vec(_) => true, // Empty vec
            FieldType::Option(_) => true, // None = zero
            FieldType::Custom(_) => true, // Depends on implementation
        }
    }
}

/// Zero semantics for Solana account fields
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ZeroSemantics {
    /// Account was never initialized
    NeverInitialized,
    
    /// Field was explicitly set to zero
    ExplicitlyZero,
    
    /// Field was cleared/reset
    Cleared,
    
    /// Zero is a valid operational value
    ValidZero,
    
    /// Account is closed (all data zeroed)
    Closed,
}

/// Wrapper for Solana program account with traverse analysis
#[derive(Debug, Clone)]
pub struct ProgramAccount {
    /// Raw Solana account data
    // #[cfg(feature = "solana")]
    // pub account: Account,
    
    /// Traverse-specific layout analysis
    pub layout: AccountLayout,
    
    /// Address of this account
    pub address: String,
    
    /// Whether the account data has been validated
    pub validated: bool,
}

impl ProgramAccount {
    /// Create a new program account wrapper
    // #[cfg(feature = "solana")]
    // pub fn new(address: Pubkey, account: Account, layout: AccountLayout) -> Self {
    //     Self {
    //         account,
    //         layout,
    //         address: address.to_string(),
    //         validated: false,
    //     }
    // }
    
    /// Create from layout only (for testing/analysis without full account)
    pub fn from_layout(address: String, layout: AccountLayout) -> Self {
        Self {
            // #[cfg(feature = "solana")]
            // account: Account::default(),
            layout,
            address,
            validated: false,
        }
    }
    
    /// Validate account data against layout
    pub fn validate(&mut self) -> Result<()> {
        // Validate discriminator if present
        // if let Some(expected_disc) = self.layout.discriminator {
        //     #[cfg(feature = "solana")]
        //     {
        //         if self.account.data.len() < 8 {
        //             return Err(SolanaError::InvalidAccountData("Account too small for discriminator".to_string()));
        //         }
        //         let actual_disc: [u8; 8] = self.account.data[0..8].try_into()
        //             .map_err(|_| SolanaError::InvalidAccountData("Invalid discriminator".to_string()))?;
        //         if actual_disc != expected_disc {
        //             return Err(SolanaError::InvalidAccountData("Discriminator mismatch".to_string()));
        //         }
        //     }
        // }
        
        // Validate account size
        // #[cfg(feature = "solana")]
        // {
        //     if self.account.data.len() as u64 != self.layout.size {
        //         return Err(SolanaError::InvalidAccountData("Size mismatch".to_string()));
        //     }
        // }
        
        self.validated = true;
        Ok(())
    }
    
    /// Extract field value by name
    pub fn get_field_value(&self, field_name: &str) -> Result<Vec<u8>> {
        let field = self.layout.get_field(field_name)
            .ok_or_else(|| SolanaError::InvalidAccountData(format!("Field '{}' not found", field_name)))?;
        
        // #[cfg(feature = "solana")]
        // {
        //     let start = field.offset as usize;
        //     let end = start + field.size as usize;
        //     
        //     if end > self.account.data.len() {
        //         return Err(SolanaError::InvalidAccountData("Field extends beyond account data".to_string()));
        //     }
        //     
        //     Ok(self.account.data[start..end].to_vec())
        // }
        
        // #[cfg(not(feature = "solana"))]
        // {
            // Return placeholder for testing
            Ok(vec![0u8; field.size as usize])
        // }
    }
} 