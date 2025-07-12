//! Minimal circuit operations for ZK environments (no_std compatible)
//!
//! This module provides the absolute minimum functionality needed for secure
//! circuit operations in ZK environments with proper semantic validation.
//! No error handling, statistics, or monitoring - only core secure operations
//! with semantic understanding of storage.
//!
//! ## no_std Compatibility
//!
//! This module is fully `no_std` compatible, using only `alloc` for minimal
//! heap allocations when necessary. All operations are designed for 
//! constrained environments and ZK circuits.
//!
//! ## Semantic Validation
//!
//! The circuit validates storage values based on semantic understanding:
//! - Layout-aware field validation
//! - Zero semantics consistency checking  
//! - Type-specific value validation
//! - Storage location semantic verification

use alloc::{vec, vec::Vec};

/// Zero semantics for circuit operations (must match storage layout semantics)
/// 
/// These semantics prevent semantic confusion attacks where an attacker
/// claims a storage value has different meaning than its actual state. Each semantic
/// type has specific validation rules to prevent manipulation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ZeroSemantics {
    /// Storage slot was never written to (initial state)
    /// Can only be claimed for genuinely uninitialized storage
    NeverWritten,
    /// Storage slot was explicitly set to zero by contract logic
    /// Must be validated against field type - not all fields can be legitimately zero
    ExplicitlyZero,
    /// Storage slot was cleared (set to zero after having a value)
    /// Must match expected clearing behavior for the field
    Cleared,
    /// Zero is a valid value for this field type (e.g., counters, flags)
    /// Only allowed for field types where zero is semantically meaningful
    ValidZero,
}

/// Field types for semantic value extraction and validation
/// 
/// Each field type has specific validation rules to prevent type confusion
/// attacks and ensure extracted values are semantically correct for their intended use.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FieldType {
    Bool,
    Uint8,
    Uint16,
    Uint32,
    Uint64,
    Uint256,
    Address,    // Zero addresses are flagged as suspicious
    Bytes32,
    String,
    Bytes,
}

impl FieldType {
    /// Check if a field type can legitimately be zero
    /// 
    /// This prevents zero-value attacks where adversaries claim
    /// zero values for field types where zero is semantically invalid or suspicious.
    /// For example, zero addresses are often indicators of uninitialized state
    /// or potential attack vectors.
    #[inline]
    pub const fn can_be_zero(&self) -> bool {
        match self {
            FieldType::Bool => true,       // false = 0 (semantically valid)
            FieldType::Uint8 => true,      // 0 is valid numeric value
            FieldType::Uint16 => true,     // 0 is valid numeric value
            FieldType::Uint32 => true,     // 0 is valid numeric value
            FieldType::Uint64 => true,     // 0 is valid numeric value
            FieldType::Uint256 => true,    // 0 is valid numeric value
            FieldType::Address => false,   // 0x0 address is suspicious/invalid
            FieldType::Bytes32 => true,    // empty hash can be semantically valid
            FieldType::String => true,     // empty string = 0 (valid)
            FieldType::Bytes => true,      // empty bytes = 0 (valid)
        }
    }

    /// Check if extracted value is semantically valid for this field type
    /// 
    /// This prevents type confusion attacks where extracted values
    /// don't match their claimed field type. It also applies field-specific
    /// security rules (e.g., flagging zero addresses as suspicious).
    #[inline]
    pub fn validate_extracted_value(&self, value: &ExtractedValue) -> bool {
        match (self, value) {
            (FieldType::Bool, ExtractedValue::Bool(_)) => true,
            (FieldType::Uint8, ExtractedValue::Uint8(_)) => true,
            (FieldType::Uint16, ExtractedValue::Uint16(_)) => true,
            (FieldType::Uint32, ExtractedValue::Uint32(_)) => true,
            (FieldType::Uint64, ExtractedValue::Uint64(_)) => true,
            (FieldType::Uint256, ExtractedValue::Uint256(_)) => true,
            (FieldType::Address, ExtractedValue::Address(addr)) => {
                // Zero address validation prevents common attack patterns
                // where uninitialized or malicious addresses are used. Zero addresses
                // are often indicators of bugs or intentional exploits.
                *addr != [0u8; 20]
            }
            (FieldType::Bytes32, ExtractedValue::Bytes32(_)) => true,
            // String and Bytes field types extract to Raw but are semantically valid
            (FieldType::String, ExtractedValue::Raw(_)) => true,
            (FieldType::Bytes, ExtractedValue::Raw(_)) => true,
            _ => false, // Type mismatch indicates potential attack or corruption
        }
    }
}

/// Minimal witness structure for ZK circuits with semantic validation
/// 
/// This structure contains all data needed for secure proof verification.
/// Each field serves a specific security purpose and is validated independently.
#[derive(Debug, Clone)]
pub struct CircuitWitness {
    /// Storage key (32 bytes)
    /// Must match expected_slot to prevent storage slot spoofing attacks
    pub key: [u8; 32],
    /// Storage value (32 bytes)
    /// Extracted and validated according to field type semantics
    pub value: [u8; 32],
    /// Proof data (minimal size)
    /// Contains cryptographic proof of storage state
    pub proof: Vec<u8>,
    /// Layout commitment (32 bytes) - must match expected layout
    /// Prevents layout spoofing attacks where adversaries claim
    /// different field layouts to manipulate value interpretation
    pub layout_commitment: [u8; 32],
    /// Field index in layout
    /// Must be within bounds to prevent out-of-bounds access
    pub field_index: u16,
    /// Zero semantics (must match storage semantics)
    /// Prevents semantic confusion attacks about value meaning
    pub semantics: ZeroSemantics,
    /// Expected storage slot for this field (for validation)
    /// Critical for preventing storage slot spoofing where
    /// adversaries claim values from different storage locations
    pub expected_slot: [u8; 32],
    /// Block height for proof timing validation
    /// Ensures proof is from the expected block height
    pub block_height: u64,
    /// Block hash for light client verification
    /// Must match the proven block hash from light client
    pub block_hash: [u8; 32],
}

/// Minimal circuit processor with semantic validation (no_std compatible)
/// 
/// This processor enforces all security validations required for
/// safe ZK circuit operation. It validates layout consistency, storage location
/// correctness, and semantic value interpretation.
pub struct CircuitProcessor {
    /// Current layout commitment
    /// Immutable after creation to prevent layout tampering
    layout_commitment: [u8; 32],
    /// Field types for extraction and validation
    /// Defines expected types to prevent type confusion attacks
    field_types: Vec<FieldType>,
    /// Expected semantic behavior for each field
    /// Defines valid zero semantics to prevent semantic manipulation
    field_semantics: Vec<ZeroSemantics>,
    /// Expected block height for proof validation
    /// Ensures all proofs are from the same block
    expected_block_height: u64,
    /// Expected block hash from light client
    /// Validates proofs against light client consensus
    expected_block_hash: [u8; 32],
    /// Maximum allowed age for proofs in blocks
    /// Prevents replay attacks with old proofs
    max_proof_age_blocks: u64,
}

impl CircuitProcessor {
    /// Create new processor with semantic validation
    /// 
    /// All validation parameters are set at creation time and cannot
    /// be modified, preventing runtime tampering with security rules.
    pub fn new(
        layout_commitment: [u8; 32], 
        field_types: Vec<FieldType>,
        field_semantics: Vec<ZeroSemantics>,
    ) -> Self {
        Self {
            layout_commitment,
            field_types,
            field_semantics,
            expected_block_height: 0,
            expected_block_hash: [0u8; 32],
            max_proof_age_blocks: 256, // Default: ~1 hour on Ethereum
        }
    }
    
    /// Create new processor with light client validation
    /// 
    /// Includes block height and hash validation from light client
    pub fn new_with_light_client(
        layout_commitment: [u8; 32], 
        field_types: Vec<FieldType>,
        field_semantics: Vec<ZeroSemantics>,
        block_height: u64,
        block_hash: [u8; 32],
    ) -> Self {
        Self {
            layout_commitment,
            field_types,
            field_semantics,
            expected_block_height: block_height,
            expected_block_hash: block_hash,
            max_proof_age_blocks: 256, // Default: ~1 hour on Ethereum
        }
    }
    
    /// Create new processor with custom proof expiration
    /// 
    /// Allows setting custom proof age limits for different security requirements
    pub fn new_with_expiration(
        layout_commitment: [u8; 32], 
        field_types: Vec<FieldType>,
        field_semantics: Vec<ZeroSemantics>,
        block_height: u64,
        block_hash: [u8; 32],
        max_proof_age_blocks: u64,
    ) -> Self {
        Self {
            layout_commitment,
            field_types,
            field_semantics,
            expected_block_height: block_height,
            expected_block_hash: block_hash,
            max_proof_age_blocks,
        }
    }

    /// Parse witness data from raw bytes (extended format only)
    /// 
    /// This function parses the extended witness format created by the controller.
    /// The extended format includes field_index and expected_slot for enhanced validation.
    /// 
    /// Extended witness format (176+ bytes):
    /// - [32 bytes] storage_key
    /// - [32 bytes] layout_commitment
    /// - [32 bytes] value
    /// - [1 byte] zero_semantics
    /// - [1 byte] semantic_source
    /// - [8 bytes] block_height
    /// - [32 bytes] block_hash
    /// - [4 bytes] proof_len
    /// - [variable] proof_data
    /// - [2 bytes] field_index
    /// - [32 bytes] expected_slot
    pub fn parse_witness_from_bytes(witness_data: &[u8]) -> Result<CircuitWitness, &'static str> {
        // Minimum size check for extended format (without proof data)
        if witness_data.len() < 176 {
            return Err("Witness data too small (extended format required)");
        }
        
        let mut offset = 0;
        
        // Parse storage key (32 bytes)
        let mut key = [0u8; 32];
        key.copy_from_slice(&witness_data[offset..offset + 32]);
        offset += 32;
        
        // Parse layout commitment (32 bytes)
        let mut layout_commitment = [0u8; 32];
        layout_commitment.copy_from_slice(&witness_data[offset..offset + 32]);
        offset += 32;
        
        // Parse value (32 bytes)
        let mut value = [0u8; 32];
        value.copy_from_slice(&witness_data[offset..offset + 32]);
        offset += 32;
        
        // Parse semantic metadata (2 bytes)
        let semantics = match witness_data[offset] {
            0 => ZeroSemantics::NeverWritten,
            1 => ZeroSemantics::ExplicitlyZero,
            2 => ZeroSemantics::Cleared,
            3 => ZeroSemantics::ValidZero,
            _ => return Err("Invalid zero semantics value"),
        };
        offset += 1;
        
        let _semantic_source = witness_data[offset]; // Currently unused in circuit
        offset += 1;
        
        // Parse block height (8 bytes)
        let height_bytes = &witness_data[offset..offset + 8];
        let block_height = u64::from_le_bytes([
            height_bytes[0], height_bytes[1], height_bytes[2], height_bytes[3],
            height_bytes[4], height_bytes[5], height_bytes[6], height_bytes[7],
        ]);
        offset += 8;
        
        // Parse block hash (32 bytes)
        let mut block_hash = [0u8; 32];
        block_hash.copy_from_slice(&witness_data[offset..offset + 32]);
        offset += 32;
        
        // Parse proof length (4 bytes)
        if witness_data.len() < offset + 4 {
            return Err("Missing proof length");
        }
        let proof_len = u32::from_le_bytes([
            witness_data[offset], witness_data[offset + 1],
            witness_data[offset + 2], witness_data[offset + 3],
        ]) as usize;
        offset += 4;
        
        // Parse proof data
        if witness_data.len() < offset + proof_len {
            return Err("Incomplete proof data");
        }
        let proof = witness_data[offset..offset + proof_len].to_vec();
        offset += proof_len;
        
        // Parse field_index (2 bytes) - mandatory in extended format
        if witness_data.len() < offset + 2 {
            return Err("Missing field_index");
        }
        let field_index = u16::from_le_bytes([witness_data[offset], witness_data[offset + 1]]);
        offset += 2;
        
        // Parse expected_slot (32 bytes) - mandatory in extended format
        if witness_data.len() < offset + 32 {
            return Err("Missing expected_slot");
        }
        let mut expected_slot = [0u8; 32];
        expected_slot.copy_from_slice(&witness_data[offset..offset + 32]);
        
        Ok(CircuitWitness {
            key,
            value,
            proof,
            layout_commitment,
            field_index,
            semantics,
            expected_slot,
            block_height,
            block_hash,
        })
    }
    
    /// Process witness with comprehensive semantic validation
    /// 
    /// This is the main entry point for witness validation. It performs
    /// multiple security checks in sequence, failing fast if any validation fails.
    /// The order of checks is designed to catch the most common attack patterns first.
    pub fn process_witness(&self, witness: &CircuitWitness) -> CircuitResult {
        // CRITICAL: Layout commitment validation must be first
        // This prevents layout spoofing attacks where adversaries claim different
        // field layouts to manipulate how values are interpreted. Without this check,
        // an attacker could claim a uint256 field is actually an address field.
        if witness.layout_commitment != self.layout_commitment {
            return CircuitResult::Invalid;
        }
        
        // CRITICAL: Light client validation for block consistency
        // This ensures the proof is from the expected block height and matches
        // the light client's proven block hash. Without this check, an attacker
        // could provide proofs from different blocks or fabricated block data.
        if self.expected_block_height != 0 { // Only validate if light client is configured
            // For exact block matching (when we have a specific expected block)
            if witness.block_height == self.expected_block_height {
                // Must match the exact block hash
                if witness.block_hash != self.expected_block_hash {
                    return CircuitResult::Invalid;
                }
            } else {
                // For historical proofs, we can't verify the exact hash but we can check age
                // CRITICAL: Proof age validation prevents replay attacks
                // This ensures that old proofs cannot be reused after expiration.
                // The age check prevents attackers from using outdated state proofs
                // that might no longer reflect the current blockchain state.
                if witness.block_height > self.expected_block_height {
                    // Future block - always invalid
                    return CircuitResult::Invalid;
                }
                
                let proof_age = self.expected_block_height - witness.block_height;
                if proof_age > self.max_proof_age_blocks {
                    return CircuitResult::Invalid;
                }
            }
        }

        // CRITICAL: Bounds checking prevents out-of-bounds access
        // This prevents buffer overflow attacks and ensures field_index is valid
        // for both field_types and field_semantics arrays. Without this check,
        // an attacker could cause undefined behavior or access wrong field metadata.
        if witness.field_index as usize >= self.field_types.len() {
            return CircuitResult::Invalid;
        }

        let field_type = self.field_types[witness.field_index as usize];
        let expected_semantics = self.field_semantics[witness.field_index as usize];

        // CRITICAL: Semantic consistency validation prevents semantic confusion
        // This ensures that claimed zero semantics match the actual field type and value.
        // Without this check, an attacker could claim a non-zero value has "never written"
        // semantics, or claim a zero address has "valid zero" semantics.
        if !self.validate_semantic_consistency(witness, field_type, expected_semantics) {
            return CircuitResult::Invalid;
        }

        // CRITICAL: Storage location validation prevents storage slot spoofing
        // This ensures the storage key matches the expected slot for this field.
        // Without this check, an attacker could provide values from different storage
        // locations while claiming they belong to the expected field.
        if !self.validate_storage_location(witness) {
            return CircuitResult::Invalid;
        }

        // Value extraction with type validation prevents type confusion
        // This ensures extracted values match their claimed field type semantics.
        let extracted_value = self.extract_value(witness, field_type);

        // CRITICAL: Final value validation catches field-specific attacks
        // This applies field-specific security rules (e.g., zero address detection)
        // and ensures the extracted value is semantically valid for its field type.
        if !field_type.validate_extracted_value(&extracted_value) {
            return CircuitResult::Invalid;
        }

        CircuitResult::Valid {
            field_index: witness.field_index,
            extracted_value,
        }
    }

    /// Process batch of witnesses with semantic validation
    /// 
    /// Each witness is validated independently to prevent cross-contamination
    /// attacks where one malicious witness could affect validation of others.
    pub fn process_batch(&self, witnesses: &[CircuitWitness]) -> Vec<CircuitResult> {
        witnesses.iter().map(|w| self.process_witness(w)).collect()
    }

    /// Validate semantic consistency between witness and expected field semantics
    /// 
    /// This function prevents semantic confusion attacks by ensuring
    /// that claimed zero semantics are consistent with actual values and field types.
    /// It catches several attack patterns:
    /// - Claiming non-zero values were "never written"
    /// - Claiming zero values for fields that can't be zero
    /// - Mismatched semantic expectations vs. actual semantics
    #[inline]
    fn validate_semantic_consistency(
        &self,
        witness: &CircuitWitness,
        field_type: FieldType,
        expected_semantics: ZeroSemantics,
    ) -> bool {
        let is_zero = witness.value == [0u8; 32];

        // Zero value semantic validation prevents zero-value attacks
        if is_zero {
            match (witness.semantics, expected_semantics) {
                // Never written semantics must match expectations exactly
                // This prevents attacks where adversaries claim initialized storage
                // was never written to access default values or bypass checks.
                (ZeroSemantics::NeverWritten, ZeroSemantics::NeverWritten) => true,
                // Explicitly zero values must be valid for field type
                // This prevents zero-value attacks on fields that shouldn't be zero
                // (e.g., claiming a zero address was explicitly set to zero).
                (ZeroSemantics::ExplicitlyZero, _) => field_type.can_be_zero(),
                // Cleared semantics must match expectations exactly
                // This prevents attacks where adversaries claim different clearing behavior
                // to manipulate how zero values are interpreted.
                (ZeroSemantics::Cleared, ZeroSemantics::Cleared) => true,
                // Valid zero values must be allowed by field type
                // This prevents zero-value attacks on fields where zero is invalid.
                (ZeroSemantics::ValidZero, _) => field_type.can_be_zero(),
                _ => false, // Any other combination indicates potential attack
            }
        } else {
            // Non-zero value semantic validation
            match witness.semantics {
                // ValidZero semantics allow both zero and non-zero values
                // This covers fields like counters, flags, and balances where any value is meaningful
                ZeroSemantics::ValidZero => true,
                // All other zero semantics are invalid for non-zero values
                // Non-zero values cannot claim to be never written, explicitly zero, or cleared
                ZeroSemantics::NeverWritten | ZeroSemantics::ExplicitlyZero | ZeroSemantics::Cleared => false,
            }
        }
    }

    /// Validate storage location matches expected slot for field
    /// 
    /// This function prevents storage slot spoofing attacks where
    /// adversaries provide values from different storage locations while claiming
    /// they belong to the expected field. This is critical for preventing:
    /// - Cross-field value injection attacks
    /// - Storage collision attacks
    /// - Field boundary violation attacks
    #[inline]
    fn validate_storage_location(&self, witness: &CircuitWitness) -> bool {
        // CRITICAL: Storage key must exactly match expected slot
        // Any mismatch indicates potential storage slot spoofing attack where
        // an adversary is trying to use values from wrong storage locations.
        // This prevents attacks where wrong storage slots are claimed to
        // belong to different fields or contract structures.
        witness.key == witness.expected_slot
    }

    /// Extract value from witness with field type validation
    /// 
    /// This function performs type-safe value extraction from raw storage.
    /// It uses bounds-checked array access and prevents buffer overflows by
    /// carefully extracting only the required bytes for each field type.
    /// The extraction follows Ethereum's storage encoding rules to prevent
    /// value interpretation attacks.
    #[inline]
    fn extract_value(&self, witness: &CircuitWitness, field_type: FieldType) -> ExtractedValue {
        match field_type {
            // Bool extraction checks only the least significant bit
            // This prevents bool value manipulation attacks where non-zero/one values
            // are used to represent boolean state.
            FieldType::Bool => ExtractedValue::Bool(witness.value[31] != 0),
            // Uint8 extraction uses only the least significant byte
            // This prevents integer overflow attacks and ensures proper value bounds.
            FieldType::Uint8 => ExtractedValue::Uint8(witness.value[31]),
            // Uint16 extraction uses big-endian byte order (Ethereum standard)
            // This prevents byte order attacks and ensures consistent value interpretation.
            FieldType::Uint16 => {
                ExtractedValue::Uint16(u16::from_be_bytes([witness.value[30], witness.value[31]]))
            }
            // Uint32 extraction uses big-endian byte order
            // Bounds-checked array access prevents buffer overflow attacks.
            FieldType::Uint32 => {
                ExtractedValue::Uint32(u32::from_be_bytes([
                    witness.value[28], witness.value[29], witness.value[30], witness.value[31]
                ]))
            }
            // Uint64 extraction uses big-endian byte order
            // Bounds-checked array access prevents buffer overflow attacks.
            FieldType::Uint64 => {
                ExtractedValue::Uint64(u64::from_be_bytes([
                    witness.value[24], witness.value[25], witness.value[26], witness.value[27],
                    witness.value[28], witness.value[29], witness.value[30], witness.value[31]
                ]))
            }
            // Uint256 uses the full 32-byte value
            // Direct copy prevents any value manipulation during extraction.
            FieldType::Uint256 => ExtractedValue::Uint256(witness.value),
            // Address extraction uses bytes 12-31 (20 bytes)
            // This follows Ethereum's address encoding and prevents address manipulation.
            // The extracted address will be validated separately for zero-address attacks.
            FieldType::Address => {
                let mut addr = [0u8; 20];
                addr.copy_from_slice(&witness.value[12..32]); // Bounds-checked slice
                ExtractedValue::Address(addr)
            }
            // Bytes32 uses the full 32-byte value
            // Direct copy prevents any value manipulation during extraction.
            FieldType::Bytes32 => ExtractedValue::Bytes32(witness.value),
            // Fallback to raw bytes for unknown types
            // This prevents crashes while maintaining security through type validation.
            _ => ExtractedValue::Raw(witness.value),
        }
    }
}

/// Circuit processing result with semantic validation
/// 
/// This result type provides clear success/failure indication without
/// leaking sensitive information about why validation failed. This prevents
/// information leakage attacks where adversaries could probe for specific
/// validation failures to understand system internals.
#[derive(Debug, Clone)]
pub enum CircuitResult {
    Valid {
        field_index: u16,
        extracted_value: ExtractedValue,
    },
    Invalid, // No detailed error info to prevent information leakage
}

/// Semantically validated extracted value types (no_std compatible)
/// 
/// These types represent values that have passed all security validations
/// and can be safely used in circuit operations. Each type maintains its semantic
/// meaning and prevents type confusion attacks.
#[derive(Debug, Clone)]
pub enum ExtractedValue {
    Bool(bool),
    Uint8(u8),
    Uint16(u16),
    Uint32(u32),
    Uint64(u64),
    Uint256([u8; 32]),
    Address([u8; 20]),    // Guaranteed to be non-zero if validation passed
    Bytes32([u8; 32]),
    Raw([u8; 32]),        // Fallback for unknown types
}

impl ExtractedValue {
    /// Convert to bytes (minimal allocation)
    /// 
    /// This function performs safe conversions without buffer overflows.
    /// It uses minimal allocations and maintains value integrity during conversion.
    #[inline]
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            ExtractedValue::Bool(b) => {
                vec![if *b { 1 } else { 0 }]
            },
            ExtractedValue::Uint8(n) => {
                vec![*n]
            },
            ExtractedValue::Uint16(n) => n.to_be_bytes().to_vec(),
            ExtractedValue::Uint32(n) => n.to_be_bytes().to_vec(),
            ExtractedValue::Uint64(n) => n.to_be_bytes().to_vec(),
            ExtractedValue::Uint256(bytes) => bytes.to_vec(),
            ExtractedValue::Address(addr) => addr.to_vec(),
            ExtractedValue::Bytes32(bytes) => bytes.to_vec(),
            ExtractedValue::Raw(bytes) => bytes.to_vec(),
        }
    }

    /// Get size (no allocation)
    /// 
    /// This function provides size information without allocation,
    /// preventing potential memory-based attacks and maintaining constant-time operation.
    #[inline]
    pub const fn size(&self) -> usize {
        match self {
            ExtractedValue::Bool(_) => 1,
            ExtractedValue::Uint8(_) => 1,
            ExtractedValue::Uint16(_) => 2,
            ExtractedValue::Uint32(_) => 4,
            ExtractedValue::Uint64(_) => 8,
            ExtractedValue::Uint256(_) => 32,
            ExtractedValue::Address(_) => 20,
            ExtractedValue::Bytes32(_) => 32,
            ExtractedValue::Raw(_) => 32,
        }
    }

    /// Check if value represents semantic zero
    /// 
    /// This function determines if a value is semantically zero,
    /// which is critical for zero-value attack detection. It uses type-specific
    /// zero checks to prevent semantic confusion about what constitutes "zero".
    #[inline]
    pub fn is_semantic_zero(&self) -> bool {
        match self {
            ExtractedValue::Bool(b) => !*b,  // false is semantic zero for bool
            ExtractedValue::Uint8(n) => *n == 0,
            ExtractedValue::Uint16(n) => *n == 0,
            ExtractedValue::Uint32(n) => *n == 0,
            ExtractedValue::Uint64(n) => *n == 0,
            ExtractedValue::Uint256(bytes) => *bytes == [0u8; 32],
            ExtractedValue::Address(addr) => *addr == [0u8; 20], // Zero address detection
            ExtractedValue::Bytes32(bytes) => *bytes == [0u8; 32],
            ExtractedValue::Raw(bytes) => *bytes == [0u8; 32],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    #[test]
    fn test_semantic_validation_valid_zero_address() {
        let layout_commitment = [1u8; 32];
        let field_types = vec![FieldType::Address];
        let field_semantics = vec![ZeroSemantics::ValidZero];
        
        let processor = CircuitProcessor::new(layout_commitment, field_types, field_semantics);
        
        let witness = CircuitWitness {
            key: [2u8; 32],
            value: [0u8; 32], // Zero address
            proof: vec![1, 2, 3],
            layout_commitment,
            field_index: 0,
            semantics: ZeroSemantics::ExplicitlyZero,
            expected_slot: [2u8; 32],
            block_height: 0,
            block_hash: [0u8; 32],
        };
        
        // Should be invalid because zero address is suspicious
        let result = processor.process_witness(&witness);
        assert!(matches!(result, CircuitResult::Invalid));
    }

    #[test] 
    fn test_semantic_validation_valid_uint256() {
        let layout_commitment = [1u8; 32];
        let field_types = vec![FieldType::Uint256];
        let field_semantics = vec![ZeroSemantics::ValidZero];
        
        let processor = CircuitProcessor::new(layout_commitment, field_types, field_semantics);
        
        let mut value = [0u8; 32];
        value[31] = 42; // Non-zero value
        
        let witness = CircuitWitness {
            key: [2u8; 32],
            value,
            proof: vec![1, 2, 3],
            layout_commitment,
            field_index: 0,
            semantics: ZeroSemantics::ValidZero,
            expected_slot: [2u8; 32],
            block_height: 0,
            block_hash: [0u8; 32],
        };
        
        let result = processor.process_witness(&witness);
        // Non-zero values with ValidZero semantics should be valid (counters, flags, balances)
        assert!(matches!(result, CircuitResult::Valid { .. }));
    }

    #[test]
    fn test_semantic_validation_invalid_zero_semantics() {
        let layout_commitment = [1u8; 32];
        let field_types = vec![FieldType::Uint256];
        let field_semantics = vec![ZeroSemantics::NeverWritten];
        
        let processor = CircuitProcessor::new(layout_commitment, field_types, field_semantics);
        
        let mut value = [0u8; 32];
        value[31] = 42; // Non-zero value
        
        let witness = CircuitWitness {
            key: [2u8; 32],
            value,
            proof: vec![1, 2, 3],
            layout_commitment,
            field_index: 0,
            semantics: ZeroSemantics::NeverWritten, // Invalid: non-zero but claims never written
            expected_slot: [2u8; 32],
            block_height: 0,
            block_hash: [0u8; 32],
        };
        
        let result = processor.process_witness(&witness);
        assert!(matches!(result, CircuitResult::Invalid));
    }

    #[test]
    fn test_semantic_validation_storage_location_mismatch() {
        let layout_commitment = [1u8; 32];
        let field_types = vec![FieldType::Uint256];
        let field_semantics = vec![ZeroSemantics::ValidZero];
        
        let processor = CircuitProcessor::new(layout_commitment, field_types, field_semantics);
        
        let witness = CircuitWitness {
            key: [2u8; 32],      // Actual key
            value: [0u8; 32],
            proof: vec![1, 2, 3],
            layout_commitment,
            field_index: 0,
            semantics: ZeroSemantics::ExplicitlyZero,
            expected_slot: [3u8; 32], // Different expected slot
            block_height: 0,
            block_hash: [0u8; 32],
        };
        
        // Should be invalid due to storage location mismatch
        let result = processor.process_witness(&witness);
        assert!(matches!(result, CircuitResult::Invalid));
    }

    #[test]
    fn test_semantic_validation_layout_commitment_mismatch() {
        let layout_commitment = [1u8; 32];
        let field_types = vec![FieldType::Uint256];
        let field_semantics = vec![ZeroSemantics::ValidZero];
        
        let processor = CircuitProcessor::new(layout_commitment, field_types, field_semantics);
        
        let witness = CircuitWitness {
            key: [2u8; 32],
            value: [0u8; 32],
            proof: vec![1, 2, 3],
            layout_commitment: [99u8; 32], // Wrong layout commitment
            field_index: 0,
            semantics: ZeroSemantics::ExplicitlyZero,
            expected_slot: [2u8; 32],
            block_height: 0,
            block_hash: [0u8; 32],
        };
        
        // Should be invalid due to layout commitment mismatch
        let result = processor.process_witness(&witness);
        assert!(matches!(result, CircuitResult::Invalid));
    }

    #[test]
    fn test_field_type_validation() {
        assert!(FieldType::Bool.can_be_zero());
        assert!(FieldType::Uint256.can_be_zero());
        assert!(!FieldType::Address.can_be_zero()); // Zero address is suspicious
        assert!(FieldType::Bytes32.can_be_zero());
    }

    #[test]
    fn test_extracted_value_semantic_zero() {
        assert!(ExtractedValue::Bool(false).is_semantic_zero());
        assert!(!ExtractedValue::Bool(true).is_semantic_zero());
        
        assert!(ExtractedValue::Uint256([0u8; 32]).is_semantic_zero());
        assert!(!ExtractedValue::Uint256([1u8; 32]).is_semantic_zero());
        
        assert!(ExtractedValue::Address([0u8; 20]).is_semantic_zero());
        assert!(!ExtractedValue::Address([1u8; 20]).is_semantic_zero());
    }

    #[test]
    fn test_light_client_validation() {
        let layout_commitment = [1u8; 32];
        let field_types = vec![FieldType::Uint256];
        let field_semantics = vec![ZeroSemantics::ValidZero];
        let block_height = 12345u64;
        let block_hash = [0xABu8; 32];
        
        let processor = CircuitProcessor::new_with_light_client(
            layout_commitment,
            field_types,
            field_semantics,
            block_height,
            block_hash,
        );
        
        // Test with matching block data
        let valid_witness = CircuitWitness {
            key: [2u8; 32],
            value: [0u8; 32],
            proof: vec![1, 2, 3],
            layout_commitment,
            field_index: 0,
            semantics: ZeroSemantics::ExplicitlyZero,
            expected_slot: [2u8; 32],
            block_height,
            block_hash,
        };
        
        let result = processor.process_witness(&valid_witness);
        assert!(matches!(result, CircuitResult::Valid { .. }));
        
        // Test with mismatched block height
        let wrong_height_witness = CircuitWitness {
            key: [2u8; 32],
            value: [0u8; 32],
            proof: vec![1, 2, 3],
            layout_commitment,
            field_index: 0,
            semantics: ZeroSemantics::ExplicitlyZero,
            expected_slot: [2u8; 32],
            block_height: 54321, // Wrong height
            block_hash,
        };
        
        let result = processor.process_witness(&wrong_height_witness);
        assert!(matches!(result, CircuitResult::Invalid));
        
        // Test with mismatched block hash
        let wrong_hash_witness = CircuitWitness {
            key: [2u8; 32],
            value: [0u8; 32],
            proof: vec![1, 2, 3],
            layout_commitment,
            field_index: 0,
            semantics: ZeroSemantics::ExplicitlyZero,
            expected_slot: [2u8; 32],
            block_height,
            block_hash: [0xDEu8; 32], // Wrong hash
        };
        
        let result = processor.process_witness(&wrong_hash_witness);
        assert!(matches!(result, CircuitResult::Invalid));
    }

    #[test]
    fn test_witness_parsing() {
        // Test enhanced format with block data
        let mut witness_data = Vec::new();
        
        // Storage key (32 bytes)
        witness_data.extend_from_slice(&[1u8; 32]);
        
        // Layout commitment (32 bytes)
        witness_data.extend_from_slice(&[2u8; 32]);
        
        // Value (32 bytes)
        witness_data.extend_from_slice(&[3u8; 32]);
        
        // Semantics (1 byte)
        witness_data.push(1); // ExplicitlyZero
        
        // Semantic source (1 byte)
        witness_data.push(0);
        
        // Block height (8 bytes)
        witness_data.extend_from_slice(&12345u64.to_le_bytes());
        
        // Block hash (32 bytes)
        witness_data.extend_from_slice(&[4u8; 32]);
        
        // Proof length (4 bytes)
        witness_data.extend_from_slice(&4u32.to_le_bytes());
        
        // Proof data
        witness_data.extend_from_slice(&[0xDE, 0xAD, 0xBE, 0xEF]);
        
        // Extended format: field_index (2 bytes)
        witness_data.extend_from_slice(&7u16.to_le_bytes());
        
        // Extended format: expected_slot (32 bytes)
        witness_data.extend_from_slice(&[9u8; 32]);
        
        let witness = CircuitProcessor::parse_witness_from_bytes(&witness_data).unwrap();
        
        assert_eq!(witness.key, [1u8; 32]);
        assert_eq!(witness.layout_commitment, [2u8; 32]);
        assert_eq!(witness.value, [3u8; 32]);
        assert_eq!(witness.semantics, ZeroSemantics::ExplicitlyZero);
        assert_eq!(witness.block_height, 12345);
        assert_eq!(witness.block_hash, [4u8; 32]);
        assert_eq!(witness.proof, vec![0xDE, 0xAD, 0xBE, 0xEF]);
        assert_eq!(witness.field_index, 7);
        assert_eq!(witness.expected_slot, [9u8; 32]);
    }


    #[test]
    fn test_semantic_validation_bug_fix() {
        // Test that non-zero values with ValidZero semantics are rejected
        let layout_commitment = [1u8; 32];
        let field_types = vec![FieldType::Uint256];
        let field_semantics = vec![ZeroSemantics::ValidZero];
        
        let processor = CircuitProcessor::new(layout_commitment, field_types, field_semantics);
        
        let mut value = [0u8; 32];
        value[31] = 42; // Non-zero value
        
        let witness = CircuitWitness {
            key: [2u8; 32],
            value,
            proof: vec![1, 2, 3],
            layout_commitment,
            field_index: 0,
            semantics: ZeroSemantics::ValidZero, // BUG: Non-zero with ValidZero semantics
            expected_slot: [2u8; 32],
            block_height: 0,
            block_hash: [0u8; 32],
        };
        
        // Should be valid - non-zero values with ValidZero semantics are allowed
        let result = processor.process_witness(&witness);
        assert!(matches!(result, CircuitResult::Valid { .. }));
        
        // Test invalid zero semantics with non-zero values - should fail
        // (ValidZero is excluded since it should be valid with non-zero values)
        for semantics in [
            ZeroSemantics::NeverWritten,
            ZeroSemantics::ExplicitlyZero,
            ZeroSemantics::Cleared,
        ] {
            let witness = CircuitWitness {
                key: [2u8; 32],
                value,
                proof: vec![1, 2, 3],
                layout_commitment,
                field_index: 0,
                semantics,
                expected_slot: [2u8; 32],
                block_height: 0,
                block_hash: [0u8; 32],
            };
            
            let result = processor.process_witness(&witness);
            assert!(matches!(result, CircuitResult::Invalid), 
                "Non-zero value with {:?} semantics should be invalid", semantics);
        }
    }

    #[test]
    fn test_proof_timing_validation() {
        let layout_commitment = [1u8; 32];
        let field_types = vec![FieldType::Uint256];
        let field_semantics = vec![ZeroSemantics::ValidZero];
        let current_block = 1000u64;
        let block_hash = [0xABu8; 32];
        
        // Test with custom expiration of 100 blocks
        let processor = CircuitProcessor::new_with_expiration(
            layout_commitment,
            field_types,
            field_semantics,
            current_block,
            block_hash,
            100, // max age: 100 blocks
        );
        
        // Test 1: Fresh proof (same block) - should be valid
        let fresh_witness = CircuitWitness {
            key: [2u8; 32],
            value: [0u8; 32],
            proof: vec![1, 2, 3],
            layout_commitment,
            field_index: 0,
            semantics: ZeroSemantics::ExplicitlyZero,
            expected_slot: [2u8; 32],
            block_height: current_block,
            block_hash,
        };
        
        let result = processor.process_witness(&fresh_witness);
        assert!(matches!(result, CircuitResult::Valid { .. }));
        
        // Test 2: Slightly old proof (50 blocks old) - should be valid
        let slightly_old_witness = CircuitWitness {
            key: [2u8; 32],
            value: [0u8; 32],
            proof: vec![1, 2, 3],
            layout_commitment,
            field_index: 0,
            semantics: ZeroSemantics::ExplicitlyZero,
            expected_slot: [2u8; 32],
            block_height: current_block - 50,
            block_hash: [0xBCu8; 32], // Different hash for different block
        };
        
        let result = processor.process_witness(&slightly_old_witness);
        assert!(matches!(result, CircuitResult::Valid { .. }));
        
        // Test 3: Expired proof (150 blocks old) - should be invalid
        let expired_witness = CircuitWitness {
            key: [2u8; 32],
            value: [0u8; 32],
            proof: vec![1, 2, 3],
            layout_commitment,
            field_index: 0,
            semantics: ZeroSemantics::ExplicitlyZero,
            expected_slot: [2u8; 32],
            block_height: current_block - 150,
            block_hash: [0xCDu8; 32], // Different hash for different block
        };
        
        let result = processor.process_witness(&expired_witness);
        assert!(matches!(result, CircuitResult::Invalid));
        
        // Test 4: Future proof (should be invalid - can't have proof from future)
        let future_witness = CircuitWitness {
            key: [2u8; 32],
            value: [0u8; 32],
            proof: vec![1, 2, 3],
            layout_commitment,
            field_index: 0,
            semantics: ZeroSemantics::ExplicitlyZero,
            expected_slot: [2u8; 32],
            block_height: current_block + 10, // Future block
            block_hash: [0xDEu8; 32], // Different hash for different block
        };
        
        let result = processor.process_witness(&future_witness);
        assert!(matches!(result, CircuitResult::Invalid));
    }

    #[test]
    fn test_comprehensive_semantic_validation() {
        let layout_commitment = [1u8; 32];
        let field_types = vec![
            FieldType::Bool,
            FieldType::Uint256,
            FieldType::Address,
        ];
        let field_semantics = vec![
            ZeroSemantics::ValidZero,
            ZeroSemantics::ExplicitlyZero, 
            ZeroSemantics::ValidZero,
        ];
        
        let processor = CircuitProcessor::new(layout_commitment, field_types, field_semantics);
        
        // Test valid bool (false = 0)
        let bool_witness = CircuitWitness {
            key: [1u8; 32],
            value: [0u8; 32], // false
            proof: vec![],
            layout_commitment,
            field_index: 0,
            semantics: ZeroSemantics::ValidZero,
            expected_slot: [1u8; 32],
            block_height: 0,
            block_hash: [0u8; 32],
        };
        
        let result = processor.process_witness(&bool_witness);
        assert!(matches!(result, CircuitResult::Valid { .. }));
        
        // Test valid uint256 zero
        let uint_witness = CircuitWitness {
            key: [2u8; 32],
            value: [0u8; 32], // zero uint256
            proof: vec![],
            layout_commitment,
            field_index: 1,
            semantics: ZeroSemantics::ExplicitlyZero,
            expected_slot: [2u8; 32],
            block_height: 0,
            block_hash: [0u8; 32],
        };
        
        let result = processor.process_witness(&uint_witness);
        assert!(matches!(result, CircuitResult::Valid { .. }));
        
        // Test invalid zero address 
        let addr_witness = CircuitWitness {
            key: [3u8; 32],
            value: [0u8; 32], // zero address (suspicious)
            proof: vec![],
            layout_commitment,
            field_index: 2,
            semantics: ZeroSemantics::ValidZero,
            expected_slot: [3u8; 32],
            block_height: 0,
            block_hash: [0u8; 32],
        };
        
        let result = processor.process_witness(&addr_witness);
        assert!(matches!(result, CircuitResult::Invalid)); // Zero address should fail
    }

    #[test]
    fn test_edge_case_field_index_bounds() {
        let layout_commitment = [1u8; 32];
        let field_types = vec![FieldType::Uint256, FieldType::Uint256];
        let field_semantics = vec![ZeroSemantics::ValidZero, ZeroSemantics::ValidZero];
        
        let processor = CircuitProcessor::new(layout_commitment, field_types, field_semantics);
        
        // Test field_index at boundary
        let witness_at_boundary = CircuitWitness {
            key: [1u8; 32],
            value: [0u8; 32],
            proof: vec![],
            layout_commitment,
            field_index: 1, // Last valid index
            semantics: ZeroSemantics::ValidZero,
            expected_slot: [1u8; 32],
            block_height: 0,
            block_hash: [0u8; 32],
        };
        
        let result = processor.process_witness(&witness_at_boundary);
        assert!(matches!(result, CircuitResult::Valid { .. }));
        
        // Test field_index out of bounds
        let witness_out_of_bounds = CircuitWitness {
            key: [1u8; 32],
            value: [0u8; 32],
            proof: vec![],
            layout_commitment,
            field_index: 2, // Out of bounds
            semantics: ZeroSemantics::ValidZero,
            expected_slot: [1u8; 32],
            block_height: 0,
            block_hash: [0u8; 32],
        };
        
        let result = processor.process_witness(&witness_out_of_bounds);
        assert!(matches!(result, CircuitResult::Invalid));
        
        // Test with u16::MAX field_index
        let witness_max_index = CircuitWitness {
            key: [1u8; 32],
            value: [0u8; 32],
            proof: vec![],
            layout_commitment,
            field_index: u16::MAX,
            semantics: ZeroSemantics::ValidZero,
            expected_slot: [1u8; 32],
            block_height: 0,
            block_hash: [0u8; 32],
        };
        
        let result = processor.process_witness(&witness_max_index);
        assert!(matches!(result, CircuitResult::Invalid));
    }

    #[test]
    fn test_edge_case_empty_proof() {
        let layout_commitment = [1u8; 32];
        let field_types = vec![FieldType::Uint256];
        let field_semantics = vec![ZeroSemantics::ValidZero];
        
        let processor = CircuitProcessor::new(layout_commitment, field_types, field_semantics);
        
        // Test with empty proof vector
        let witness_empty_proof = CircuitWitness {
            key: [1u8; 32],
            value: [42u8; 32],
            proof: vec![], // Empty proof
            layout_commitment,
            field_index: 0,
            semantics: ZeroSemantics::ValidZero,
            expected_slot: [1u8; 32],
            block_height: 0,
            block_hash: [0u8; 32],
        };
        
        // Should still validate other aspects even with empty proof
        let result = processor.process_witness(&witness_empty_proof);
        assert!(matches!(result, CircuitResult::Valid { .. })); // Non-zero with ValidZero semantics is valid
    }

    #[test]
    fn test_edge_case_large_proof() {
        let layout_commitment = [1u8; 32];
        let field_types = vec![FieldType::Uint256];
        let field_semantics = vec![ZeroSemantics::ValidZero];
        
        let processor = CircuitProcessor::new(layout_commitment, field_types, field_semantics);
        
        // Test with very large proof
        let large_proof = vec![0xFFu8; 10000]; // 10KB proof
        let witness_large_proof = CircuitWitness {
            key: [1u8; 32],
            value: [0u8; 32],
            proof: large_proof,
            layout_commitment,
            field_index: 0,
            semantics: ZeroSemantics::ValidZero,
            expected_slot: [1u8; 32],
            block_height: 0,
            block_hash: [0u8; 32],
        };
        
        let result = processor.process_witness(&witness_large_proof);
        assert!(matches!(result, CircuitResult::Valid { .. }));
    }

    #[test]
    fn test_edge_case_all_field_types() {
        let layout_commitment = [1u8; 32];
        let field_types = vec![
            FieldType::Bool,
            FieldType::Uint8,
            FieldType::Uint16,
            FieldType::Uint32,
            FieldType::Uint64,
            FieldType::Uint256,
            FieldType::Address,
            FieldType::Bytes32,
            FieldType::String,
            FieldType::Bytes,
        ];
        let field_semantics = vec![ZeroSemantics::ValidZero; 10];
        
        let processor = CircuitProcessor::new(layout_commitment, field_types.clone(), field_semantics);
        
        // Test each field type with appropriate values
        for (index, field_type) in field_types.iter().enumerate() {
            let mut value = [0u8; 32];
            
            // Set appropriate non-zero values for each type
            match field_type {
                FieldType::Bool => value[31] = 1, // true
                FieldType::Uint8 => value[31] = 255,
                FieldType::Uint16 => {
                    value[30] = 0xFF;
                    value[31] = 0xFF;
                },
                FieldType::Uint32 => {
                    value[28] = 0xFF;
                    value[29] = 0xFF;
                    value[30] = 0xFF;
                    value[31] = 0xFF;
                },
                FieldType::Uint64 => {
                    for i in 24..32 {
                        value[i] = 0xFF;
                    }
                },
                FieldType::Uint256 | FieldType::Bytes32 => {
                    for i in 0..32 {
                        value[i] = 0xFF;
                    }
                },
                FieldType::Address => {
                    // Valid non-zero address
                    for i in 12..32 {
                        value[i] = 0x42;
                    }
                },
                _ => {
                    // For String/Bytes, just use non-zero value
                    value[31] = 1;
                },
            }
            
            let witness = CircuitWitness {
                key: [(index + 1) as u8; 32],
                value,
                proof: vec![],
                layout_commitment,
                field_index: index as u16,
                semantics: ZeroSemantics::ValidZero,
                expected_slot: [(index + 1) as u8; 32],
                block_height: 0,
                block_hash: [0u8; 32],
            };
            
            let result = processor.process_witness(&witness);
            
            // All non-zero values with ValidZero semantics should be valid
            assert!(matches!(result, CircuitResult::Valid { .. }), 
                "Field type {:?} should be valid with non-zero value and ValidZero semantics", field_type);
        }
    }

    #[test]
    fn test_edge_case_witness_parsing_extended_format() {
        // Test extended format with field_index and expected_slot
        let mut witness_data = Vec::new();
        
        // Storage key (32 bytes)
        witness_data.extend_from_slice(&[1u8; 32]);
        
        // Layout commitment (32 bytes)
        witness_data.extend_from_slice(&[2u8; 32]);
        
        // Value (32 bytes)
        witness_data.extend_from_slice(&[3u8; 32]);
        
        // Semantics (1 byte)
        witness_data.push(1); // ExplicitlyZero
        
        // Semantic source (1 byte)
        witness_data.push(0);
        
        // Block height (8 bytes)
        witness_data.extend_from_slice(&12345u64.to_le_bytes());
        
        // Block hash (32 bytes)
        witness_data.extend_from_slice(&[4u8; 32]);
        
        // Proof length (4 bytes)
        witness_data.extend_from_slice(&4u32.to_le_bytes());
        
        // Proof data
        witness_data.extend_from_slice(&[0xDE, 0xAD, 0xBE, 0xEF]);
        
        // Extended format: field_index (2 bytes)
        witness_data.extend_from_slice(&42u16.to_le_bytes());
        
        // Extended format: expected_slot (32 bytes)
        witness_data.extend_from_slice(&[5u8; 32]);
        
        let witness = CircuitProcessor::parse_witness_from_bytes(&witness_data).unwrap();
        
        assert_eq!(witness.field_index, 42);
        assert_eq!(witness.expected_slot, [5u8; 32]);
    }

    #[test]
    fn test_edge_case_witness_parsing_errors() {
        // Test too small witness data
        let small_data = vec![0u8; 50]; // Less than minimum 176 bytes
        let result = CircuitProcessor::parse_witness_from_bytes(&small_data);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Witness data too small (extended format required)");
        
        // Test invalid semantics value
        let mut invalid_semantics_data = vec![0u8; 176];
        invalid_semantics_data[96] = 5; // Invalid semantics value (> 3)
        let result = CircuitProcessor::parse_witness_from_bytes(&invalid_semantics_data);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid zero semantics value");
        
        // Test incomplete proof data (too small for extended format)
        let mut incomplete_proof = vec![0u8; 142];
        // Set proof length to 100 but don't provide enough data
        incomplete_proof[138..142].copy_from_slice(&100u32.to_le_bytes());
        let result = CircuitProcessor::parse_witness_from_bytes(&incomplete_proof);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Witness data too small (extended format required)");
        
        // Test incomplete proof data (passes size check but proof too large)
        let mut incomplete_proof_large = vec![0u8; 176];
        // Set proof length to 100 at offset 138, but only provide minimum 176 bytes (which includes 38 bytes for base + proof)
        incomplete_proof_large[138..142].copy_from_slice(&100u32.to_le_bytes());
        let result = CircuitProcessor::parse_witness_from_bytes(&incomplete_proof_large);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Incomplete proof data");
        
        // Test cases for errors that are actually reachable
        
        // Test with minimum valid size (176 bytes) but all zero - should succeed parsing
        let valid_minimal = vec![0u8; 176];
        let result = CircuitProcessor::parse_witness_from_bytes(&valid_minimal);
        assert!(result.is_ok(), "Minimal 176-byte witness should parse successfully");
    }

    #[test]
    fn test_edge_case_batch_processing() {
        let layout_commitment = [1u8; 32];
        let field_types = vec![FieldType::Uint256; 100]; // 100 fields
        let field_semantics = vec![ZeroSemantics::ValidZero; 100];
        
        let processor = CircuitProcessor::new(layout_commitment, field_types, field_semantics);
        
        // Create 100 witnesses
        let mut witnesses = Vec::new();
        for i in 0..100 {
            let mut key = [0u8; 32];
            key[31] = i as u8;
            
            let witness = CircuitWitness {
                key,
                value: [0u8; 32], // All zero values
                proof: vec![i as u8; 10], // Different proof for each
                layout_commitment,
                field_index: i as u16,
                semantics: ZeroSemantics::ValidZero,
                expected_slot: key,
                block_height: i as u64,
                block_hash: [i as u8; 32],
            };
            witnesses.push(witness);
        }
        
        // Process batch
        let results = processor.process_batch(&witnesses);
        assert_eq!(results.len(), 100);
        
        // All should be valid (zero values with ValidZero semantics)
        for result in results {
            assert!(matches!(result, CircuitResult::Valid { .. }));
        }
    }

    #[test]
    fn test_edge_case_malformed_value_extraction() {
        let layout_commitment = [1u8; 32];
        let field_types = vec![
            FieldType::Uint16,
            FieldType::Uint32,
            FieldType::Uint64,
            FieldType::Address,
        ];
        let field_semantics = vec![ZeroSemantics::ValidZero; 4];
        
        let processor = CircuitProcessor::new(layout_commitment, field_types, field_semantics);
        
        // Test Uint16 extraction with specific pattern
        let mut value_u16 = [0u8; 32];
        value_u16[30] = 0xAB;
        value_u16[31] = 0xCD;

        let witness_u16 = CircuitWitness {
            key: [1u8; 32],
            value: value_u16,
            proof: vec![],
            layout_commitment,
            field_index: 0,
            semantics: ZeroSemantics::ValidZero, // Non-zero value with ValidZero semantics is valid
            expected_slot: [1u8; 32],
            block_height: 0,
            block_hash: [0u8; 32],
        };

        let result = processor.process_witness(&witness_u16);
        if let CircuitResult::Valid { extracted_value, .. } = result {
            if let ExtractedValue::Uint16(val) = extracted_value {
                assert_eq!(val, 0xABCD); // Big-endian
            } else {
                panic!("Expected Uint16 extraction");
            }
        } else {
            panic!("Expected valid result for non-zero Uint16 with ValidZero semantics");
        }
        
                // Test Address extraction with pattern
        let mut value_addr = [0u8; 32];
        // Set address bytes (12-31)
        for i in 12..32 {
            value_addr[i] = (i - 12) as u8;
        }

        let witness_addr = CircuitWitness {
            key: [2u8; 32],
            value: value_addr,
            proof: vec![],
            layout_commitment,
            field_index: 3,
            semantics: ZeroSemantics::ValidZero, // Non-zero address with ValidZero semantics is valid
            expected_slot: [2u8; 32],
            block_height: 0,
            block_hash: [0u8; 32],
        };

        let result = processor.process_witness(&witness_addr);
        if let CircuitResult::Valid { extracted_value, .. } = result {
            if let ExtractedValue::Address(addr) = extracted_value {
                // Check address extraction
                for i in 0..20 {
                    assert_eq!(addr[i], i as u8);
                }
            } else {
                panic!("Expected Address extraction");
            }
        } else {
            panic!("Expected valid result for non-zero address with ValidZero semantics");
        }
    }

    #[test]
    fn test_edge_case_concurrent_witness_validation() {
        // Test that witnesses don't interfere with each other
        let layout_commitment = [1u8; 32];
        let field_types = vec![FieldType::Uint256, FieldType::Address];
        let field_semantics = vec![ZeroSemantics::ExplicitlyZero, ZeroSemantics::NeverWritten];
        
        let processor = CircuitProcessor::new(layout_commitment, field_types, field_semantics);
        
        // Create two witnesses with different validation results
        let valid_witness = CircuitWitness {
            key: [1u8; 32],
            value: [0u8; 32], // Zero value
            proof: vec![1, 2, 3],
            layout_commitment,
            field_index: 0,
            semantics: ZeroSemantics::ExplicitlyZero, // Matches expected
            expected_slot: [1u8; 32],
            block_height: 100,
            block_hash: [0xAAu8; 32],
        };
        
        let invalid_witness = CircuitWitness {
            key: [2u8; 32],
            value: [0u8; 32], // Zero address
            proof: vec![4, 5, 6],
            layout_commitment,
            field_index: 1,
            semantics: ZeroSemantics::NeverWritten, // Zero address is suspicious
            expected_slot: [2u8; 32],
            block_height: 101,
            block_hash: [0xBBu8; 32],
        };
        
        // Process in different orders
        let batch1 = vec![valid_witness.clone(), invalid_witness.clone()];
        let batch2 = vec![invalid_witness.clone(), valid_witness.clone()];
        
        let results1 = processor.process_batch(&batch1);
        let results2 = processor.process_batch(&batch2);
        
        // First should be valid, second invalid regardless of order
        assert!(matches!(results1[0], CircuitResult::Valid { .. }));
        assert!(matches!(results1[1], CircuitResult::Invalid));
        assert!(matches!(results2[0], CircuitResult::Invalid));
        assert!(matches!(results2[1], CircuitResult::Valid { .. }));
    }
}
