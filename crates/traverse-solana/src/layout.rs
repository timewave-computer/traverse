//! Solana IDL-based storage layout compilation
//!
//! This module implements layout compilation for Solana programs using
//! Anchor IDL files and program introspection.

use crate::{AccountLayout, FieldLayout, FieldType, SolanaError, SolanaResult, AccountType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Conditional Solana SDK imports
#[cfg(feature = "solana")]
use solana_sdk::account::Account;

#[cfg(feature = "anchor")]
use crate::anchor::{SolanaIdl, IdlAccount, IdlType};

/// Layout information for a Solana program compiled from IDL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolanaLayout {
    /// Program ID
    pub program_id: String,
    /// Account layouts keyed by discriminator
    pub accounts: HashMap<String, AccountLayout>,
    /// Instruction layouts
    pub instructions: HashMap<String, Vec<FieldLayout>>,
}

/// Solana-specific layout compiler that processes IDL files
pub struct SolanaLayoutCompiler {
    /// Whether to include discriminator in layout calculations
    pub include_discriminator: bool,
}

impl SolanaLayoutCompiler {
    /// Create a new Solana layout compiler
    pub fn new() -> Self {
        Self {
            include_discriminator: true,
        }
    }

    /// Compile layout from IDL string (requires anchor feature)
    #[cfg(feature = "anchor")]
    pub fn compile_from_idl(&self, idl_data: &str) -> SolanaResult<SolanaLayout> {
        let idl: SolanaIdl = serde_json::from_str(idl_data)
            .map_err(|e| SolanaError::IdlParsingError(format!("Failed to parse IDL: {}", e)))?;

        let mut accounts = HashMap::new();
        let mut instructions = HashMap::new();

        // Process account definitions
        for account_def in &idl.accounts {
            let layout = self.compile_account_layout(account_def)?;
            // Use account name as key (could also use discriminator)
            accounts.insert(account_def.name.clone(), layout);
        }

        // Process instruction definitions
        for instruction_def in &idl.instructions {
            let fields = self.compile_instruction_fields(&instruction_def.args)?;
            instructions.insert(instruction_def.name.clone(), fields);
        }

        Ok(SolanaLayout {
            program_id: idl.metadata.address.clone(),
            accounts,
            instructions,
        })
    }

    /// Fallback when anchor feature is not enabled
    #[cfg(not(feature = "anchor"))]
    pub fn compile_from_idl(&self, _idl_data: &str) -> SolanaResult<SolanaLayout> {
        Err(SolanaError::ConfigurationError(
            "Anchor feature not enabled. Cannot parse IDL files.".into()
        ))
    }

    /// Compile layout from program account introspection (requires solana feature)
    #[cfg(feature = "solana")]
    pub fn compile_from_program_account(&self, account: &Account) -> SolanaResult<AccountLayout> {
        // This would analyze the account data to infer layout
        // For now, return a basic implementation
        let _ = account; // Suppress unused warning
        
        Ok(AccountLayout {
            account_type: AccountType::System { owner: "11111111111111111111111111111111".to_string() },
            address: "unknown".to_string(),
            data_layout: vec![],
            size: 0,
            initialized: false,
            discriminator: None,
        })
    }

    /// Fallback when solana feature is not enabled
    #[cfg(not(feature = "solana"))]
    pub fn compile_from_program_account(&self, _account_data: &[u8]) -> SolanaResult<AccountLayout> {
        Err(SolanaError::ConfigurationError(
            "Solana SDK feature not enabled. Cannot analyze program accounts.".into()
        ))
    }

    /// Compile layout for a specific account type (requires anchor feature)
    #[cfg(feature = "anchor")]
    fn compile_account_layout(&self, account_def: &IdlAccount) -> SolanaResult<AccountLayout> {
        let mut fields = Vec::new();
        let mut offset = 0;

        // Add discriminator field if enabled
        if self.include_discriminator {
            fields.push(FieldLayout {
                name: "discriminator".to_string(),
                field_type: FieldType::Bytes8,
                offset,
                size: 8,
                zero_semantics: crate::ZeroSemantics::NeverWritten,
            });
            offset += 8;
        }

        // Process account fields
        for field in &account_def.fields {
            let field_layout = self.compile_field_layout(field, offset)?;
            offset += field_layout.size;
            fields.push(field_layout);
        }

        Ok(AccountLayout {
            commitment: self.compute_layout_commitment(&fields),
            fields,
            total_size: offset,
        })
    }

    /// Compile instruction parameter fields (requires anchor feature)
    #[cfg(feature = "anchor")]
    fn compile_instruction_fields(&self, _args: &[crate::anchor::IdlField]) -> SolanaResult<Vec<FieldLayout>> {
        // For now, return empty - would implement instruction parameter layout
        Ok(vec![])
    }

    /// Compile individual field layout (requires anchor feature)
    #[cfg(feature = "anchor")]
    fn compile_field_layout(&self, field: &crate::anchor::IdlField, offset: usize) -> SolanaResult<FieldLayout> {
        let (field_type, size) = self.idl_type_to_field_type(&field.type_)?;
        
        Ok(FieldLayout {
            name: field.name.clone(),
            field_type,
            offset,
            size,
            zero_semantics: self.infer_zero_semantics(&field.name, &field_type),
        })
    }

    /// Convert IDL type to field type (requires anchor feature)
    #[cfg(feature = "anchor")]
    fn idl_type_to_field_type(&self, idl_type: &IdlType) -> SolanaResult<(FieldType, usize)> {
        match idl_type {
            IdlType::Bool => Ok((FieldType::Bool, 1)),
            IdlType::U8 => Ok((FieldType::U8, 1)),
            IdlType::I8 => Ok((FieldType::I8, 1)),
            IdlType::U16 => Ok((FieldType::U16, 2)),
            IdlType::I16 => Ok((FieldType::I16, 2)),
            IdlType::U32 => Ok((FieldType::U32, 4)),
            IdlType::I32 => Ok((FieldType::I32, 4)),
            IdlType::U64 => Ok((FieldType::U64, 8)),
            IdlType::I64 => Ok((FieldType::I64, 8)),
            IdlType::U128 => Ok((FieldType::U128, 16)),
            IdlType::I128 => Ok((FieldType::I128, 16)),
            IdlType::PublicKey => Ok((FieldType::PublicKey, 32)),
            IdlType::String => Ok((FieldType::String, 4)), // Length prefix
            IdlType::Bytes => Ok((FieldType::Bytes, 4)), // Length prefix
            IdlType::Array { inner, size } => {
                let (inner_type, inner_size) = self.idl_type_to_field_type(inner)?;
                Ok((FieldType::Array(Box::new(inner_type)), inner_size * size))
            }
            IdlType::Vec { inner: _ } => {
                // Variable length, return size of length prefix
                Ok((FieldType::Vec, 4))
            }
            IdlType::Option { inner } => {
                let (inner_type, inner_size) = self.idl_type_to_field_type(inner)?;
                // Option adds 1 byte for discriminant
                Ok((FieldType::Option(Box::new(inner_type)), 1 + inner_size))
            }
            IdlType::Defined { name } => {
                // For defined types, we'd need to look up the definition
                // For now, treat as opaque data
                Ok((FieldType::Defined(name.clone()), 32)) // Assume 32 bytes
            }
        }
    }

    /// Infer zero semantics for a field based on name and type
    fn infer_zero_semantics(&self, field_name: &str, field_type: &FieldType) -> crate::ZeroSemantics {
        match field_type {
            FieldType::PublicKey => {
                // Zero pubkey often indicates unset/default
                if field_name.contains("authority") || field_name.contains("owner") {
                    crate::ZeroSemantics::NeverWritten
                } else {
                    crate::ZeroSemantics::ExplicitlyZero
                }
            }
            FieldType::U64 | FieldType::U128 => {
                // Numeric fields can often be legitimately zero
                if field_name.contains("amount") || field_name.contains("balance") {
                    crate::ZeroSemantics::ValidZero
                } else {
                    crate::ZeroSemantics::ExplicitlyZero
                }
            }
            FieldType::Bool => crate::ZeroSemantics::ValidZero, // false is a valid boolean value
            _ => crate::ZeroSemantics::ExplicitlyZero, // Default to explicitly zero
        }
    }

    /// Compute layout commitment hash
    fn compute_layout_commitment(&self, fields: &[FieldLayout]) -> String {
        use sha2::{Digest, Sha256};
        
        let mut hasher = Sha256::new();
        
        // Hash field information
        for field in fields {
            hasher.update(field.name.as_bytes());
            hasher.update(&[field.field_type.discriminant()]);
            hasher.update(&field.offset.to_le_bytes());
            hasher.update(&field.size.to_le_bytes());
        }
        
        let hash = hasher.finalize();
        hex::encode(hash)
    }
}

impl Default for SolanaLayoutCompiler {
    fn default() -> Self {
        Self::new()
    }
}

impl FieldType {
    /// Get a discriminant value for hashing
    fn discriminant(&self) -> u8 {
        match self {
            FieldType::Bool => 0,
            FieldType::U8 => 1,
            FieldType::I8 => 2,
            FieldType::U16 => 3,
            FieldType::I16 => 4,
            FieldType::U32 => 5,
            FieldType::I32 => 6,
            FieldType::U64 => 7,
            FieldType::I64 => 8,
            FieldType::U128 => 9,
            FieldType::I128 => 10,
            FieldType::Pubkey => 11,
            FieldType::PublicKey => 12,
            FieldType::String => 13,
            FieldType::Bytes(_) => 14,
            FieldType::Bytes8 => 15,
            FieldType::Array(_) => 16,
            FieldType::Vec(_) => 17,
            FieldType::Option(_) => 18,
            FieldType::Custom(_) => 19,
            FieldType::Defined(_) => 20,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layout_compiler_creation() {
        let compiler = SolanaLayoutCompiler::new();
        assert!(compiler.include_discriminator);
    }

    #[test]
    fn test_layout_commitment_computation() {
        let compiler = SolanaLayoutCompiler::new();
        let fields = vec![
            FieldLayout {
                name: "test_field".to_string(),
                field_type: FieldType::U64,
                offset: 0,
                size: 8,
                zero_semantics: crate::ZeroSemantics::ValidZero,
            }
        ];
        
        let commitment = compiler.compute_layout_commitment(&fields);
        assert!(!commitment.is_empty());
        assert_eq!(commitment.len(), 64); // SHA256 hex string
    }

    #[test]
    fn test_zero_semantics_inference() {
        let compiler = SolanaLayoutCompiler::new();
        
        let authority_semantics = compiler.infer_zero_semantics("authority", &FieldType::PublicKey);
        assert_eq!(authority_semantics, crate::ZeroSemantics::NeverWritten);
        
        let amount_semantics = compiler.infer_zero_semantics("amount", &FieldType::U64);
        assert_eq!(amount_semantics, crate::ZeroSemantics::ValidZero);
        
        let bool_semantics = compiler.infer_zero_semantics("flag", &FieldType::Bool);
        assert_eq!(bool_semantics, crate::ZeroSemantics::ValidZero);
    }

    #[cfg(not(feature = "anchor"))]
    #[test]
    fn test_idl_compilation_without_anchor_feature() {
        let compiler = SolanaLayoutCompiler::new();
        let result = compiler.compile_from_idl("{}");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Anchor feature not enabled"));
    }

    #[cfg(not(feature = "solana"))]
    #[test]
    fn test_program_account_compilation_without_solana_feature() {
        let compiler = SolanaLayoutCompiler::new();
        let result = compiler.compile_from_program_account(&[]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Solana SDK feature not enabled"));
    }
} 