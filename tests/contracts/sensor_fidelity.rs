//! Contract tests for Hardware Sensor Fidelity
//!
//! Enforces SENSOR-001..006 contracts from docs/contracts/feature_sensors.yml
//!
//! These tests read the source code as text and assert that forbidden patterns
//! are absent and required patterns are present. This prevents regressions
//! where real hardware connections silently return simulated data.

use std::fs;
use std::path::{Path, PathBuf};

/// Find the workspace root by walking up from CARGO_MANIFEST_DIR until we find
/// a Cargo.toml containing [workspace]. This works regardless of which crate
/// compiles the test.
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
            .expect("CONTRACT TEST SETUP: Could not find workspace root (no [workspace] Cargo.toml found)");
    }
}

/// Helper: read a source file relative to the workspace root.
fn read_source(relative_path: &str) -> String {
    let root = workspace_root();
    let path = root.join(relative_path);
    fs::read_to_string(&path).unwrap_or_else(|e| {
        panic!(
            "CONTRACT TEST SETUP: Could not read '{}': {}. \
             This file must exist for sensor fidelity contracts.",
            path.display(),
            e
        )
    })
}

/// Extract the Serial match arm block from the read_sensors function.
/// Looks for the pattern: TransportInner::Serial ... => { ... }
/// and returns the text of that block.
fn extract_serial_arm(source: &str) -> String {
    // Find read_sensors function first
    let read_sensors_start = source
        .find("fn read_sensors")
        .expect("CONTRACT TEST SETUP: read_sensors() function not found in transport.rs");

    let after_read_sensors = &source[read_sensors_start..];

    // Find the Serial match arm within read_sensors
    let serial_marker = "TransportInner::Serial";
    let serial_pos = after_read_sensors
        .find(serial_marker)
        .expect("CONTRACT TEST SETUP: TransportInner::Serial arm not found in read_sensors()");

    let after_serial = &after_read_sensors[serial_pos..];

    // Find the opening brace of the match arm body
    let body_start = after_serial
        .find("=> {")
        .or_else(|| after_serial.find("=>{"))
        .expect("CONTRACT TEST SETUP: Could not find Serial match arm body");

    let body_content = &after_serial[body_start..];

    // Track brace depth to find the end of this match arm
    let mut depth = 0;
    let mut end = 0;
    for (i, ch) in body_content.char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    end = i + 1;
                    break;
                }
            }
            _ => {}
        }
    }

    if end == 0 {
        panic!("CONTRACT TEST SETUP: Could not find end of Serial match arm body");
    }

    body_content[..end].to_string()
}

/// Extract the Bluetooth match arm block from the read_sensors function.
fn extract_bluetooth_arm(source: &str) -> String {
    let read_sensors_start = source
        .find("fn read_sensors")
        .expect("CONTRACT TEST SETUP: read_sensors() function not found in transport.rs");

    let after_read_sensors = &source[read_sensors_start..];

    let bt_marker = "TransportInner::Bluetooth";
    let bt_pos = after_read_sensors
        .find(bt_marker)
        .expect("CONTRACT TEST SETUP: TransportInner::Bluetooth arm not found in read_sensors()");

    let after_bt = &after_read_sensors[bt_pos..];

    let body_start = after_bt
        .find("=> {")
        .or_else(|| after_bt.find("=>{"))
        .expect("CONTRACT TEST SETUP: Could not find Bluetooth match arm body");

    let body_content = &after_bt[body_start..];

    let mut depth = 0;
    let mut end = 0;
    for (i, ch) in body_content.char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    end = i + 1;
                    break;
                }
            }
            _ => {}
        }
    }

    if end == 0 {
        panic!("CONTRACT TEST SETUP: Could not find end of Bluetooth match arm body");
    }

    body_content[..end].to_string()
}

// ============================================================
// SENSOR-001: Serial transport must use protocol commands,
//             not read_simulated() or Default::default()
// ============================================================

#[test]
fn sensor_001_serial_must_not_call_read_simulated() {
    let source = read_source("crates/mbot-companion/src/transport.rs");
    let serial_arm = extract_serial_arm(&source);

    assert!(
        !serial_arm.contains("read_simulated"),
        "\n\nCONTRACT VIOLATION: SENSOR-001\n\
         File: crates/mbot-companion/src/transport.rs\n\
         Issue: Serial match arm in read_sensors() calls read_simulated().\n\
         When connected to real hardware via serial, ALL sensor data must come\n\
         from protocol commands, not from simulated values.\n\
         \n\
         Found 'read_simulated' in Serial arm:\n{}\n\
         \n\
         See: docs/contracts/feature_sensors.yml (SENSOR-001)\n",
        serial_arm
    );
}

#[test]
fn sensor_001_serial_must_not_use_default_for_sensors() {
    let source = read_source("crates/mbot-companion/src/transport.rs");
    let serial_arm = extract_serial_arm(&source);

    assert!(
        !serial_arm.contains("..Default::default()"),
        "\n\nCONTRACT VIOLATION: SENSOR-001\n\
         File: crates/mbot-companion/src/transport.rs\n\
         Issue: Serial match arm uses '..Default::default()' to fill MBotSensors.\n\
         This means some sensor fields get zero/default values instead of real\n\
         hardware readings. Every field must be populated from a protocol command.\n\
         \n\
         Found '..Default::default()' in Serial arm:\n{}\n\
         \n\
         See: docs/contracts/feature_sensors.yml (SENSOR-001)\n",
        serial_arm
    );
}

// ============================================================
// SENSOR-002: Bluetooth must NOT silently fall back to simulated
// ============================================================

#[test]
fn sensor_002_bluetooth_must_not_call_read_simulated() {
    let source = read_source("crates/mbot-companion/src/transport.rs");
    let bt_arm = extract_bluetooth_arm(&source);

    assert!(
        !bt_arm.contains("read_simulated"),
        "\n\nCONTRACT VIOLATION: SENSOR-002\n\
         File: crates/mbot-companion/src/transport.rs\n\
         Issue: Bluetooth match arm in read_sensors() calls read_simulated().\n\
         Users connecting via Bluetooth expect real sensor data. The Bluetooth\n\
         transport must either read from the BLE notification characteristic\n\
         or return an explicit error -- never silently fake data.\n\
         \n\
         Found 'read_simulated' in Bluetooth arm:\n{}\n\
         \n\
         See: docs/contracts/feature_sensors.yml (SENSOR-002)\n",
        bt_arm
    );
}

// ============================================================
// SENSOR-003: No hardcoded magic-number fallbacks
// ============================================================

#[test]
fn sensor_003_no_hardcoded_fallback_100() {
    let source = read_source("crates/mbot-companion/src/transport.rs");

    assert!(
        !source.contains("unwrap_or(100.0)"),
        "\n\nCONTRACT VIOLATION: SENSOR-003\n\
         File: crates/mbot-companion/src/transport.rs\n\
         Issue: Found 'unwrap_or(100.0)' -- hardcoded fallback hides sensor failures.\n\
         Use last-known-good values instead and log a warning.\n\
         \n\
         See: docs/contracts/feature_sensors.yml (SENSOR-003)\n"
    );
}

#[test]
fn sensor_003_no_hardcoded_fallback_05() {
    let source = read_source("crates/mbot-companion/src/transport.rs");

    assert!(
        !source.contains("unwrap_or(0.5)"),
        "\n\nCONTRACT VIOLATION: SENSOR-003\n\
         File: crates/mbot-companion/src/transport.rs\n\
         Issue: Found 'unwrap_or(0.5)' -- hardcoded fallback hides sensor failures.\n\
         Use last-known-good values instead and log a warning.\n\
         \n\
         See: docs/contracts/feature_sensors.yml (SENSOR-003)\n"
    );
}

// ============================================================
// SENSOR-005: Protocol must have read commands for ALL sensors
// ============================================================

#[test]
fn sensor_005_protocol_has_read_ultrasonic_cmd() {
    let source = read_source("crates/mbot-companion/src/protocol.rs");

    assert!(
        source.contains("fn read_ultrasonic_cmd"),
        "\n\nCONTRACT VIOLATION: SENSOR-005\n\
         File: crates/mbot-companion/src/protocol.rs\n\
         Issue: Missing read_ultrasonic_cmd() function.\n\
         Protocol must define read commands for all sensor types.\n\
         \n\
         See: docs/contracts/feature_sensors.yml (SENSOR-005)\n"
    );
}

#[test]
fn sensor_005_protocol_has_read_sound_cmd() {
    let source = read_source("crates/mbot-companion/src/protocol.rs");

    assert!(
        source.contains("fn read_sound_cmd"),
        "\n\nCONTRACT VIOLATION: SENSOR-005\n\
         File: crates/mbot-companion/src/protocol.rs\n\
         Issue: Missing read_sound_cmd() function.\n\
         Protocol must define read commands for all sensor types:\n\
         ultrasonic, sound, light, gyro, quad RGB.\n\
         \n\
         See: docs/contracts/feature_sensors.yml (SENSOR-005)\n"
    );
}

#[test]
fn sensor_005_protocol_has_read_light_cmd() {
    let source = read_source("crates/mbot-companion/src/protocol.rs");

    assert!(
        source.contains("fn read_light_cmd"),
        "\n\nCONTRACT VIOLATION: SENSOR-005\n\
         File: crates/mbot-companion/src/protocol.rs\n\
         Issue: Missing read_light_cmd() function.\n\
         Protocol must define read commands for all sensor types:\n\
         ultrasonic, sound, light, gyro, quad RGB.\n\
         \n\
         See: docs/contracts/feature_sensors.yml (SENSOR-005)\n"
    );
}

#[test]
fn sensor_005_protocol_has_read_gyro_cmd() {
    let source = read_source("crates/mbot-companion/src/protocol.rs");

    assert!(
        source.contains("fn read_gyro_cmd"),
        "\n\nCONTRACT VIOLATION: SENSOR-005\n\
         File: crates/mbot-companion/src/protocol.rs\n\
         Issue: Missing read_gyro_cmd() function.\n\
         Protocol must define read commands for all sensor types:\n\
         ultrasonic, sound, light, gyro, quad RGB.\n\
         \n\
         See: docs/contracts/feature_sensors.yml (SENSOR-005)\n"
    );
}

#[test]
fn sensor_005_protocol_has_read_quad_rgb_cmd() {
    let source = read_source("crates/mbot-companion/src/protocol.rs");

    assert!(
        source.contains("fn read_quad_rgb_cmd"),
        "\n\nCONTRACT VIOLATION: SENSOR-005\n\
         File: crates/mbot-companion/src/protocol.rs\n\
         Issue: Missing read_quad_rgb_cmd() function.\n\
         Protocol must define read commands for all sensor types:\n\
         ultrasonic, sound, light, gyro, quad RGB.\n\
         \n\
         See: docs/contracts/feature_sensors.yml (SENSOR-005)\n"
    );
}

// ============================================================
// SENSOR-001 (continued): Transport must CALL all protocol
//                          read functions for serial connections
// ============================================================

#[test]
fn sensor_001_transport_calls_read_ultrasonic() {
    let source = read_source("crates/mbot-companion/src/transport.rs");

    assert!(
        source.contains("read_ultrasonic_cmd"),
        "\n\nCONTRACT VIOLATION: SENSOR-001\n\
         File: crates/mbot-companion/src/transport.rs\n\
         Issue: transport.rs never calls protocol::read_ultrasonic_cmd().\n\
         The serial transport must send read commands for ALL sensors.\n\
         \n\
         See: docs/contracts/feature_sensors.yml (SENSOR-001)\n"
    );
}

#[test]
fn sensor_001_transport_calls_read_sound() {
    let source = read_source("crates/mbot-companion/src/transport.rs");

    assert!(
        source.contains("read_sound_cmd"),
        "\n\nCONTRACT VIOLATION: SENSOR-001\n\
         File: crates/mbot-companion/src/transport.rs\n\
         Issue: transport.rs never calls protocol::read_sound_cmd().\n\
         The serial transport must send read commands for ALL sensors:\n\
         ultrasonic, sound, light, gyro, quad RGB.\n\
         \n\
         See: docs/contracts/feature_sensors.yml (SENSOR-001)\n"
    );
}

#[test]
fn sensor_001_transport_calls_read_light() {
    let source = read_source("crates/mbot-companion/src/transport.rs");

    assert!(
        source.contains("read_light_cmd"),
        "\n\nCONTRACT VIOLATION: SENSOR-001\n\
         File: crates/mbot-companion/src/transport.rs\n\
         Issue: transport.rs never calls protocol::read_light_cmd().\n\
         The serial transport must send read commands for ALL sensors:\n\
         ultrasonic, sound, light, gyro, quad RGB.\n\
         \n\
         See: docs/contracts/feature_sensors.yml (SENSOR-001)\n"
    );
}

#[test]
fn sensor_001_transport_calls_read_gyro() {
    let source = read_source("crates/mbot-companion/src/transport.rs");

    assert!(
        source.contains("read_gyro_cmd"),
        "\n\nCONTRACT VIOLATION: SENSOR-001\n\
         File: crates/mbot-companion/src/transport.rs\n\
         Issue: transport.rs never calls protocol::read_gyro_cmd().\n\
         The serial transport must send read commands for ALL sensors:\n\
         ultrasonic, sound, light, gyro, quad RGB.\n\
         \n\
         See: docs/contracts/feature_sensors.yml (SENSOR-001)\n"
    );
}

#[test]
fn sensor_001_transport_calls_read_quad_rgb() {
    let source = read_source("crates/mbot-companion/src/transport.rs");

    assert!(
        source.contains("read_quad_rgb_cmd"),
        "\n\nCONTRACT VIOLATION: SENSOR-001\n\
         File: crates/mbot-companion/src/transport.rs\n\
         Issue: transport.rs never calls protocol::read_quad_rgb_cmd().\n\
         The serial transport must send read commands for ALL sensors:\n\
         ultrasonic, sound, light, gyro, quad RGB.\n\
         \n\
         See: docs/contracts/feature_sensors.yml (SENSOR-001)\n"
    );
}

// ============================================================
// SENSOR-001: Verify read_simulated() is ONLY called from
//             TransportInner::Simulated, nowhere else
// ============================================================

#[test]
fn sensor_001_read_simulated_only_from_simulated_arm() {
    let source = read_source("crates/mbot-companion/src/transport.rs");

    // Count how many times read_simulated appears in the read_sensors function
    let read_sensors_start = source
        .find("fn read_sensors")
        .expect("read_sensors not found");

    // Find the end of read_sensors by looking for the next top-level fn
    let after_fn = &source[read_sensors_start..];
    let fn_end = after_fn[1..]
        .find("\n    pub ")
        .or_else(|| after_fn[1..].find("\n    fn "))
        .map(|pos| pos + 1)
        .unwrap_or(after_fn.len());

    let read_sensors_body = &after_fn[..fn_end];

    // Count occurrences of read_simulated in read_sensors
    let sim_count = read_sensors_body.matches("read_simulated").count();

    // There should be at most 1 call (from the Simulated arm)
    assert!(
        sim_count <= 1,
        "\n\nCONTRACT VIOLATION: SENSOR-001 / SENSOR-002\n\
         File: crates/mbot-companion/src/transport.rs\n\
         Issue: read_simulated() is called {} times in read_sensors().\n\
         It should only be called once, from TransportInner::Simulated.\n\
         Serial and Bluetooth arms must use protocol commands instead.\n\
         \n\
         See: docs/contracts/feature_sensors.yml\n",
        sim_count
    );
}
