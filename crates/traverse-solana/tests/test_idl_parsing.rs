//! Integration tests for Solana IDL parsing
//!
//! These tests verify that the IDL parsing functionality works correctly
//! with real Anchor program IDL files.

use traverse_solana::anchor::IdlParser;

const SAMPLE_IDL: &str = r#"{
    "version": "0.1.0",
    "name": "test_program",
    "programId": "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM",
    "instructions": [
        {
            "name": "initialize",
            "accounts": [
                {
                    "name": "user",
                    "isMut": false,
                    "isSigner": true
                }
            ],
            "args": [
                {
                    "name": "amount",
                    "type": "u64"
                }
            ]
        }
    ],
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
                    },
                    {
                        "name": "is_active",
                        "type": "bool"
                    }
                ]
            }
        }
    ],
    "types": [],
    "events": [],
    "errors": [],
    "constants": []
}"#;

#[test]
fn test_parse_sample_idl() {
    let idl = IdlParser::parse_idl(SAMPLE_IDL).unwrap();
    
    assert_eq!(idl.name, "test_program");
    assert_eq!(idl.program_id, "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM");
    assert_eq!(idl.instructions.len(), 1);
    assert_eq!(idl.accounts.len(), 1);
    
    // Test instruction parsing
    let instruction = &idl.instructions[0];
    assert_eq!(instruction.name, "initialize");
    assert_eq!(instruction.accounts.len(), 1);
    assert_eq!(instruction.args.len(), 1);
    
    // Test account parsing
    let account = &idl.accounts[0];
    assert_eq!(account.name, "UserAccount");
}

#[test]
fn test_extract_account_layouts() {
    let idl = IdlParser::parse_idl(SAMPLE_IDL).unwrap();
    let layouts = IdlParser::extract_account_layouts(&idl).unwrap();
    
    assert_eq!(layouts.len(), 1);
    
    let layout = &layouts[0];
    assert_eq!(layout.name, "UserAccount");
    assert_eq!(layout.fields.len(), 3);
    
    // Check field types
    assert_eq!(layout.fields[0].name, "authority");
    assert_eq!(layout.fields[0].field_type, "pubkey");
    
    assert_eq!(layout.fields[1].name, "balance");
    assert_eq!(layout.fields[1].field_type, "u64");
    
    assert_eq!(layout.fields[2].name, "is_active");
    assert_eq!(layout.fields[2].field_type, "bool");
}

#[test]
fn test_idl_parsing_error_handling() {
    // Test invalid JSON
    let result = IdlParser::parse_idl("{ invalid json }");
    assert!(result.is_err());
    
    // Test missing required fields
    let incomplete_idl = r#"{"version": "0.1.0"}"#;
    let result = IdlParser::parse_idl(incomplete_idl);
    assert!(result.is_err());
}

#[test]
fn test_complex_field_types() {
    let complex_idl = r#"{
        "version": "0.1.0",
        "name": "complex_program",
        "programId": "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM",
        "instructions": [],
        "accounts": [
            {
                "name": "ComplexAccount", 
                "type": {
                    "kind": "struct",
                    "fields": [
                        {
                            "name": "simple_field",
                            "type": "u32"
                        },
                        {
                            "name": "array_field",
                            "type": {
                                "kind": "array",
                                "type": "u8",
                                "size": 32
                            }
                        },
                        {
                            "name": "option_field",
                            "type": {
                                "kind": "option",
                                "type": "u64"
                            }
                        }
                    ]
                }
            }
        ],
        "types": [],
        "events": [],
        "errors": [],
        "constants": []
    }"#;
    
    let idl = IdlParser::parse_idl(complex_idl).unwrap();
    let layouts = IdlParser::extract_account_layouts(&idl).unwrap();
    
    assert_eq!(layouts.len(), 1);
    let layout = &layouts[0];
    assert_eq!(layout.fields.len(), 3);
    
    // Test complex type parsing
    assert_eq!(layout.fields[0].field_type, "u32");
    assert_eq!(layout.fields[1].field_type, "[u8; 32]");
    assert_eq!(layout.fields[2].field_type, "Option<u64>");
} 