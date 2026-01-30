//! E2E tests for quirks system (STORY-PERS-006, issue #26)
//!
//! These tests verify that quirks trigger correctly, respect cooldowns,
//! and can coexist without conflicts.

use mbot_core::personality::quirks::{Quirk, QuirkConfig, QuirkEngine};

/// Test that all quirks have valid default configurations
#[test]
fn test_all_quirks_have_default_configs() {
    for quirk in Quirk::all() {
        let config = QuirkConfig::default_for(*quirk);
        assert_eq!(config.quirk, *quirk);
        assert!(
            config.activation_chance >= 0.0 && config.activation_chance <= 1.0,
            "Quirk {:?} has invalid activation chance",
            quirk
        );
    }
}

/// Test quirk engine can add and remove quirks
#[test]
fn test_quirk_engine_add_remove() {
    let mut engine = QuirkEngine::new();

    // Add quirks
    engine.add_quirk(QuirkConfig::default_for(Quirk::RandomSigh));
    engine.add_quirk(QuirkConfig::default_for(Quirk::SpinWhenHappy));
    assert_eq!(engine.active_quirks().len(), 2);

    // Remove quirk
    engine.remove_quirk(Quirk::RandomSigh);
    assert_eq!(engine.active_quirks().len(), 1);

    // Clear all
    engine.clear();
    assert_eq!(engine.active_quirks().len(), 0);
}

/// Test that quirks respect cooldown periods (I-PERS-018)
#[test]
fn test_quirk_cooldowns() {
    let mut engine = QuirkEngine::new();
    let quirk = Quirk::RandomSigh;
    engine.add_quirk(QuirkConfig::default_for(quirk));

    let start_time = 1000;

    // Not on cooldown initially
    assert!(!engine.is_on_cooldown(quirk, start_time));

    // Record activation
    engine.record_activation(quirk, start_time);

    // On cooldown immediately after
    assert!(engine.is_on_cooldown(quirk, start_time + 1));
    assert!(engine.is_on_cooldown(quirk, start_time + 5000));

    // Off cooldown after cooldown period
    let cooldown = quirk.default_cooldown_ms();
    assert!(!engine.is_on_cooldown(quirk, start_time + cooldown + 1));
}

/// Test that multiple quirks can coexist (I-PERS-019)
#[test]
fn test_multiple_quirks_coexist() {
    let mut engine = QuirkEngine::new();

    // Add multiple quirks
    engine.add_quirk(QuirkConfig::default_for(Quirk::RandomSigh));
    engine.add_quirk(QuirkConfig::default_for(Quirk::SpinWhenHappy));
    engine.add_quirk(QuirkConfig::default_for(Quirk::BackUpWhenScared));

    // All should be active
    assert_eq!(engine.active_quirks().len(), 3);

    // Each should have independent cooldowns
    engine.record_activation(Quirk::RandomSigh, 1000);
    assert!(engine.is_on_cooldown(Quirk::RandomSigh, 1001));
    assert!(!engine.is_on_cooldown(Quirk::SpinWhenHappy, 1001));
    assert!(!engine.is_on_cooldown(Quirk::BackUpWhenScared, 1001));
}

/// Test that safety-related quirks have no cooldown (I-PERS-016)
#[test]
fn test_safety_quirk_no_cooldown() {
    let quirk = Quirk::BackUpWhenScared;
    let config = QuirkConfig::default_for(quirk);

    assert_eq!(
        config.cooldown_ms, 0,
        "BackUpWhenScared is safety-related and should have 0 cooldown"
    );

    let mut engine = QuirkEngine::new();
    engine.add_quirk(config);

    // Can activate repeatedly without cooldown
    engine.record_activation(quirk, 1000);
    assert!(!engine.is_on_cooldown(quirk, 1001));
    engine.record_activation(quirk, 1001);
    assert!(!engine.is_on_cooldown(quirk, 1002));
}

/// Test quirk string parsing
#[test]
fn test_quirk_from_string() {
    assert_eq!(Quirk::from_str("random_sigh"), Some(Quirk::RandomSigh));
    assert_eq!(Quirk::from_str("spin_when_happy"), Some(Quirk::SpinWhenHappy));
    assert_eq!(Quirk::from_str("invalid_quirk"), None);
}

/// Test quirk roundtrip (to_str -> from_str)
#[test]
fn test_quirk_roundtrip() {
    for quirk in Quirk::all() {
        let str_repr = quirk.to_str();
        let parsed = Quirk::from_str(str_repr);
        assert_eq!(
            parsed,
            Some(*quirk),
            "Roundtrip failed for {:?}",
            quirk
        );
    }
}

/// Test that activation chances are configurable (I-PERS-017)
#[test]
fn test_configurable_activation_chance() {
    let mut config = QuirkConfig::default_for(Quirk::RandomSigh);

    // Default is 0.15
    assert!((config.activation_chance - 0.15).abs() < 0.001);

    // Can be modified
    config.activation_chance = 0.5;
    assert!((config.activation_chance - 0.5).abs() < 0.001);
}

/// Test that quirks have descriptions
#[test]
fn test_quirk_descriptions() {
    for quirk in Quirk::all() {
        let desc = quirk.description();
        assert!(
            !desc.is_empty(),
            "Quirk {:?} should have a description",
            quirk
        );
    }
}

/// Test SpinWhenHappy quirk configuration
#[test]
fn test_spin_when_happy_config() {
    let config = QuirkConfig::default_for(Quirk::SpinWhenHappy);

    // Should trigger on high coherence
    assert!(matches!(
        config.trigger,
        mbot_core::personality::quirks::QuirkTrigger::StateThreshold { .. }
    ));

    // Should have reasonable activation chance
    assert!(config.activation_chance >= 0.1 && config.activation_chance <= 0.5);

    // Should have a cooldown
    assert!(config.cooldown_ms > 0);
}

/// Test BackUpWhenScared quirk configuration
#[test]
fn test_back_up_when_scared_config() {
    let config = QuirkConfig::default_for(Quirk::BackUpWhenScared);

    // Should trigger on sudden stimulus
    assert!(matches!(
        config.trigger,
        mbot_core::personality::quirks::QuirkTrigger::Stimulus { .. }
    ));

    // Should have high activation chance (almost always)
    assert!(config.activation_chance >= 0.8);

    // Should have no cooldown (safety)
    assert_eq!(config.cooldown_ms, 0);
}

/// Test NightOwl quirk configuration
#[test]
fn test_night_owl_config() {
    let config = QuirkConfig::default_for(Quirk::NightOwl);

    // Should trigger on low light environment
    assert!(matches!(
        config.trigger,
        mbot_core::personality::quirks::QuirkTrigger::Environment { .. }
    ));

    // Should have 100% activation (always active when condition met)
    assert!((config.activation_chance - 1.0).abs() < 0.001);

    // Should have no cooldown (continuous modifier)
    assert_eq!(config.cooldown_ms, 0);
}

/// Test ChaseTail quirk configuration
#[test]
fn test_chase_tail_config() {
    let config = QuirkConfig::default_for(Quirk::ChaseTail);

    // Should trigger on idle
    assert!(matches!(
        config.trigger,
        mbot_core::personality::quirks::QuirkTrigger::Idle { .. }
    ));

    // Should have low activation chance (rare behavior)
    assert!(config.activation_chance <= 0.2);

    // Should have long cooldown
    assert!(config.cooldown_ms >= 20_000);
}

/// Test that quirks don't duplicate in engine
#[test]
fn test_no_duplicate_quirks() {
    let mut engine = QuirkEngine::new();

    engine.add_quirk(QuirkConfig::default_for(Quirk::RandomSigh));
    engine.add_quirk(QuirkConfig::default_for(Quirk::RandomSigh));
    engine.add_quirk(QuirkConfig::default_for(Quirk::RandomSigh));

    // Should only have one instance
    assert_eq!(engine.active_quirks().len(), 1);
}

/// Test add_quirk_from_str convenience method
#[test]
fn test_add_quirk_from_str() {
    let mut engine = QuirkEngine::new();

    engine.add_quirk_from_str("random_sigh");
    engine.add_quirk_from_str("spin_when_happy");
    assert_eq!(engine.active_quirks().len(), 2);

    // Invalid quirk should be ignored
    engine.add_quirk_from_str("invalid_quirk");
    assert_eq!(engine.active_quirks().len(), 2);
}

/// Test that personalities can have quirks assigned
#[test]
fn test_personality_with_quirks() {
    let personality = mbot_core::personality::Personality::builder()
        .id("quirky")
        .name("Quirky")
        .quirk("spin_when_happy")
        .quirk("random_sigh")
        .quirk("chase_tail")
        .build()
        .expect("Should build");

    assert_eq!(personality.quirks.len(), 3);
    assert!(personality.quirks.contains(&"spin_when_happy".to_string()));
    assert!(personality.quirks.contains(&"random_sigh".to_string()));
    assert!(personality.quirks.contains(&"chase_tail".to_string()));
}

/// Test loading personality quirks into engine
#[test]
fn test_load_personality_quirks_into_engine() {
    let personality = mbot_core::personality::presets::ExtendedPreset::Curious.to_personality();

    let mut engine = QuirkEngine::new();
    for quirk_str in &personality.quirks {
        engine.add_quirk_from_str(quirk_str);
    }

    // Curious Cleo should have at least 2 quirks
    assert!(engine.active_quirks().len() >= 2);
}

/// Integration test: Preset personalities with quirks
#[test]
fn test_presets_with_quirks() {
    use mbot_core::personality::presets::ExtendedPreset;

    // Curious Cleo should have "investigate_all" quirk
    let cleo = ExtendedPreset::Curious.to_personality();
    assert!(cleo.quirks.contains(&"investigate_all".to_string()));

    // Grumpy Gus should have "reluctant_participation" quirk
    let gus = ExtendedPreset::Grumpy.to_personality();
    assert!(gus.quirks.contains(&"reluctant_participation".to_string()));

    // Bouncy Betty should have "spin_when_happy" quirk
    let betty = ExtendedPreset::Excitable.to_personality();
    assert!(betty.quirks.contains(&"spin_when_happy".to_string()));
}

/// Test that continuous modifier quirks (NightOwl, EarlyBird) work correctly
#[test]
fn test_continuous_modifier_quirks() {
    let night_owl = QuirkConfig::default_for(Quirk::NightOwl);
    let early_bird = QuirkConfig::default_for(Quirk::EarlyBird);

    // Both should have no cooldown (continuous)
    assert_eq!(night_owl.cooldown_ms, 0);
    assert_eq!(early_bird.cooldown_ms, 0);

    // Both should always activate when condition is met
    assert!((night_owl.activation_chance - 1.0).abs() < 0.001);
    assert!((early_bird.activation_chance - 1.0).abs() < 0.001);

    // Both should be parameter modifiers
    assert!(matches!(
        night_owl.behavior,
        mbot_core::personality::quirks::QuirkBehavior::ParameterModifier { .. }
    ));
    assert!(matches!(
        early_bird.behavior,
        mbot_core::personality::quirks::QuirkBehavior::ParameterModifier { .. }
    ));
}
