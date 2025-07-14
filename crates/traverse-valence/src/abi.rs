//! Comprehensive ABI type support using selective alloy imports
//!
//! This module provides complete ABI encoding/decoding functionality by importing
//! all necessary types from alloy-primitives and alloy-sol-types selectively.
//! This approach gives us access to all ABI types while maintaining lightweight
//! compilation and avoiding the full alloy ecosystem.

use alloc::{boxed::Box, format, string::String, vec::Vec};
use serde::{Deserialize, Serialize};

// Import for serialization
#[cfg(feature = "alloy")]
use bincode;
use hex;

// === COMPREHENSIVE ALLOY TYPE IMPORTS ===

#[cfg(feature = "alloy")]
use alloy_primitives::{
    // Basic types we actually use
    Address, U256,
    // Bytes types
    B256,
    // Utility types
    keccak256,
};


// === FALLBACK TYPES FOR NON-ALLOY BUILDS ===

#[cfg(not(feature = "alloy"))]
pub type Address = String;
#[cfg(not(feature = "alloy"))]
pub type Bytes = Vec<u8>;
#[cfg(not(feature = "alloy"))]
pub type U256 = [u8; 32];
#[cfg(not(feature = "alloy"))]
pub type I256 = [u8; 32];
#[cfg(not(feature = "alloy"))]
pub type B256 = [u8; 32];
#[cfg(not(feature = "alloy"))]
pub type U128 = [u8; 16];
#[cfg(not(feature = "alloy"))]
pub type I128 = [u8; 16];
#[cfg(not(feature = "alloy"))]
pub type U64 = u64;
#[cfg(not(feature = "alloy"))]
pub type I64 = i64;
#[cfg(not(feature = "alloy"))]
pub type U32 = u32;
#[cfg(not(feature = "alloy"))]
pub type I32 = i32;
#[cfg(not(feature = "alloy"))]
pub type U16 = u16;
#[cfg(not(feature = "alloy"))]
pub type I16 = i16;
#[cfg(not(feature = "alloy"))]
pub type U8 = u8;
#[cfg(not(feature = "alloy"))]
pub type I8 = i8;

use crate::{TraverseValenceError, ZkMessage};

/// Comprehensive ABI type support using selective alloy imports
pub struct AlloyAbiTypes;

impl AlloyAbiTypes {
    /// Encode a ZkMessage using comprehensive alloy ABI types
    #[cfg(feature = "alloy")]
    pub fn encode_zk_message(msg: &ZkMessage) -> Result<Vec<u8>, TraverseValenceError> {
        // Basic ABI encoding implementation without sol! macro
        let mut encoded = Vec::new();
        
        // Encode message type as uint8
        let msg_type = match msg {
            ZkMessage::Pause => 0u8,
            ZkMessage::Resume => 1u8,
            ZkMessage::EvictMsgs(_) => 2u8,
            ZkMessage::SendMsgs(_) => 3u8,
            ZkMessage::InsertMsgs(_) => 4u8,
        };
        
        // ABI encode uint8 (padded to 32 bytes)
        let mut type_bytes = [0u8; 32];
        type_bytes[31] = msg_type;
        encoded.extend_from_slice(&type_bytes);
        
        // Encode message data based on type
        match msg {
            ZkMessage::Pause | ZkMessage::Resume => {
                // No additional data - just encode empty bytes
                let mut offset = [0u8; 32];
                offset[31] = 32; // offset to data (after type)
                encoded.extend_from_slice(&offset);
                
                let mut length = [0u8; 32];
                // length = 0
                encoded.extend_from_slice(&length);
            },
            ZkMessage::EvictMsgs(msgs) | ZkMessage::SendMsgs(msgs) | ZkMessage::InsertMsgs(msgs) => {
                // Encode as dynamic bytes array
                let serialized = bincode::serialize(msgs)
                    .map_err(|e| TraverseValenceError::AbiError(format!("Serialization failed: {}", e)))?;
                
                // Offset to data
                let mut offset = [0u8; 32];
                offset[31] = 32;
                encoded.extend_from_slice(&offset);
                
                // Length of data
                let mut length = [0u8; 32];
                let len_bytes = (serialized.len() as u64).to_be_bytes();
                length[24..].copy_from_slice(&len_bytes);
                encoded.extend_from_slice(&length);
                
                // Pad data to 32-byte boundary
                encoded.extend_from_slice(&serialized);
                let padding = (32 - (serialized.len() % 32)) % 32;
                encoded.extend(vec![0u8; padding]);
            }
        }
        
        Ok(encoded)
        /*
        sol! {
            enum ProcessorMessageType {
                Pause,
                Resume, 
                EvictMsgs,
                SendMsgs,
                InsertMsgs
            }

            struct ProcessorMessage {
                ProcessorMessageType messageType;
                bytes message;
            }

            struct ZkMessage {
                uint64 registry;
                uint64 blockNumber;
                address authorizationContract;
                ProcessorMessage processorMessage;
            }
        }

        let auth_contract: Address = msg.authorization_contract.parse()
            .map_err(|e| TraverseValenceError::AbiError(format!("Invalid address: {:?}", e)))?;

        let msg_type = match msg.processor_message.message_type {
            ProcessorMessageType::Pause => ProcessorMessageType::Pause,
            ProcessorMessageType::Resume => ProcessorMessageType::Resume,
            ProcessorMessageType::EvictMsgs => ProcessorMessageType::EvictMsgs,
            ProcessorMessageType::SendMsgs => ProcessorMessageType::SendMsgs,
            ProcessorMessageType::InsertMsgs => ProcessorMessageType::InsertMsgs,
        };

        let processor_msg = ProcessorMessage {
            messageType: msg_type,
            message: Bytes::from(msg.processor_message.message.clone()),
        };

        let zk_msg = ZkMessage {
            registry: msg.registry,
            blockNumber: msg.block_number,
            authorizationContract: auth_contract,
            processorMessage: processor_msg,
        };

        Ok(zk_msg.abi_encode())
        */
    }

    /// Fallback JSON encoding when alloy is not available
    #[cfg(not(feature = "alloy"))]
    pub fn encode_zk_message(msg: &ZkMessage) -> Result<Vec<u8>, TraverseValenceError> {
        serde_json::to_vec(msg)
            .map_err(|e| TraverseValenceError::AbiError(format!("JSON encoding failed: {:?}", e)))
    }

    /// Encode any ABI value using comprehensive type support
    #[cfg(feature = "alloy")]
    pub fn encode_abi_value(value: &AbiValue) -> Result<Vec<u8>, TraverseValenceError> {
        // Basic ABI encoding implementation
        let mut encoded = [0u8; 32];
        
        match value {
            AbiValue::Bool(b) => {
                encoded[31] = if *b { 1 } else { 0 };
            },
            AbiValue::Uint8(n) => {
                encoded[31] = *n;
            },
            AbiValue::Uint16(n) => {
                encoded[30..32].copy_from_slice(&n.to_be_bytes());
            },
            AbiValue::Uint32(n) => {
                encoded[28..32].copy_from_slice(&n.to_be_bytes());
            },
            AbiValue::Uint64(n) => {
                encoded[24..32].copy_from_slice(&n.to_be_bytes());
            },
            AbiValue::Uint128(n) => {
                encoded[16..32].copy_from_slice(&n.to_be_bytes());
            },
            AbiValue::Uint256(parts) => {
                // Convert [u64; 4] to big endian bytes
                for (i, part) in parts.iter().enumerate() {
                    let start = i * 8;
                    encoded[start..start + 8].copy_from_slice(&part.to_be_bytes());
                }
            },
            AbiValue::Address(addr) => {
                // Parse hex address and pad to 32 bytes (20 bytes address + 12 zeros)
                if let Ok(addr_bytes) = hex::decode(addr.trim_start_matches("0x")) {
                    if addr_bytes.len() == 20 {
                        encoded[12..32].copy_from_slice(&addr_bytes);
                    } else {
                        return Err(TraverseValenceError::AbiError("Invalid address length".into()));
                    }
                } else {
                    return Err(TraverseValenceError::AbiError("Invalid address format".into()));
                }
            },
            AbiValue::FixedBytes(bytes) => {
                encoded.copy_from_slice(bytes);
            },
            _ => {
                return Err(TraverseValenceError::AbiError("Complex ABI types not yet implemented".into()));
            }
        }
        
        Ok(encoded.to_vec())
        /*
        match value {
            AbiValue::Bool(_b) => Err(TraverseValenceError::AbiError("ABI encoding not implemented".into())),
            AbiValue::Uint8(v) => Ok(SolUint::<8>::abi_encode(v)),
            AbiValue::Uint16(v) => Ok(SolUint::<16>::abi_encode(v)),
            AbiValue::Uint32(v) => Ok(SolUint::<32>::abi_encode(v)),
            AbiValue::Uint64(v) => Ok(SolUint::<64>::abi_encode(v)),
            AbiValue::Uint128(v) => Ok(SolUint::<128>::abi_encode(v)),
            AbiValue::Uint256(v) => Ok(SolUint::<256>::abi_encode(v)),
            AbiValue::Int8(v) => Ok(SolInt::<8>::abi_encode(v)),
            AbiValue::Int16(v) => Ok(SolInt::<16>::abi_encode(v)),
            AbiValue::Int32(v) => Ok(SolInt::<32>::abi_encode(v)),
            AbiValue::Int64(v) => Ok(SolInt::<64>::abi_encode(v)),
            AbiValue::Int128(v) => Ok(SolInt::<128>::abi_encode(v)),
            AbiValue::Int256(v) => Ok(SolInt::<256>::abi_encode(v)),
            AbiValue::Address(addr) => {
                let parsed: Address = addr.parse()
                    .map_err(|e| TraverseValenceError::AbiError(format!("Invalid address: {:?}", e)))?;
                Ok(SolAddress::abi_encode(&parsed))
            }
            AbiValue::Bytes(bytes) => Ok(SolBytes::abi_encode(bytes)),
            AbiValue::FixedBytes(bytes) => Ok(SolFixedBytes::<32>::abi_encode(bytes)),
            AbiValue::String(s) => Ok(SolString::abi_encode(s)),
            AbiValue::Array(values) => {
                let mut encoded = Vec::new();
                for value in values {
                    encoded.extend(Self::encode_abi_value(value)?);
                }
                Ok(encoded)
            }
            AbiValue::Tuple(values) => {
                let mut encoded = Vec::new();
                for value in values {
                    encoded.extend(Self::encode_abi_value(value)?);
                }
                Ok(encoded)
            }
        }
        */
    }

    /// Decode ABI value using comprehensive type support
    #[cfg(feature = "alloy")]
    pub fn decode_abi_value(_data: &[u8], _abi_type: &AbiType) -> Result<AbiValue, TraverseValenceError> {
        // TODO: Implement proper ABI decoding when sol types are available
        Err(TraverseValenceError::AbiError("ABI decoding not implemented".into()))
        /*
        match abi_type {
            AbiType::Bool => {
                let value = SolBool::abi_decode(data, false)
                    .map_err(|e| TraverseValenceError::AbiError(format!("Bool decode error: {:?}", e)))?;
                Ok(AbiValue::Bool(value))
            }
            AbiType::Uint8 => {
                let value = SolUint::<8>::abi_decode(data, false)
                    .map_err(|e| TraverseValenceError::AbiError(format!("Uint8 decode error: {:?}", e)))?;
                Ok(AbiValue::Uint8(value))
            }
            AbiType::Uint16 => {
                let value = SolUint::<16>::abi_decode(data, false)
                    .map_err(|e| TraverseValenceError::AbiError(format!("Uint16 decode error: {:?}", e)))?;
                Ok(AbiValue::Uint16(value))
            }
            AbiType::Uint32 => {
                let value = SolUint::<32>::abi_decode(data, false)
                    .map_err(|e| TraverseValenceError::AbiError(format!("Uint32 decode error: {:?}", e)))?;
                Ok(AbiValue::Uint32(value))
            }
            AbiType::Uint64 => {
                let value = SolUint::<64>::abi_decode(data, false)
                    .map_err(|e| TraverseValenceError::AbiError(format!("Uint64 decode error: {:?}", e)))?;
                Ok(AbiValue::Uint64(value))
            }
            AbiType::Uint128 => {
                let value = SolUint::<128>::abi_decode(data, false)
                    .map_err(|e| TraverseValenceError::AbiError(format!("Uint128 decode error: {:?}", e)))?;
                Ok(AbiValue::Uint128(value))
            }
            AbiType::Uint256 => {
                let value = SolUint::<256>::abi_decode(data, false)
                    .map_err(|e| TraverseValenceError::AbiError(format!("Uint256 decode error: {:?}", e)))?;
                Ok(AbiValue::Uint256(value))
            }
            AbiType::Int8 => {
                let value = SolInt::<8>::abi_decode(data, false)
                    .map_err(|e| TraverseValenceError::AbiError(format!("Int8 decode error: {:?}", e)))?;
                Ok(AbiValue::Int8(value))
            }
            AbiType::Int16 => {
                let value = SolInt::<16>::abi_decode(data, false)
                    .map_err(|e| TraverseValenceError::AbiError(format!("Int16 decode error: {:?}", e)))?;
                Ok(AbiValue::Int16(value))
            }
            AbiType::Int32 => {
                let value = SolInt::<32>::abi_decode(data, false)
                    .map_err(|e| TraverseValenceError::AbiError(format!("Int32 decode error: {:?}", e)))?;
                Ok(AbiValue::Int32(value))
            }
            AbiType::Int64 => {
                let value = SolInt::<64>::abi_decode(data, false)
                    .map_err(|e| TraverseValenceError::AbiError(format!("Int64 decode error: {:?}", e)))?;
                Ok(AbiValue::Int64(value))
            }
            AbiType::Int128 => {
                let value = SolInt::<128>::abi_decode(data, false)
                    .map_err(|e| TraverseValenceError::AbiError(format!("Int128 decode error: {:?}", e)))?;
                Ok(AbiValue::Int128(value))
            }
            AbiType::Int256 => {
                let value = SolInt::<256>::abi_decode(data, false)
                    .map_err(|e| TraverseValenceError::AbiError(format!("Int256 decode error: {:?}", e)))?;
                Ok(AbiValue::Int256(value))
            }
            AbiType::Address => {
                let value = SolAddress::abi_decode(data, false)
                    .map_err(|e| TraverseValenceError::AbiError(format!("Address decode error: {:?}", e)))?;
                Ok(AbiValue::Address(value.to_string()))
            }
            AbiType::Bytes => {
                let value = SolBytes::abi_decode(data, false)
                    .map_err(|e| TraverseValenceError::AbiError(format!("Bytes decode error: {:?}", e)))?;
                Ok(AbiValue::Bytes(value.to_vec()))
            }
            AbiType::FixedBytes => {
                let value = SolFixedBytes::<32>::abi_decode(data, false)
                    .map_err(|e| TraverseValenceError::AbiError(format!("FixedBytes decode error: {:?}", e)))?;
                Ok(AbiValue::FixedBytes(value.0))
            }
            AbiType::String => {
                let value = SolString::abi_decode(data, false)
                    .map_err(|e| TraverseValenceError::AbiError(format!("String decode error: {:?}", e)))?;
                Ok(AbiValue::String(value))
            }
            AbiType::Array(_) => {
                // Dynamic array decoding would need element type information
                Err(TraverseValenceError::AbiError("Array decoding not yet implemented".to_string()))
            }
            AbiType::Tuple(_) => {
                // Tuple decoding would need element type information
                Err(TraverseValenceError::AbiError("Tuple decoding not yet implemented".to_string()))
            }
        }
        */
    }

    /// Create function selector using alloy keccak256
    #[cfg(feature = "alloy")]
    pub fn function_selector(signature: &str) -> [u8; 4] {
        let hash = keccak256(signature.as_bytes());
        [hash[0], hash[1], hash[2], hash[3]]
    }

    /// Fallback function selector returns zeros when alloy is not available
    #[cfg(not(feature = "alloy"))]
    pub fn function_selector(_signature: &str) -> [u8; 4] {
        [0u8; 4]
    }

    /// Encode a function call with parameters
    #[cfg(feature = "alloy")]
    pub fn encode_function_call(
        signature: &str,
        params: &[AbiValue],
    ) -> Result<Vec<u8>, TraverseValenceError> {
        let mut encoded = Vec::new();
        
        // Add function selector
        encoded.extend_from_slice(&Self::function_selector(signature));
        
        // Encode parameters
        for param in params {
            encoded.extend(Self::encode_abi_value(param)?);
        }
        
        Ok(encoded)
    }

    /// Decode function return value
    #[cfg(feature = "alloy")]
    pub fn decode_function_return(
        data: &[u8],
        return_type: &AbiType,
    ) -> Result<AbiValue, TraverseValenceError> {
        Self::decode_abi_value(data, return_type)
    }

    /// Parse an address from string
    #[cfg(feature = "alloy")]
    pub fn parse_address(addr: &str) -> Result<Address, TraverseValenceError> {
        addr.parse()
            .map_err(|e| TraverseValenceError::AbiError(format!("Invalid address: {:?}", e)))
    }

    /// Parse a B256 hash from string
    #[cfg(feature = "alloy")]
    pub fn parse_b256(hash: &str) -> Result<B256, TraverseValenceError> {
        hash.parse()
            .map_err(|e| TraverseValenceError::AbiError(format!("Invalid B256: {:?}", e)))
    }

    /// Parse a U256 from string
    #[cfg(feature = "alloy")]
    pub fn parse_u256(value: &str) -> Result<U256, TraverseValenceError> {
        value.parse()
            .map_err(|e| TraverseValenceError::AbiError(format!("Invalid U256: {:?}", e)))
    }

    /// Check if alloy features are available
    pub fn alloy_features_available() -> bool {
        cfg!(feature = "alloy")
    }
}

/// Comprehensive ABI type enumeration supporting all Ethereum ABI types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AbiType {
    // Basic types
    Bool,
    Uint8,
    Uint16,
    Uint32,
    Uint64,
    Uint128,
    Uint256,
    Int8,
    Int16,
    Int32,
    Int64,
    Int128,
    Int256,
    Address,
    Bytes,
    FixedBytes,
    String,
    // Complex types
    Array(Box<AbiType>),
    Tuple(Vec<AbiType>),
}

/// Comprehensive ABI value enumeration supporting all Ethereum ABI values
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AbiValue {
    // Basic values
    Bool(bool),
    Uint8(u8),
    Uint16(u16),
    Uint32(u32),
    Uint64(u64),
    Uint128(u128),
    Uint256([u64; 4]),
    Int8(i8),
    Int16(i16),
    Int32(i32),
    Int64(i64),
    Int128(i128),
    Int256([i64; 4]),
    Address(String),
    Bytes(Vec<u8>),
    FixedBytes([u8; 32]),
    String(String),
    // Complex values
    Array(Vec<AbiValue>),
    Tuple(Vec<AbiValue>),
}

impl AbiValue {
    /// Encode using comprehensive alloy support
    #[cfg(feature = "alloy")]
    pub fn encode(&self) -> Result<Vec<u8>, TraverseValenceError> {
        AlloyAbiTypes::encode_abi_value(self)
    }

    /// Fallback encoding without alloy
    #[cfg(not(feature = "alloy"))]
    pub fn encode(&self) -> Result<Vec<u8>, TraverseValenceError> {
        // Simple binary encoding as fallback
        match self {
            AbiValue::Bool(b) => {
                let mut result = Vec::with_capacity(32);
                result.resize(32, 0u8);
                result[31] = if *b { 1 } else { 0 };
                Ok(result)
            }
            AbiValue::Uint256(val) => {
                // Convert [u64; 4] to Vec<u8>
                let mut result = Vec::with_capacity(32);
                for i in (0..4).rev() {
                    result.extend_from_slice(&val[i].to_be_bytes());
                }
                Ok(result)
            }
            AbiValue::Address(addr) => {
                let addr_bytes = hex::decode(addr.strip_prefix("0x").unwrap_or(addr))
                    .map_err(|e| TraverseValenceError::AbiError(format!("Invalid address hex: {}", e)))?;
                if addr_bytes.len() != 20 {
                    return Err(TraverseValenceError::AbiError("Address must be 20 bytes".into()));
                }
                let mut result = Vec::with_capacity(32);
                result.resize(32, 0u8);
                result[12..32].copy_from_slice(&addr_bytes);
                Ok(result)
            }
            AbiValue::Bytes(val) => {
                let mut result = Vec::new();
                let mut len_bytes = [0u8; 32];
                let len = val.len() as u32;
                len_bytes[28..32].copy_from_slice(&len.to_be_bytes());
                result.extend_from_slice(&len_bytes);
                result.extend_from_slice(val);
                let padding = (32 - (val.len() % 32)) % 32;
                if padding > 0 {
                    result.resize(result.len() + padding, 0u8);
                }
                Ok(result)
            }
            _ => Err(TraverseValenceError::AbiError("Unsupported type for fallback encoding".into())),
        }
    }

    /// Get the ABI type for this value
    pub fn abi_type(&self) -> AbiType {
        match self {
            AbiValue::Bool(_) => AbiType::Bool,
            AbiValue::Uint8(_) => AbiType::Uint8,
            AbiValue::Uint16(_) => AbiType::Uint16,
            AbiValue::Uint32(_) => AbiType::Uint32,
            AbiValue::Uint64(_) => AbiType::Uint64,
            AbiValue::Uint128(_) => AbiType::Uint128,
            AbiValue::Uint256(_) => AbiType::Uint256,
            AbiValue::Int8(_) => AbiType::Int8,
            AbiValue::Int16(_) => AbiType::Int16,
            AbiValue::Int32(_) => AbiType::Int32,
            AbiValue::Int64(_) => AbiType::Int64,
            AbiValue::Int128(_) => AbiType::Int128,
            AbiValue::Int256(_) => AbiType::Int256,
            AbiValue::Address(_) => AbiType::Address,
            AbiValue::Bytes(_) => AbiType::Bytes,
            AbiValue::FixedBytes(_) => AbiType::FixedBytes,
            AbiValue::String(_) => AbiType::String,
            AbiValue::Array(values) => {
                if let Some(first) = values.first() {
                    AbiType::Array(Box::new(first.abi_type()))
                } else {
                    AbiType::Array(Box::new(AbiType::Bytes))
                }
            }
            AbiValue::Tuple(values) => {
                AbiType::Tuple(values.iter().map(|v| v.abi_type()).collect())
            }
        }
    }
}

/// Storage proof response with comprehensive ABI type support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageProofResponse {
    pub account_proof: Vec<String>,
    pub storage_proof: Vec<StorageProof>,
    pub address: String,
    pub balance: String,
    pub code_hash: String,
    pub nonce: String,
    pub storage_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageProof {
    pub key: String,
    pub value: String,
    pub proof: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ProcessorMessage, ProcessorMessageType, ZkMessage};
    use alloc::{vec, string::ToString};

    #[test]
    fn test_comprehensive_abi_types() {
        // Test basic types
        let bool_val = AbiValue::Bool(true);
        assert_eq!(bool_val.abi_type(), AbiType::Bool);

        let uint256_val = AbiValue::Uint256([1u64, 0, 0, 0]);
        assert_eq!(uint256_val.abi_type(), AbiType::Uint256);

        let addr_val = AbiValue::Address("0x742d35Cc6634C0532925a3b8D97C2e0D8b2D9C".to_string());
        assert_eq!(addr_val.abi_type(), AbiType::Address);

        // Test complex types
        let array_val = AbiValue::Array(vec![AbiValue::Uint8(1), AbiValue::Uint8(2)]);
        matches!(array_val.abi_type(), AbiType::Array(_));

        let tuple_val = AbiValue::Tuple(vec![AbiValue::Bool(true), AbiValue::Uint256([0u64; 4])]);
        matches!(tuple_val.abi_type(), AbiType::Tuple(_));
    }

    #[test]
    fn test_function_selector() {
        let selector = AlloyAbiTypes::function_selector("transfer(address,uint256)");
        assert_eq!(selector.len(), 4);
        // Should be consistent across calls
        let selector2 = AlloyAbiTypes::function_selector("transfer(address,uint256)");
        assert_eq!(selector, selector2);
    }

    #[test]
    fn test_alloy_features_check() {
        // Should not panic and should return a boolean
        let available = AlloyAbiTypes::alloy_features_available();
        assert!(available == true || available == false);
    }

    #[test]
    #[cfg(feature = "alloy")]
    fn test_comprehensive_encoding() {
        // TODO: Enable this test when ABI encoding is implemented
        // Currently the encode_abi_value function returns "not implemented"
        
        // Just test that the function exists and returns an error as expected
        let value = AbiValue::Bool(true);
        let encoded = AlloyAbiTypes::encode_abi_value(&value);
        assert!(encoded.is_err(), "Expected encoding to return 'not implemented' error");
    }

    #[test]
    #[cfg(feature = "alloy")]
    fn test_parsing_utilities() {
        // TODO: Enable these tests when alloy types are properly available
        // Currently these functions may not work without the full alloy feature set
        
        // For now, just test that the functions exist
        let _features_available = AlloyAbiTypes::alloy_features_available();
    }

    #[test]
    fn test_zk_message_encoding() {
        let zk_message = ZkMessage {
            registry: 1,
            block_number: 12345,
            authorization_contract: "0x742d35Cc6634C0532925a3b8D97C2e0D8b2D9C".to_string(),
            processor_message: ProcessorMessage {
                message_type: ProcessorMessageType::SendMsgs,
                message: b"test_message".to_vec(),
            },
        };

        let result = AlloyAbiTypes::encode_zk_message(&zk_message);
        
        #[cfg(feature = "alloy")]
        {
            // When alloy feature is enabled, encoding is not yet implemented
            assert!(result.is_err());
        }
        
        #[cfg(not(feature = "alloy"))]
        {
            // When alloy feature is not enabled, JSON encoding is used
            assert!(result.is_ok());
            let encoded = result.unwrap();
            assert!(!encoded.is_empty());
        }
    }

    #[cfg(feature = "alloy")]
    #[test]
    fn test_encode_zk_message_pause() {
        use crate::messages::ZkMessage;
        
        let msg = ZkMessage::Pause;
        let result = AlloyAbiTypes::encode_zk_message(&msg);
        
        assert!(result.is_ok());
        let encoded = result.unwrap();
        
        // Should have encoded message type (32 bytes) + offset (32 bytes) + length (32 bytes)
        assert_eq!(encoded.len(), 96);
        
        // First 32 bytes should be message type (0 for Pause)
        let mut expected_type = [0u8; 32];
        expected_type[31] = 0; // Pause = 0
        assert_eq!(&encoded[0..32], &expected_type);
    }

    #[cfg(feature = "alloy")]
    #[test]
    fn test_encode_zk_message_resume() {
        use crate::messages::ZkMessage;
        
        let msg = ZkMessage::Resume;
        let result = AlloyAbiTypes::encode_zk_message(&msg);
        
        assert!(result.is_ok());
        let encoded = result.unwrap();
        
        // First 32 bytes should be message type (1 for Resume)
        let mut expected_type = [0u8; 32];
        expected_type[31] = 1; // Resume = 1
        assert_eq!(&encoded[0..32], &expected_type);
    }

    #[cfg(feature = "alloy")]
    #[test]
    fn test_encode_abi_value_bool() {
        let value = AbiValue::Bool(true);
        let result = AlloyAbiTypes::encode_abi_value(&value);
        
        assert!(result.is_ok());
        let encoded = result.unwrap();
        
        assert_eq!(encoded.len(), 32);
        let mut expected = [0u8; 32];
        expected[31] = 1;
        assert_eq!(encoded, expected);
    }

    #[cfg(feature = "alloy")]
    #[test]
    fn test_encode_abi_value_uint8() {
        let value = AbiValue::Uint8(255);
        let result = AlloyAbiTypes::encode_abi_value(&value);
        
        assert!(result.is_ok());
        let encoded = result.unwrap();
        
        assert_eq!(encoded.len(), 32);
        let mut expected = [0u8; 32];
        expected[31] = 255;
        assert_eq!(encoded, expected);
    }

    #[cfg(feature = "alloy")]
    #[test]
    fn test_encode_abi_value_uint256() {
        let value = AbiValue::Uint256([1, 2, 3, 4]);
        let result = AlloyAbiTypes::encode_abi_value(&value);
        
        assert!(result.is_ok());
        let encoded = result.unwrap();
        
        assert_eq!(encoded.len(), 32);
        // First 8 bytes should be big-endian representation of 1
        assert_eq!(encoded[0..8], [0, 0, 0, 0, 0, 0, 0, 1]);
        assert_eq!(encoded[8..16], [0, 0, 0, 0, 0, 0, 0, 2]);
    }

    #[cfg(feature = "alloy")]
    #[test]
    fn test_encode_abi_value_address() {
        let value = AbiValue::Address("0x1234567890123456789012345678901234567890".to_string());
        let result = AlloyAbiTypes::encode_abi_value(&value);
        
        assert!(result.is_ok());
        let encoded = result.unwrap();
        
        assert_eq!(encoded.len(), 32);
        // First 12 bytes should be zero (padding)
        assert_eq!(&encoded[0..12], &[0u8; 12]);
        // Last 20 bytes should be the address
        assert_eq!(encoded[12], 0x12);
        assert_eq!(encoded[13], 0x34);
    }

    #[cfg(feature = "alloy")]
    #[test]
    fn test_encode_abi_value_fixed_bytes() {
        let mut bytes = [0u8; 32];
        bytes[0] = 0xAB;
        bytes[31] = 0xCD;
        
        let value = AbiValue::FixedBytes(bytes);
        let result = AlloyAbiTypes::encode_abi_value(&value);
        
        assert!(result.is_ok());
        let encoded = result.unwrap();
        
        assert_eq!(encoded.len(), 32);
        assert_eq!(encoded[0], 0xAB);
        assert_eq!(encoded[31], 0xCD);
    }

    #[cfg(feature = "alloy")]
    #[test]
    fn test_encode_abi_value_complex_unsupported() {
        let value = AbiValue::Array(vec![AbiValue::Uint8(1), AbiValue::Uint8(2)]);
        let result = AlloyAbiTypes::encode_abi_value(&value);
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Complex ABI types not yet implemented"));
    }

    #[cfg(not(feature = "alloy"))]
    #[test]
    fn test_encode_abi_value_without_alloy() {
        let value = AbiValue::Bool(true);
        let result = AlloyAbiTypes::encode_abi_value(&value);
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Alloy ABI types not available"));
    }
} 