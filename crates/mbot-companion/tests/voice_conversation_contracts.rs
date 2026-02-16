//! Contract tests for Voice Conversation System
//!
//! Enforces VCONV-001..010 contracts from docs/contracts/feature_voice_conversation.yml
//!
//! These tests scan source code as text to verify structural invariants.
//! They run without any hardware or API keys.

use std::fs;
use std::path::{Path, PathBuf};

fn workspace_root() -> PathBuf {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let mut dir = Path::new(manifest_dir);
    loop {
        let cargo_toml = dir.join("Cargo.toml");
        if cargo_toml.exists() {
            if let Ok(contents) = fs::read_to_string(&cargo_toml) {
                if contents.contains("[workspace]") {
                    return dir.to_path_buf();
                }
            }
        }
        dir = dir
            .parent()
            .expect("CONTRACT TEST SETUP: Could not find workspace root");
    }
}

fn read_source(relative_path: &str) -> String {
    let root = workspace_root();
    let path = root.join(relative_path);
    fs::read_to_string(&path).unwrap_or_else(|e| {
        panic!(
            "CONTRACT TEST SETUP: Could not read '{}': {}",
            path.display(),
            e
        )
    })
}

// ============================================================
// VCONV-001: R2-D2 voice uses ONLY play_tone() API
// ============================================================

#[test]
fn vconv_001_robot_speak_uses_play_tone() {
    let source = read_source("crates/mbot-companion/src/transport.rs");

    assert!(
        source.contains("robot_speak"),
        "\n\nCONTRACT VIOLATION: VCONV-001\n\
         File: crates/mbot-companion/src/transport.rs\n\
         Issue: Missing robot_speak() method.\n\
         Transport must have a robot_speak function for R2-D2 voice.\n\
         \n\
         See: docs/contracts/feature_voice_conversation.yml (VCONV-001)\n"
    );
}

#[test]
fn vconv_001_robot_speak_contains_play_tone() {
    let source = read_source("crates/mbot-companion/src/transport.rs");

    // Find the robot_speak function and check it uses play_tone
    if let Some(start) = source.find("fn robot_speak") {
        let after = &source[start..];
        let fn_end = after[1..]
            .find("\n    pub ")
            .or_else(|| after[1..].find("\n    fn "))
            .or_else(|| after[1..].find("\n    // ==="))
            .map(|pos| pos + 1)
            .unwrap_or(after.len());
        let fn_body = &after[..fn_end];

        assert!(
            fn_body.contains("play_tone"),
            "\n\nCONTRACT VIOLATION: VCONV-001\n\
             File: crates/mbot-companion/src/transport.rs\n\
             Issue: robot_speak() does not use play_tone().\n\
             R2-D2 voice must use ONLY cyberpi.audio.play_tone() API.\n\
             \n\
             See: docs/contracts/feature_voice_conversation.yml (VCONV-001)\n"
        );
    }
    // If robot_speak doesn't exist yet, vconv_001_robot_speak_uses_play_tone catches it
}

// ============================================================
// VCONV-008: Tones synchronized with display text
// ============================================================

#[test]
fn vconv_008_robot_speak_updates_display() {
    let source = read_source("crates/mbot-companion/src/transport.rs");

    if let Some(start) = source.find("fn robot_speak") {
        let after = &source[start..];
        let fn_end = after[1..]
            .find("\n    pub ")
            .or_else(|| after[1..].find("\n    fn "))
            .or_else(|| after[1..].find("\n    // ==="))
            .map(|pos| pos + 1)
            .unwrap_or(after.len());
        let fn_body = &after[..fn_end];

        assert!(
            fn_body.contains("console.println") || fn_body.contains("display_print_script"),
            "\n\nCONTRACT VIOLATION: VCONV-008\n\
             File: crates/mbot-companion/src/transport.rs\n\
             Issue: robot_speak() does not update CyberPi display.\n\
             Tones must be synchronized with display text.\n\
             \n\
             See: docs/contracts/feature_voice_conversation.yml (VCONV-008)\n"
        );
    }
}

// ============================================================
// VCONV-004: Voice commands pass through SafetyFilter
// ============================================================

#[test]
fn vconv_004_safety_filter_exists() {
    let root = workspace_root();
    let safety_path = root.join("crates/mbot-companion/src/brain/planner/safety.rs");

    assert!(
        safety_path.exists(),
        "\n\nCONTRACT VIOLATION: VCONV-004\n\
         File: crates/mbot-companion/src/brain/planner/safety.rs\n\
         Issue: SafetyFilter module does not exist.\n\
         Voice commands must pass through SafetyFilter before execution.\n\
         \n\
         See: docs/contracts/feature_voice_conversation.yml (VCONV-004)\n"
    );
}

// ============================================================
// VCONV-005: API keys from env vars only
// ============================================================

#[test]
fn vconv_005_no_hardcoded_api_keys_in_transport() {
    let source = read_source("crates/mbot-companion/src/transport.rs");

    let forbidden = ["sk-", "ELEVENLABS_API_KEY=\"", "OPENAI_API_KEY=\""];
    for pattern in &forbidden {
        assert!(
            !source.contains(pattern),
            "\n\nCONTRACT VIOLATION: VCONV-005\n\
             File: crates/mbot-companion/src/transport.rs\n\
             Issue: Found hardcoded API key pattern '{}'.\n\
             API keys must come from env vars only.\n\
             \n\
             See: docs/contracts/feature_voice_conversation.yml (VCONV-005)\n",
            pattern
        );
    }
}

// ============================================================
// VCONV-009: Motor commands from voice clamped [-100, 100]
// ============================================================

#[test]
fn vconv_009_safety_filter_clamps_motors() {
    let root = workspace_root();
    let safety_path = root.join("crates/mbot-companion/src/brain/planner/safety.rs");

    if safety_path.exists() {
        let source = fs::read_to_string(&safety_path).unwrap();
        assert!(
            source.contains("clamp") || source.contains("-100") || source.contains("100"),
            "\n\nCONTRACT VIOLATION: VCONV-009\n\
             File: crates/mbot-companion/src/brain/planner/safety.rs\n\
             Issue: SafetyFilter does not clamp motor speeds.\n\
             Motor commands from voice must be clamped to [-100, 100].\n\
             \n\
             See: docs/contracts/feature_voice_conversation.yml (VCONV-009)\n"
        );
    }
    // If safety.rs doesn't exist yet, vconv_004 catches it
}

// ============================================================
// VCONV-001 (continued): No raw audio streaming to CyberPi
// ============================================================

#[test]
fn vconv_001_no_raw_audio_streaming() {
    let source = read_source("crates/mbot-companion/src/transport.rs");

    let forbidden = ["write_audio", "stream_audio", "send_wav", "send_mp3", "play_wav"];
    for pattern in &forbidden {
        assert!(
            !source.contains(pattern),
            "\n\nCONTRACT VIOLATION: VCONV-001\n\
             File: crates/mbot-companion/src/transport.rs\n\
             Issue: Found raw audio pattern '{}' in transport.\n\
             R2-D2 voice must use ONLY play_tone(), no raw audio streaming.\n\
             \n\
             See: docs/contracts/feature_voice_conversation.yml (VCONV-001)\n",
            pattern
        );
    }
}
