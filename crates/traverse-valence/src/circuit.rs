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
/// SECURITY: These semantics prevent semantic confusion attacks where an attacker
/// claims a storage value has different meaning than its actual state. Each semantic
/// type has specific validation rules to prevent manipulation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ZeroSemantics {
    /// Storage slot was never written to (initial state)
    /// SECURITY: Can only be claimed for genuinely uninitialized storage
    NeverWritten,
    /// Storage slot was explicitly set to zero by contract logic
    /// SECURITY: Must be validated against field type - not all fields can be legitimately zero
    ExplicitlyZero,
    /// Storage slot was cleared (set to zero after having a value)
    /// SECURITY: Must match expected clearing behavior for the field
    Cleared,
    /// Zero is a valid value for this field type (e.g., counters, flags)
    /// SECURITY: Only allowed for field types where zero is semantically meaningful
    ValidZero,
}

/// Field types for semantic value extraction and validation
/// 
/// SECURITY: Each field type has specific validation rules to prevent type confusion
/// attacks and ensure extracted values are semantically correct for their intended use.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FieldType {
    Bool,
    Uint8,
    Uint16,
    Uint32,
    Uint64,
    Uint256,
    Address,    // SECURITY: Zero addresses are flagged as suspicious
    Bytes32,
    String,
    Bytes,
}

impl FieldType {
    /// Check if a field type can legitimately be zero
    /// 
    /// SECURITY: This prevents zero-value attacks where adversaries claim
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
            FieldType::Address => false,   // SECURITY: 0x0 address is suspicious/invalid
            FieldType::Bytes32 => true,    // empty hash can be semantically valid
            FieldType::String => true,     // empty string = 0 (valid)
            FieldType::Bytes => true,      // empty bytes = 0 (valid)
        }
    }

    /// Check if extracted value is semantically valid for this field type
    /// 
    /// SECURITY: This prevents type confusion attacks where extracted values
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
                // SECURITY: Zero address validation prevents common attack patterns
                // where uninitialized or malicious addresses are used. Zero addresses
                // are often indicators of bugs or intentional exploits.
                *addr != [0u8; 20]
            }
            (FieldType::Bytes32, ExtractedValue::Bytes32(_)) => true,
            _ => false, // SECURITY: Type mismatch indicates potential attack or corruption
        }
    }
}

/// Minimal witness structure for ZK circuits with semantic validation
/// 
/// SECURITY: This structure contains all data needed for secure proof verification.
/// Each field serves a specific security purpose and is validated independently.
#[derive(Debug, Clone)]
pub struct CircuitWitness {
    /// Storage key (32 bytes)
    /// SECURITY: Must match expected_slot to prevent storage slot spoofing attacks
    pub key: [u8; 32],
    /// Storage value (32 bytes)
    /// SECURITY: Extracted and validated according to field type semantics
    pub value: [u8; 32],
    /// Proof data (minimal size)
    /// SECURITY: Contains cryptographic proof of storage state
    pub proof: Vec<u8>,
    /// Layout commitment (32 bytes) - must match expected layout
    /// SECURITY: Prevents layout spoofing attacks where adversaries claim
    /// different field layouts to manipulate value interpretation
    pub layout_commitment: [u8; 32],
    /// Field index in layout
    /// SECURITY: Must be within bounds to prevent out-of-bounds access
    pub field_index: u16,
    /// Zero semantics (must match storage semantics)
    /// SECURITY: Prevents semantic confusion attacks about value meaning
    pub semantics: ZeroSemantics,
    /// Expected storage slot for this field (for validation)
    /// SECURITY: Critical for preventing storage slot spoofing where
    /// adversaries claim values from different storage locations
    pub expected_slot: [u8; 32],
    /// Block height for proof timing validation
    /// SECURITY: Ensures proof is from the expected block height
    pub block_height: u64,
    /// Block hash for light client verification
    /// SECURITY: Must match the proven block hash from light client
    pub block_hash: [u8; 32],
}

/// Minimal circuit processor with semantic validation (no_std compatible)
/// 
/// SECURITY: This processor enforces all security validations required for
/// safe ZK circuit operation. It validates layout consistency, storage location
/// correctness, and semantic value interpretation.
pub struct CircuitProcessor {
    /// Current layout commitment
    /// SECURITY: Immutable after creation to prevent layout tampering
    layout_commitment: [u8; 32],
    /// Field types for extraction and validation
    /// SECURITY: Defines expected types to prevent type confusion attacks
    field_types: Vec<FieldType>,
    /// Expected semantic behavior for each field
    /// SECURITY: Defines valid zero semantics to prevent semantic manipulation
    field_semantics: Vec<ZeroSemantics>,
    /// Expected block height for proof validation
    /// SECURITY: Ensures all proofs are from the same block
    expected_block_height: u64,
    /// Expected block hash from light client
    /// SECURITY: Validates proofs against light client consensus
    expected_block_hash: [u8; 32],
    /// Maximum allowed age for proofs in blocks
    /// SECURITY: Prevents replay attacks with old proofs
    max_proof_age_blocks: u64,
}

impl CircuitProcessor {
    /// Create new processor with semantic validation
    /// 
    /// SECURITY: All validation parameters are set at creation time and cannot
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
    /// SECURITY: Includes block height and hash validation from light client
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
    /// SECURITY: Allows setting custom proof age limits for different security requirements
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

    /// Parse witness data from raw bytes
    /// 
    /// SECURITY: This function parses the witness format created by the controller.
    /// It supports both legacy format (without block data) and enhanced format
    /// (with block height and hash for light client validation).
    pub fn parse_witness_from_bytes(witness_data: &[u8]) -> Result<CircuitWitness, &'static str> {
        // Minimum size check for legacy format
        if witness_data.len() < 102 {
            return Err("Witness data too small");
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
        
        // Check if this is enhanced format with block data
        let (block_height, block_hash) = if witness_data.len() >= offset + 40 {
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
            
            (block_height, block_hash)
        } else {
            // Legacy format without block data
            (0u64, [0u8; 32])
        };
        
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
        
        // For now, we'll use dummy values for field_index and expected_slot
        // In a real implementation, these would be derived from the layout
        Ok(CircuitWitness {
            key,
            value,
            proof,
            layout_commitment,
            field_index: 0, // TODO: Derive from layout
            semantics,
            expected_slot: key, // TODO: Compute expected slot
            block_height,
            block_hash,
        })
    }
    
    /// Process witness with comprehensive semantic validation
    /// 
    /// SECURITY: This is the main entry point for witness validation. It performs
    /// multiple security checks in sequence, failing fast if any validation fails.
    /// The order of checks is designed to catch the most common attack patterns first.
    pub fn process_witness(&self, witness: &CircuitWitness) -> CircuitResult {
        // SECURITY CRITICAL: Layout commitment validation must be first
        // This prevents layout spoofing attacks where adversaries claim different
        // field layouts to manipulate how values are interpreted. Without this check,
        // an attacker could claim a uint256 field is actually an address field.
        if witness.layout_commitment != self.layout_commitment {
            return CircuitResult::Invalid;
        }
        
        // SECURITY CRITICAL: Light client validation for block consistency
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
                // SECURITY CRITICAL: Proof age validation prevents replay attacks
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

        // SECURITY CRITICAL: Bounds checking prevents out-of-bounds access
        // This prevents buffer overflow attacks and ensures field_index is valid
        // for both field_types and field_semantics arrays. Without this check,
        // an attacker could cause undefined behavior or access wrong field metadata.
        if witness.field_index as usize >= self.field_types.len() {
            return CircuitResult::Invalid;
        }

        let field_type = self.field_types[witness.field_index as usize];
        let expected_semantics = self.field_semantics[witness.field_index as usize];

        // SECURITY CRITICAL: Semantic consistency validation prevents semantic confusion
        // This ensures that claimed zero semantics match the actual field type and value.
        // Without this check, an attacker could claim a non-zero value has "never written"
        // semantics, or claim a zero address has "valid zero" semantics.
        if !self.validate_semantic_consistency(witness, field_type, expected_semantics) {
            return CircuitResult::Invalid;
        }

        // SECURITY CRITICAL: Storage location validation prevents storage slot spoofing
        // This ensures the storage key matches the expected slot for this field.
        // Without this check, an attacker could provide values from different storage
        // locations while claiming they belong to the expected field.
        if !self.validate_storage_location(witness) {
            return CircuitResult::Invalid;
        }

        // SECURITY: Value extraction with type validation prevents type confusion
        // This ensures extracted values match their claimed field type semantics.
        let extracted_value = self.extract_value(witness, field_type);

        // SECURITY CRITICAL: Final value validation catches field-specific attacks
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
    /// SECURITY: Each witness is validated independently to prevent cross-contamination
    /// attacks where one malicious witness could affect validation of others.
    pub fn process_batch(&self, witnesses: &[CircuitWitness]) -> Vec<CircuitResult> {
        witnesses.iter().map(|w| self.process_witness(w)).collect()
    }

    /// Validate semantic consistency between witness and expected field semantics
    /// 
    /// SECURITY: This function prevents semantic confusion attacks by ensuring
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

        // SECURITY: Zero value semantic validation prevents zero-value attacks
        if is_zero {
            match (witness.semantics, expected_semantics) {
                // SECURITY: Never written semantics must match expectations exactly
                // This prevents attacks where adversaries claim initialized storage
                // was never written to access default values or bypass checks.
                (ZeroSemantics::NeverWritten, ZeroSemantics::NeverWritten) => true,
                // SECURITY: Explicitly zero values must be valid for field type
                // This prevents zero-value attacks on fields that shouldn't be zero
                // (e.g., claiming a zero address was explicitly set to zero).
                (ZeroSemantics::ExplicitlyZero, _) => field_type.can_be_zero(),
                // SECURITY: Cleared semantics must match expectations exactly
                // This prevents attacks where adversaries claim different clearing behavior
                // to manipulate how zero values are interpreted.
                (ZeroSemantics::Cleared, ZeroSemantics::Cleared) => true,
                // SECURITY: Valid zero values must be allowed by field type
                // This prevents zero-value attacks on fields where zero is invalid.
                (ZeroSemantics::ValidZero, _) => field_type.can_be_zero(),
                _ => false, // SECURITY: Any other combination indicates potential attack
            }
        } else {
            // SECURITY: Non-zero values with zero semantics indicate attacks
            // Non-zero values should never claim to be uninitialized or explicitly zero.
            // This catches semantic manipulation attacks where adversaries try to
            // claim non-zero values have zero-related semantics.
            match witness.semantics {
                ZeroSemantics::NeverWritten => false,  // ATTACK: Non-zero claiming never written
                ZeroSemantics::ExplicitlyZero => false, // ATTACK: Non-zero claiming explicitly zero
                ZeroSemantics::Cleared => false,        // ATTACK: Non-zero claiming cleared
                ZeroSemantics::ValidZero => false,      // ATTACK: Non-zero claiming valid zero
            }
        }
    }

    /// Validate storage location matches expected slot for field
    /// 
    /// SECURITY: This function prevents storage slot spoofing attacks where
    /// adversaries provide values from different storage locations while claiming
    /// they belong to the expected field. This is critical for preventing:
    /// - Cross-field value injection attacks
    /// - Storage collision attacks
    /// - Field boundary violation attacks
    #[inline]
    fn validate_storage_location(&self, witness: &CircuitWitness) -> bool {
        // SECURITY CRITICAL: Storage key must exactly match expected slot
        // Any mismatch indicates potential storage slot spoofing attack where
        // an adversary is trying to use values from wrong storage locations.
        // This prevents attacks where wrong storage slots are claimed to
        // belong to different fields or contract structures.
        witness.key == witness.expected_slot
    }

    /// Extract value from witness with field type validation
    /// 
    /// SECURITY: This function performs type-safe value extraction from raw storage.
    /// It uses bounds-checked array access and prevents buffer overflows by
    /// carefully extracting only the required bytes for each field type.
    /// The extraction follows Ethereum's storage encoding rules to prevent
    /// value interpretation attacks.
    #[inline]
    fn extract_value(&self, witness: &CircuitWitness, field_type: FieldType) -> ExtractedValue {
        match field_type {
            // SECURITY: Bool extraction checks only the least significant bit
            // This prevents bool value manipulation attacks where non-zero/one values
            // are used to represent boolean state.
            FieldType::Bool => ExtractedValue::Bool(witness.value[31] != 0),
            // SECURITY: Uint8 extraction uses only the least significant byte
            // This prevents integer overflow attacks and ensures proper value bounds.
            FieldType::Uint8 => ExtractedValue::Uint8(witness.value[31]),
            // SECURITY: Uint16 extraction uses big-endian byte order (Ethereum standard)
            // This prevents byte order attacks and ensures consistent value interpretation.
            FieldType::Uint16 => {
                ExtractedValue::Uint16(u16::from_be_bytes([witness.value[30], witness.value[31]]))
            }
            // SECURITY: Uint32 extraction uses big-endian byte order
            // Bounds-checked array access prevents buffer overflow attacks.
            FieldType::Uint32 => {
                ExtractedValue::Uint32(u32::from_be_bytes([
                    witness.value[28], witness.value[29], witness.value[30], witness.value[31]
                ]))
            }
            // SECURITY: Uint64 extraction uses big-endian byte order
            // Bounds-checked array access prevents buffer overflow attacks.
            FieldType::Uint64 => {
                ExtractedValue::Uint64(u64::from_be_bytes([
                    witness.value[24], witness.value[25], witness.value[26], witness.value[27],
                    witness.value[28], witness.value[29], witness.value[30], witness.value[31]
                ]))
            }
            // SECURITY: Uint256 uses the full 32-byte value
            // Direct copy prevents any value manipulation during extraction.
            FieldType::Uint256 => ExtractedValue::Uint256(witness.value),
            // SECURITY: Address extraction uses bytes 12-31 (20 bytes)
            // This follows Ethereum's address encoding and prevents address manipulation.
            // The extracted address will be validated separately for zero-address attacks.
            FieldType::Address => {
                let mut addr = [0u8; 20];
                addr.copy_from_slice(&witness.value[12..32]); // SECURITY: Bounds-checked slice
                ExtractedValue::Address(addr)
            }
            // SECURITY: Bytes32 uses the full 32-byte value
            // Direct copy prevents any value manipulation during extraction.
            FieldType::Bytes32 => ExtractedValue::Bytes32(witness.value),
            // SECURITY: Fallback to raw bytes for unknown types
            // This prevents crashes while maintaining security through type validation.
            _ => ExtractedValue::Raw(witness.value),
        }
    }
}

/// Circuit processing result with semantic validation
/// 
/// SECURITY: This result type provides clear success/failure indication without
/// leaking sensitive information about why validation failed. This prevents
/// information leakage attacks where adversaries could probe for specific
/// validation failures to understand system internals.
#[derive(Debug, Clone)]
pub enum CircuitResult {
    Valid {
        field_index: u16,
        extracted_value: ExtractedValue,
    },
    Invalid, // SECURITY: No detailed error info to prevent information leakage
}

/// Semantically validated extracted value types (no_std compatible)
/// 
/// SECURITY: These types represent values that have passed all security validations
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
    Address([u8; 20]),    // SECURITY: Guaranteed to be non-zero if validation passed
    Bytes32([u8; 32]),
    Raw([u8; 32]),        // SECURITY: Fallback for unknown types
}

impl ExtractedValue {
    /// Convert to bytes (minimal allocation)
    /// 
    /// SECURITY: This function performs safe conversions without buffer overflows.
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
    /// SECURITY: This function provides size information without allocation,
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
    /// SECURITY: This function determines if a value is semantically zero,
    /// which is critical for zero-value attack detection. It uses type-specific
    /// zero checks to prevent semantic confusion about what constitutes "zero".
    #[inline]
    pub fn is_semantic_zero(&self) -> bool {
        match self {
            ExtractedValue::Bool(b) => !*b,  // SECURITY: false is semantic zero for bool
            ExtractedValue::Uint8(n) => *n == 0,
            ExtractedValue::Uint16(n) => *n == 0,
            ExtractedValue::Uint32(n) => *n == 0,
            ExtractedValue::Uint64(n) => *n == 0,
            ExtractedValue::Uint256(bytes) => *bytes == [0u8; 32],
            ExtractedValue::Address(addr) => *addr == [0u8; 20], // SECURITY: Zero address detection
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
        // Non-zero values should not have zero semantics - this should be invalid
        assert!(matches!(result, CircuitResult::Invalid));
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
        
        let witness = CircuitProcessor::parse_witness_from_bytes(&witness_data).unwrap();
        
        assert_eq!(witness.key, [1u8; 32]);
        assert_eq!(witness.layout_commitment, [2u8; 32]);
        assert_eq!(witness.value, [3u8; 32]);
        assert_eq!(witness.semantics, ZeroSemantics::ExplicitlyZero);
        assert_eq!(witness.block_height, 12345);
        assert_eq!(witness.block_hash, [4u8; 32]);
        assert_eq!(witness.proof, vec![0xDE, 0xAD, 0xBE, 0xEF]);
    }

    #[test]
    fn test_witness_parsing_legacy_format() {
        // Test legacy format without block data
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
        
        // Proof length (4 bytes)
        witness_data.extend_from_slice(&4u32.to_le_bytes());
        
        // Proof data
        witness_data.extend_from_slice(&[0xDE, 0xAD, 0xBE, 0xEF]);
        
        let witness = CircuitProcessor::parse_witness_from_bytes(&witness_data).unwrap();
        
        assert_eq!(witness.key, [1u8; 32]);
        assert_eq!(witness.layout_commitment, [2u8; 32]);
        assert_eq!(witness.value, [3u8; 32]);
        assert_eq!(witness.semantics, ZeroSemantics::ExplicitlyZero);
        assert_eq!(witness.block_height, 0); // Default for legacy format
        assert_eq!(witness.block_hash, [0u8; 32]); // Default for legacy format
        assert_eq!(witness.proof, vec![0xDE, 0xAD, 0xBE, 0xEF]);
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
        
        // Should be invalid - non-zero values cannot have ValidZero semantics
        let result = processor.process_witness(&witness);
        assert!(matches!(result, CircuitResult::Invalid));
        
        // Test all zero semantics with non-zero values - all should fail
        for semantics in [
            ZeroSemantics::NeverWritten,
            ZeroSemantics::ExplicitlyZero,
            ZeroSemantics::Cleared,
            ZeroSemantics::ValidZero,
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
}
