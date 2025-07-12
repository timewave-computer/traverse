//! Core types and traits for chain-independent ZK storage path generation
//!
//! This crate provides the foundational abstractions for generating ZK-compatible
//! contract storage proofs across different blockchain architectures.
//!
//! # Features
//!
//! - **no_std compatible**: Can be used in circuit environments without std
//! - **Chain-independent**: Extensible to multiple blockchain architectures
//! - **Deterministic**: Layout commitments ensure reproducible behavior
//! - **Circuit-ready**: Optimized for RISC-V compilation and ZK circuits
//! - **Constrained environments**: Memory-efficient data structures and algorithms
//!
//! # Usage
//!
//! For circuit environments (no_std):
//! ```toml
//! [dependencies]
//! traverse-core = { version = "0.1", default-features = false, features = ["no-std"] }
//! ```
//!
//! For constrained environments (embedded, ZK circuits):
//! ```toml
//! [dependencies]
//! traverse-core = { version = "0.1", default-features = false, features = ["constrained"] }
//! ```
//!
//! For std environments (CLI, tools):
//! ```toml
//! [dependencies]
//! traverse-core = { version = "0.1", features = ["std", "serde_json"] }
//! ```

#![no_std]
#![cfg_attr(not(feature = "std"), no_main)]

extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

// Module declarations
pub mod error;
pub mod key;
pub mod layout;
pub mod semantic;
pub mod traits;

// Constrained environment support
#[cfg(any(feature = "no-std", feature = "constrained", feature = "embedded"))]
pub mod constrained;

// Re-export all public types and traits for convenience
pub use error::TraverseError;
pub use key::{Key, SemanticStorageProof, StaticKeyPath, StorageSemantics, ZeroSemantics};
pub use layout::{LayoutInfo, StorageEntry, TypeInfo};
pub use semantic::{ResolvedSemantics, SemanticResolver, SemanticSource, StorageSemanticsExt};
pub use traits::KeyResolver;

#[cfg(feature = "std")]
pub use traits::{LayoutCompiler, ProofFetcher};

// Re-export constrained types when available
#[cfg(any(feature = "no-std", feature = "constrained", feature = "embedded"))]
pub use constrained::{
    ConstrainedLayoutInfo, ConstrainedStorageEntry, ConstrainedFieldType,
    ConstrainedKeyResolver, MemoryUsage,
};

#[cfg(all(not(feature = "std"), any(feature = "no-std", feature = "constrained")))]
pub use constrained::ConstrainedMemoryPool;

pub mod prelude {
    //! Common functionality available in all environments
    //! Common imports for traverse-core
    
    pub use crate::{
        TraverseError, Key, ZeroSemantics, StorageSemantics,
        LayoutInfo, StorageEntry, KeyResolver,
    };
    
    #[cfg(feature = "std")]
    pub use crate::{LayoutCompiler, ProofFetcher};
    
    #[cfg(any(feature = "no-std", feature = "constrained", feature = "embedded"))]
    pub use crate::{ConstrainedLayoutInfo, ConstrainedKeyResolver};
}
