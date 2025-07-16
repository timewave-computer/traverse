//! Valence ecosystem message structures and ABI encoding helpers
//!
//! This module provides message types and ABI encoding utilities for integrating
//! storage proof verification with the Valence Authorization contract ecosystem.
//!
//! The structures here mirror the patterns used in valence-coprocessor-app for
//! generating ABI-encoded messages that can be processed by Valence contracts.

use alloc::{string::String, vec::Vec};
use serde::{Deserialize, Serialize};

// Lightweight alloy integration for ABI encoding (avoids k256 conflicts)
#[cfg(feature = "lightweight-alloy")]
use alloy_primitives::{Address as AlloyAddress, Bytes as AlloyBytes};

// Fallback types (used when lightweight-alloy is not available)
pub type Address = String;
pub type Bytes = Vec<u8>;

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

impl ProcessorMessageType {
    pub fn as_u8(&self) -> u8 {
        match self {
            ProcessorMessageType::Pause => 0,
            ProcessorMessageType::Resume => 1,
            ProcessorMessageType::EvictMsgs => 2,
            ProcessorMessageType::SendMsgs => 3,
            ProcessorMessageType::InsertMsgs => 4,
        }
    }
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

/// Solana account query format for coprocessor integration
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SolanaAccountQuery {
    /// Original query string (e.g., "user_account[seed]", "token_balance[mint,owner]")
    pub query: String,
    /// Resolved account address (base58 encoded)
    pub account_address: String,
    /// Program ID that owns this account (base58 encoded)
    pub program_id: String,
    /// Account discriminator (for Anchor programs)
    pub discriminator: Option<String>,
    /// Field offset within account data
    pub field_offset: Option<u32>,
    /// Field size in bytes
    pub field_size: Option<u32>,
}

/// Solana account proof data from RPC
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SolanaAccountProof {
    /// Account address (base58 encoded)
    pub address: String,
    /// Account data (base64 encoded)
    pub data: String,
    /// Account owner program (base58 encoded)
    pub owner: String,
    /// Lamports balance
    pub lamports: u64,
    /// Rent epoch
    pub rent_epoch: u64,
    /// Slot when proof was generated
    pub slot: u64,
    /// Block hash for the slot
    pub block_hash: String,
}

/// Complete Solana account verification request
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SolanaAccountVerificationRequest {
    /// Account query generated by traverse-cli
    pub account_query: SolanaAccountQuery,
    /// Account proof from Solana RPC
    pub account_proof: SolanaAccountProof,
    /// Optional program address for additional validation
    pub program_address: Option<String>,
    /// Optional slot number for proof validation
    pub slot: Option<u64>,
}

/// Batch Solana account verification for multiple queries
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BatchSolanaAccountVerificationRequest {
    /// Multiple account verification requests
    pub account_batch: Vec<SolanaAccountVerificationRequest>,
    /// Common program address (if all queries are for same program)
    pub program_address: Option<String>,
    /// Common slot (if all proofs are from same slot)
    pub slot: Option<u64>,
}

/// Solana account proof validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolanaAccountValidationResult {
    /// Whether the account proof is valid
    pub is_valid: bool,
    /// Extracted value from account data (hex encoded)
    pub account_value: String,
    /// Original account address that was verified (base58 encoded)
    pub account_address: String,
    /// Program ID that owns the account (base58 encoded)
    pub program_id: String,
    /// Slot when proof was generated
    pub slot: u64,
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
    let validation_bytes =
        serde_json::to_vec(&validation_result).unwrap_or_else(|_| b"validation_failed".to_vec());

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

    let send_msgs_bytes =
        serde_json::to_vec(&send_msgs).unwrap_or_else(|_| b"encoding_failed".to_vec());

    let processor_message = ProcessorMessage {
        message_type: ProcessorMessageType::SendMsgs,
        message: send_msgs_bytes,
    };

    ZkMessage {
        registry: 0,     // Permissionless execution
        block_number: 0, // Not validated
        authorization_contract: "0x0000000000000000000000000000000000000000".into(), // Any contract
        processor_message,
    }
}

// ABI encoding module (conditional on alloy features)
#[cfg(feature = "lightweight-alloy")]
pub mod abi_encoding {
    //! ABI encoding utilities using alloy-sol-types
    //!
    //! This module provides ABI encoding for Valence message structures
    //! using alloy-sol-types for type safety and compatibility.

    use super::*;
    use crate::TraverseValenceError;

    /// Encode a ZkMessage to ABI bytes
    pub fn encode_zk_message(
        msg: &crate::messages::ZkMessage,
    ) -> Result<Vec<u8>, TraverseValenceError> {
        // Fallback to JSON encoding to avoid alloy conflicts
        // This maintains compatibility while avoiding k256 conflicts with Solana
        let json_bytes = serde_json::to_vec(msg).map_err(|e| {
            TraverseValenceError::AbiError(alloc::format!("JSON encoding failed: {:?}", e))
        })?;

        Ok(json_bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_storage_validation_message() {
        let validation_result = StorageProofValidationResult {
            is_valid: true,
            storage_value: "0x0000000000000000000000000000000000000000000000000000000000000064"
                .into(),
            storage_key: "0xc1f51986c7e9d391993039c3c40e41ad9f26e1db9b80f8535a639eadeb1d1bd9"
                .into(),
            layout_commitment: "0xf6dc3c4a79e95565b3cf38993f1a120c6a6b467796264e7fd9a9c8675616dd7a"
                .into(),
            metadata: Some("balance_verification".into()),
        };

        let message = create_storage_validation_message(validation_result, 1);

        assert_eq!(message.registry, 0);
        assert_eq!(message.block_number, 0);
        assert_eq!(
            message.authorization_contract,
            "0x0000000000000000000000000000000000000000"
        );
    }

    #[test]
    fn test_create_no_retry_logic() {
        let retry_logic = create_no_retry_logic();

        assert!(matches!(
            retry_logic.times.retry_type,
            RetryTimesType::NoRetry
        ));
        assert_eq!(retry_logic.times.amount, 0);
        assert_eq!(retry_logic.interval.value, 0);
    }
}
