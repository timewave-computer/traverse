//! Valence ecosystem message structures and ABI encoding helpers
//! 
//! This module provides message types and ABI encoding utilities for integrating
//! storage proof verification with the Valence Authorization contract ecosystem.
//! 
//! The structures here mirror the patterns used in valence-coprocessor-app for
//! generating ABI-encoded messages that can be processed by Valence contracts.

use alloc::{string::String, vec::Vec};
use serde::{Deserialize, Serialize};

// TODO: Enable when alloy sol_types feature is available
// #[cfg(feature = "alloy")]
// use alloy::primitives::{Address, Bytes, U256};

// #[cfg(feature = "alloy")]
// use alloy::sol_types::{sol, SolValue};

/// Duration type for Valence messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DurationType {
    Height,
    Time,
}

/// Duration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Duration {
    pub duration_type: DurationType,
    pub value: u64,
}

/// Retry times type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RetryTimesType {
    NoRetry,
    Indefinitely,
    Amount,
}

/// Retry times structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryTimes {
    pub retry_type: RetryTimesType,
    pub amount: u64,
}

/// Retry logic structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryLogic {
    pub times: RetryTimes,
    pub interval: Duration,
}

/// Atomic function structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtomicFunction {
    pub contract_address: String,
}

/// Atomic subroutine structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtomicSubroutine {
    pub functions: Vec<AtomicFunction>,
    pub retry_logic: RetryLogic,
}

/// Subroutine type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SubroutineType {
    Atomic,
    NonAtomic,
}

/// Subroutine structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subroutine {
    pub subroutine_type: SubroutineType,
    pub subroutine: Vec<u8>,
}

/// Priority enum
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Priority {
    Medium,
    High,
}

/// SendMsgs structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendMsgs {
    pub execution_id: u64,
    pub priority: Priority,
    pub subroutine: Subroutine,
    pub expiration_time: u64,
    pub messages: Vec<Vec<u8>>,
}

/// ProcessorMessage type enum
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProcessorMessageType {
    Pause,
    Resume,
    EvictMsgs,
    SendMsgs,
    InsertMsgs,
}

/// ProcessorMessage structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessorMessage {
    pub message_type: ProcessorMessageType,
    pub message: Vec<u8>,
}

/// ZkMessage structure for Valence Authorization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZkMessage {
    pub registry: u64,
    pub block_number: u64,
    pub authorization_contract: String,
    pub processor_message: ProcessorMessage,
}

/// Storage proof validation result that can be included in Valence messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageProofValidationResult {
    /// Whether the storage proof is valid
    pub is_valid: bool,
    /// Extracted value from storage (hex encoded)
    pub storage_value: String,
    /// Original storage key that was verified (hex encoded) 
    pub storage_key: String,
    /// Layout commitment that was verified (hex encoded)
    pub layout_commitment: String,
    /// Additional validation metadata
    pub metadata: Option<String>,
}

/// Create default retry logic for atomic execution (no retry)
pub fn create_no_retry_logic() -> RetryLogic {
    RetryLogic {
        times: RetryTimes {
            retry_type: RetryTimesType::NoRetry,
            amount: 0,
        },
        interval: Duration {
            duration_type: DurationType::Time,
            value: 0,
        },
    }
}

/// Create a simple atomic subroutine with a single contract call
pub fn create_atomic_subroutine(contract_address: String) -> AtomicSubroutine {
    AtomicSubroutine {
        functions: alloc::vec![AtomicFunction { contract_address }],
        retry_logic: create_no_retry_logic(),
    }
}

/// Helper to create a basic ZkMessage for storage proof validation
/// 
/// This can be extended by applications to include their specific contract calls
/// that depend on verified storage values.
pub fn create_storage_validation_message(
    validation_result: StorageProofValidationResult,
    execution_id: u64,
) -> ZkMessage {
    // Encode the validation result as the message payload
    let validation_bytes = serde_json::to_vec(&validation_result)
        .unwrap_or_else(|_| b"validation_failed".to_vec());

    // Create a basic SendMsgs with the validation result
    let send_msgs = SendMsgs {
        execution_id,
        priority: Priority::Medium,
        subroutine: Subroutine {
            subroutine_type: SubroutineType::Atomic,
            subroutine: validation_bytes.clone(),
        },
        expiration_time: 0,
        messages: alloc::vec![validation_bytes],
    };

    let send_msgs_bytes = serde_json::to_vec(&send_msgs)
        .unwrap_or_else(|_| b"encoding_failed".to_vec());

    let processor_message = ProcessorMessage {
        message_type: ProcessorMessageType::SendMsgs,
        message: send_msgs_bytes,
    };

    ZkMessage {
        registry: 0, // Permissionless execution
        block_number: 0, // Not validated
        authorization_contract: "0x0000000000000000000000000000000000000000".into(), // Any contract
        processor_message,
    }
}

#[cfg(feature = "alloy")]
pub mod abi_encoding {
    //! ABI encoding utilities using alloy-sol-types
    //! 
    //! This module provides ABI encoding for Valence message structures
    //! using alloy-sol-types for type safety and compatibility.
    
    use super::*;
    use crate::TraverseValenceError;
    
    // Define Valence contract types using alloy-sol-types
    // sol! {
    //     /// Duration type for Valence messages
    //     enum DurationType {
    //         Height,
    //         Time
    //     }

    //     /// Duration structure
    //     struct Duration {
    //         DurationType durationType;
    //         uint64 value;
    //     }

    //     /// Retry times type
    //     enum RetryTimesType {
    //         NoRetry,
    //         Indefinitely,
    //         Amount
    //     }

    //     /// Retry times structure
    //     struct RetryTimes {
    //         RetryTimesType retryType;
    //         uint64 amount;
    //     }

    //     /// Retry logic structure
    //     struct RetryLogic {
    //         RetryTimes times;
    //         Duration interval;
    //     }

    //     /// Atomic function structure
    //     struct AtomicFunction {
    //         address contractAddress;
    //     }

    //     /// Atomic subroutine structure
    //     struct AtomicSubroutine {
    //         AtomicFunction[] functions;
    //         RetryLogic retryLogic;
    //     }

    //     /// Subroutine type
    //     enum SubroutineType {
    //         Atomic,
    //         NonAtomic
    //     }

    //     /// Subroutine structure
    //     struct Subroutine {
    //         SubroutineType subroutineType;
    //         bytes subroutine;
    //     }

    //     /// Priority enum
    //     enum Priority {
    //         Medium,
    //         High
    //     }

    //     /// SendMsgs structure
    //     struct SendMsgs {
    //         uint64 executionId;
    //         Priority priority;
    //         Subroutine subroutine;
    //         uint64 expirationTime;
    //         bytes[] messages;
    //     }

    //     /// ProcessorMessage type enum
    //     enum ProcessorMessageType {
    //         Pause,
    //         Resume,
    //         EvictMsgs,
    //         SendMsgs,
    //         InsertMsgs
    //     }

    //     /// ProcessorMessage structure
    //     struct ProcessorMessage {
    //         ProcessorMessageType messageType;
    //         bytes message;
    //     }

    //     /// ZkMessage structure for Valence Authorization
    //     struct ZkMessage {
    //         uint64 registry;
    //         uint64 blockNumber;
    //         address authorizationContract;
    //         ProcessorMessage processorMessage;
    //     }
    // }
    
    /// Encode a ZkMessage to ABI bytes
    pub fn encode_zk_message(msg: &ZkMessage) -> Result<Vec<u8>, TraverseValenceError> {
        // For now, return JSON encoding until alloy sol_types is available
        // This maintains compatibility while avoiding compilation issues
        let json_bytes = serde_json::to_vec(msg)
            .map_err(|e| TraverseValenceError::AbiError(alloc::format!("JSON encoding failed: {:?}", e)))?;
        
        Ok(json_bytes)
        
        // TODO: Enable when alloy sol_types is properly configured
        // Convert our message types to alloy types for ABI encoding
        // let contract_address: Address = msg.authorization_contract.parse()
        //     .map_err(|e| TraverseValenceError::AbiError(alloc::format!("Invalid address: {:?}", e)))?;
        
        // let processor_message = ProcessorMessage {
        //     messageType: match msg.processor_message.message_type {
        //         crate::messages::ProcessorMessageType::Pause => ProcessorMessageType::Pause,
        //         crate::messages::ProcessorMessageType::Resume => ProcessorMessageType::Resume,
        //         crate::messages::ProcessorMessageType::EvictMsgs => ProcessorMessageType::EvictMsgs,
        //         crate::messages::ProcessorMessageType::SendMsgs => ProcessorMessageType::SendMsgs,
        //         crate::messages::ProcessorMessageType::InsertMsgs => ProcessorMessageType::InsertMsgs,
        //     },
        //     message: Bytes::from(msg.processor_message.message.clone()),
        // };
        
        // let zk_message = ZkMessage {
        //     registry: msg.registry,
        //     blockNumber: msg.block_number,
        //     authorizationContract: contract_address,
        //     processorMessage: processor_message,
        // };
        
        // // Encode to ABI bytes
        // Ok(zk_message.abi_encode())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_storage_validation_message() {
        let validation_result = StorageProofValidationResult {
            is_valid: true,
            storage_value: "0x0000000000000000000000000000000000000000000000000000000000000064".into(),
            storage_key: "0xc1f51986c7e9d391993039c3c40e41ad9f26e1db9b80f8535a639eadeb1d1bd9".into(),
            layout_commitment: "0xf6dc3c4a79e95565b3cf38993f1a120c6a6b467796264e7fd9a9c8675616dd7a".into(),
            metadata: Some("balance_verification".into()),
        };

        let message = create_storage_validation_message(validation_result, 1);
        
        assert_eq!(message.registry, 0);
        assert_eq!(message.block_number, 0);
        assert_eq!(message.authorization_contract, "0x0000000000000000000000000000000000000000");
    }
    
    #[test]
    fn test_create_no_retry_logic() {
        let retry_logic = create_no_retry_logic();
        
        assert!(matches!(retry_logic.times.retry_type, RetryTimesType::NoRetry));
        assert_eq!(retry_logic.times.amount, 0);
        assert_eq!(retry_logic.interval.value, 0);
    }
} 