//! E2E tests for preset personalities (STORY-PERS-003, issue #18)
//!
//! These tests verify that all preset personalities are valid, distinct,
//! and pass the Kitchen Table Test.

use mbot_core::personality::presets::ExtendedPreset;

/// Test that all preset personalities are valid and can be loaded
#[test]
fn test_all_presets_valid() {
    for preset in ExtendedPreset::all() {
        let personality = preset.to_personality();
        assert!(
            personality.validate().is_ok(),
            "Preset {:?} should be valid",
            preset
        );
    }
}

/// Test that each preset has a unique icon
#[test]
fn test_presets_have_unique_icons() {
    let presets = ExtendedPreset::all();
    let mut icons: Vec<char> = presets.iter().map(|p| p.icon()).collect();
    icons.sort_unstable();
    icons.dedup();

    assert_eq!(
        icons.len(),
        presets.len(),
        "All presets should have unique icons"
    );
}

/// Test that each preset has a non-empty description
#[test]
fn test_presets_have_descriptions() {
    for preset in ExtendedPreset::all() {
        let desc = preset.description();
        assert!(!desc.is_empty(), "Preset {:?} should have a description", preset);
    }
}

/// Test that each preset has a defining trait (I-PERS-008)
#[test]
fn test_presets_have_defining_traits() {
    for preset in ExtendedPreset::all() {
        let trait_desc = preset.defining_trait();
        assert!(
            !trait_desc.is_empty(),
            "Preset {:?} should have a defining trait",
            preset
        );
    }
}

/// Test that Curious Cleo has high curiosity drive
#[test]
fn test_curious_cleo_characteristics() {
    let personality = ExtendedPreset::Curious.to_personality();
    assert!(
        personality.curiosity_drive() >= 0.9,
        "Curious Cleo should have curiosity_drive >= 0.9"
    );
    assert_eq!(personality.name, "Curious Cleo");
    assert!(personality.quirks.contains(&"investigate_all".to_string()));
}

/// Test that Nervous Nellie has high startle sensitivity
#[test]
fn test_nervous_nellie_characteristics() {
    let personality = ExtendedPreset::Timid.to_personality();
    assert!(
        personality.startle_sensitivity() >= 0.9,
        "Nervous Nellie should have startle_sensitivity >= 0.9"
    );
    assert!(personality.tension_baseline() >= 0.7);
    assert!(personality.quirks.contains(&"back_up_when_scared".to_string()));
}

/// Test that Chill Charlie has low reactivity
#[test]
fn test_chill_charlie_characteristics() {
    let personality = ExtendedPreset::Zen.to_personality();
    assert!(
        personality.startle_sensitivity() <= 0.2,
        "Chill Charlie should have low startle sensitivity"
    );
    assert!(personality.coherence_baseline() >= 0.7);
    assert!(personality.recovery_speed() >= 0.9);
}

/// Test that Bouncy Betty has high energy
#[test]
fn test_bouncy_betty_characteristics() {
    let personality = ExtendedPreset::Excitable.to_personality();
    assert!(
        personality.energy_baseline() >= 0.9,
        "Bouncy Betty should have energy_baseline >= 0.9"
    );
    assert!(personality.movement_expressiveness() >= 0.95);
    assert!(personality.quirks.contains(&"spin_when_happy".to_string()));
}

/// Test that Grumpy Gus has low coherence and reluctance
#[test]
fn test_grumpy_gus_characteristics() {
    let personality = ExtendedPreset::Grumpy.to_personality();
    assert!(
        personality.coherence_baseline() <= 0.2,
        "Grumpy Gus should have low coherence"
    );
    assert!(personality.quirks.contains(&"reluctant_participation".to_string()));
}

/// Test that presets are balanced (I-PERS-009)
/// No single personality should dominate all positive traits
#[test]
fn test_presets_are_balanced() {
    let presets = ExtendedPreset::all();

    // Count how many "high" values each preset has
    for preset in presets {
        let p = preset.to_personality();

        let high_count = [
            p.tension_baseline() > 0.7,
            p.coherence_baseline() > 0.7,
            p.energy_baseline() > 0.7,
            p.startle_sensitivity() > 0.7,
            p.recovery_speed() > 0.7,
            p.curiosity_drive() > 0.7,
            p.movement_expressiveness() > 0.7,
            p.sound_expressiveness() > 0.7,
            p.light_expressiveness() > 0.7,
        ]
        .iter()
        .filter(|&&x| x)
        .count();

        // No preset should have more than 6 out of 9 parameters at high values
        assert!(
            high_count <= 6,
            "Preset {:?} has too many high values ({}), violating balance",
            preset,
            high_count
        );
    }
}

/// Test that presets are distinguishable within 30 seconds (I-PERS-007)
/// This is a smoke test - real validation would require simulation
#[test]
fn test_presets_distinguishable() {
    let presets = ExtendedPreset::all();

    // Each preset should have at least one parameter that's significantly different
    // from the default (0.5)
    for preset in presets {
        let p = preset.to_personality();

        let has_extreme = [
            (p.tension_baseline() - 0.5).abs() > 0.2,
            (p.coherence_baseline() - 0.5).abs() > 0.2,
            (p.energy_baseline() - 0.5).abs() > 0.2,
            (p.startle_sensitivity() - 0.5).abs() > 0.2,
            (p.recovery_speed() - 0.5).abs() > 0.2,
            (p.curiosity_drive() - 0.5).abs() > 0.2,
            (p.movement_expressiveness() - 0.5).abs() > 0.2,
            (p.sound_expressiveness() - 0.5).abs() > 0.2,
            (p.light_expressiveness() - 0.5).abs() > 0.2,
        ]
        .iter()
        .any(|&x| x);

        assert!(
            has_extreme,
            "Preset {:?} is too close to neutral (not distinguishable)",
            preset
        );
    }
}

/// Kitchen Table Test: All presets should have safe values
#[test]
fn test_kitchen_table_safety() {
    for preset in ExtendedPreset::all() {
        let p = preset.to_personality();

        // All parameters should be within safe bounds
        assert!(
            p.tension_baseline() >= 0.0 && p.tension_baseline() <= 1.0,
            "Preset {:?} has unsafe tension_baseline",
            preset
        );
        assert!(
            p.coherence_baseline() >= 0.0 && p.coherence_baseline() <= 1.0,
            "Preset {:?} has unsafe coherence_baseline",
            preset
        );
        assert!(
            p.energy_baseline() >= 0.0 && p.energy_baseline() <= 1.0,
            "Preset {:?} has unsafe energy_baseline",
            preset
        );

        // Validation should pass
        assert!(
            p.validate().is_ok(),
            "Preset {:?} failed Kitchen Table Test validation",
            preset
        );
    }
}

/// Test that presets can be serialized and deserialized
#[test]
fn test_presets_serialization() {
    for preset in ExtendedPreset::all() {
        let original = preset.to_personality();
        let json = original.to_json().expect("Should serialize");
        let restored = mbot_core::personality::Personality::from_json(&json)
            .expect("Should deserialize");

        assert_eq!(original.id, restored.id);
        assert_eq!(original.name, restored.name);
        assert!((original.tension_baseline() - restored.tension_baseline()).abs() < 0.001);
        assert_eq!(original.quirks, restored.quirks);
    }
}
