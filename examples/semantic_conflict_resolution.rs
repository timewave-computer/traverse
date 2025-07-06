//! Semantic Conflict Resolution Example
//!
//! This example demonstrates how to detect and resolve semantic conflicts
//! when storage values have ambiguous zero meanings in Ethereum contracts.
//!
//! It shows:
//! - Different zero semantic types (never_written, explicitly_zero, cleared, valid_zero)
//! - Conflict detection between declared and validated semantics
//! - Resolution strategies for semantic conflicts
//! - Integration with semantic-aware business logic

use std::collections::HashMap;
use traverse_core::{LayoutInfo, StorageEntry, StorageSemantics, TypeInfo, ZeroSemantics};

/// Mock contract with different semantic scenarios
#[allow(dead_code)]
struct MockContract {
    name: String,
    storage_values: HashMap<String, u64>,
    semantic_specifications: HashMap<String, ZeroSemantics>,
    historical_events: HashMap<String, Vec<MockEvent>>,
}

/// Mock blockchain event for semantic validation
#[derive(Clone)]
#[allow(dead_code)]
struct MockEvent {
    event_type: String,
    old_value: u64,
    new_value: u64,
    block_number: u64,
}

impl MockContract {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            storage_values: HashMap::new(),
            semantic_specifications: HashMap::new(),
            historical_events: HashMap::new(),
        }
    }

    fn add_storage_slot(&mut self, slot: &str, value: u64, declared_semantics: ZeroSemantics) {
        self.storage_values.insert(slot.to_string(), value);
        self.semantic_specifications
            .insert(slot.to_string(), declared_semantics);
    }

    fn add_historical_event(&mut self, slot: &str, event: MockEvent) {
        self.historical_events
            .entry(slot.to_string())
            .or_default()
            .push(event);
    }

    fn get_storage_value(&self, slot: &str) -> Option<u64> {
        self.storage_values.get(slot).copied()
    }

    fn get_declared_semantics(&self, slot: &str) -> Option<ZeroSemantics> {
        self.semantic_specifications.get(slot).copied()
    }

    fn get_historical_events(&self, slot: &str) -> Vec<MockEvent> {
        self.historical_events
            .get(slot)
            .cloned()
            .unwrap_or_default()
    }
}

/// Semantic validator that analyzes blockchain events to determine actual semantics
struct SemanticValidator;

impl SemanticValidator {
    fn validate_zero_semantics(
        &self,
        contract: &MockContract,
        slot: &str,
    ) -> Option<ZeroSemantics> {
        let events = contract.get_historical_events(slot);
        let current_value = contract.get_storage_value(slot)?;

        if current_value != 0 {
            // Non-zero values don't need semantic validation
            return None;
        }

        // Analyze events to determine actual semantics
        if events.is_empty() {
            Some(ZeroSemantics::NeverWritten)
        } else {
            // Check for explicit zero writes
            let has_zero_write = events.iter().any(|e| e.new_value == 0);
            let has_nonzero_write = events.iter().any(|e| e.new_value != 0);

            match (has_zero_write, has_nonzero_write) {
                (true, false) => Some(ZeroSemantics::ExplicitlyZero),
                (true, true) => Some(ZeroSemantics::Cleared),
                (false, true) => Some(ZeroSemantics::Cleared), // Written non-zero, now zero
                (false, false) => Some(ZeroSemantics::NeverWritten),
            }
        }
    }

    fn resolve_semantic_conflict(
        &self,
        declared: ZeroSemantics,
        validated: ZeroSemantics,
    ) -> StorageSemantics {
        if declared == validated {
            // No conflict
            StorageSemantics::new(declared)
        } else {
            // Conflict detected - use validated semantics
            StorageSemantics::with_validation(declared, validated)
        }
    }
}

/// Demonstrates semantic conflict resolution for different scenarios
fn demonstrate_semantic_scenarios() -> Result<(), Box<dyn std::error::Error>> {
    println!("Semantic Conflict Resolution Example");
    println!("=====================================");
    println!();

    let validator = SemanticValidator;

    // Scenario 1: Never Written (no conflict)
    println!("Scenario 1: Never Written Storage Slot");
    println!("--------------------------------------");

    let mut contract1 = MockContract::new("UnusedContract");
    contract1.add_storage_slot("0", 0, ZeroSemantics::NeverWritten);
    // No events added - truly never written

    let declared = contract1.get_declared_semantics("0").unwrap();
    let validated = validator.validate_zero_semantics(&contract1, "0");

    println!("Storage slot 0:");
    println!("   • Value: {}", contract1.get_storage_value("0").unwrap());
    println!("   • Declared: {:?}", declared);
    println!("   • Validated: {:?}", validated);

    if let Some(validated_semantics) = validated {
        let resolved = validator.resolve_semantic_conflict(declared, validated_semantics);
        println!("   • Resolution: {:?}", resolved.zero_meaning);
        println!("   • Has conflict: {}", resolved.has_conflict());

        if resolved.has_conflict() {
            println!("   Semantic conflict detected!");
        } else {
            println!("   No semantic conflict");
        }
    }
    println!();

    // Scenario 2: Explicitly Zero (no conflict)
    println!("Scenario 2: Explicitly Zero Storage Slot");
    println!("----------------------------------------");

    let mut contract2 = MockContract::new("InitializedContract");
    contract2.add_storage_slot("0", 0, ZeroSemantics::ExplicitlyZero);
    contract2.add_historical_event(
        "0",
        MockEvent {
            event_type: "SSTORE".to_string(),
            old_value: 0,
            new_value: 0,
            block_number: 100,
        },
    );

    let declared = contract2.get_declared_semantics("0").unwrap();
    let validated = validator.validate_zero_semantics(&contract2, "0");

    println!("Storage slot 0:");
    println!("   • Value: {}", contract2.get_storage_value("0").unwrap());
    println!("   • Declared: {:?}", declared);
    println!("   • Validated: {:?}", validated);

    if let Some(validated_semantics) = validated {
        let resolved = validator.resolve_semantic_conflict(declared, validated_semantics);
        println!("   • Resolution: {:?}", resolved.zero_meaning);
        println!("   • Has conflict: {}", resolved.has_conflict());

        if resolved.has_conflict() {
            println!("   Semantic conflict detected!");
        } else {
            println!("   No semantic conflict");
        }
    }
    println!();

    // Scenario 3: Semantic Conflict - Declared Never Written but Actually Written
    println!("Scenario 3: Semantic Conflict - Never Written vs Explicitly Zero");
    println!("------------------------------------------------------------------");

    let mut contract3 = MockContract::new("ConflictContract");
    contract3.add_storage_slot("0", 0, ZeroSemantics::NeverWritten);
    contract3.add_historical_event(
        "0",
        MockEvent {
            event_type: "SSTORE".to_string(),
            old_value: 0,
            new_value: 0,
            block_number: 200,
        },
    );

    let declared = contract3.get_declared_semantics("0").unwrap();
    let validated = validator.validate_zero_semantics(&contract3, "0");

    println!("Storage slot 0:");
    println!("   • Value: {}", contract3.get_storage_value("0").unwrap());
    println!("   • Declared: {:?}", declared);
    println!("   • Validated: {:?}", validated);

    if let Some(validated_semantics) = validated {
        let resolved = validator.resolve_semantic_conflict(declared, validated_semantics);
        println!("   • Resolution: {:?}", resolved.zero_meaning);
        println!("   • Has conflict: {}", resolved.has_conflict());

        if resolved.has_conflict() {
            println!("   SEMANTIC CONFLICT DETECTED!");
            println!("      Developer declared 'never_written' but blockchain shows write event");
            println!("      Resolution: Using validated semantics (explicitly_zero)");
        } else {
            println!("   No semantic conflict");
        }
    }
    println!();

    // Scenario 4: Cleared Value Conflict
    println!("Scenario 4: Semantic Conflict - Never Written vs Cleared");
    println!("--------------------------------------------------------");

    let mut contract4 = MockContract::new("ClearedContract");
    contract4.add_storage_slot("0", 0, ZeroSemantics::NeverWritten);
    contract4.add_historical_event(
        "0",
        MockEvent {
            event_type: "SSTORE".to_string(),
            old_value: 0,
            new_value: 1000,
            block_number: 300,
        },
    );
    contract4.add_historical_event(
        "0",
        MockEvent {
            event_type: "SSTORE".to_string(),
            old_value: 1000,
            new_value: 0,
            block_number: 400,
        },
    );

    let declared = contract4.get_declared_semantics("0").unwrap();
    let validated = validator.validate_zero_semantics(&contract4, "0");

    println!("Storage slot 0:");
    println!("   • Value: {}", contract4.get_storage_value("0").unwrap());
    println!("   • Declared: {:?}", declared);
    println!("   • Validated: {:?}", validated);

    if let Some(validated_semantics) = validated {
        let resolved = validator.resolve_semantic_conflict(declared, validated_semantics);
        println!("   • Resolution: {:?}", resolved.zero_meaning);
        println!("   • Has conflict: {}", resolved.has_conflict());

        if resolved.has_conflict() {
            println!("   SEMANTIC CONFLICT DETECTED!");
            println!("      Developer declared 'never_written' but blockchain shows non-zero value was cleared");
            println!("      Resolution: Using validated semantics (cleared)");
        } else {
            println!("   No semantic conflict");
        }
    }
    println!();

    // Scenario 5: Valid Zero (no conflict)
    println!("Scenario 5: Valid Zero Operational State");
    println!("----------------------------------------");

    let mut contract5 = MockContract::new("OperationalContract");
    contract5.add_storage_slot("0", 0, ZeroSemantics::ValidZero);
    contract5.add_historical_event(
        "0",
        MockEvent {
            event_type: "SSTORE".to_string(),
            old_value: 0,
            new_value: 0,
            block_number: 500,
        },
    );

    let declared = contract5.get_declared_semantics("0").unwrap();
    let validated = validator.validate_zero_semantics(&contract5, "0");

    println!("Storage slot 0:");
    println!("   • Value: {}", contract5.get_storage_value("0").unwrap());
    println!("   • Declared: {:?}", declared);
    println!("   • Validated: {:?}", validated);

    if let Some(validated_semantics) = validated {
        let resolved = validator.resolve_semantic_conflict(declared, validated_semantics);
        println!("   • Resolution: {:?}", resolved.zero_meaning);
        println!("   • Has conflict: {}", resolved.has_conflict());

        if resolved.has_conflict() {
            println!("   Semantic conflict detected!");
        } else {
            println!("   No semantic conflict");
        }
    }
    println!();

    // Business Logic Integration
    println!("Business Logic Integration");
    println!("=========================");

    let contracts = vec![
        ("UnusedContract", &contract1),
        ("InitializedContract", &contract2),
        ("ConflictContract", &contract3),
        ("ClearedContract", &contract4),
        ("OperationalContract", &contract5),
    ];

    for (name, contract) in contracts {
        let value = contract.get_storage_value("0").unwrap();
        let declared = contract.get_declared_semantics("0").unwrap();
        let validated = validator.validate_zero_semantics(contract, "0");

        let business_interpretation = if let Some(validated_semantics) = validated {
            let resolved = validator.resolve_semantic_conflict(declared, validated_semantics);

            match (value, resolved.zero_meaning) {
                (0, ZeroSemantics::NeverWritten) => "System never initialized - requires setup",
                (0, ZeroSemantics::ExplicitlyZero) => "System initialized and ready",
                (0, ZeroSemantics::Cleared) => "System was active but cleared - may need reset",
                (0, ZeroSemantics::ValidZero) => "System operational with zero value",
                (n, _) => &format!("System active with value {}", n),
            }
        } else {
            "Unable to determine semantic meaning"
        };

        println!("{}: {}", name, business_interpretation);
    }

    println!();
    println!("CLI Integration Example");
    println!("======================");
    println!("# Generate semantic storage proof with conflict resolution");
    println!("traverse generate-proof \\");
    println!("  --contract 0x1234567890123456789012345678901234567890 \\");
    println!("  --slot 0x0000000000000000000000000000000000000000000000000000000000000000 \\");
    println!("  --zero-means never_written \\");
    println!("  --validate-semantics \\");
    println!("  --resolve-conflicts \\");
    println!("  --rpc-url $ETHEREUM_RPC_URL");

    Ok(())
}

/// Creates a semantic storage layout for testing
fn create_semantic_layout() -> LayoutInfo {
    let mut layout = LayoutInfo {
        contract_name: "SemanticExample".to_string(),
        storage: Vec::new(),
        types: Vec::new(),
    };

    // Add storage entries with different semantic specifications
    layout.storage.push(StorageEntry {
        label: "never_written_slot".to_string(),
        slot: "0".to_string(),
        offset: 0,
        type_name: "t_uint256".to_string(),
        zero_semantics: ZeroSemantics::NeverWritten,
    });

    layout.storage.push(StorageEntry {
        label: "explicitly_zero_slot".to_string(),
        slot: "1".to_string(),
        offset: 0,
        type_name: "t_uint256".to_string(),
        zero_semantics: ZeroSemantics::ExplicitlyZero,
    });

    layout.storage.push(StorageEntry {
        label: "cleared_slot".to_string(),
        slot: "2".to_string(),
        offset: 0,
        type_name: "t_uint256".to_string(),
        zero_semantics: ZeroSemantics::Cleared,
    });

    layout.storage.push(StorageEntry {
        label: "valid_zero_slot".to_string(),
        slot: "3".to_string(),
        offset: 0,
        type_name: "t_uint256".to_string(),
        zero_semantics: ZeroSemantics::ValidZero,
    });

    // Add type information
    layout.types.push(TypeInfo {
        label: "t_uint256".to_string(),
        number_of_bytes: "32".to_string(),
        encoding: "inplace".to_string(),
        base: None,
        key: None,
        value: None,
    });

    layout
}

/// Main example function
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Demonstrate semantic conflict resolution
    demonstrate_semantic_scenarios()?;

    // Show layout integration
    println!();
    println!("Semantic Layout Integration");
    println!("===========================");

    let layout = create_semantic_layout();
    println!(
        "Layout: {} (commitment: 0x{})",
        layout.contract_name,
        hex::encode(layout.commitment())
    );

    for entry in &layout.storage {
        println!(
            "   • {} (slot {}) → {:?}",
            entry.label, entry.slot, entry.zero_semantics
        );
    }

    println!();
    println!("Key Takeaways:");
    println!("• Semantic conflicts occur when declared and validated semantics differ");
    println!("• Event analysis helps validate actual zero meanings");
    println!("• Resolution strategies prefer validated over declared semantics");
    println!("• Business logic adapts based on resolved semantic meanings");
    println!("• CLI integration provides semantic validation and conflict resolution");

    Ok(())
}
