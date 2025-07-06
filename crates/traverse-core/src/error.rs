//! Error types for traverse-core
//!
//! This module defines the error types that can occur during layout compilation,
//! path resolution, and proof generation operations.

#[cfg(feature = "std")]
use alloc::boxed::Box;
use alloc::string::String;

/// Errors that can occur during layout compilation and path resolution
///
/// This enum covers all possible error conditions that can arise when working
/// with storage layouts and path resolution across different blockchain architectures.
#[derive(Debug)]
pub enum TraverseError {
    /// Layout compilation failed with the given error message
    LayoutCompilation(String),
    /// Path resolution failed with the given error message  
    PathResolution(String),
    /// Proof generation failed with the given error message
    ProofGeneration(String),
    /// Invalid query format with details about what went wrong
    InvalidQuery(String),
    /// Invalid storage layout (conflicts, overlaps, etc.)
    InvalidLayout(String),
    /// Network error for HTTP requests (e.g., API calls)
    Network(String),
    /// IO error (only available with std feature)
    #[cfg(feature = "std")]
    Io(Box<dyn std::error::Error + Send + Sync>),
    /// Serialization error with error details
    Serialization(String),
}

impl core::fmt::Display for TraverseError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            TraverseError::LayoutCompilation(msg) => {
                write!(f, "Layout compilation failed: {}", msg)
            }
            TraverseError::PathResolution(msg) => write!(f, "Path resolution failed: {}", msg),
            TraverseError::ProofGeneration(msg) => write!(f, "Proof generation failed: {}", msg),
            TraverseError::InvalidQuery(msg) => write!(f, "Invalid query format: {}", msg),
            TraverseError::InvalidLayout(msg) => write!(f, "Invalid storage layout: {}", msg),
            TraverseError::Network(msg) => write!(f, "Network error: {}", msg),
            #[cfg(feature = "std")]
            TraverseError::Io(err) => write!(f, "IO error: {}", err),
            TraverseError::Serialization(msg) => write!(f, "Serialization error: {}", msg),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for TraverseError {}

#[cfg(feature = "std")]
impl From<std::io::Error> for TraverseError {
    fn from(err: std::io::Error) -> Self {
        TraverseError::Io(Box::new(err))
    }
}

#[cfg(feature = "serde_json")]
impl From<serde_json::Error> for TraverseError {
    fn from(err: serde_json::Error) -> Self {
        TraverseError::Serialization(alloc::format!("{}", err))
    }
}
