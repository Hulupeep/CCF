//! E2E tests for personality persistence (STORY-PERS-005, issue #23)
//!
//! These tests verify that personalities can be saved and loaded
//! across restarts with data integrity.

use mbot_core::personality::{Personality, persistence::PersonalityStorage};
use std::fs;
use std::path::PathBuf;

fn test_storage_path(test_name: &str) -> PathBuf {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let counter = COUNTER.fetch_add(1, Ordering::SeqCst);

    let mut path = std::env::temp_dir();
    path.push(format!("mbot_e2e_personality_{}_{}.json", test_name, counter));
    path
}

/// Test that personality survives restart (simulated)
#[test]
fn test_personality_survives_restart() {
    let path = test_storage_path("restart");
    let storage = PersonalityStorage::new(path.clone());

    // Configure as "Bouncy Betty"
    let original = mbot_core::personality::presets::ExtendedPreset::Excitable.to_personality();

    // Save
    storage.save(&original).expect("Should save");
    assert!(storage.exists());

    // Simulate restart by creating new storage instance
    let storage2 = PersonalityStorage::new(path.clone());

    // Load
    let loaded = storage2.load().expect("Should load").expect("File should exist");

    // Verify it's still Bouncy Betty
    assert_eq!(loaded.id, original.id);
    assert_eq!(loaded.name, original.name);
    assert!((loaded.energy_baseline() - original.energy_baseline()).abs() < 0.001);

    // Cleanup
    fs::remove_file(&path).ok();
}

/// Test that personality is saved when changed
#[test]
fn test_save_on_change() {
    let path = test_storage_path("save_on_change");
    let storage = PersonalityStorage::new(path.clone());

    // Start as Curious Cleo
    let cleo = mbot_core::personality::presets::ExtendedPreset::Curious.to_personality();
    storage.save(&cleo).expect("Should save");

    // Switch to Nervous Nellie
    let nellie = mbot_core::personality::presets::ExtendedPreset::Timid.to_personality();
    storage.save(&nellie).expect("Should save");

    // Load should return Nervous Nellie
    let loaded = storage.load().expect("Should load").expect("File should exist");
    assert_eq!(loaded.id, "preset-timid");
    assert_eq!(loaded.name, "Nervous Nellie");

    // Cleanup
    fs::remove_file(&path).ok();
}

/// Test loading personality on startup
#[test]
fn test_load_on_startup() {
    let path = test_storage_path("startup");
    let storage = PersonalityStorage::new(path.clone());

    // Pre-save Grumpy Gus
    let gus = mbot_core::personality::presets::ExtendedPreset::Grumpy.to_personality();
    storage.save(&gus).expect("Should save");

    // Simulate startup
    let storage2 = PersonalityStorage::new(path.clone());
    let loaded = storage2.load().expect("Should load").expect("File should exist");

    // Should be Grumpy Gus
    assert_eq!(loaded.id, "preset-grumpy");
    assert_eq!(loaded.name, "Grumpy Gus");
    assert!((loaded.coherence_baseline() - 0.2).abs() < 0.001);

    // Cleanup
    fs::remove_file(&path).ok();
}

/// Test handling of missing personality file
#[test]
fn test_handle_missing_file() {
    let path = test_storage_path("missing");
    let storage = PersonalityStorage::new(path.clone());

    // No file exists
    assert!(!storage.exists());

    // Load should return None
    let result = storage.load().expect("Should not error");
    assert!(result.is_none());

    // load_or_default should return default
    let loaded = storage.load_or_default();
    assert_eq!(loaded.id, "default");
}

/// Test handling of corrupt personality file
#[test]
fn test_handle_corrupt_file() {
    let path = test_storage_path("corrupt");
    let storage = PersonalityStorage::new(path.clone());

    // Write corrupt JSON
    fs::write(&path, "{ this is not valid json }").expect("Should write");

    // Load should fail gracefully
    let result = storage.load();
    assert!(result.is_err());

    // load_or_default should return default and backup corrupt file
    let loaded = storage.load_or_default();
    assert_eq!(loaded.id, "default");

    // Backup should exist
    let backup_path = path.with_extension("corrupt.bak");
    assert!(backup_path.exists());

    // Cleanup
    fs::remove_file(&path).ok();
    fs::remove_file(&backup_path).ok();
}

/// Test preservation of custom parameter modifications
#[test]
fn test_preserve_custom_parameters() {
    let path = test_storage_path("custom");
    let storage = PersonalityStorage::new(path.clone());

    // Start with Curious Cleo
    let mut cleo = mbot_core::personality::presets::ExtendedPreset::Curious.to_personality();

    // Modify curiosity_drive
    cleo.set_curiosity_drive(0.5).expect("Should set");

    // Save
    storage.save(&cleo).expect("Should save");

    // Load
    let loaded = storage.load().expect("Should load").expect("File should exist");

    // curiosity_drive should be 0.5, not the original 0.9
    assert!((loaded.curiosity_drive() - 0.5).abs() < 0.001);

    // Other parameters should remain unchanged
    assert_eq!(loaded.id, "preset-curious");
    assert_eq!(loaded.name, "Curious Cleo");

    // Cleanup
    fs::remove_file(&path).ok();
}

/// Test that quirks are preserved across saves/loads
#[test]
fn test_preserve_quirks() {
    let path = test_storage_path("quirks");
    let storage = PersonalityStorage::new(path.clone());

    // Create personality with quirks
    let personality = Personality::builder()
        .id("quirky")
        .name("Quirky")
        .quirk("spin_when_happy")
        .quirk("random_sigh")
        .quirk("chase_tail")
        .build()
        .expect("Should build");

    storage.save(&personality).expect("Should save");

    // Load
    let loaded = storage.load().expect("Should load").expect("File should exist");

    // All quirks should be preserved
    assert_eq!(loaded.quirks.len(), 3);
    assert!(loaded.quirks.contains(&"spin_when_happy".to_string()));
    assert!(loaded.quirks.contains(&"random_sigh".to_string()));
    assert!(loaded.quirks.contains(&"chase_tail".to_string()));

    // Cleanup
    fs::remove_file(&path).ok();
}

/// Test file version migration (forward compatibility)
#[test]
fn test_version_compatibility() {
    let path = test_storage_path("version");
    let storage = PersonalityStorage::new(path.clone());

    // Save with current version
    let personality = Personality::default();
    storage.save(&personality).expect("Should save");

    // Read and verify version is correct
    let json = fs::read_to_string(&path).expect("Should read");
    assert!(json.contains("\"version\":1"));

    // Load should succeed
    let loaded = storage.load().expect("Should load");
    assert!(loaded.is_some());

    // Cleanup
    fs::remove_file(&path).ok();
}

/// Test checksum validation prevents tampering (I-PERS-013)
#[test]
fn test_checksum_validation() {
    let path = test_storage_path("checksum");
    let storage = PersonalityStorage::new(path.clone());

    // Save personality
    let personality = Personality::builder()
        .id("test")
        .name("Test")
        .tension_baseline(0.3)
        .build()
        .expect("Should build");

    storage.save(&personality).expect("Should save");

    // Tamper with the file
    let mut json = fs::read_to_string(&path).expect("Should read");
    json = json.replace("\"tension_baseline\":0.3", "\"tension_baseline\":0.9");
    fs::write(&path, json).expect("Should write");

    // Load should fail due to checksum mismatch
    let result = storage.load();
    assert!(result.is_err());

    // Cleanup
    fs::remove_file(&path).ok();
}

/// Power cycle simulation test
#[test]
fn test_power_cycle_simulation() {
    let path = test_storage_path("power_cycle");

    // Cycle 1: Boot, configure, save
    {
        let storage = PersonalityStorage::new(path.clone());
        let bouncy = mbot_core::personality::presets::ExtendedPreset::Excitable.to_personality();
        storage.save(&bouncy).expect("Should save");
    }

    // Cycle 2: Boot, load, modify, save
    {
        let storage = PersonalityStorage::new(path.clone());
        let mut loaded = storage.load().expect("Should load").expect("File should exist");
        assert_eq!(loaded.name, "Bouncy Betty");

        loaded.set_energy_baseline(0.7).expect("Should set");
        storage.save(&loaded).expect("Should save");
    }

    // Cycle 3: Boot, load, verify
    {
        let storage = PersonalityStorage::new(path.clone());
        let loaded = storage.load().expect("Should load").expect("File should exist");
        assert_eq!(loaded.name, "Bouncy Betty");
        assert!((loaded.energy_baseline() - 0.7).abs() < 0.001);
    }

    // Cleanup
    fs::remove_file(&path).ok();
}
