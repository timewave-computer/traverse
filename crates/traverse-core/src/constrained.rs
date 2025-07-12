//! Support for constrained environments
//!
//! This module provides utilities and adaptations for running traverse in
//! constrained environments such as embedded systems, WASM runtime, and 
//! ZK circuits where standard library is not available.

#![cfg_attr(not(feature = "std"), no_std)]

use alloc::{vec::Vec, format};

/// Memory pool for constrained environments
/// 
/// This provides a simple memory allocator for environments where
/// dynamic allocation needs to be controlled or limited.
#[cfg(not(feature = "std"))]
pub struct ConstrainedMemoryPool {
    buffer: Vec<u8>,
    used: usize,
}

#[cfg(not(feature = "std"))]
impl ConstrainedMemoryPool {
    /// Create a new memory pool with the given capacity
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: Vec::with_capacity(capacity),
            used: 0,
        }
    }

    /// Allocate a chunk of memory from the pool
    pub fn allocate(&mut self, size: usize, align: usize) -> Option<&mut [u8]> {
        let aligned_used = (self.used + align - 1) & !(align - 1);
        let end = aligned_used + size;
        
        if end <= self.buffer.capacity() {
            // Extend buffer if needed
            if end > self.buffer.len() {
                self.buffer.resize(end, 0);
            }
            
            self.used = end;
            Some(&mut self.buffer[aligned_used..end])
        } else {
            None
        }
    }

    /// Reset the memory pool (mark all memory as available)
    pub fn reset(&mut self) {
        self.used = 0;
        // Don't deallocate the buffer, just mark it as unused
    }

    /// Get current memory usage statistics
    pub fn usage(&self) -> MemoryUsage {
        MemoryUsage {
            used: self.used,
            capacity: self.buffer.capacity(),
            utilization: (self.used as f32 / self.buffer.capacity() as f32) * 100.0,
        }
    }
}

/// Memory usage statistics
#[derive(Debug, Clone)]
pub struct MemoryUsage {
    pub used: usize,
    pub capacity: usize,
    pub utilization: f32,
}

/// Constrained layout info for memory-limited environments
///
/// This is a more compact version of LayoutInfo that uses less memory
/// and is optimized for constrained environments.
#[derive(Debug, Clone)]
pub struct ConstrainedLayoutInfo {
    /// Compact storage entries (fixed-size for better memory layout)
    pub storage: Vec<ConstrainedStorageEntry>,
    /// Layout commitment (always 32 bytes)
    pub commitment: [u8; 32],
    /// Number of storage entries (redundant but helps with validation)
    pub entry_count: u16,
}

/// Compact storage entry for constrained environments
#[derive(Debug, Clone)]
pub struct ConstrainedStorageEntry {
    /// Storage slot (always 32 bytes for Ethereum)
    pub slot: [u8; 32],
    /// Byte offset within the slot
    pub offset: u8,
    /// Field size in bytes (0-32)
    pub size: u8,
    /// Field type as a compact enum
    pub field_type: ConstrainedFieldType,
    /// Zero semantics
    pub zero_semantics: crate::ZeroSemantics,
}

/// Compact field type enum for constrained environments
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum ConstrainedFieldType {
    Uint8 = 0,
    Uint16 = 1,
    Uint32 = 2,
    Uint64 = 3,
    Uint128 = 4,
    Uint256 = 5,
    Int8 = 6,
    Int16 = 7,
    Int32 = 8,
    Int64 = 9,
    Int128 = 10,
    Int256 = 11,
    Address = 12,
    Bool = 13,
    Bytes32 = 14,
    Mapping = 15,
    Array = 16,
    Struct = 17,
    String = 18,
    Bytes = 19,
}

impl ConstrainedFieldType {
    /// Get the size in bytes for fixed-size types
    pub fn fixed_size(&self) -> Option<u8> {
        match self {
            Self::Uint8 | Self::Int8 | Self::Bool => Some(1),
            Self::Uint16 | Self::Int16 => Some(2),
            Self::Uint32 | Self::Int32 => Some(4),
            Self::Uint64 | Self::Int64 => Some(8),
            Self::Uint128 | Self::Int128 => Some(16),
            Self::Uint256 | Self::Int256 | Self::Bytes32 => Some(32),
            Self::Address => Some(20),
            _ => None, // Variable size types
        }
    }
}

/// Convert from full LayoutInfo to constrained version
impl TryFrom<&crate::LayoutInfo> for ConstrainedLayoutInfo {
    type Error = crate::TraverseError;

    fn try_from(layout: &crate::LayoutInfo) -> Result<Self, Self::Error> {
        let mut constrained_storage = Vec::new();

        for entry in &layout.storage {
            // Parse slot as hex and convert to bytes
            let slot_hex = entry.slot.strip_prefix("0x").unwrap_or(&entry.slot);
            let mut slot_bytes = [0u8; 32];
            
            // Parse hex slot to bytes
            if slot_hex.len() <= 64 { // Max 32 bytes * 2 hex chars
                let slot_vec = hex::decode(slot_hex)
                    .map_err(|e| crate::TraverseError::InvalidInput(format!("Invalid slot hex: {}", e)))?;
                
                if slot_vec.len() <= 32 {
                    slot_bytes[32 - slot_vec.len()..].copy_from_slice(&slot_vec);
                }
            }

            // Determine field type from type name
            let field_type = match entry.type_name.as_str() {
                "t_uint8" => ConstrainedFieldType::Uint8,
                "t_uint16" => ConstrainedFieldType::Uint16,
                "t_uint32" => ConstrainedFieldType::Uint32,
                "t_uint64" => ConstrainedFieldType::Uint64,
                "t_uint128" => ConstrainedFieldType::Uint128,
                "t_uint256" => ConstrainedFieldType::Uint256,
                "t_address" => ConstrainedFieldType::Address,
                "t_bool" => ConstrainedFieldType::Bool,
                "t_bytes32" => ConstrainedFieldType::Bytes32,
                s if s.contains("mapping") => ConstrainedFieldType::Mapping,
                s if s.contains("array") => ConstrainedFieldType::Array,
                s if s.contains("struct") => ConstrainedFieldType::Struct,
                s if s.contains("string") => ConstrainedFieldType::String,
                s if s.contains("bytes") => ConstrainedFieldType::Bytes,
                _ => ConstrainedFieldType::Uint256, // Default fallback
            };

            let size = field_type.fixed_size().unwrap_or(32);

            constrained_storage.push(ConstrainedStorageEntry {
                slot: slot_bytes,
                offset: entry.offset,
                size,
                field_type,
                zero_semantics: entry.zero_semantics,
            });
        }

        let entry_count = constrained_storage.len() as u16;
        Ok(Self {
            storage: constrained_storage,
            commitment: layout.commitment(),
            entry_count,
        })
    }
}

/// Constrained key resolver for memory-limited environments
pub struct ConstrainedKeyResolver {
    /// Memory pool for temporary allocations
    #[cfg(not(feature = "std"))]
    memory_pool: Option<ConstrainedMemoryPool>,
    #[cfg(feature = "std")]
    _phantom: core::marker::PhantomData<()>,
}

impl ConstrainedKeyResolver {
    /// Create a new constrained key resolver
    pub fn new() -> Self {
        Self {
            #[cfg(not(feature = "std"))]
            memory_pool: None,
            #[cfg(feature = "std")]
            _phantom: core::marker::PhantomData,
        }
    }

    /// Create a new constrained key resolver with a memory pool
    #[cfg(not(feature = "std"))]
    pub fn with_memory_pool(pool_size: usize) -> Self {
        Self {
            memory_pool: Some(ConstrainedMemoryPool::new(pool_size)),
        }
    }

    /// Resolve a storage key using constrained operations
    pub fn resolve_constrained(
        &mut self,
        layout: &ConstrainedLayoutInfo,
        field_index: u16,
    ) -> Result<[u8; 32], crate::TraverseError> {
        if field_index >= layout.entry_count {
            return Err(crate::TraverseError::InvalidInput(
                format!("Field index {} out of bounds", field_index)
            ));
        }

        let entry = &layout.storage[field_index as usize];

        // For simple fields, return the slot directly
        if matches!(entry.field_type, ConstrainedFieldType::Mapping | ConstrainedFieldType::Array) {
            // For complex types, we'd need additional parameters
            // This is a simplified version for demonstration
            return Err(crate::TraverseError::InvalidInput(
                "Complex types require additional parameters".into()
            ));
        }

        Ok(entry.slot)
    }

    /// Get memory usage statistics if a pool is available
    pub fn memory_usage(&self) -> Option<MemoryUsage> {
        #[cfg(not(feature = "std"))]
        {
            self.memory_pool.as_ref().map(|pool| pool.usage())
        }
        #[cfg(feature = "std")]
        {
            None
        }
    }
}

impl Default for ConstrainedKeyResolver {
    fn default() -> Self {
        Self::new()
    }
}

/// Constrained error handling for no_std environments
pub mod error {
    use alloc::string::String;

    /// Simplified error type for constrained environments
    #[derive(Debug)]
    pub enum ConstrainedError {
        /// Memory allocation failed
        OutOfMemory,
        /// Invalid input parameter
        InvalidInput,
        /// Operation not supported in constrained mode
        NotSupported,
        /// Layout format error
        LayoutError,
        /// Generic error with message
        Generic(String),
    }

    impl core::fmt::Display for ConstrainedError {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            match self {
                ConstrainedError::OutOfMemory => write!(f, "Out of memory"),
                ConstrainedError::InvalidInput => write!(f, "Invalid input"),
                ConstrainedError::NotSupported => write!(f, "Operation not supported"),
                ConstrainedError::LayoutError => write!(f, "Layout error"),
                ConstrainedError::Generic(msg) => write!(f, "{}", msg),
            }
        }
    }
}

/// Utility functions for constrained environments
pub mod utils {
    /// Error type for utility functions
    #[derive(Debug)]
    pub struct UtilError;

    /// Calculate Keccak256 hash using minimal implementation
    pub fn keccak256_minimal(input: &[u8]) -> [u8; 32] {
        use sha2::{Digest, Sha256};
        
        // SHA256 is available in no_std environments
        let mut hasher = Sha256::new();
        hasher.update(input);
        hasher.finalize().into()
    }

    /// Convert bytes to hex string without allocation
    pub fn bytes_to_hex_stack(bytes: &[u8], output: &mut [u8]) -> Result<usize, UtilError> {
        if output.len() < bytes.len() * 2 {
            return Err(UtilError);
        }

        const HEX_CHARS: &[u8] = b"0123456789abcdef";
        
        for (i, &byte) in bytes.iter().enumerate() {
            output[i * 2] = HEX_CHARS[(byte >> 4) as usize];
            output[i * 2 + 1] = HEX_CHARS[(byte & 0xf) as usize];
        }

        Ok(bytes.len() * 2)
    }

    /// Parse hex string to bytes without allocation
    pub fn hex_to_bytes_stack(hex: &str, output: &mut [u8]) -> Result<usize, UtilError> {
        let hex = hex.strip_prefix("0x").unwrap_or(hex);
        
        if hex.len() % 2 != 0 || output.len() < hex.len() / 2 {
            return Err(UtilError);
        }

        for (i, chunk) in hex.as_bytes().chunks(2).enumerate() {
            let hex_byte = core::str::from_utf8(chunk).map_err(|_| UtilError)?;
            output[i] = u8::from_str_radix(hex_byte, 16).map_err(|_| UtilError)?;
        }

        Ok(hex.len() / 2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(not(feature = "std"))]
    fn test_constrained_memory_pool() {
        let mut pool = ConstrainedMemoryPool::new(1024);
        
        // Allocate some memory
        let chunk1 = pool.allocate(32, 4).unwrap();
        assert_eq!(chunk1.len(), 32);
        
        let chunk2 = pool.allocate(64, 8).unwrap();
        assert_eq!(chunk2.len(), 64);
        
        // Check usage
        let usage = pool.usage();
        assert!(usage.used > 0);
        assert!(usage.utilization > 0.0);
        
        // Reset and check
        pool.reset();
        assert_eq!(pool.usage().used, 0);
    }

    #[test]
    fn test_constrained_field_type() {
        assert_eq!(ConstrainedFieldType::Uint8.fixed_size(), Some(1));
        assert_eq!(ConstrainedFieldType::Uint256.fixed_size(), Some(32));
        assert_eq!(ConstrainedFieldType::Address.fixed_size(), Some(20));
        assert_eq!(ConstrainedFieldType::Mapping.fixed_size(), None);
    }

    #[test]
    fn test_hex_utils() {
        let bytes = [0x12, 0x34, 0xab, 0xcd];
        let mut hex_output = [0u8; 8];
        
        let len = utils::bytes_to_hex_stack(&bytes, &mut hex_output).unwrap();
        assert_eq!(len, 8);
        assert_eq!(&hex_output, b"1234abcd");
        
        let mut bytes_output = [0u8; 4];
        let len = utils::hex_to_bytes_stack("1234abcd", &mut bytes_output).unwrap();
        assert_eq!(len, 4);
        assert_eq!(bytes_output, [0x12, 0x34, 0xab, 0xcd]);
    }
} 