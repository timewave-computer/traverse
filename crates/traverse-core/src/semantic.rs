//! Dynamic semantic resolution for storage proofs
//!
//! This module provides logic to dynamically resolve the final semantic meaning
//! of zero values by combining declared semantics with event-validated semantics.

use crate::{StorageSemantics, ZeroSemantics};
use alloc::{format, string::String};
use serde::{Deserialize, Serialize};

/// Result of dynamic semantic resolution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedSemantics {
    /// Final semantic meaning to use
    pub final_semantics: ZeroSemantics,
    /// Source of the final semantics
    pub source: SemanticSource,
    /// Whether there were conflicts during resolution
    pub has_conflicts: bool,
    /// Description of any conflicts found
    pub conflict_description: Option<String>,
}

/// Source of semantic information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SemanticSource {
    /// Used declared semantics (no validation available)
    Declared,
    /// Used event-validated semantics
    EventValidated,
    /// Used declared semantics despite conflicts (developer override)
    DeclaredOverride,
}

/// Dynamic semantic resolver
pub struct SemanticResolver;

impl SemanticResolver {
    /// Resolve final semantics using validation results
    ///
    /// # Arguments
    ///
    /// * `storage_semantics` - The storage semantics with declared and validated values
    /// * `allow_override` - Whether to allow developer override of conflicts
    ///
    /// # Returns
    ///
    /// Resolved semantics with source information
    pub fn resolve(
        storage_semantics: &StorageSemantics,
        allow_override: bool,
    ) -> ResolvedSemantics {
        match storage_semantics.validated_semantics {
            Some(validated) => {
                // We have validated semantics from events
                if validated == storage_semantics.declared_semantics {
                    // No conflict - validated matches declared
                    ResolvedSemantics {
                        final_semantics: validated,
                        source: SemanticSource::EventValidated,
                        has_conflicts: false,
                        conflict_description: None,
                    }
                } else {
                    // Conflict between declared and validated
                    if allow_override {
                        // Allow developer to override validation
                        ResolvedSemantics {
                            final_semantics: storage_semantics.declared_semantics,
                            source: SemanticSource::DeclaredOverride,
                            has_conflicts: true,
                            conflict_description: Some(format!(
                                "Declared {:?} conflicts with validated {:?}, using declared (override)",
                                storage_semantics.declared_semantics, validated
                            )),
                        }
                    } else {
                        // Use validated semantics by default
                        ResolvedSemantics {
                            final_semantics: validated,
                            source: SemanticSource::EventValidated,
                            has_conflicts: true,
                            conflict_description: Some(format!(
                                "Declared {:?} conflicts with validated {:?}, using validated",
                                storage_semantics.declared_semantics, validated
                            )),
                        }
                    }
                }
            }
            None => {
                // No validated semantics available - use declared
                ResolvedSemantics {
                    final_semantics: storage_semantics.declared_semantics,
                    source: SemanticSource::Declared,
                    has_conflicts: false,
                    conflict_description: None,
                }
            }
        }
    }

    /// Resolve semantics with validation preference
    ///
    /// Always prefer validated semantics over declared when available,
    /// regardless of conflicts. This is the recommended approach.
    pub fn resolve_prefer_validated(storage_semantics: &StorageSemantics) -> ResolvedSemantics {
        Self::resolve(storage_semantics, false)
    }

    /// Resolve semantics with developer override
    ///
    /// Allow developers to override validation conflicts.
    /// Use this when developers have better context than event analysis.
    pub fn resolve_allow_override(storage_semantics: &StorageSemantics) -> ResolvedSemantics {
        Self::resolve(storage_semantics, true)
    }

    /// Check if semantic resolution would produce conflicts
    pub fn has_conflicts(storage_semantics: &StorageSemantics) -> bool {
        if let Some(validated) = storage_semantics.validated_semantics {
            validated != storage_semantics.declared_semantics
        } else {
            false
        }
    }

    /// Get conflict description if any
    pub fn get_conflict_description(storage_semantics: &StorageSemantics) -> Option<String> {
        if let Some(validated) = storage_semantics.validated_semantics {
            if validated != storage_semantics.declared_semantics {
                Some(format!(
                    "Semantic conflict: declared {:?} vs validated {:?}",
                    storage_semantics.declared_semantics, validated
                ))
            } else {
                None
            }
        } else {
            None
        }
    }
}

/// Extension trait for StorageSemantics to add resolution methods
pub trait StorageSemanticsExt {
    /// Resolve final semantics preferring validation
    fn resolve(&self) -> ResolvedSemantics;

    /// Resolve final semantics allowing developer override
    fn resolve_with_override(&self) -> ResolvedSemantics;

    /// Check for conflicts
    fn has_conflicts(&self) -> bool;

    /// Get the final semantic meaning to use
    fn final_semantics(&self) -> ZeroSemantics;
}

impl StorageSemanticsExt for StorageSemantics {
    fn resolve(&self) -> ResolvedSemantics {
        SemanticResolver::resolve_prefer_validated(self)
    }

    fn resolve_with_override(&self) -> ResolvedSemantics {
        SemanticResolver::resolve_allow_override(self)
    }

    fn has_conflicts(&self) -> bool {
        SemanticResolver::has_conflicts(self)
    }

    fn final_semantics(&self) -> ZeroSemantics {
        self.resolve().final_semantics
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_no_validation() {
        let semantics = StorageSemantics::new(ZeroSemantics::ExplicitlyZero);
        let resolved = semantics.resolve();

        assert_eq!(resolved.final_semantics, ZeroSemantics::ExplicitlyZero);
        assert_eq!(resolved.source, SemanticSource::Declared);
        assert!(!resolved.has_conflicts);
    }

    #[test]
    fn test_resolve_validation_matches() {
        let mut semantics = StorageSemantics::new(ZeroSemantics::ExplicitlyZero);
        semantics.validated_semantics = Some(ZeroSemantics::ExplicitlyZero);

        let resolved = semantics.resolve();

        assert_eq!(resolved.final_semantics, ZeroSemantics::ExplicitlyZero);
        assert_eq!(resolved.source, SemanticSource::EventValidated);
        assert!(!resolved.has_conflicts);
    }

    #[test]
    fn test_resolve_validation_conflict() {
        let mut semantics = StorageSemantics::new(ZeroSemantics::NeverWritten);
        semantics.validated_semantics = Some(ZeroSemantics::ExplicitlyZero);

        let resolved = semantics.resolve();

        assert_eq!(resolved.final_semantics, ZeroSemantics::ExplicitlyZero);
        assert_eq!(resolved.source, SemanticSource::EventValidated);
        assert!(resolved.has_conflicts);
    }

    #[test]
    fn test_resolve_with_override() {
        let mut semantics = StorageSemantics::new(ZeroSemantics::NeverWritten);
        semantics.validated_semantics = Some(ZeroSemantics::ExplicitlyZero);

        let resolved = semantics.resolve_with_override();

        assert_eq!(resolved.final_semantics, ZeroSemantics::NeverWritten);
        assert_eq!(resolved.source, SemanticSource::DeclaredOverride);
        assert!(resolved.has_conflicts);
    }

    #[test]
    fn test_has_conflicts() {
        let mut semantics = StorageSemantics::new(ZeroSemantics::NeverWritten);
        assert!(!semantics.has_conflicts());

        semantics.validated_semantics = Some(ZeroSemantics::NeverWritten);
        assert!(!semantics.has_conflicts());

        semantics.validated_semantics = Some(ZeroSemantics::ExplicitlyZero);
        assert!(semantics.has_conflicts());
    }

    #[test]
    fn test_final_semantics() {
        let mut semantics = StorageSemantics::new(ZeroSemantics::NeverWritten);
        assert_eq!(semantics.final_semantics(), ZeroSemantics::NeverWritten);

        semantics.validated_semantics = Some(ZeroSemantics::ExplicitlyZero);
        assert_eq!(semantics.final_semantics(), ZeroSemantics::ExplicitlyZero);
    }
}
