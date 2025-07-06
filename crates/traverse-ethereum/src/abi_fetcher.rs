//! ABI fetcher for retrieving contract ABIs from various sources
//!
//! This module provides functionality to fetch contract Application Binary Interfaces (ABIs)
//! from various sources including Etherscan API, and convert them to storage layouts.

use serde::{Deserialize, Serialize};
use traverse_core::{LayoutInfo, StorageEntry, TraverseError, TypeInfo, ZeroSemantics};

/// Etherscan API response for contract ABI
#[derive(Debug, Deserialize)]
struct EtherscanAbiResponse {
    status: String,
    message: String,
    result: String, // ABI JSON as string
}

/// Simplified ABI structure for storage layout inference
#[derive(Debug, Deserialize, Serialize)]
struct AbiItem {
    #[serde(rename = "type")]
    item_type: String,
    name: Option<String>,
    inputs: Option<Vec<AbiInput>>,
    outputs: Option<Vec<AbiOutput>>,
    #[serde(rename = "stateMutability")]
    state_mutability: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct AbiInput {
    name: String,
    #[serde(rename = "type")]
    input_type: String,
    #[serde(rename = "internalType")]
    internal_type: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct AbiOutput {
    name: String,
    #[serde(rename = "type")]
    output_type: String,
    #[serde(rename = "internalType")]
    internal_type: Option<String>,
}

/// ABI fetcher for retrieving contract ABIs from various sources
pub struct AbiFetcher {
    /// Etherscan API key
    pub etherscan_api_key: Option<String>,
    /// Base URL for Etherscan API (default: mainnet)
    pub etherscan_base_url: String,
    /// HTTP client for making requests
    client: reqwest::Client,
}

impl AbiFetcher {
    /// Create a new ABI fetcher
    ///
    /// # Arguments
    ///
    /// * `etherscan_api_key` - Optional Etherscan API key for higher rate limits
    /// * `etherscan_base_url` - Base URL for Etherscan API (None = mainnet)
    ///
    /// # Returns
    ///
    /// New AbiFetcher instance
    pub fn new(etherscan_api_key: Option<String>, etherscan_base_url: Option<String>) -> Self {
        let base_url = etherscan_base_url.unwrap_or_else(|| "https://api.etherscan.io".to_string());

        Self {
            etherscan_api_key,
            etherscan_base_url: base_url,
            client: reqwest::Client::new(),
        }
    }

    /// Fetch contract ABI from Etherscan
    ///
    /// # Arguments
    ///
    /// * `contract_address` - Ethereum contract address (with or without 0x prefix)
    ///
    /// # Returns
    ///
    /// * `Ok(String)` - ABI JSON string
    /// * `Err(TraverseError)` - Failed to fetch ABI
    pub async fn fetch_abi_from_etherscan(
        &self,
        contract_address: &str,
    ) -> Result<String, TraverseError> {
        let clean_address = contract_address
            .strip_prefix("0x")
            .unwrap_or(contract_address);

        let mut url = format!(
            "{}/api?module=contract&action=getabi&address=0x{}",
            self.etherscan_base_url, clean_address
        );

        if let Some(ref api_key) = self.etherscan_api_key {
            url.push_str(&format!("&apikey={}", api_key));
        }

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| TraverseError::Network(format!("Failed to fetch ABI: {}", e)))?;

        let response_text = response
            .text()
            .await
            .map_err(|e| TraverseError::Network(format!("Failed to read response: {}", e)))?;

        let etherscan_response: EtherscanAbiResponse = serde_json::from_str(&response_text)
            .map_err(|e| {
                TraverseError::Serialization(format!("Failed to parse Etherscan response: {}", e))
            })?;

        if etherscan_response.status != "1" {
            return Err(TraverseError::Network(format!(
                "Etherscan API error: {}",
                etherscan_response.message
            )));
        }

        Ok(etherscan_response.result)
    }

    /// Generate a realistic storage layout from contract ABI
    ///
    /// This creates an estimated storage layout based on common ERC20/ERC721 patterns
    /// and the functions available in the ABI. For accurate layouts, use forge inspect.
    ///
    /// # Arguments
    ///
    /// * `contract_address` - Contract address for metadata
    /// * `abi_json` - ABI JSON string
    ///
    /// # Returns
    ///
    /// * `Ok(LayoutInfo)` - Estimated storage layout
    /// * `Err(TraverseError)` - Failed to generate layout
    pub fn generate_layout_from_abi(
        &self,
        contract_address: &str,
        abi_json: &str,
    ) -> Result<LayoutInfo, TraverseError> {
        let abi: Vec<AbiItem> = serde_json::from_str(abi_json)
            .map_err(|e| TraverseError::Serialization(format!("Failed to parse ABI: {}", e)))?;

        let mut storage = Vec::new();

        // Detect contract patterns and generate typical storage layout
        let has_balance_of = abi
            .iter()
            .any(|item| item.name.as_ref().is_some_and(|name| name == "balanceOf"));

        let has_total_supply = abi
            .iter()
            .any(|item| item.name.as_ref().is_some_and(|name| name == "totalSupply"));

        let has_owner = abi
            .iter()
            .any(|item| item.name.as_ref().is_some_and(|name| name == "owner"));

        let has_name = abi
            .iter()
            .any(|item| item.name.as_ref().is_some_and(|name| name == "name"));

        let has_symbol = abi
            .iter()
            .any(|item| item.name.as_ref().is_some_and(|name| name == "symbol"));

        let has_decimals = abi
            .iter()
            .any(|item| item.name.as_ref().is_some_and(|name| name == "decimals"));

        let has_allowance = abi
            .iter()
            .any(|item| item.name.as_ref().is_some_and(|name| name == "allowance"));

        // Generate ERC20-style storage layout if patterns match
        if has_balance_of && has_total_supply {
            // Standard ERC20 layout pattern
            storage.push(StorageEntry {
                label: "_balances".to_string(),
                slot: "0".to_string(),
                offset: 0,
                type_name: "t_mapping_address_uint256".to_string(),
                zero_semantics: ZeroSemantics::NeverWritten,
            });

            if has_allowance {
                storage.push(StorageEntry {
                    label: "_allowances".to_string(),
                    slot: "1".to_string(),
                    offset: 0,
                    type_name: "t_mapping_address_mapping_address_uint256".to_string(),
                    zero_semantics: ZeroSemantics::NeverWritten,
                });
            }

            storage.push(StorageEntry {
                label: "_totalSupply".to_string(),
                slot: "2".to_string(),
                offset: 0,
                type_name: "t_uint256".to_string(),
                zero_semantics: ZeroSemantics::NeverWritten,
            });

            if has_name {
                storage.push(StorageEntry {
                    label: "_name".to_string(),
                    slot: "3".to_string(),
                    offset: 0,
                    type_name: "t_string_storage".to_string(),
                    zero_semantics: ZeroSemantics::NeverWritten,
                });
            }

            if has_symbol {
                storage.push(StorageEntry {
                    label: "_symbol".to_string(),
                    slot: "4".to_string(),
                    offset: 0,
                    type_name: "t_string_storage".to_string(),
                    zero_semantics: ZeroSemantics::NeverWritten,
                });
            }

            if has_decimals {
                storage.push(StorageEntry {
                    label: "_decimals".to_string(),
                    slot: "5".to_string(),
                    offset: 0,
                    type_name: "t_uint8".to_string(),
                    zero_semantics: ZeroSemantics::NeverWritten,
                });
            }
        }

        // Add owner if present
        if has_owner {
            storage.push(StorageEntry {
                label: "_owner".to_string(),
                slot: "6".to_string(),
                offset: 0,
                type_name: "t_address".to_string(),
                zero_semantics: ZeroSemantics::NeverWritten,
            });
        }

        // Add common administrative storage slots
        let admin_functions = ["paused", "pause", "unpause"];
        let has_pause = abi.iter().any(|item| {
            item.name
                .as_ref()
                .is_some_and(|name| admin_functions.iter().any(|func| name == func))
        });

        if has_pause {
            storage.push(StorageEntry {
                label: "_paused".to_string(),
                slot: "100".to_string(), // Common pattern: admin vars at higher slots
                offset: 0,
                type_name: "t_bool".to_string(),
                zero_semantics: ZeroSemantics::NeverWritten,
            });
        }

        // Generate type definitions
        let types = Self::generate_standard_types();

        // Get contract name from address or use default
        let contract_name = format!(
            "Contract_{}",
            contract_address
                .strip_prefix("0x")
                .unwrap_or(contract_address)
        );

        Ok(LayoutInfo {
            contract_name,
            storage,
            types,
        })
    }

    /// Generate standard type definitions used in most contracts
    fn generate_standard_types() -> Vec<TypeInfo> {
        vec![
            TypeInfo {
                label: "t_address".to_string(),
                number_of_bytes: "20".to_string(),
                encoding: "inplace".to_string(),
                base: None,
                key: None,
                value: None,
            },
            TypeInfo {
                label: "t_uint256".to_string(),
                number_of_bytes: "32".to_string(),
                encoding: "inplace".to_string(),
                base: None,
                key: None,
                value: None,
            },
            TypeInfo {
                label: "t_uint8".to_string(),
                number_of_bytes: "1".to_string(),
                encoding: "inplace".to_string(),
                base: None,
                key: None,
                value: None,
            },
            TypeInfo {
                label: "t_bool".to_string(),
                number_of_bytes: "1".to_string(),
                encoding: "inplace".to_string(),
                base: None,
                key: None,
                value: None,
            },
            TypeInfo {
                label: "t_string_storage".to_string(),
                number_of_bytes: "32".to_string(),
                encoding: "bytes".to_string(),
                base: None,
                key: None,
                value: None,
            },
            TypeInfo {
                label: "t_mapping_address_uint256".to_string(),
                number_of_bytes: "32".to_string(),
                encoding: "mapping".to_string(),
                base: None,
                key: Some("t_address".to_string()),
                value: Some("t_uint256".to_string()),
            },
            TypeInfo {
                label: "t_mapping_address_mapping_address_uint256".to_string(),
                number_of_bytes: "32".to_string(),
                encoding: "mapping".to_string(),
                base: None,
                key: Some("t_address".to_string()),
                value: Some("t_mapping_address_uint256".to_string()),
            },
        ]
    }

    /// Fetch ABI and generate layout in one call
    ///
    /// # Arguments
    ///
    /// * `contract_address` - Ethereum contract address
    ///
    /// # Returns
    ///
    /// * `Ok(LayoutInfo)` - Contract storage layout
    /// * `Err(TraverseError)` - Failed to fetch ABI or generate layout
    pub async fn fetch_and_generate_layout(
        &self,
        contract_address: &str,
    ) -> Result<LayoutInfo, TraverseError> {
        println!("Fetching ABI for contract: {}", contract_address);
        let abi_json = self.fetch_abi_from_etherscan(contract_address).await?;

        println!("Generating storage layout from ABI...");
        let layout = self.generate_layout_from_abi(contract_address, &abi_json)?;

        println!(
            "Generated layout with {} storage entries",
            layout.storage.len()
        );
        Ok(layout)
    }
}
