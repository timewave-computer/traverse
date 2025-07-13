//! Anchor IDL parsing and analysis
//!
//! This module provides functionality to parse Anchor IDL files and extract
//! account structure information for storage layout compilation.

use crate::{SolanaError, SolanaResult};
use serde::{Deserialize, Serialize};
use alloc::{format, string::String, vec::Vec, collections::BTreeMap};

/// Complete Anchor IDL structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolanaIdl {
    /// IDL version
    pub version: String,
    /// Program name
    pub name: String,
    /// Program ID
    #[serde(rename = "programId")]
    pub program_id: String,
    /// Program instructions
    pub instructions: Vec<IdlInstruction>,
    /// Account definitions
    pub accounts: Vec<IdlAccount>,
    /// Type definitions
    pub types: Vec<IdlType>,
    /// Event definitions
    #[serde(default)]
    pub events: Vec<IdlEvent>,
    /// Error definitions
    #[serde(default)]
    pub errors: Vec<IdlError>,
    /// Constants
    #[serde(default)]
    pub constants: Vec<IdlConstant>,
    /// Metadata
    #[serde(default)]
    pub metadata: Option<IdlMetadata>,
}

/// Anchor instruction definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdlInstruction {
    /// Instruction name
    pub name: String,
    /// Instruction accounts
    pub accounts: Vec<IdlAccountItem>,
    /// Instruction arguments
    pub args: Vec<IdlField>,
    /// Return type
    #[serde(default)]
    pub returns: Option<IdlType>,
}

/// Account item in instruction context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdlAccountItem {
    /// Account name
    pub name: String,
    /// Whether account is mutable
    #[serde(rename = "isMut")]
    pub is_mut: bool,
    /// Whether account is signer
    #[serde(rename = "isSigner")]
    pub is_signer: bool,
    /// Whether account is optional
    #[serde(default)]
    pub is_optional: bool,
    /// Account description
    #[serde(default)]
    pub docs: Vec<String>,
    /// PDA information
    #[serde(default)]
    pub pda: Option<IdlPda>,
}

/// PDA (Program Derived Account) definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdlPda {
    /// PDA seeds
    pub seeds: Vec<IdlSeed>,
    /// Program ID for derivation
    #[serde(rename = "programId")]
    pub program_id: Option<String>,
}

/// PDA seed definition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum IdlSeed {
    /// Constant seed value
    #[serde(rename = "const")]
    Const {
        /// Seed type
        #[serde(rename = "type")]
        seed_type: String,
        /// Seed value
        value: serde_json::Value,
    },
    /// Account field seed
    #[serde(rename = "account")]
    Account {
        /// Account name
        account: String,
        /// Account field path
        path: Option<String>,
    },
    /// Argument seed
    #[serde(rename = "arg")]
    Arg {
        /// Argument name
        arg: String,
        /// Argument path
        path: Option<String>,
    },
}

/// Account type definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdlAccount {
    /// Account name
    pub name: String,
    /// Account discriminator
    #[serde(default)]
    pub discriminator: Option<Vec<u8>>,
    /// Account type definition
    #[serde(rename = "type")]
    pub account_type: IdlAccountType,
}

/// Account type variants
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum IdlAccountType {
    /// Struct account type
    #[serde(rename = "struct")]
    Struct {
        /// Struct fields
        fields: Vec<IdlField>,
    },
    /// Enum account type
    #[serde(rename = "enum")]
    Enum {
        /// Enum variants
        variants: Vec<IdlEnumVariant>,
    },
}

/// Field definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdlField {
    /// Field name
    pub name: String,
    /// Field type
    #[serde(rename = "type")]
    pub field_type: IdlType,
    /// Field documentation
    #[serde(default)]
    pub docs: Vec<String>,
}

/// Enum variant definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdlEnumVariant {
    /// Variant name
    pub name: String,
    /// Variant fields (if any)
    #[serde(default)]
    pub fields: Option<IdlEnumFields>,
}

/// Enum variant fields
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum IdlEnumFields {
    /// Named fields
    Named(Vec<IdlField>),
    /// Tuple fields
    Tuple(Vec<IdlType>),
}

/// Type definition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum IdlType {
    /// Primitive type (string)
    Primitive(String),
    /// Complex type definition
    Complex {
        /// Type name/kind
        #[serde(flatten)]
        kind: IdlTypeKind,
    },
    /// Defined type reference
    Defined {
        /// Defined type name
        defined: String,
    },
}

/// Complex type kinds
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum IdlTypeKind {
    /// Option type
    Option {
        /// Inner type
        #[serde(rename = "type")]
        inner: Box<IdlType>,
    },
    /// Vector type
    Vec {
        /// Element type
        #[serde(rename = "type")]
        element: Box<IdlType>,
    },
    /// Array type
    Array {
        /// Element type
        #[serde(rename = "type")]
        element: Box<IdlType>,
        /// Array size
        size: u32,
    },
    /// Struct type
    Struct {
        /// Struct fields
        fields: Vec<IdlField>,
    },
    /// Enum type
    Enum {
        /// Enum variants
        variants: Vec<IdlEnumVariant>,
    },
}

/// Event definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdlEvent {
    /// Event name
    pub name: String,
    /// Event fields
    pub fields: Vec<IdlField>,
}

/// Error definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdlError {
    /// Error code
    pub code: u32,
    /// Error name
    pub name: String,
    /// Error message
    #[serde(default)]
    pub msg: Option<String>,
}

/// Constant definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdlConstant {
    /// Constant name
    pub name: String,
    /// Constant type
    #[serde(rename = "type")]
    pub constant_type: IdlType,
    /// Constant value
    pub value: serde_json::Value,
}

/// IDL metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdlMetadata {
    /// Address
    #[serde(default)]
    pub address: Option<String>,
    /// Origin
    #[serde(default)]
    pub origin: Option<String>,
    /// Upgrade authority
    #[serde(rename = "upgradeAuthority", default)]
    pub upgrade_authority: Option<String>,
}

/// IDL parser for Anchor programs
pub struct IdlParser;

impl IdlParser {
    /// Parse IDL from JSON string
    pub fn parse_idl(json_content: &str) -> SolanaResult<SolanaIdl> {
        serde_json::from_str(json_content)
            .map_err(|e| SolanaError::InvalidIdl(format!("Failed to parse IDL JSON: {}", e)))
    }

    /// Extract account layouts from IDL
    pub fn extract_account_layouts(idl: &SolanaIdl) -> SolanaResult<Vec<AccountLayoutInfo>> {
        let mut layouts = Vec::new();

        for account in &idl.accounts {
            let layout = Self::convert_account_to_layout(account, &idl.types)?;
            layouts.push(layout);
        }

        Ok(layouts)
    }

    /// Convert IDL account to layout info
    fn convert_account_to_layout(
        account: &IdlAccount,
        types: &[IdlType],
    ) -> SolanaResult<AccountLayoutInfo> {
        let mut fields = Vec::new();

        match &account.account_type {
            IdlAccountType::Struct { fields: struct_fields } => {
                for field in struct_fields {
                    let layout_field = Self::convert_field_to_layout(field, types)?;
                    fields.push(layout_field);
                }
            }
            IdlAccountType::Enum { variants } => {
                // For enums, create a discriminator field plus variant fields
                fields.push(FieldLayoutInfo {
                    name: "__discriminator".to_string(),
                    field_type: "u8".to_string(),
                    offset: 0,
                    size: 1,
                    description: "Enum discriminator".to_string(),
                });

                // Add variant fields (simplified - real implementation would be more complex)
                for (i, variant) in variants.iter().enumerate() {
                    if let Some(variant_fields) = &variant.fields {
                        match variant_fields {
                            IdlEnumFields::Named(named_fields) => {
                                for field in named_fields {
                                    let mut layout_field = Self::convert_field_to_layout(field, types)?;
                                    layout_field.name = format!("{}_{}", variant.name, layout_field.name);
                                    fields.push(layout_field);
                                }
                            }
                            IdlEnumFields::Tuple(tuple_types) => {
                                for (j, tuple_type) in tuple_types.iter().enumerate() {
                                    let field_name = format!("{}_field_{}", variant.name, j);
                                    let type_info = Self::get_type_info(tuple_type, types)?;
                                    fields.push(FieldLayoutInfo {
                                        name: field_name,
                                        field_type: type_info.type_name,
                                        offset: 0, // Would need proper offset calculation
                                        size: type_info.size,
                                        description: format!("Tuple field {} in variant {}", j, variant.name),
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(AccountLayoutInfo {
            name: account.name.clone(),
            discriminator: account.discriminator.clone().unwrap_or_default(),
            fields,
            total_size: Self::calculate_total_size(&fields),
        })
    }

    /// Convert IDL field to layout field
    fn convert_field_to_layout(
        field: &IdlField,
        types: &[IdlType],
    ) -> SolanaResult<FieldLayoutInfo> {
        let type_info = Self::get_type_info(&field.field_type, types)?;
        
        Ok(FieldLayoutInfo {
            name: field.name.clone(),
            field_type: type_info.type_name,
            offset: 0, // Would need proper offset calculation in real implementation
            size: type_info.size,
            description: field.docs.join(" "),
        })
    }

    /// Get type information from IDL type
    fn get_type_info(idl_type: &IdlType, _types: &[IdlType]) -> SolanaResult<TypeInfo> {
        match idl_type {
            IdlType::Primitive(name) => {
                let (size, type_name) = match name.as_str() {
                    "bool" => (1, "bool"),
                    "u8" => (1, "u8"),
                    "i8" => (1, "i8"),
                    "u16" => (2, "u16"),
                    "i16" => (2, "i16"),
                    "u32" => (4, "u32"),
                    "i32" => (4, "i32"),
                    "u64" => (8, "u64"),
                    "i64" => (8, "i64"),
                    "u128" => (16, "u128"),
                    "i128" => (16, "i128"),
                    "f32" => (4, "f32"),
                    "f64" => (8, "f64"),
                    "publicKey" | "pubkey" => (32, "pubkey"),
                    "string" => (0, "string"), // Variable size
                    "bytes" => (0, "bytes"), // Variable size
                    _ => return Err(SolanaError::InvalidIdl(format!("Unknown primitive type: {}", name))),
                };
                Ok(TypeInfo {
                    type_name: type_name.to_string(),
                    size,
                })
            }
            IdlType::Complex { kind } => {
                match kind {
                    IdlTypeKind::Option { inner } => {
                        let inner_info = Self::get_type_info(inner, _types)?;
                        Ok(TypeInfo {
                            type_name: format!("Option<{}>", inner_info.type_name),
                            size: 1 + inner_info.size, // 1 byte for Some/None + inner size
                        })
                    }
                    IdlTypeKind::Vec { element } => {
                        let _element_info = Self::get_type_info(element, _types)?;
                        Ok(TypeInfo {
                            type_name: format!("Vec<{}>", _element_info.type_name),
                            size: 0, // Variable size
                        })
                    }
                    IdlTypeKind::Array { element, size } => {
                        let element_info = Self::get_type_info(element, _types)?;
                        Ok(TypeInfo {
                            type_name: format!("[{}; {}]", element_info.type_name, size),
                            size: element_info.size * (*size as usize),
                        })
                    }
                    IdlTypeKind::Struct { fields } => {
                        let mut total_size = 0;
                        for field in fields {
                            let field_info = Self::get_type_info(&field.field_type, _types)?;
                            total_size += field_info.size;
                        }
                        Ok(TypeInfo {
                            type_name: "struct".to_string(),
                            size: total_size,
                        })
                    }
                    IdlTypeKind::Enum { variants: _ } => {
                        Ok(TypeInfo {
                            type_name: "enum".to_string(),
                            size: 1, // Discriminator size, actual size varies
                        })
                    }
                }
            }
            IdlType::Defined { defined } => {
                Ok(TypeInfo {
                    type_name: defined.clone(),
                    size: 0, // Would need to resolve from types array
                })
            }
        }
    }

    /// Calculate total size of fields
    fn calculate_total_size(fields: &[FieldLayoutInfo]) -> usize {
        fields.iter().map(|f| f.size).sum()
    }

    /// Extract PDAs from IDL
    pub fn extract_pdas(idl: &SolanaIdl) -> Vec<PdaInfo> {
        let mut pdas = Vec::new();

        for instruction in &idl.instructions {
            for account in &instruction.accounts {
                if let Some(pda) = &account.pda {
                    pdas.push(PdaInfo {
                        account_name: account.name.clone(),
                        seeds: pda.seeds.clone(),
                        program_id: pda.program_id.clone(),
                    });
                }
            }
        }

        pdas
    }
}

/// Account layout information extracted from IDL
#[derive(Debug, Clone)]
pub struct AccountLayoutInfo {
    /// Account type name
    pub name: String,
    /// Account discriminator bytes
    pub discriminator: Vec<u8>,
    /// Account fields
    pub fields: Vec<FieldLayoutInfo>,
    /// Total account size (if fixed)
    pub total_size: usize,
}

/// Field layout information
#[derive(Debug, Clone)]
pub struct FieldLayoutInfo {
    /// Field name
    pub name: String,
    /// Field type
    pub field_type: String,
    /// Byte offset in account data
    pub offset: usize,
    /// Field size in bytes
    pub size: usize,
    /// Field description
    pub description: String,
}

/// Type information
#[derive(Debug, Clone)]
pub struct TypeInfo {
    /// Type name
    pub type_name: String,
    /// Type size in bytes (0 for variable size)
    pub size: usize,
}

/// PDA information extracted from IDL
#[derive(Debug, Clone)]
pub struct PdaInfo {
    /// Account name that uses this PDA
    pub account_name: String,
    /// PDA seeds
    pub seeds: Vec<IdlSeed>,
    /// Program ID for derivation
    pub program_id: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_idl() {
        let idl_json = r#"{
            "version": "0.1.0",
            "name": "test_program",
            "programId": "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM",
            "instructions": [],
            "accounts": [
                {
                    "name": "UserAccount",
                    "type": {
                        "kind": "struct",
                        "fields": [
                            {
                                "name": "authority",
                                "type": "publicKey"
                            },
                            {
                                "name": "balance",
                                "type": "u64"
                            }
                        ]
                    }
                }
            ],
            "types": []
        }"#;

        let idl = IdlParser::parse_idl(idl_json).unwrap();
        assert_eq!(idl.name, "test_program");
        assert_eq!(idl.accounts.len(), 1);
        assert_eq!(idl.accounts[0].name, "UserAccount");

        let layouts = IdlParser::extract_account_layouts(&idl).unwrap();
        assert_eq!(layouts.len(), 1);
        assert_eq!(layouts[0].fields.len(), 2);
        assert_eq!(layouts[0].fields[0].name, "authority");
        assert_eq!(layouts[0].fields[0].field_type, "pubkey");
        assert_eq!(layouts[0].fields[1].name, "balance");
        assert_eq!(layouts[0].fields[1].field_type, "u64");
    }

    #[test]
    fn test_type_info_primitives() {
        let bool_info = IdlParser::get_type_info(&IdlType::Primitive("bool".to_string()), &[]).unwrap();
        assert_eq!(bool_info.size, 1);
        assert_eq!(bool_info.type_name, "bool");

        let u64_info = IdlParser::get_type_info(&IdlType::Primitive("u64".to_string()), &[]).unwrap();
        assert_eq!(u64_info.size, 8);
        assert_eq!(u64_info.type_name, "u64");

        let pubkey_info = IdlParser::get_type_info(&IdlType::Primitive("publicKey".to_string()), &[]).unwrap();
        assert_eq!(pubkey_info.size, 32);
        assert_eq!(pubkey_info.type_name, "pubkey");
    }

    #[test]
    fn test_invalid_idl_parsing() {
        let invalid_json = "{ invalid json }";
        let result = IdlParser::parse_idl(invalid_json);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SolanaError::InvalidIdl(_)));
    }
} 