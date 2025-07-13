//! Solana IDL parsing and account resolution example
//!
//! This example demonstrates how to parse Anchor IDL files and resolve 
//! Solana account addresses using traverse-solana.

#[cfg(feature = "solana")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use traverse_solana::anchor::IdlParser;
    use traverse_solana::resolver::{SolanaKeyResolver, SolanaQuery};

    println!("🔧 Solana IDL Parsing and Account Resolution Example");
    println!("=====================================================");

    // Sample token program IDL (simplified)
    let sample_idl = r#"{
        "version": "0.1.0",
        "name": "token_program",
        "programId": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
        "instructions": [
            {
                "name": "transfer",
                "accounts": [
                    {
                        "name": "source",
                        "isMut": true,
                        "isSigner": false
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
                "name": "TokenAccount",
                "type": {
                    "kind": "struct",
                    "fields": [
                        {
                            "name": "mint",
                            "type": "publicKey"
                        },
                        {
                            "name": "owner",
                            "type": "publicKey"
                        },
                        {
                            "name": "amount",
                            "type": "u64"
                        },
                        {
                            "name": "is_frozen",
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

    // Step 1: Parse IDL
    println!("\n📋 Step 1: Parsing IDL");
    let idl = IdlParser::parse_idl(sample_idl)?;
    println!("✓ Program: {} ({})", idl.name, idl.program_id);
    println!("  - Instructions: {}", idl.instructions.len());
    println!("  - Accounts: {}", idl.accounts.len());

    // Step 2: Extract account layouts
    println!("\n🏗️  Step 2: Extracting Account Layouts");
    let layouts = IdlParser::extract_account_layouts(&idl)?;
    
    for layout in &layouts {
        println!("Account: {}", layout.name);
        println!("  - Fields: {}", layout.fields.len());
        println!("  - Total size: {} bytes", layout.total_size);
        
        for field in &layout.fields {
            println!("    {} ({}) - {} bytes at offset {}", 
                field.name, field.field_type, field.size, field.offset);
        }
    }

    // Step 3: Demonstrate query parsing and resolution
    println!("\n🎯 Step 3: Query Resolution Examples");
    
    let resolver = SolanaKeyResolver::with_program_id(idl.program_id.clone());
    
    // Example queries
    let example_queries = vec![
        "token_account",  // Direct access
        "token_account[9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM]",  // PDA with seed
        "token_balance[mint123,owner456]",  // ATA pattern
        "token_account.amount",  // Field access
    ];

    for query_str in example_queries {
        match SolanaKeyResolver::parse_query(query_str) {
            Ok(query) => {
                println!("\nQuery: '{}'", query_str);
                println!("  Type: {:?}", query);
                
                match resolver.resolve_account_address(&query) {
                    Ok(address) => {
                        println!("  ✓ Resolved: {}", address);
                    }
                    Err(e) => {
                        println!("  ⚠️ Error: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("Query '{}' - Parse Error: {}", query_str, e);
            }
        }
    }

    // Step 4: Demonstrate batch resolution
    println!("\n📦 Step 4: Batch Query Resolution");
    
    let batch_queries = vec![
        SolanaQuery::Direct { 
            account_name: "token_mint".to_string() 
        },
        SolanaQuery::PDA { 
            account_name: "user_vault".to_string(),
            seeds: vec!["vault".to_string(), "user123".to_string()]
        },
    ];

    let batch_results = resolver.resolve_batch_queries(&batch_queries);
    println!("Batch resolution results:");
    
    for (i, result) in batch_results.iter().enumerate() {
        match result {
            Ok(address) => println!("  Query {}: ✓ {}", i, address),
            Err(e) => println!("  Query {}: ✗ {}", i, e),
        }
    }

    // Step 5: Field key resolution example
    println!("\n🔑 Step 5: Field Key Resolution");
    
    let account_address = "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM";
    let field_path = "amount";
    let field_offset = 64; // Hypothetical offset for amount field
    
    let field_key = resolver.resolve_field_key(account_address, field_path, field_offset)?;
    println!("Field key for {}.{}: {}", account_address, field_path, field_key);

    println!("\n✅ Solana example completed successfully!");
    println!("\nKey takeaways:");
    println!("- IDL parsing extracts program structure and account layouts");
    println!("- Query resolution supports PDAs, ATAs, and field access patterns");
    println!("- Batch operations enable efficient multi-account resolution");
    println!("- Field keys provide deterministic identifiers for account fields");

    Ok(())
}

#[cfg(not(feature = "solana"))]
fn main() {
    eprintln!("❌ This example requires the 'solana' feature to be enabled.");
    eprintln!("Run with: cargo run --example solana_idl_example --features solana");
    std::process::exit(1);
} 