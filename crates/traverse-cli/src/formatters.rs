//! Output formatting and data conversion utilities
//!
//! This module provides functions for converting between different data formats
//! and handling output to files or stdout.

use crate::cli::OutputFormat;
use anyhow::Result;
use base64::{engine::general_purpose::STANDARD, Engine};
use serde::{Deserialize, Serialize};
use std::path::Path;
use traverse_core::{Key, LayoutInfo, StaticKeyPath};

/// Coprocessor-compatible storage query format
#[derive(Serialize, Deserialize)]
pub struct CoprocessorStorageQuery {
    /// Original query string
    pub query: String,
    /// Pre-computed storage key (hex encoded)
    pub storage_key: String,
    /// Layout commitment for verification (hex encoded)
    pub layout_commitment: String,
    /// Field size in bytes
    pub field_size: Option<u8>,
    /// Byte offset within storage slot
    pub offset: Option<u8>,
}

/// Helper function to write output to file or stdout
pub fn write_output(content: &str, output_path: Option<&Path>) -> Result<()> {
    if let Some(path) = output_path {
        std::fs::write(path, content)?;
        println!("Output written to {}", path.display());
    } else {
        println!("{}", content);
    }
    Ok(())
}

/// Helper function to load layout from file
pub fn load_layout(layout_path: &Path) -> Result<LayoutInfo> {
    let content = std::fs::read_to_string(layout_path)?;
    let layout: LayoutInfo = serde_json::from_str(&content)?;
    Ok(layout)
}

/// Convert storage key to hex string with proper padding
fn storage_key_to_hex(key: &Key) -> String {
    match key {
        Key::Fixed(key) => hex::encode(key),
        Key::Variable(key) => {
            let mut padded = [0u8; 32];
            let len = std::cmp::min(key.len(), 32);
            padded[32 - len..].copy_from_slice(&key[key.len() - len..]);
            hex::encode(padded)
        }
    }
}

/// Convert storage path to coprocessor format
pub fn path_to_coprocessor_query(path: &StaticKeyPath, query: &str) -> CoprocessorStorageQuery {
    CoprocessorStorageQuery {
        query: query.to_string(),
        storage_key: storage_key_to_hex(&path.key),
        layout_commitment: hex::encode(path.layout_commitment),
        field_size: path.field_size,
        offset: path.offset,
    }
}

/// Format storage path based on output format
pub fn format_storage_path(
    path: &StaticKeyPath,
    query: &str,
    format: &OutputFormat,
) -> Result<String> {
    match format {
        OutputFormat::Traverse => serde_json::to_string_pretty(path).map_err(Into::into),
        OutputFormat::CoprocessorJson => {
            let coprocessor_payload = path_to_coprocessor_query(path, query);
            serde_json::to_string_pretty(&coprocessor_payload).map_err(Into::into)
        }
        OutputFormat::Toml => {
            let coprocessor_payload = path_to_coprocessor_query(path, query);
            toml::to_string_pretty(&coprocessor_payload).map_err(Into::into)
        }
        OutputFormat::Binary => match &path.key {
            Key::Fixed(bytes) => {
                let binary_data = if let Some(offset) = path.offset {
                    let mut result = bytes.to_vec();
                    result.push(offset);
                    result
                } else {
                    bytes.to_vec()
                };
                Ok(format!("{}: {}", path.name, STANDARD.encode(&binary_data)))
            }
            Key::Variable(bytes) => {
                let binary_data = if let Some(offset) = path.offset {
                    let mut result = bytes.clone();
                    result.push(offset);
                    result
                } else {
                    bytes.clone()
                };
                Ok(format!("{}: {}", path.name, STANDARD.encode(&binary_data)))
            }
        },
        OutputFormat::Base64 => match &path.key {
            Key::Fixed(bytes) => {
                let binary_data = if let Some(offset) = path.offset {
                    let mut result = bytes.to_vec();
                    result.push(offset);
                    result
                } else {
                    bytes.to_vec()
                };
                Ok(format!("{}: {}", path.name, STANDARD.encode(&binary_data)))
            }
            Key::Variable(bytes) => {
                let binary_data = if let Some(offset) = path.offset {
                    let mut result = bytes.clone();
                    result.push(offset);
                    result
                } else {
                    bytes.clone()
                };
                Ok(format!("{}: {}", path.name, STANDARD.encode(&binary_data)))
            }
        },
    }
}

/// Format multiple storage paths based on output format
pub fn format_storage_paths(paths: &[StaticKeyPath], format: &OutputFormat) -> Result<String> {
    match format {
        OutputFormat::Traverse => serde_json::to_string_pretty(paths).map_err(Into::into),
        OutputFormat::CoprocessorJson => {
            let coprocessor_payloads: Vec<CoprocessorStorageQuery> = paths
                .iter()
                .map(|path| path_to_coprocessor_query(path, path.name))
                .collect();
            serde_json::to_string_pretty(&coprocessor_payloads).map_err(Into::into)
        }
        OutputFormat::Toml => {
            #[derive(Serialize)]
            struct TomlOutput {
                queries: Vec<CoprocessorStorageQuery>,
            }
            let coprocessor_payloads: Vec<CoprocessorStorageQuery> = paths
                .iter()
                .map(|path| path_to_coprocessor_query(path, path.name))
                .collect();
            let output = TomlOutput {
                queries: coprocessor_payloads,
            };
            toml::to_string_pretty(&output).map_err(Into::into)
        }
        OutputFormat::Binary => {
            let coprocessor_payloads: Vec<CoprocessorStorageQuery> = paths
                .iter()
                .map(|path| path_to_coprocessor_query(path, path.name))
                .collect();
            let binary_data = bincode::serialize(&coprocessor_payloads)?;
            Ok(format!(
                "Binary data: {} bytes\nBase64: {}",
                binary_data.len(),
                STANDARD.encode(&binary_data)
            ))
        }
        OutputFormat::Base64 => {
            let coprocessor_payloads: Vec<CoprocessorStorageQuery> = paths
                .iter()
                .map(|path| path_to_coprocessor_query(path, path.name))
                .collect();
            let binary_data = bincode::serialize(&coprocessor_payloads)?;
            Ok(format!(
                "Binary data: {} bytes\nBase64: {}",
                binary_data.len(),
                STANDARD.encode(&binary_data)
            ))
        }
    }
}
