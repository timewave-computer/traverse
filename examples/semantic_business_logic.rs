//! Semantic Business Logic Integration Example  
//!
//! This example demonstrates how semantic storage proofs integrate with business logic
//! in real-world DeFi applications, showing how different zero meanings affect
//! authorization, risk assessment, and operational decisions.
//!
//! It covers:
//! - Semantic-aware authorization systems
//! - Risk assessment based on zero semantics
//! - Operational state management with semantic context
//! - Multi-contract semantic orchestration

use std::collections::HashMap;
use traverse_core::{LayoutInfo, StorageEntry, StorageSemantics, TypeInfo, ZeroSemantics};
// Removed unused import: serde_json::json

/// DeFi protocol state with semantic awareness
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct DeFiProtocolState {
    name: String,
    total_deposits: u64,
    total_borrows: u64,
    user_positions: HashMap<String, UserPosition>,
    semantic_context: HashMap<String, StorageSemantics>,
}

/// User position with semantic metadata
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct UserPosition {
    deposited: u64,
    borrowed: u64,
    collateral_ratio: f64,
    semantic_flags: Vec<String>,
}

/// Semantic authorization result
#[derive(Debug)]
struct SemanticAuthorizationResult {
    authorized: bool,
    reason: String,
    risk_level: RiskLevel,
    semantic_basis: Vec<String>,
    required_actions: Vec<String>,
}

/// Risk assessment levels
#[derive(Debug, Clone, PartialEq, PartialOrd)]
enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Business logic engine with semantic awareness
struct SemanticBusinessLogic {
    protocol_configs: HashMap<String, ProtocolConfig>,
}

/// Protocol configuration with semantic parameters
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct ProtocolConfig {
    min_collateral_ratio: f64,
    max_utilization_rate: f64,
    emergency_pause_threshold: f64,
    semantic_overrides: HashMap<ZeroSemantics, f64>, // Risk multipliers
}

impl SemanticBusinessLogic {
    fn new() -> Self {
        let mut configs = HashMap::new();

        // Default DeFi protocol configuration
        configs.insert(
            "default".to_string(),
            ProtocolConfig {
                min_collateral_ratio: 1.5,
                max_utilization_rate: 0.8,
                emergency_pause_threshold: 0.95,
                semantic_overrides: {
                    let mut overrides = HashMap::new();
                    overrides.insert(ZeroSemantics::NeverWritten, 2.0); // Double risk - uninitialized
                    overrides.insert(ZeroSemantics::ExplicitlyZero, 1.0); // Normal risk
                    overrides.insert(ZeroSemantics::Cleared, 1.5); // Higher risk - was active
                    overrides.insert(ZeroSemantics::ValidZero, 1.0); // Normal risk
                    overrides
                },
            },
        );

        Self {
            protocol_configs: configs,
        }
    }

    /// Authorize user action based on semantic context
    fn authorize_user_action(
        &self,
        protocol_state: &DeFiProtocolState,
        user_id: &str,
        action: &str,
        amount: u64,
    ) -> SemanticAuthorizationResult {
        let config = self.protocol_configs.get("default").unwrap();

        // Get user position
        let user_position = protocol_state.user_positions.get(user_id);

        match action {
            "deposit" => self.authorize_deposit(protocol_state, user_position, amount, config),
            "withdraw" => self.authorize_withdrawal(protocol_state, user_position, amount, config),
            "borrow" => self.authorize_borrow(protocol_state, user_position, amount, config),
            "repay" => self.authorize_repay(protocol_state, user_position, amount, config),
            _ => SemanticAuthorizationResult {
                authorized: false,
                reason: format!("Unknown action: {}", action),
                risk_level: RiskLevel::Critical,
                semantic_basis: vec![],
                required_actions: vec![],
            },
        }
    }

    /// Authorize deposit with semantic context
    fn authorize_deposit(
        &self,
        protocol_state: &DeFiProtocolState,
        user_position: Option<&UserPosition>,
        amount: u64,
        config: &ProtocolConfig,
    ) -> SemanticAuthorizationResult {
        let mut semantic_basis = vec![];
        let mut required_actions = vec![];
        let mut risk_level = RiskLevel::Low;

        // Check if user has never deposited before
        if user_position.is_none() {
            // Check protocol initialization semantics
            if let Some(total_deposits_semantics) =
                protocol_state.semantic_context.get("total_deposits")
            {
                match total_deposits_semantics.zero_meaning {
                    ZeroSemantics::NeverWritten => {
                        semantic_basis
                            .push("Protocol never initialized - first deposit".to_string());
                        risk_level = RiskLevel::Medium;
                        required_actions.push("Initialize protocol state".to_string());
                    }
                    ZeroSemantics::ExplicitlyZero => {
                        semantic_basis.push("Protocol initialized and ready".to_string());
                    }
                    ZeroSemantics::Cleared => {
                        semantic_basis.push(
                            "Protocol was active but cleared - may need reinitialization"
                                .to_string(),
                        );
                        risk_level = RiskLevel::High;
                        required_actions.push("Verify protocol state integrity".to_string());
                    }
                    ZeroSemantics::ValidZero => {
                        semantic_basis.push("Protocol operational with zero deposits".to_string());
                    }
                }
            }
        }

        // Calculate semantic risk multiplier
        let semantic_multiplier = self.calculate_semantic_risk_multiplier(protocol_state, config);
        let adjusted_amount = (amount as f64 * semantic_multiplier) as u64;

        // Check utilization after deposit
        let new_total_deposits = protocol_state.total_deposits + adjusted_amount;
        let utilization_rate = if new_total_deposits > 0 {
            protocol_state.total_borrows as f64 / new_total_deposits as f64
        } else {
            0.0
        };

        if utilization_rate > config.max_utilization_rate {
            return SemanticAuthorizationResult {
                authorized: false,
                reason: format!(
                    "Deposit would exceed max utilization rate ({:.2}%)",
                    config.max_utilization_rate * 100.0
                ),
                risk_level: RiskLevel::High,
                semantic_basis,
                required_actions,
            };
        }

        SemanticAuthorizationResult {
            authorized: true,
            reason: format!(
                "Deposit authorized with semantic risk multiplier {:.2}x",
                semantic_multiplier
            ),
            risk_level,
            semantic_basis,
            required_actions,
        }
    }

    /// Authorize withdrawal with semantic context
    fn authorize_withdrawal(
        &self,
        protocol_state: &DeFiProtocolState,
        user_position: Option<&UserPosition>,
        amount: u64,
        config: &ProtocolConfig,
    ) -> SemanticAuthorizationResult {
        let mut semantic_basis = vec![];
        let required_actions = vec![];

        // Check if user has position
        let user_pos = match user_position {
            Some(pos) => pos,
            None => {
                // Check zero balance semantics
                if let Some(balance_semantics) =
                    protocol_state.semantic_context.get("user_balances")
                {
                    match balance_semantics.zero_meaning {
                        ZeroSemantics::NeverWritten => {
                            semantic_basis
                                .push("User never deposited - no balance to withdraw".to_string());
                        }
                        ZeroSemantics::ExplicitlyZero => {
                            semantic_basis.push(
                                "User balance explicitly zero - no funds available".to_string(),
                            );
                        }
                        ZeroSemantics::Cleared => {
                            semantic_basis.push(
                                "User balance was cleared - funds may have been withdrawn"
                                    .to_string(),
                            );
                        }
                        ZeroSemantics::ValidZero => {
                            semantic_basis.push("User balance is zero (valid state)".to_string());
                        }
                    }
                }

                return SemanticAuthorizationResult {
                    authorized: false,
                    reason: "No user position found".to_string(),
                    risk_level: RiskLevel::Medium,
                    semantic_basis,
                    required_actions,
                };
            }
        };

        // Check if sufficient balance
        if user_pos.deposited < amount {
            return SemanticAuthorizationResult {
                authorized: false,
                reason: format!("Insufficient balance: {} < {}", user_pos.deposited, amount),
                risk_level: RiskLevel::Medium,
                semantic_basis,
                required_actions,
            };
        }

        // Check collateral ratio after withdrawal
        let remaining_collateral = user_pos.deposited - amount;
        let new_collateral_ratio = if user_pos.borrowed > 0 {
            remaining_collateral as f64 / user_pos.borrowed as f64
        } else {
            f64::INFINITY
        };

        if new_collateral_ratio < config.min_collateral_ratio {
            return SemanticAuthorizationResult {
                authorized: false,
                reason: format!(
                    "Withdrawal would violate collateral ratio: {:.2} < {:.2}",
                    new_collateral_ratio, config.min_collateral_ratio
                ),
                risk_level: RiskLevel::High,
                semantic_basis,
                required_actions: vec!["Repay loans or deposit more collateral".to_string()],
            };
        }

        SemanticAuthorizationResult {
            authorized: true,
            reason: format!(
                "Withdrawal authorized - collateral ratio remains {:.2}",
                new_collateral_ratio
            ),
            risk_level: RiskLevel::Low,
            semantic_basis,
            required_actions,
        }
    }

    /// Authorize borrow with semantic context
    fn authorize_borrow(
        &self,
        protocol_state: &DeFiProtocolState,
        user_position: Option<&UserPosition>,
        amount: u64,
        config: &ProtocolConfig,
    ) -> SemanticAuthorizationResult {
        let mut semantic_basis = vec![];
        let required_actions = vec![];

        // Check if user has collateral
        let user_pos = match user_position {
            Some(pos) => pos,
            None => {
                return SemanticAuthorizationResult {
                    authorized: false,
                    reason: "No collateral deposited".to_string(),
                    risk_level: RiskLevel::High,
                    semantic_basis: vec![
                        "User has no position - cannot borrow without collateral".to_string()
                    ],
                    required_actions: vec!["Deposit collateral first".to_string()],
                };
            }
        };

        // Calculate semantic-adjusted collateral requirements
        let semantic_multiplier = self.calculate_semantic_risk_multiplier(protocol_state, config);
        let adjusted_min_collateral = config.min_collateral_ratio * semantic_multiplier;

        // Check collateral ratio after borrow
        let new_borrowed = user_pos.borrowed + amount;
        let collateral_ratio = if new_borrowed > 0 {
            user_pos.deposited as f64 / new_borrowed as f64
        } else {
            f64::INFINITY
        };

        if collateral_ratio < adjusted_min_collateral {
            return SemanticAuthorizationResult {
                authorized: false,
                reason: format!(
                    "Insufficient collateral ratio: {:.2} < {:.2} (semantic-adjusted)",
                    collateral_ratio, adjusted_min_collateral
                ),
                risk_level: RiskLevel::High,
                semantic_basis: vec![format!(
                    "Semantic risk multiplier: {:.2}x",
                    semantic_multiplier
                )],
                required_actions: vec!["Deposit more collateral".to_string()],
            };
        }

        // Check protocol liquidity
        let available_liquidity = protocol_state
            .total_deposits
            .saturating_sub(protocol_state.total_borrows);
        if amount > available_liquidity {
            return SemanticAuthorizationResult {
                authorized: false,
                reason: format!(
                    "Insufficient protocol liquidity: {} > {}",
                    amount, available_liquidity
                ),
                risk_level: RiskLevel::High,
                semantic_basis,
                required_actions: vec!["Wait for more deposits or borrow less".to_string()],
            };
        }

        semantic_basis.push(format!(
            "Semantic-adjusted collateral ratio: {:.2}",
            collateral_ratio
        ));

        SemanticAuthorizationResult {
            authorized: true,
            reason: format!(
                "Borrow authorized with collateral ratio {:.2}",
                collateral_ratio
            ),
            risk_level: RiskLevel::Low,
            semantic_basis,
            required_actions,
        }
    }

    /// Authorize repay with semantic context
    fn authorize_repay(
        &self,
        _protocol_state: &DeFiProtocolState,
        user_position: Option<&UserPosition>,
        amount: u64,
        _config: &ProtocolConfig,
    ) -> SemanticAuthorizationResult {
        let user_pos = match user_position {
            Some(pos) => pos,
            None => {
                return SemanticAuthorizationResult {
                    authorized: false,
                    reason: "No outstanding loans".to_string(),
                    risk_level: RiskLevel::Low,
                    semantic_basis: vec!["User has no position".to_string()],
                    required_actions: vec![],
                };
            }
        };

        // Repayment is generally always allowed (up to outstanding balance)
        let repay_amount = std::cmp::min(amount, user_pos.borrowed);

        SemanticAuthorizationResult {
            authorized: true,
            reason: format!(
                "Repay authorized: {} (capped at outstanding balance)",
                repay_amount
            ),
            risk_level: RiskLevel::Low,
            semantic_basis: vec!["Repayment reduces risk".to_string()],
            required_actions: vec![],
        }
    }

    /// Calculate semantic risk multiplier based on protocol state
    fn calculate_semantic_risk_multiplier(
        &self,
        protocol_state: &DeFiProtocolState,
        config: &ProtocolConfig,
    ) -> f64 {
        let mut multiplier = 1.0;

        // Analyze each semantic context
        for (field, semantics) in &protocol_state.semantic_context {
            if let Some(field_multiplier) = config.semantic_overrides.get(&semantics.zero_meaning) {
                match field.as_str() {
                    "total_deposits" => multiplier *= field_multiplier,
                    "total_borrows" => multiplier *= field_multiplier * 0.5, // Less impact
                    "user_balances" => multiplier *= field_multiplier * 0.8, // Moderate impact
                    _ => {}
                }
            }
        }

        multiplier
    }

    /// Assess overall protocol risk based on semantic context
    fn assess_protocol_risk(&self, protocol_state: &DeFiProtocolState) -> (RiskLevel, Vec<String>) {
        let mut risk_factors = vec![];
        let mut max_risk = RiskLevel::Low;

        // Analyze semantic contexts
        for (field, semantics) in &protocol_state.semantic_context {
            if semantics.has_conflict() {
                risk_factors.push(format!(
                    "Semantic conflict in {}: declared {:?} vs validated {:?}",
                    field, semantics.declared_semantics, semantics.validated_semantics
                ));
                max_risk = RiskLevel::High;
            }

            match (&semantics.zero_meaning, field.as_str()) {
                (ZeroSemantics::NeverWritten, "total_deposits") => {
                    risk_factors.push("Protocol never initialized - high risk".to_string());
                    max_risk = RiskLevel::High;
                }
                (ZeroSemantics::Cleared, "total_deposits") => {
                    risk_factors.push("Protocol deposits were cleared - investigate".to_string());
                    if max_risk == RiskLevel::Low {
                        max_risk = RiskLevel::Medium;
                    }
                }
                (ZeroSemantics::Cleared, "user_balances") => {
                    risk_factors.push("User balances were cleared - verify integrity".to_string());
                    if max_risk == RiskLevel::Low {
                        max_risk = RiskLevel::Medium;
                    }
                }
                _ => {}
            }
        }

        // Check utilization rate
        let utilization = if protocol_state.total_deposits > 0 {
            protocol_state.total_borrows as f64 / protocol_state.total_deposits as f64
        } else {
            0.0
        };

        if utilization > 0.9 {
            risk_factors.push(format!(
                "High utilization rate: {:.1}%",
                utilization * 100.0
            ));
            max_risk = RiskLevel::Critical;
        } else if utilization > 0.8 {
            risk_factors.push(format!(
                "Elevated utilization rate: {:.1}%",
                utilization * 100.0
            ));
            if max_risk < RiskLevel::Medium {
                max_risk = RiskLevel::Medium;
            }
        }

        (max_risk, risk_factors)
    }
}

/// Demonstrates semantic business logic in DeFi scenarios
fn demonstrate_semantic_business_logic() -> Result<(), Box<dyn std::error::Error>> {
    println!("Semantic Business Logic Integration Example");
    println!("==========================================");
    println!();

    let business_logic = SemanticBusinessLogic::new();

    // Scenario 1: New Protocol (Never Written)
    println!("Scenario 1: New Protocol Deployment");
    println!("-----------------------------------");

    let mut protocol_state = DeFiProtocolState {
        name: "NewProtocol".to_string(),
        total_deposits: 0,
        total_borrows: 0,
        user_positions: HashMap::new(),
        semantic_context: HashMap::new(),
    };

    protocol_state.semantic_context.insert(
        "total_deposits".to_string(),
        StorageSemantics::new(ZeroSemantics::NeverWritten),
    );

    // Try to authorize first deposit
    let auth_result =
        business_logic.authorize_user_action(&protocol_state, "user1", "deposit", 1000);

    println!("First deposit authorization:");
    println!("   • Authorized: {}", auth_result.authorized);
    println!("   • Reason: {}", auth_result.reason);
    println!("   • Risk Level: {:?}", auth_result.risk_level);
    println!("   • Semantic Basis: {:?}", auth_result.semantic_basis);
    println!("   • Required Actions: {:?}", auth_result.required_actions);
    println!();

    // Scenario 2: Initialized Protocol (Explicitly Zero)
    println!("Scenario 2: Initialized Protocol");
    println!("--------------------------------");

    let mut protocol_state2 = DeFiProtocolState {
        name: "InitializedProtocol".to_string(),
        total_deposits: 0,
        total_borrows: 0,
        user_positions: HashMap::new(),
        semantic_context: HashMap::new(),
    };

    protocol_state2.semantic_context.insert(
        "total_deposits".to_string(),
        StorageSemantics::new(ZeroSemantics::ExplicitlyZero),
    );

    let auth_result2 =
        business_logic.authorize_user_action(&protocol_state2, "user1", "deposit", 1000);

    println!("Deposit to initialized protocol:");
    println!("   • Authorized: {}", auth_result2.authorized);
    println!("   • Reason: {}", auth_result2.reason);
    println!("   • Risk Level: {:?}", auth_result2.risk_level);
    println!("   • Semantic Basis: {:?}", auth_result2.semantic_basis);
    println!();

    // Scenario 3: Protocol with Conflicts
    println!("Scenario 3: Protocol with Semantic Conflicts");
    println!("--------------------------------------------");

    let mut protocol_state3 = DeFiProtocolState {
        name: "ConflictProtocol".to_string(),
        total_deposits: 0,
        total_borrows: 0,
        user_positions: HashMap::new(),
        semantic_context: HashMap::new(),
    };

    protocol_state3.semantic_context.insert(
        "total_deposits".to_string(),
        StorageSemantics::with_validation(ZeroSemantics::NeverWritten, ZeroSemantics::Cleared),
    );

    let (risk_level, risk_factors) = business_logic.assess_protocol_risk(&protocol_state3);

    println!("Protocol risk assessment:");
    println!("   • Risk Level: {:?}", risk_level);
    println!("   • Risk Factors: {:?}", risk_factors);
    println!();

    // Scenario 4: User Borrowing with Semantic Context
    println!("Scenario 4: User Borrowing with Semantic Risk Assessment");
    println!("--------------------------------------------------------");

    let mut protocol_state4 = DeFiProtocolState {
        name: "LendingProtocol".to_string(),
        total_deposits: 10000,
        total_borrows: 2000,
        user_positions: HashMap::new(),
        semantic_context: HashMap::new(),
    };

    // User has deposited before
    protocol_state4.user_positions.insert(
        "user1".to_string(),
        UserPosition {
            deposited: 1000,
            borrowed: 0,
            collateral_ratio: 0.0,
            semantic_flags: vec!["verified".to_string()],
        },
    );

    protocol_state4.semantic_context.insert(
        "total_deposits".to_string(),
        StorageSemantics::new(ZeroSemantics::ExplicitlyZero),
    );

    protocol_state4.semantic_context.insert(
        "user_balances".to_string(),
        StorageSemantics::with_validation(ZeroSemantics::NeverWritten, ZeroSemantics::Cleared),
    );

    let borrow_auth =
        business_logic.authorize_user_action(&protocol_state4, "user1", "borrow", 500);

    println!("Borrow authorization with semantic conflicts:");
    println!("   • Authorized: {}", borrow_auth.authorized);
    println!("   • Reason: {}", borrow_auth.reason);
    println!("   • Risk Level: {:?}", borrow_auth.risk_level);
    println!("   • Semantic Basis: {:?}", borrow_auth.semantic_basis);
    println!("   • Required Actions: {:?}", borrow_auth.required_actions);
    println!();

    // Scenario 5: Withdrawal from Cleared Account
    println!("Scenario 5: Withdrawal from Cleared Account");
    println!("-------------------------------------------");

    let mut protocol_state5 = DeFiProtocolState {
        name: "ClearedProtocol".to_string(),
        total_deposits: 5000,
        total_borrows: 1000,
        user_positions: HashMap::new(),
        semantic_context: HashMap::new(),
    };

    protocol_state5.semantic_context.insert(
        "user_balances".to_string(),
        StorageSemantics::new(ZeroSemantics::Cleared),
    );

    let withdraw_auth =
        business_logic.authorize_user_action(&protocol_state5, "user1", "withdraw", 100);

    println!("Withdrawal from cleared account:");
    println!("   • Authorized: {}", withdraw_auth.authorized);
    println!("   • Reason: {}", withdraw_auth.reason);
    println!("   • Risk Level: {:?}", withdraw_auth.risk_level);
    println!("   • Semantic Basis: {:?}", withdraw_auth.semantic_basis);
    println!();

    // Semantic Business Logic Summary
    println!("Semantic Business Logic Summary");
    println!("==============================");
    println!("Never Written: Higher risk multiplier (2.0x) for uninitialized protocols");
    println!("Explicitly Zero: Normal risk (1.0x) for properly initialized systems");
    println!("Cleared: Elevated risk (1.5x) for systems that were active but cleared");
    println!("Valid Zero: Normal risk (1.0x) for operational zero states");
    println!("Conflicts: Automatic risk escalation when declared ≠ validated");
    println!("Context-aware: Different risk weights for different protocol components");
    println!();
    println!("Integration Points:");
    println!("• Authorization systems use semantic context for risk assessment");
    println!("• Collateral requirements adjusted based on semantic risk multipliers");
    println!("• Protocol health monitoring includes semantic conflict detection");
    println!("• Business logic adapts to semantic meanings of zero values");
    println!("• User actions guided by semantic-aware risk assessment");

    Ok(())
}

/// Create semantic layout for DeFi protocol
fn create_defi_semantic_layout() -> LayoutInfo {
    let mut layout = LayoutInfo {
        contract_name: "DeFiProtocol".to_string(),
        storage: Vec::new(),
        types: Vec::new(),
    };

    // Protocol-level storage
    layout.storage.push(StorageEntry {
        label: "totalDeposits".to_string(),
        slot: "0".to_string(),
        offset: 0,
        type_name: "t_uint256".to_string(),
        zero_semantics: ZeroSemantics::ExplicitlyZero, // Initialized to zero
    });

    layout.storage.push(StorageEntry {
        label: "totalBorrows".to_string(),
        slot: "1".to_string(),
        offset: 0,
        type_name: "t_uint256".to_string(),
        zero_semantics: ZeroSemantics::ExplicitlyZero, // Initialized to zero
    });

    // User-level storage
    layout.storage.push(StorageEntry {
        label: "userBalances".to_string(),
        slot: "2".to_string(),
        offset: 0,
        type_name: "t_mapping(t_address,t_uint256)".to_string(),
        zero_semantics: ZeroSemantics::NeverWritten, // Most users never deposit
    });

    layout.storage.push(StorageEntry {
        label: "userBorrows".to_string(),
        slot: "3".to_string(),
        offset: 0,
        type_name: "t_mapping(t_address,t_uint256)".to_string(),
        zero_semantics: ZeroSemantics::NeverWritten, // Most users never borrow
    });

    // System state
    layout.storage.push(StorageEntry {
        label: "paused".to_string(),
        slot: "4".to_string(),
        offset: 0,
        type_name: "t_bool".to_string(),
        zero_semantics: ZeroSemantics::ExplicitlyZero, // Initialized to false
    });

    layout.storage.push(StorageEntry {
        label: "lastUpdateTimestamp".to_string(),
        slot: "5".to_string(),
        offset: 0,
        type_name: "t_uint256".to_string(),
        zero_semantics: ZeroSemantics::NeverWritten, // Only set when first used
    });

    // Add type definitions
    layout.types.push(TypeInfo {
        label: "t_uint256".to_string(),
        number_of_bytes: "32".to_string(),
        encoding: "inplace".to_string(),
        base: None,
        key: None,
        value: None,
    });

    layout.types.push(TypeInfo {
        label: "t_address".to_string(),
        number_of_bytes: "20".to_string(),
        encoding: "inplace".to_string(),
        base: None,
        key: None,
        value: None,
    });

    layout.types.push(TypeInfo {
        label: "t_bool".to_string(),
        number_of_bytes: "1".to_string(),
        encoding: "inplace".to_string(),
        base: None,
        key: None,
        value: None,
    });

    layout.types.push(TypeInfo {
        label: "t_mapping(t_address,t_uint256)".to_string(),
        number_of_bytes: "32".to_string(),
        encoding: "mapping".to_string(),
        base: None,
        key: Some("t_address".to_string()),
        value: Some("t_uint256".to_string()),
    });

    layout
}

/// Main function demonstrating semantic business logic
fn main() -> Result<(), Box<dyn std::error::Error>> {
    demonstrate_semantic_business_logic()?;

    println!();
    println!("DeFi Protocol Semantic Layout");
    println!("=============================");

    let layout = create_defi_semantic_layout();
    println!(
        "Protocol: {} (commitment: 0x{})",
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
    println!("CLI Usage for DeFi Protocol:");
    println!("traverse generate-proof \\");
    println!("  --contract 0xDeFiProtocolAddress \\");
    println!("  --query \"userBalances[0x742d35Cc6aB8B23c0532C65C6b555f09F9d40894]\" \\");
    println!("  --zero-means never_written \\");
    println!("  --validate-semantics \\");
    println!("  --business-logic defi \\");
    println!("  --rpc-url $ETHEREUM_RPC_URL");

    Ok(())
}
