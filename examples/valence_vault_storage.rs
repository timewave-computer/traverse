//! Valence One Way Vault Storage Query Example
//! 
//! This example demonstrates how to use traverse to query the withdrawRequests storage
//! from the Valence One Way Vault (vTEST) deployed at
//! 0xf2b85c389a771035a9bd147d4bf87987a7f9cf98
//!
//! The contract is an EIP-1967 transparent proxy with implementation at
//! 0x425de7d367027bea8896631e69bf0606d7d7ce6f
//!
//! Reference: https://etherscan.io/address/0xf2b85c389a771035a9bd147d4bf87987a7f9cf98#readProxyContract#F30

use std::{format, println};
use traverse_core::{KeyResolver, LayoutInfo, StaticKeyPath, Key, StorageEntry, TypeInfo};
use traverse_ethereum::{EthereumKeyResolver, AbiFetcher};

#[cfg(feature = "client")]
use valence_domain_clients::clients::ethereum::EthereumClient;

#[cfg(feature = "client")]
use valence_domain_clients::evm::request_provider_client::RequestProviderClient;

#[cfg(feature = "client")]
use alloy::providers::Provider;

/// Example contract address for Valence One Way Vault
const VALENCE_VAULT_ADDRESS: &str = "0xf2b85c389a771035a9bd147d4bf87987a7f9cf98";

/// Example implementation address 
const IMPLEMENTATION_ADDRESS: &str = "0x425de7d367027bea8896631e69bf0606d7d7ce6f";

/// Default RPC endpoint (user should replace with their own)
const DEFAULT_RPC_URL: &str = "https://mainnet.infura.io/v3/YOUR_PROJECT_ID";

/// Helper function to extract bytes from Key enum
fn extract_key_bytes(key: &Key) -> [u8; 32] {
    match key {
        Key::Fixed(key) => *key,
        Key::Variable(key) => {
            let mut fixed = [0u8; 32];
            let len = std::cmp::min(key.len(), 32);
            fixed[32-len..].copy_from_slice(&key[key.len()-len..]);
            fixed
        }
    }
}

#[cfg(feature = "client")]
/// Fetch real storage data from Ethereum using valence-domain-clients
async fn fetch_real_storage_data(rpc_url: &str, contract_addr: &str, storage_key: [u8; 32]) -> Result<String, Box<dyn std::error::Error>> {
    println!("🌐 Fetching real storage data from Ethereum...");
    println!("   RPC: {}", rpc_url);
    println!("   Contract: {}", contract_addr);
    println!("   Storage Key: 0x{}", hex::encode(storage_key));
    
    // Create an Ethereum client using valence-domain-clients  
    let dummy_mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    let client = EthereumClient::new(rpc_url, dummy_mnemonic, None)?;
    
    // Get the provider to make raw RPC calls
    let provider = client.get_request_provider().await?;
    
    // Parse contract address
    let contract_address: alloy::primitives::Address = contract_addr.parse()?;
    
    // Convert storage key to alloy format
    let slot_b256 = alloy::primitives::B256::from_slice(&storage_key);
    
    // Get storage proof using eth_getProof
    println!("   Calling eth_getProof...");
    let proof_response = provider.get_proof(contract_address, vec![slot_b256]).await?;
    
    if proof_response.storage_proof.is_empty() {
        return Err("No storage proof returned".into());
    }
    
    let storage_proof = &proof_response.storage_proof[0];
    let value_hex = format!("0x{}", hex::encode(storage_proof.value.to_be_bytes()));
    
    println!("   ✅ Storage value: {}", value_hex);
    println!("   ✅ Proof nodes: {}", storage_proof.proof.len());
    
    Ok(value_hex)
}

#[cfg(not(feature = "client"))]
/// Error when client feature is not enabled - no mock mode
async fn fetch_real_storage_data(_rpc_url: &str, _contract_addr: &str, _storage_key: [u8; 32]) -> Result<String, Box<dyn std::error::Error>> {
    Err("Client feature not enabled. Build with: cargo build --features client".into())
}

/// Customize the layout for Valence Vault contract by adding withdrawRequests field
fn customize_layout_for_valence_vault(mut layout: LayoutInfo) -> LayoutInfo {
    // Add withdrawRequests storage entry
    layout.storage.push(StorageEntry {
        label: "_withdrawRequests".to_string(),
        slot: "7".to_string(),
        offset: 0,
        type_name: "t_uint64".to_string(),
    });
    
    // Add t_uint64 type definition
    layout.types.push(TypeInfo {
        label: "t_uint64".to_string(),
        number_of_bytes: "8".to_string(),
        encoding: "inplace".to_string(),
        base: None,
        key: None,
        value: None,
    });
    
    layout
}

/// Demonstrate querying withdraw requests storage with real RPC calls
async fn query_withdraw_requests_live(layout: &LayoutInfo, rpc_url: &str) -> Result<StaticKeyPath, Box<dyn std::error::Error>> {
    let resolver = EthereumKeyResolver;
    let query = "_withdrawRequests";
    
    println!("🔍 Querying withdraw requests storage (LIVE)...");
    println!("   Contract: {}", VALENCE_VAULT_ADDRESS);
    println!("   Query: {}", query);
    
    let path = resolver.resolve(layout, query)
        .map_err(|e| format!("Storage key resolution failed: {}", e))?;
    
    println!("✅ Storage key resolved:");
    let storage_key = extract_key_bytes(&path.key);
    println!("   Key: 0x{}", hex::encode(storage_key));
    println!("   Layout commitment: 0x{}", hex::encode(path.layout_commitment));
    
    // Fetch real storage value
    let storage_value = fetch_real_storage_data(rpc_url, VALENCE_VAULT_ADDRESS, storage_key).await?;
    println!("   Real Storage Value: {}", storage_value);
    
    // Decode withdraw requests as uint64
    if let Ok(value_bytes) = hex::decode(storage_value.strip_prefix("0x").unwrap_or(&storage_value)) {
        if value_bytes.len() >= 8 {
            let withdraw_requests = u64::from_be_bytes(value_bytes[24..32].try_into().unwrap_or([0u8; 8]));
            println!("   📊 Decoded Withdraw Requests: {} requests", withdraw_requests);
        }
    }
    
    // Generate coprocessor-compatible JSON output
    let coprocessor_json = serde_json::json!({
        "storage_query": {
            "query": query,
            "storage_key": hex::encode(storage_key),
            "layout_commitment": hex::encode(path.layout_commitment),
            "field_size": path.field_size,
            "offset": path.offset,
            "storage_value": storage_value
        }
    });
    
    println!("\n📋 Coprocessor JSON format:");
    let output_json = serde_json::json!({
        "contract_address": VALENCE_VAULT_ADDRESS,
        "chain": "ethereum",
        "network": "mainnet",
        "withdrawRequests_query": coprocessor_json
    });
    println!("{}", serde_json::to_string_pretty(&output_json).unwrap());
    
    Ok(path)
}

/// Main example function - with real ABI fetching and RPC calls
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        run_example().await
    })
}

async fn run_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Valence One Way Vault Withdraw Requests Query Example");
    let separator = "=".repeat(60);
    println!("{}", separator);
    println!();
    println!("This example demonstrates querying the withdrawRequests storage from");
    println!("the Valence One Way Vault contract using REAL ABI fetching and");
    println!("RPC calls to fetch actual on-chain storage data.");
    println!();
    println!("Contract Details:");
    println!("  • Proxy: {}", VALENCE_VAULT_ADDRESS);
    println!("  • Implementation: {}", IMPLEMENTATION_ADDRESS);
    println!("  • Token: Valence One Way Vault (vTEST)");
    println!("  • Query: withdrawRequests() storage slot");
    println!();
    
    // Get RPC URL from environment or use default
    let rpc_url = std::env::var("ETHEREUM_RPC_URL").unwrap_or_else(|_| DEFAULT_RPC_URL.to_string());
    
    if rpc_url.contains("YOUR_PROJECT_ID") {
        println!("⚠️  WARNING: Using default RPC URL with placeholder API key");
        println!("   Set ETHEREUM_RPC_URL environment variable to use a real RPC endpoint");
        println!("   Example: export ETHEREUM_RPC_URL=https://mainnet.infura.io/v3/your_actual_key");
        println!();
    } else {
        println!("🌐 Using RPC endpoint: {}", rpc_url);
        println!();
    }
    
    // Get Etherscan API key from environment
    let etherscan_api_key = std::env::var("ETHERSCAN_API_KEY").ok();
    
    if etherscan_api_key.is_none() {
        println!("⚠️  WARNING: No Etherscan API key provided");
        println!("   Set ETHERSCAN_API_KEY environment variable for higher rate limits");
        println!("   Example: export ETHERSCAN_API_KEY=your_etherscan_api_key");
        println!("   ⚠️  Proceeding without API key (lower rate limits)");
        println!();
    } else {
        println!("🔑 Using Etherscan API key for ABI fetching");
        println!();
    }
    
    // Create ABI fetcher and fetch real contract layout
    let abi_fetcher = AbiFetcher::new(etherscan_api_key, None);
    
    println!("📥 Fetching real contract ABI and generating storage layout...");
    let layout = match abi_fetcher.fetch_and_generate_layout(VALENCE_VAULT_ADDRESS).await {
        Ok(layout) => {
            println!("✅ Successfully fetched ABI and generated layout!");
            customize_layout_for_valence_vault(layout)
        }
        Err(e) => {
            println!("❌ Failed to fetch ABI: {}", e);
            println!("   This example requires a valid Etherscan API key or network access");
            println!("   Set ETHERSCAN_API_KEY environment variable and try again");
            return Err(format!("ABI fetching failed: {}", e).into());
        }
    };
    
    println!("📐 Layout loaded with {} storage entries", layout.storage.len());
    println!("   Contract name: {}", layout.contract_name);
    println!("   Layout commitment: 0x{}", hex::encode(layout.commitment()));
    
    // Show detected storage entries
    println!("\n📋 Detected storage entries:");
    for entry in &layout.storage {
        println!("   • {} (slot {}, type: {})", entry.label, entry.slot, entry.type_name);
    }
    println!();
    
    // Query withdraw requests with live data
    let requests_path = query_withdraw_requests_live(&layout, &rpc_url).await?;
    
    println!("\n🔗 Real Integration Steps Completed:");
    println!("✅ 1. Fetched contract ABI from Etherscan");
    println!("✅ 2. Generated storage layout from ABI");
    println!("✅ 3. Resolved withdrawRequests storage key");
    println!("✅ 4. Made RPC call to fetch storage value");
    println!("✅ 5. Generated coprocessor-compatible JSON");
    println!();
    
    println!("🔑 Ready-to-use storage key for ZK proof generation:");
    println!("  WithdrawRequests key: 0x{}", hex::encode(extract_key_bytes(&requests_path.key)));
    
    println!("\n📝 Example CLI command for proof generation:");
    println!("# Generate real storage proof with traverse-cli");
    println!("cargo run -p traverse-cli --features client -- generate-proof \\");
    println!("  --slot 0x{} \\", hex::encode(extract_key_bytes(&requests_path.key)));
    println!("  --contract {} \\", VALENCE_VAULT_ADDRESS);
    println!("  --rpc {} \\", rpc_url);
    println!("  --output withdrawrequests_proof.json");
    
    println!("\n✨ Example complete! Real ABI fetched, withdrawRequests storage queried!");
    println!("   Ready for ZK coprocessor integration with actual contract data.");
    
    Ok(())
}
