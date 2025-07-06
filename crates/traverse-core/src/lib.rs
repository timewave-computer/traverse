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
//!
//! # Usage
//!
//! For circuit environments (no_std):
//! ```toml
//! [dependencies]
//! traverse-core = { version = "0.1", default-features = false }
//! ```
//!
//! For std environments (CLI, tools):
//! ```toml
//! [dependencies]
//! traverse-core = { version = "0.1", features = ["std", "serde_json"] }
//! ```

#![no_std]

extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

// Module declarations
pub mod error;
pub mod key;
pub mod layout;
pub mod semantic;
pub mod traits;

// Re-export all public types and traits for convenience
pub use error::TraverseError;
pub use key::{Key, SemanticStorageProof, StaticKeyPath, StorageSemantics, ZeroSemantics};
pub use layout::{LayoutInfo, StorageEntry, TypeInfo};
pub use semantic::{ResolvedSemantics, SemanticResolver, SemanticSource, StorageSemanticsExt};
pub use traits::KeyResolver;

#[cfg(feature = "std")]
pub use traits::{LayoutCompiler, ProofFetcher};
