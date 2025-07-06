//! Indexer service integration for event-based semantic validation
//!
//! This module provides interfaces to external blockchain indexing services
//! to validate semantic declarations against actual blockchain state.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use traverse_core::{TraverseError, ZeroSemantics};

/// Event data returned by indexer services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageEvent {
    /// Block number where the event occurred
    pub block_number: u64,
    /// Transaction hash
    pub transaction_hash: String,
    /// Contract address
    pub contract_address: String,
    /// Storage slot affected
    pub storage_slot: String,
    /// Previous value (if available)
    pub previous_value: Option<String>,
    /// New value
    pub new_value: String,
    /// Event type (write, clear, etc.)
    pub event_type: StorageEventType,
}

/// Types of storage events
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum StorageEventType {
    /// First write to a slot
    FirstWrite,
    /// Update to existing value
    Update,
    /// Value cleared to zero
    Cleared,
    /// Value set to zero (but not cleared)
    SetToZero,
}

/// Result of semantic validation against events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Whether the declared semantics match the event history
    pub is_valid: bool,
    /// Detected semantic meaning based on events
    pub detected_semantics: ZeroSemantics,
    /// Conflicting events if any
    pub conflicts: Vec<SemanticConflict>,
}

/// Represents a conflict between declared and detected semantics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticConflict {
    /// Declared semantic meaning
    pub declared: ZeroSemantics,
    /// Detected semantic meaning from events
    pub detected: ZeroSemantics,
    /// Evidence supporting the conflict
    pub evidence: Vec<StorageEvent>,
    /// Description of the conflict
    pub description: String,
}

/// Trait for blockchain indexer services
pub trait IndexerService {
    /// Get storage events for a specific slot
    ///
    /// # Arguments
    ///
    /// * `contract_address` - Contract address to query
    /// * `storage_slot` - Storage slot to get events for
    /// * `from_block` - Starting block number (None = from genesis)
    /// * `to_block` - Ending block number (None = latest)
    ///
    /// # Returns
    ///
    /// Vector of storage events affecting the slot
    fn get_storage_events(
        &self,
        contract_address: &str,
        storage_slot: &str,
        from_block: Option<u64>,
        to_block: Option<u64>,
    ) -> impl std::future::Future<Output = Result<Vec<StorageEvent>, TraverseError>> + Send;

    /// Get the current value of a storage slot
    fn get_current_value(
        &self,
        contract_address: &str,
        storage_slot: &str,
    ) -> impl std::future::Future<Output = Result<String, TraverseError>> + Send;

    /// Get service name for identification
    fn service_name(&self) -> &str;
}

/// Validates semantic declarations against blockchain events
pub struct SemanticValidator<T: IndexerService> {
    indexer: T,
}

impl<T: IndexerService> SemanticValidator<T> {
    /// Create a new semantic validator
    pub fn new(indexer: T) -> Self {
        Self { indexer }
    }

    /// Validate semantic declaration against blockchain events
    ///
    /// # Arguments
    ///
    /// * `contract_address` - Contract address
    /// * `storage_slot` - Storage slot to validate
    /// * `declared_semantics` - What the developer declared
    /// * `current_value` - Current value of the slot (if known)
    ///
    /// # Returns
    ///
    /// Validation result with conflicts and confidence level
    pub async fn validate_semantics(
        &self,
        contract_address: &str,
        storage_slot: &str,
        declared_semantics: ZeroSemantics,
        current_value: Option<&str>,
    ) -> Result<ValidationResult, TraverseError> {
        // Get events for this storage slot
        let events = self
            .indexer
            .get_storage_events(
                contract_address,
                storage_slot,
                None, // from genesis
                None, // to latest
            )
            .await?;

        // Get current value if not provided
        let current_val = if let Some(val) = current_value {
            val.to_string()
        } else {
            self.indexer
                .get_current_value(contract_address, storage_slot)
                .await?
        };

        // Analyze events to determine actual semantics
        let detected = self.analyze_events(&events, &current_val);

        // Check for conflicts
        let conflicts = self.detect_conflicts(declared_semantics, detected, &events);

        Ok(ValidationResult {
            is_valid: conflicts.is_empty(),
            detected_semantics: detected,
            conflicts,
        })
    }

    /// Analyze events to determine semantic meaning
    fn analyze_events(&self, events: &[StorageEvent], current_value: &str) -> ZeroSemantics {
        let is_zero = current_value
            == "0x0000000000000000000000000000000000000000000000000000000000000000"
            || current_value == "0x0"
            || current_value == "0";

        if events.is_empty() {
            // No events found
            if is_zero {
                ZeroSemantics::NeverWritten
            } else {
                // Non-zero value but no events - possibly pre-existing data
                ZeroSemantics::ValidZero
            }
        } else if is_zero {
            // Current value is zero, check event history
            let has_non_zero = events.iter().any(|e| {
                e.new_value != "0x0000000000000000000000000000000000000000000000000000000000000000"
                    && e.new_value != "0x0"
                    && e.new_value != "0"
            });

            if has_non_zero {
                // Was non-zero, now zero
                let last_event = events.last();
                if let Some(event) = last_event {
                    match event.event_type {
                        StorageEventType::Cleared => ZeroSemantics::Cleared,
                        StorageEventType::SetToZero => ZeroSemantics::ExplicitlyZero,
                        _ => ZeroSemantics::Cleared, // Default assumption
                    }
                } else {
                    ZeroSemantics::Cleared
                }
            } else {
                // All events were zero writes
                ZeroSemantics::ExplicitlyZero
            }
        } else {
            // Current value is non-zero
            ZeroSemantics::ValidZero
        }
    }

    /// Detect conflicts between declared and detected semantics
    fn detect_conflicts(
        &self,
        declared: ZeroSemantics,
        detected: ZeroSemantics,
        events: &[StorageEvent],
    ) -> Vec<SemanticConflict> {
        let mut conflicts = Vec::new();

        // Only create conflicts if there's a meaningful disagreement
        if declared != detected {
            // Check for specific conflict patterns
            match (declared, detected) {
                (ZeroSemantics::NeverWritten, _) if !events.is_empty() => {
                    conflicts.push(SemanticConflict {
                        declared,
                        detected,
                        evidence: events.to_vec(),
                        description: format!(
                            "Declared 'never_written' but found {} storage events",
                            events.len()
                        ),
                    });
                }
                (ZeroSemantics::ExplicitlyZero, ZeroSemantics::Cleared) => {
                    let clear_events: Vec<_> = events
                        .iter()
                        .filter(|e| e.event_type == StorageEventType::Cleared)
                        .cloned()
                        .collect();

                    if !clear_events.is_empty() {
                        conflicts.push(SemanticConflict {
                            declared,
                            detected,
                            evidence: clear_events,
                            description:
                                "Declared 'explicitly_zero' but evidence shows value was cleared"
                                    .to_string(),
                        });
                    }
                }
                (ZeroSemantics::ValidZero, ZeroSemantics::NeverWritten) => {
                    conflicts.push(SemanticConflict {
                        declared,
                        detected,
                        evidence: Vec::new(),
                        description: "Declared 'valid_zero' but no write events found".to_string(),
                    });
                }
                _ => {
                    // Other mismatches - create general conflict
                    conflicts.push(SemanticConflict {
                        declared,
                        detected,
                        evidence: events.to_vec(),
                        description: format!(
                            "Semantic mismatch: declared {:?} but detected {:?}",
                            declared, detected
                        ),
                    });
                }
            }
        }

        conflicts
    }
}

/// Mock indexer service for testing
pub struct MockIndexerService {
    name: String,
    mock_events: HashMap<String, Vec<StorageEvent>>,
    mock_values: HashMap<String, String>,
}

impl MockIndexerService {
    /// Create a new mock indexer
    pub fn new(name: String) -> Self {
        Self {
            name,
            mock_events: HashMap::new(),
            mock_values: HashMap::new(),
        }
    }

    /// Add mock events for testing
    pub fn add_mock_events(&mut self, key: String, events: Vec<StorageEvent>) {
        self.mock_events.insert(key, events);
    }

    /// Add mock current value
    pub fn add_mock_value(&mut self, key: String, value: String) {
        self.mock_values.insert(key, value);
    }

    /// Create key for lookups
    fn make_key(&self, contract_address: &str, storage_slot: &str) -> String {
        format!("{}:{}", contract_address, storage_slot)
    }
}

impl IndexerService for MockIndexerService {
    fn get_storage_events(
        &self,
        contract_address: &str,
        storage_slot: &str,
        _from_block: Option<u64>,
        _to_block: Option<u64>,
    ) -> impl std::future::Future<Output = Result<Vec<StorageEvent>, TraverseError>> + Send {
        let key = self.make_key(contract_address, storage_slot);
        let result = self.mock_events.get(&key).cloned().unwrap_or_default();
        async move { Ok(result) }
    }

    fn get_current_value(
        &self,
        contract_address: &str,
        storage_slot: &str,
    ) -> impl std::future::Future<Output = Result<String, TraverseError>> + Send {
        let key = self.make_key(contract_address, storage_slot);
        let result = self.mock_values.get(&key).cloned().unwrap_or_else(|| {
            "0x0000000000000000000000000000000000000000000000000000000000000000".to_string()
        });
        async move { Ok(result) }
    }

    fn service_name(&self) -> &str {
        &self.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_semantic_validation_never_written() {
        let mock_indexer = MockIndexerService::new("test".to_string());
        let validator = SemanticValidator::new(mock_indexer);

        let result = validator
            .validate_semantics(
                "0x123",
                "0x0",
                ZeroSemantics::NeverWritten,
                Some("0x0000000000000000000000000000000000000000000000000000000000000000"),
            )
            .await
            .unwrap();

        assert!(result.is_valid);
        assert_eq!(result.detected_semantics, ZeroSemantics::NeverWritten);
    }

    #[tokio::test]
    async fn test_semantic_validation_conflict() {
        let mut mock_indexer = MockIndexerService::new("test".to_string());

        // Add mock events showing the slot was written to
        mock_indexer.add_mock_events(
            "0x123:0x0".to_string(),
            vec![StorageEvent {
                block_number: 100,
                transaction_hash: "0xabc".to_string(),
                contract_address: "0x123".to_string(),
                storage_slot: "0x0".to_string(),
                previous_value: None,
                new_value: "0x1".to_string(),
                event_type: StorageEventType::FirstWrite,
            }],
        );

        let validator = SemanticValidator::new(mock_indexer);

        let result = validator
            .validate_semantics(
                "0x123",
                "0x0",
                ZeroSemantics::NeverWritten, // Declared never written
                Some("0x0000000000000000000000000000000000000000000000000000000000000000"), // But current value is zero
            )
            .await
            .unwrap();

        assert!(!result.is_valid); // Should have conflicts
        assert_eq!(result.conflicts.len(), 1);
        assert_eq!(result.conflicts[0].declared, ZeroSemantics::NeverWritten);
    }

    #[tokio::test]
    async fn test_semantic_validation_explicitly_zero() {
        let mut mock_indexer = MockIndexerService::new("test".to_string());

        // Add events showing slot was explicitly set to zero
        mock_indexer.add_mock_events(
            "0x456:0x1".to_string(),
            vec![StorageEvent {
                block_number: 200,
                transaction_hash: "0xdef".to_string(),
                contract_address: "0x456".to_string(),
                storage_slot: "0x1".to_string(),
                previous_value: None,
                new_value: "0x0000000000000000000000000000000000000000000000000000000000000000"
                    .to_string(),
                event_type: StorageEventType::SetToZero,
            }],
        );

        let validator = SemanticValidator::new(mock_indexer);

        let result = validator
            .validate_semantics(
                "0x456",
                "0x1",
                ZeroSemantics::ExplicitlyZero,
                Some("0x0000000000000000000000000000000000000000000000000000000000000000"),
            )
            .await
            .unwrap();

        assert!(result.is_valid);
        assert_eq!(result.detected_semantics, ZeroSemantics::ExplicitlyZero);
        assert!(result.conflicts.is_empty());
    }

    #[tokio::test]
    async fn test_semantic_validation_cleared() {
        let mut mock_indexer = MockIndexerService::new("test".to_string());

        // Add events showing slot was non-zero then cleared
        mock_indexer.add_mock_events(
            "0x789:0x2".to_string(),
            vec![
                StorageEvent {
                    block_number: 100,
                    transaction_hash: "0x111".to_string(),
                    contract_address: "0x789".to_string(),
                    storage_slot: "0x2".to_string(),
                    previous_value: None,
                    new_value: "0x42".to_string(),
                    event_type: StorageEventType::FirstWrite,
                },
                StorageEvent {
                    block_number: 200,
                    transaction_hash: "0x222".to_string(),
                    contract_address: "0x789".to_string(),
                    storage_slot: "0x2".to_string(),
                    previous_value: Some("0x42".to_string()),
                    new_value: "0x0000000000000000000000000000000000000000000000000000000000000000"
                        .to_string(),
                    event_type: StorageEventType::Cleared,
                },
            ],
        );

        let validator = SemanticValidator::new(mock_indexer);

        let result = validator
            .validate_semantics(
                "0x789",
                "0x2",
                ZeroSemantics::Cleared,
                Some("0x0000000000000000000000000000000000000000000000000000000000000000"),
            )
            .await
            .unwrap();

        assert!(result.is_valid);
        assert_eq!(result.detected_semantics, ZeroSemantics::Cleared);
        assert!(result.conflicts.is_empty());
    }

    #[tokio::test]
    async fn test_semantic_validation_valid_zero() {
        let mut mock_indexer = MockIndexerService::new("test".to_string());

        // Add events showing slot has valid non-zero value
        mock_indexer.add_mock_events(
            "0xabc:0x3".to_string(),
            vec![StorageEvent {
                block_number: 300,
                transaction_hash: "0x333".to_string(),
                contract_address: "0xabc".to_string(),
                storage_slot: "0x3".to_string(),
                previous_value: None,
                new_value: "0x100".to_string(),
                event_type: StorageEventType::FirstWrite,
            }],
        );

        mock_indexer.add_mock_value("0xabc:0x3".to_string(), "0x100".to_string());

        let validator = SemanticValidator::new(mock_indexer);

        let result = validator
            .validate_semantics(
                "0xabc",
                "0x3",
                ZeroSemantics::ValidZero,
                None, // Let it fetch current value
            )
            .await
            .unwrap();

        assert!(result.is_valid);
        assert_eq!(result.detected_semantics, ZeroSemantics::ValidZero);
        assert!(result.conflicts.is_empty());
    }

    #[tokio::test]
    async fn test_semantic_conflict_explicitly_zero_vs_cleared() {
        let mut mock_indexer = MockIndexerService::new("test".to_string());

        // Add events showing slot was cleared (but developer claims explicitly zero)
        mock_indexer.add_mock_events(
            "0xdef:0x4".to_string(),
            vec![
                StorageEvent {
                    block_number: 100,
                    transaction_hash: "0x444".to_string(),
                    contract_address: "0xdef".to_string(),
                    storage_slot: "0x4".to_string(),
                    previous_value: None,
                    new_value: "0x99".to_string(),
                    event_type: StorageEventType::FirstWrite,
                },
                StorageEvent {
                    block_number: 200,
                    transaction_hash: "0x555".to_string(),
                    contract_address: "0xdef".to_string(),
                    storage_slot: "0x4".to_string(),
                    previous_value: Some("0x99".to_string()),
                    new_value: "0x0000000000000000000000000000000000000000000000000000000000000000"
                        .to_string(),
                    event_type: StorageEventType::Cleared,
                },
            ],
        );

        let validator = SemanticValidator::new(mock_indexer);

        let result = validator
            .validate_semantics(
                "0xdef",
                "0x4",
                ZeroSemantics::ExplicitlyZero, // Declared explicitly zero
                Some("0x0000000000000000000000000000000000000000000000000000000000000000"),
            )
            .await
            .unwrap();

        assert!(!result.is_valid); // Should detect conflict
        assert_eq!(result.detected_semantics, ZeroSemantics::Cleared);
        assert_eq!(result.conflicts.len(), 1);
        assert_eq!(result.conflicts[0].declared, ZeroSemantics::ExplicitlyZero);
        assert_eq!(result.conflicts[0].detected, ZeroSemantics::Cleared);
        assert_eq!(result.conflicts[0].evidence.len(), 1); // Should have the clear event as evidence
    }

    #[tokio::test]
    async fn test_semantic_validation_multiple_events() {
        let mut mock_indexer = MockIndexerService::new("test".to_string());

        // Add multiple events showing complex history
        mock_indexer.add_mock_events(
            "0x123:0x5".to_string(),
            vec![
                StorageEvent {
                    block_number: 100,
                    transaction_hash: "0x666".to_string(),
                    contract_address: "0x123".to_string(),
                    storage_slot: "0x5".to_string(),
                    previous_value: None,
                    new_value: "0x10".to_string(),
                    event_type: StorageEventType::FirstWrite,
                },
                StorageEvent {
                    block_number: 150,
                    transaction_hash: "0x777".to_string(),
                    contract_address: "0x123".to_string(),
                    storage_slot: "0x5".to_string(),
                    previous_value: Some("0x10".to_string()),
                    new_value: "0x20".to_string(),
                    event_type: StorageEventType::Update,
                },
                StorageEvent {
                    block_number: 200,
                    transaction_hash: "0x888".to_string(),
                    contract_address: "0x123".to_string(),
                    storage_slot: "0x5".to_string(),
                    previous_value: Some("0x20".to_string()),
                    new_value: "0x30".to_string(),
                    event_type: StorageEventType::Update,
                },
            ],
        );

        mock_indexer.add_mock_value("0x123:0x5".to_string(), "0x30".to_string());

        let validator = SemanticValidator::new(mock_indexer);

        let result = validator
            .validate_semantics("0x123", "0x5", ZeroSemantics::ValidZero, None)
            .await
            .unwrap();

        assert!(result.is_valid);
        assert_eq!(result.detected_semantics, ZeroSemantics::ValidZero);
        assert!(result.conflicts.is_empty());
    }

    #[tokio::test]
    async fn test_mock_indexer_service_interface() {
        let mut mock_indexer = MockIndexerService::new("test_service".to_string());

        // Test service name
        assert_eq!(mock_indexer.service_name(), "test_service");

        // Test adding and retrieving mock data
        mock_indexer.add_mock_value("test:slot".to_string(), "0x42".to_string());

        let value = mock_indexer
            .get_current_value("test", "slot")
            .await
            .unwrap();
        assert_eq!(value, "0x42");

        // Test default value for non-existent slot
        let default_value = mock_indexer
            .get_current_value("missing", "slot")
            .await
            .unwrap();
        assert_eq!(
            default_value,
            "0x0000000000000000000000000000000000000000000000000000000000000000"
        );

        // Test empty events for non-existent slot
        let events = mock_indexer
            .get_storage_events("missing", "slot", None, None)
            .await
            .unwrap();
        assert!(events.is_empty());
    }

    #[tokio::test]
    async fn test_storage_event_types() {
        // Test that all event types can be created and compared
        let events = [
            StorageEventType::FirstWrite,
            StorageEventType::Update,
            StorageEventType::Cleared,
            StorageEventType::SetToZero,
        ];

        assert_eq!(events.len(), 4);
        assert_ne!(StorageEventType::FirstWrite, StorageEventType::Update);
        assert_ne!(StorageEventType::Cleared, StorageEventType::SetToZero);
    }

    #[tokio::test]
    async fn test_semantic_conflict_serialization() {
        // Test that conflict structures can be serialized/deserialized
        let conflict = SemanticConflict {
            declared: ZeroSemantics::NeverWritten,
            detected: ZeroSemantics::ExplicitlyZero,
            evidence: vec![StorageEvent {
                block_number: 100,
                transaction_hash: "0xtest".to_string(),
                contract_address: "0x123".to_string(),
                storage_slot: "0x0".to_string(),
                previous_value: None,
                new_value: "0x0".to_string(),
                event_type: StorageEventType::SetToZero,
            }],
            description: "Test conflict".to_string(),
        };

        // Should be able to serialize and deserialize
        let serialized = serde_json::to_string(&conflict).unwrap();
        let deserialized: SemanticConflict = serde_json::from_str(&serialized).unwrap();

        assert_eq!(conflict.declared, deserialized.declared);
        assert_eq!(conflict.detected, deserialized.detected);
        assert_eq!(conflict.description, deserialized.description);
        assert_eq!(conflict.evidence.len(), deserialized.evidence.len());
    }
}
