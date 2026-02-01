//! Contract tests for AI Learning (feature_ai_learning.yml)
//!
//! Tests enforcement of invariants I-AI-001 through I-AI-007

use std::fs;
use std::path::Path;

/// Test I-AI-001: Learning rates must be bounded
#[test]
fn test_i_ai_001_bounded_learning_rates() {
    let prediction_rs = fs::read_to_string(
        "crates/mbot-core/src/learning/prediction.rs"
    ).expect("Could not read prediction.rs");

    // Check that we use clamp() for confidence/probability values
    assert!(
        prediction_rs.contains(".clamp(0.0, 1.0)"),
        "I-AI-001: Learning parameters must be bounded with clamp()"
    );
}

/// Test I-AI-003: Storage must be bounded
#[test]
fn test_i_ai_003_bounded_storage() {
    let prediction_rs = fs::read_to_string(
        "crates/mbot-core/src/learning/prediction.rs"
    ).expect("Could not read prediction.rs");

    // Check for max_history_size and VecDeque with capacity
    assert!(
        prediction_rs.contains("max_history_size") || prediction_rs.contains("max_patterns"),
        "I-AI-003: Must have storage size limits"
    );

    assert!(
        prediction_rs.contains("VecDeque::with_capacity"),
        "I-AI-003: Should use bounded collections"
    );

    // Check for LRU eviction logic
    assert!(
        prediction_rs.contains("pop_front") || prediction_rs.contains("remove"),
        "I-AI-003: Must have eviction strategy for bounded storage"
    );
}

/// Test I-AI-004: Predictions must include reasoning
#[test]
fn test_i_ai_004_observable_predictions() {
    let prediction_rs = fs::read_to_string(
        "crates/mbot-core/src/learning/prediction.rs"
    ).expect("Could not read prediction.rs");

    // Check for reasoning field in Prediction struct
    assert!(
        prediction_rs.contains("reasoning: String"),
        "I-AI-004: Predictions must have reasoning field"
    );

    // Check for confidence field
    assert!(
        prediction_rs.contains("confidence: f32"),
        "I-AI-004: Predictions must have confidence scores"
    );
}

/// Test I-AI-005: Minimum confidence threshold for proactive actions
#[test]
fn test_i_ai_005_confidence_threshold() {
    let prediction_rs = fs::read_to_string(
        "crates/mbot-core/src/learning/prediction.rs"
    ).expect("Could not read prediction.rs");

    // Check for min_confidence field
    assert!(
        prediction_rs.contains("min_confidence"),
        "I-AI-005: Must have confidence threshold setting"
    );

    // Check for confidence comparison before acting
    assert!(
        prediction_rs.contains("confidence >= self.settings.min_confidence") ||
        prediction_rs.contains("confidence < self.settings.min_confidence"),
        "I-AI-005: Must check confidence threshold before actions"
    );

    // Check default threshold is 0.7 (70%)
    assert!(
        prediction_rs.contains("min_confidence: 0.7"),
        "I-AI-005: Default confidence threshold should be 0.7 (70%)"
    );
}

/// Test I-AI-006: Minimum observations required for patterns
#[test]
fn test_i_ai_006_min_observations() {
    let prediction_rs = fs::read_to_string(
        "crates/mbot-core/src/learning/prediction.rs"
    ).expect("Could not read prediction.rs");

    // Check for min_observations field
    assert!(
        prediction_rs.contains("min_observations"),
        "I-AI-006: Must have minimum observations setting"
    );

    // Check for observations field in Pattern
    assert!(
        prediction_rs.contains("observations: u32"),
        "I-AI-006: Patterns must track observation count"
    );

    // Check for observation threshold check
    assert!(
        prediction_rs.contains("observations >= self.settings.min_observations") ||
        prediction_rs.contains("observations >= settings.min_observations"),
        "I-AI-006: Must check minimum observations before using patterns"
    );

    // Check default minimum is 10
    assert!(
        prediction_rs.contains("min_observations: 10"),
        "I-AI-006: Default minimum observations should be 10"
    );
}

/// Test I-AI-007: User can disable predictions and clear data
#[test]
fn test_i_ai_007_user_control() {
    let prediction_rs = fs::read_to_string(
        "crates/mbot-core/src/learning/prediction.rs"
    ).expect("Could not read prediction.rs");

    // Check for enabled toggle
    assert!(
        prediction_rs.contains("enabled: bool"),
        "I-AI-007: Must have enabled/disabled toggle"
    );

    // Check for enabled check before predictions
    assert!(
        prediction_rs.contains("if !self.settings.enabled") ||
        prediction_rs.contains("if self.settings.enabled"),
        "I-AI-007: Must check if predictions are enabled"
    );

    // Check for clear_all_data function
    assert!(
        prediction_rs.contains("clear_all_data") ||
        prediction_rs.contains("clear") && prediction_rs.contains("patterns"),
        "I-AI-007: Must provide method to clear all data"
    );
}

/// Test contract file exists
#[test]
fn test_contract_file_exists() {
    assert!(
        Path::new("docs/contracts/feature_ai_learning.yml").exists(),
        "AI learning contract file must exist"
    );
}

/// Test contract is registered in index
#[test]
fn test_contract_registered() {
    let index = fs::read_to_string(
        "docs/contracts/CONTRACT_INDEX.yml"
    ).expect("Could not read CONTRACT_INDEX.yml");

    assert!(
        index.contains("feature_ai_learning"),
        "AI learning contract must be registered in CONTRACT_INDEX.yml"
    );

    // Check all invariants are covered
    for invariant in &["I-AI-001", "I-AI-002", "I-AI-003", "I-AI-004", "I-AI-005", "I-AI-006", "I-AI-007"] {
        assert!(
            index.contains(invariant),
            "Invariant {} must be in CONTRACT_INDEX.yml", invariant
        );
    }
}

#[test]
fn test_no_std_compatibility() {
    let prediction_rs = fs::read_to_string(
        "crates/mbot-core/src/learning/prediction.rs"
    ).expect("Could not read prediction.rs");

    // Should not directly use std:: (except in cfg blocks)
    let std_uses: Vec<&str> = prediction_rs
        .lines()
        .filter(|line| line.contains("use std::") && !line.contains("#[cfg"))
        .collect();

    // Check that std is only used in cfg(not(feature = "no_std")) blocks
    for line in std_uses {
        let context: Vec<&str> = prediction_rs
            .lines()
            .collect();

        // Find this line in context
        if let Some(idx) = context.iter().position(|l| *l == line) {
            // Check a few lines before for cfg attribute
            let has_cfg = (idx.saturating_sub(5)..idx)
                .any(|i| context.get(i)
                     .map(|l| l.contains("cfg(not(feature = \"no_std\"))"))
                     .unwrap_or(false));

            assert!(
                has_cfg,
                "std:: usage must be guarded by cfg(not(feature = \"no_std\")): {}",
                line
            );
        }
    }
}
