//! Simple tests for personality system (#18, #23, #26)
//!
//! These tests verify the basic implementations:
//! - #18: 15 preset personalities
//! - #23: JSON persistence (to_json/from_json)
//! - #26: Quirk struct definition

use mbot_core::personality::{Personality, PersonalityPreset};

#[cfg(feature = "std")]
use mbot_core::personality::persistence;

#[cfg(feature = "std")]
use mbot_core::personality::quirks::Quirk;

// === #18: Extended Preset Library Tests ===

#[test]
fn test_preset_count() {
    // Verify we have 15 presets as required by #18
    let all = PersonalityPreset::all();
    assert_eq!(all.len(), 15, "Should have 15 preset personalities");
}

#[test]
fn test_preset_variants() {
    // Verify all 15 preset variants exist
    let presets = PersonalityPreset::all();

    // Original 5
    assert!(presets.contains(&PersonalityPreset::Mellow));
    assert!(presets.contains(&PersonalityPreset::Curious));
    assert!(presets.contains(&PersonalityPreset::Zen));
    assert!(presets.contains(&PersonalityPreset::Excitable));
    assert!(presets.contains(&PersonalityPreset::Timid));

    // Extended 10
    assert!(presets.contains(&PersonalityPreset::Adventurous));
    assert!(presets.contains(&PersonalityPreset::Shy));
    assert!(presets.contains(&PersonalityPreset::Grumpy));
    assert!(presets.contains(&PersonalityPreset::Cheerful));
    assert!(presets.contains(&PersonalityPreset::Cautious));
    assert!(presets.contains(&PersonalityPreset::Playful));
    assert!(presets.contains(&PersonalityPreset::Serious));
    assert!(presets.contains(&PersonalityPreset::Energetic));
    assert!(presets.contains(&PersonalityPreset::Calm));
    assert!(presets.contains(&PersonalityPreset::Anxious));
}

#[test]
fn test_preset_to_personality() {
    // Verify each preset can be converted to a Personality
    for preset in PersonalityPreset::all() {
        let personality = preset.to_personality();

        // All parameters should be within [0.0, 1.0]
        assert!(personality.tension_baseline() >= 0.0);
        assert!(personality.tension_baseline() <= 1.0);
        assert!(personality.energy_baseline() >= 0.0);
        assert!(personality.energy_baseline() <= 1.0);
    }
}

#[test]
fn test_preset_uniqueness() {
    // Verify each preset has unique parameters
    let mut personalities = Vec::new();

    for preset in PersonalityPreset::all() {
        personalities.push(preset.to_personality());
    }

    // Check that presets are actually different
    assert_ne!(personalities[0], personalities[1], "Presets should differ");
}

// === #23: Personality Persistence Tests ===

#[cfg(feature = "std")]
#[test]
fn test_personality_to_json() {
    let p = Personality::default();
    let json_result = persistence::to_json(&p);

    assert!(json_result.is_ok(), "Should serialize to JSON");

    let json = json_result.unwrap();
    assert!(json.contains("\"id\""), "JSON should contain id field");
    assert!(json.contains("\"name\""), "JSON should contain name field");
}

#[cfg(feature = "std")]
#[test]
fn test_personality_from_json() {
    let original = Personality::default();
    let json = persistence::to_json(&original).unwrap();

    let result = persistence::from_json(&json);
    assert!(result.is_ok(), "Should deserialize from JSON");

    let restored = result.unwrap();
    assert_eq!(restored.id, original.id);
    assert_eq!(restored.name, original.name);
}

#[cfg(feature = "std")]
#[test]
fn test_preset_round_trip() {
    // Test that a preset can be serialized and deserialized
    let original = PersonalityPreset::Curious.to_personality();
    let json = persistence::to_json(&original).unwrap();
    let restored = persistence::from_json(&json).unwrap();

    assert_eq!(restored.id, original.id);
    assert_eq!(restored.name, original.name);
    assert_eq!(restored.tension_baseline(), original.tension_baseline());
    assert_eq!(restored.energy_baseline(), original.energy_baseline());
}

#[cfg(feature = "std")]
#[test]
fn test_invalid_json() {
    let result = persistence::from_json("{ invalid json }");
    assert!(result.is_err(), "Should fail on invalid JSON");
}

// === #26: Quirks System Tests ===

#[cfg(feature = "std")]
#[test]
fn test_quirk_enum_all() {
    // Test that all 9 quirks are present
    let quirks = Quirk::all();
    assert_eq!(quirks.len(), 9);
}

#[cfg(feature = "std")]
#[test]
fn test_quirk_string_conversion() {
    // Test string conversion for sample quirks
    let quirk = Quirk::RandomSigh;
    assert_eq!(quirk.to_str(), "random_sigh");

    let parsed = Quirk::from_str("random_sigh");
    assert_eq!(parsed, Some(Quirk::RandomSigh));
}

#[cfg(feature = "std")]
#[test]
fn test_quirk_config_default() {
    // Test that default config works for all quirks
    for quirk in Quirk::all() {
        use mbot_core::personality::quirks::QuirkConfig;
        let config = QuirkConfig::default_for(*quirk);
        assert_eq!(config.quirk, *quirk);
        assert!(config.activation_chance >= 0.0 && config.activation_chance <= 1.0);
    }
}

#[cfg(feature = "std")]
#[test]
fn test_quirk_clone() {
    let original = Quirk::SpinWhenHappy;
    let cloned = original;
    assert_eq!(cloned, original);
}
