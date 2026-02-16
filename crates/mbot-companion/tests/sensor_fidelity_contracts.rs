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

/// Extract the serial sensor-reading code. Checks both the read_sensors()
/// dispatch arm and the read_sensors_serial() function body (CyberPi
/// HalocodeProtocol uses dispatch to dedicated functions).
fn extract_serial_sensor_code(source: &str) -> String {
    let mut result = String::new();

    // Capture the read_sensors() dispatch arm for Serial
    if let Some(start) = source.find("fn read_sensors") {
        let after = &source[start..];
        if let Some(serial_pos) = after.find("TransportInner::Serial") {
            // Grab 200 chars after the match to capture the arm
            let arm_start = start + serial_pos;
            let arm_end = (arm_start + 200).min(source.len());
            result.push_str(&source[arm_start..arm_end]);
            result.push('\n');
        }
    }

    // Capture the read_sensors_serial() function body
    if let Some(start) = source.find("fn read_sensors_serial") {
        let end = source[start + 1..]
            .find("\n    // ===")
            .or_else(|| source[start + 1..].find("\n    pub "))
            .or_else(|| source[start + 1..].find("\n    fn "))
            .map(|pos| start + 1 + pos)
            .unwrap_or(source.len());
        result.push_str(&source[start..end]);
    }

    result
}

/// Extract the Bluetooth sensor-reading code. Checks both the read_sensors()
/// dispatch arm and the read_sensors_ble() function body.
fn extract_bluetooth_sensor_code(source: &str) -> String {
    let mut result = String::new();

    // Capture the read_sensors() dispatch arm for Bluetooth
    if let Some(start) = source.find("fn read_sensors") {
        let after = &source[start..];
        if let Some(bt_pos) = after.find("TransportInner::Bluetooth") {
            let arm_start = start + bt_pos;
            let arm_end = (arm_start + 200).min(source.len());
            result.push_str(&source[arm_start..arm_end]);
            result.push('\n');
        }
    }

    // Capture the read_sensors_ble() function body
    if let Some(start) = source.find("fn read_sensors_ble") {
        let end = source[start + 1..]
            .find("\n    // ===")
            .or_else(|| source[start + 1..].find("\n    pub "))
            .or_else(|| source[start + 1..].find("\n    fn "))
            .map(|pos| start + 1 + pos)
            .unwrap_or(source.len());
        result.push_str(&source[start..end]);
    }

    result
}

// ============================================================
// SENSOR-001: Serial transport must use protocol commands,
//             not read_simulated() or Default::default()
// ============================================================

#[test]
fn sensor_001_serial_must_not_call_read_simulated() {
    let source = read_source("crates/mbot-companion/src/transport.rs");
    let serial_code = extract_serial_sensor_code(&source);

    assert!(
        !serial_code.contains("read_simulated"),
        "\n\nCONTRACT VIOLATION: SENSOR-001\n\
         File: crates/mbot-companion/src/transport.rs\n\
         Issue: Serial match arm in read_sensors() calls read_simulated().\n\
         When connected to real hardware via serial, ALL sensor data must come\n\
         from protocol commands, not from simulated values.\n\
         \n\
         Found 'read_simulated' in Serial arm:\n{}\n\
         \n\
         See: docs/contracts/feature_sensors.yml (SENSOR-001)\n",
        serial_code
    );
}

#[test]
fn sensor_001_serial_must_not_use_default_for_sensors() {
    let source = read_source("crates/mbot-companion/src/transport.rs");
    let serial_code = extract_serial_sensor_code(&source);

    assert!(
        !serial_code.contains("..Default::default()"),
        "\n\nCONTRACT VIOLATION: SENSOR-001\n\
         File: crates/mbot-companion/src/transport.rs\n\
         Issue: Serial match arm uses '..Default::default()' to fill MBotSensors.\n\
         This means some sensor fields get zero/default values instead of real\n\
         hardware readings. Every field must be populated from a protocol command.\n\
         \n\
         Found '..Default::default()' in Serial arm:\n{}\n\
         \n\
         See: docs/contracts/feature_sensors.yml (SENSOR-001)\n",
        serial_code
    );
}

// ============================================================
// SENSOR-002: Bluetooth must NOT silently fall back to simulated
// ============================================================

#[test]
fn sensor_002_bluetooth_must_not_call_read_simulated() {
    let source = read_source("crates/mbot-companion/src/transport.rs");
    let bt_code = extract_bluetooth_sensor_code(&source);

    assert!(
        !bt_code.contains("read_simulated"),
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
        bt_code
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

// NOTE: CyberPi HalocodeProtocol uses Python expression scripts, not MegaPi binary commands.
// Function naming: read_*_script() instead of read_*_cmd()
// Sensor mapping: brightness (light), loudness (sound), accel (replaces quad_rgb)

#[test]
fn sensor_005_protocol_has_read_ultrasonic() {
    let source = read_source("crates/mbot-companion/src/protocol.rs");

    assert!(
        source.contains("fn read_ultrasonic_script"),
        "\n\nCONTRACT VIOLATION: SENSOR-005\n\
         File: crates/mbot-companion/src/protocol.rs\n\
         Issue: Missing read_ultrasonic_script() function.\n\
         Protocol must define read scripts for all sensor types.\n\
         \n\
         See: docs/contracts/feature_sensors.yml (SENSOR-005)\n"
    );
}

#[test]
fn sensor_005_protocol_has_read_loudness() {
    let source = read_source("crates/mbot-companion/src/protocol.rs");

    assert!(
        source.contains("fn read_loudness_script"),
        "\n\nCONTRACT VIOLATION: SENSOR-005\n\
         File: crates/mbot-companion/src/protocol.rs\n\
         Issue: Missing read_loudness_script() function.\n\
         Protocol must define read scripts for all sensor types.\n\
         \n\
         See: docs/contracts/feature_sensors.yml (SENSOR-005)\n"
    );
}

#[test]
fn sensor_005_protocol_has_read_brightness() {
    let source = read_source("crates/mbot-companion/src/protocol.rs");

    assert!(
        source.contains("fn read_brightness_script"),
        "\n\nCONTRACT VIOLATION: SENSOR-005\n\
         File: crates/mbot-companion/src/protocol.rs\n\
         Issue: Missing read_brightness_script() function.\n\
         Protocol must define read scripts for all sensor types.\n\
         \n\
         See: docs/contracts/feature_sensors.yml (SENSOR-005)\n"
    );
}

#[test]
fn sensor_005_protocol_has_read_gyro() {
    let source = read_source("crates/mbot-companion/src/protocol.rs");

    assert!(
        source.contains("fn read_gyro_script"),
        "\n\nCONTRACT VIOLATION: SENSOR-005\n\
         File: crates/mbot-companion/src/protocol.rs\n\
         Issue: Missing read_gyro_script() function.\n\
         Protocol must define read scripts for all sensor types.\n\
         \n\
         See: docs/contracts/feature_sensors.yml (SENSOR-005)\n"
    );
}

#[test]
fn sensor_005_protocol_has_read_accel() {
    let source = read_source("crates/mbot-companion/src/protocol.rs");

    assert!(
        source.contains("fn read_accel_script"),
        "\n\nCONTRACT VIOLATION: SENSOR-005\n\
         File: crates/mbot-companion/src/protocol.rs\n\
         Issue: Missing read_accel_script() function.\n\
         Protocol must define read scripts for all sensor types.\n\
         \n\
         See: docs/contracts/feature_sensors.yml (SENSOR-005)\n"
    );
}

// ============================================================
// SENSOR-001 (continued): Transport must CALL all protocol
//                          read functions for serial connections
// ============================================================

// NOTE: CyberPi HalocodeProtocol uses read_*_script() naming

#[test]
fn sensor_001_transport_calls_read_ultrasonic() {
    let source = read_source("crates/mbot-companion/src/transport.rs");

    assert!(
        source.contains("read_ultrasonic_script"),
        "\n\nCONTRACT VIOLATION: SENSOR-001\n\
         File: crates/mbot-companion/src/transport.rs\n\
         Issue: transport.rs never calls protocol::read_ultrasonic_script().\n\
         The transport must send read scripts for ALL sensors.\n\
         \n\
         See: docs/contracts/feature_sensors.yml (SENSOR-001)\n"
    );
}

#[test]
fn sensor_001_transport_calls_read_loudness() {
    let source = read_source("crates/mbot-companion/src/transport.rs");

    assert!(
        source.contains("read_loudness_script"),
        "\n\nCONTRACT VIOLATION: SENSOR-001\n\
         File: crates/mbot-companion/src/transport.rs\n\
         Issue: transport.rs never calls protocol::read_loudness_script().\n\
         The transport must send read scripts for ALL sensors.\n\
         \n\
         See: docs/contracts/feature_sensors.yml (SENSOR-001)\n"
    );
}

#[test]
fn sensor_001_transport_calls_read_brightness() {
    let source = read_source("crates/mbot-companion/src/transport.rs");

    assert!(
        source.contains("read_brightness_script"),
        "\n\nCONTRACT VIOLATION: SENSOR-001\n\
         File: crates/mbot-companion/src/transport.rs\n\
         Issue: transport.rs never calls protocol::read_brightness_script().\n\
         The transport must send read scripts for ALL sensors.\n\
         \n\
         See: docs/contracts/feature_sensors.yml (SENSOR-001)\n"
    );
}

#[test]
fn sensor_001_transport_calls_read_gyro() {
    let source = read_source("crates/mbot-companion/src/transport.rs");

    assert!(
        source.contains("read_gyro_script"),
        "\n\nCONTRACT VIOLATION: SENSOR-001\n\
         File: crates/mbot-companion/src/transport.rs\n\
         Issue: transport.rs never calls protocol::read_gyro_script().\n\
         The transport must send read scripts for ALL sensors.\n\
         \n\
         See: docs/contracts/feature_sensors.yml (SENSOR-001)\n"
    );
}

#[test]
fn sensor_001_transport_calls_read_accel() {
    let source = read_source("crates/mbot-companion/src/transport.rs");

    assert!(
        source.contains("read_accel_script"),
        "\n\nCONTRACT VIOLATION: SENSOR-001\n\
         File: crates/mbot-companion/src/transport.rs\n\
         Issue: transport.rs never calls protocol::read_accel_script().\n\
         The transport must send read scripts for ALL sensors.\n\
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
