//! Trait definitions for traverse-core
//!
//! This module contains the core traits that define the interfaces for
//! different operations in the traverse system: layout compilation,
//! key resolution, and proof fetching.

#[allow(unused_imports)]
use crate::{LayoutInfo, SemanticStorageProof, StaticKeyPath, TraverseError, ZeroSemantics};
use alloc::vec::Vec;

/// Trait for compiling contract layouts from chain-specific sources
///
/// This trait abstracts the process of converting chain-specific contract
/// definitions (like Solidity ABIs) into the canonical `LayoutInfo` format.
///
/// **Note**: This trait is only available with the `std` feature since it
/// involves file I/O operations. Circuit code typically uses pre-compiled
/// `LayoutInfo` structs.
///
/// # Examples
///
/// ```rust,ignore
/// use traverse_core::LayoutCompiler;
/// use std::path::Path;
///
/// struct MyLayoutCompiler;
///
/// impl LayoutCompiler for MyLayoutCompiler {
///     fn compile_layout(&self, abi_path: &Path) -> Result<LayoutInfo, TraverseError> {
///         // Implementation specific to your blockchain
///         todo!()
///     }
/// }
/// ```
#[cfg(feature = "std")]
pub trait LayoutCompiler {
    /// Compile layout information from an ABI or contract source
    ///
    /// # Arguments
    ///
    /// * `abi_path` - Path to the contract ABI or source file
    ///
    /// # Returns
    ///
    /// * `Ok(LayoutInfo)` - Successfully compiled layout
    /// * `Err(TraverseError)` - Compilation failed with error details
    fn compile_layout(&self, abi_path: &std::path::Path) -> Result<LayoutInfo, TraverseError>;
}

/// Trait for resolving human-readable queries into storage keys
///
/// This trait handles the core functionality of converting storage queries
/// (like `balances[0x123...]` or `owner`) into deterministic storage keys
/// that can be used for blockchain state queries.
///
/// Implementations should support:
/// - Simple field access (`owner`, `totalSupply`)
/// - Mapping access (`balances[address]`, `allowances[owner][spender]`)
/// - Array access (`items[index]`)
/// - Struct field access (`user.balance`)
///
/// # Circuit Compatibility
///
/// This trait is available in both std and no_std environments, making it
/// suitable for use in ZK circuits where dynamic path resolution may be needed.
pub trait KeyResolver {
    /// Resolve a query string into a static key path
    ///
    /// # Arguments
    ///
    /// * `layout` - The contract layout information
    /// * `query` - The storage query string (e.g., "balances[0x123...]")
    ///
    /// # Returns
    ///
    /// * `Ok(StaticKeyPath)` - Successfully resolved path
    /// * `Err(TraverseError)` - Resolution failed with error details
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let path = resolver.resolve(&layout, "balances[0x742d35Cc6634C0532925a3b8D97C2e0D8b2D9C]")?;
    /// println!("Storage key: {:?}", path.key);
    /// ```
    fn resolve(&self, layout: &LayoutInfo, query: &str) -> Result<StaticKeyPath, TraverseError>;

    /// Resolve all possible paths from a layout
    ///
    /// Generates storage paths for all simple fields in the layout.
    /// Mappings and arrays are not included since they require specific keys/indices.
    ///
    /// # Arguments
    ///
    /// * `layout` - The contract layout information
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<StaticKeyPath>)` - All resolvable paths
    /// * `Err(TraverseError)` - Resolution failed with error details
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let all_paths = resolver.resolve_all(&layout)?;
    /// for path in all_paths {
    ///     println!("Field: {}, Key: {:?}", path.name, path.key);
    /// }
    /// ```
    fn resolve_all(&self, layout: &LayoutInfo) -> Result<Vec<StaticKeyPath>, TraverseError>;
}

/// Trait for fetching storage proofs from blockchain nodes
///
/// This trait abstracts the process of fetching storage proofs from different
/// blockchain RPC endpoints. It handles the network communication and proof
/// formatting needed for ZK verification.
///
/// **Note**: This trait is only available with the `std` feature since it
/// involves network I/O operations. Circuit code typically receives
/// `SemanticStorageProof` structs directly.
///
/// # Examples
///
/// ```rust,ignore
/// use traverse_core::ProofFetcher;
///
/// struct MyProofFetcher {
///     rpc_url: String,
///     contract_address: String,
/// }
///
/// impl ProofFetcher for MyProofFetcher {
///     fn fetch(&self, key: &[u8; 32]) -> Result<CoprocessorQueryPayload, TraverseError> {
///         // Implementation specific to your blockchain
///         todo!()
///     }
/// }
/// ```
#[cfg(feature = "std")]
pub trait ProofFetcher {
    /// Fetch a storage proof for the given key
    ///
    /// # Arguments
    ///
    /// * `key` - The storage key to fetch a proof for
    ///
    /// # Returns
    ///
    /// * `Ok(SemanticStorageProof)` - Successfully fetched proof with semantics
    /// * `Err(TraverseError)` - Fetch failed with error details
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let key = [0u8; 32]; // Your storage key
    /// let zero_meaning = ZeroSemantics::ExplicitlyZero;
    /// let proof = fetcher.fetch(&key, zero_meaning)?;
    /// // Submit proof to ZK coprocessor
    /// ```
    fn fetch(
        &self,
        key: &[u8; 32],
        zero_semantics: ZeroSemantics,
    ) -> Result<SemanticStorageProof, TraverseError>;
}
