//! Contract tests for BLE Transport (HalocodeProtocol over Bluetooth)
//!
//! Enforces BLE-001..005 contracts from docs/contracts/feature_ble_transport.yml
//!
//! These tests scan source code as text to verify structural invariants.
//! They run without Bluetooth hardware.

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
// BLE-001: BLE transport uses same f3/f4 protocol as serial
// ============================================================

#[test]
fn ble_001_uses_protocol_sensor_read() {
    let source = read_source("crates/mbot-companion/src/transport.rs");

    // BLE code must use protocol::sensor_read for frame construction
    assert!(
        source.contains("protocol::sensor_read"),
        "\n\nCONTRACT VIOLATION: BLE-001\n\
         File: crates/mbot-companion/src/transport.rs\n\
         Issue: BLE code does not use protocol::sensor_read() for frame construction.\n\
         BLE must use the same f3/f4 protocol module as serial.\n\
         \n\
         See: docs/contracts/feature_ble_transport.yml (BLE-001)\n"
    );
}

#[test]
fn ble_001_uses_protocol_command() {
    let source = read_source("crates/mbot-companion/src/transport.rs");

    assert!(
        source.contains("protocol::command"),
        "\n\nCONTRACT VIOLATION: BLE-001\n\
         File: crates/mbot-companion/src/transport.rs\n\
         Issue: BLE code does not use protocol::command() for fire-and-forget frames.\n\
         BLE must use the same f3/f4 protocol module as serial.\n\
         \n\
         See: docs/contracts/feature_ble_transport.yml (BLE-001)\n"
    );
}

#[test]
fn ble_001_uses_f3_parser() {
    let source = read_source("crates/mbot-companion/src/transport.rs");

    assert!(
        source.contains("parser.feed"),
        "\n\nCONTRACT VIOLATION: BLE-001\n\
         File: crates/mbot-companion/src/transport.rs\n\
         Issue: BLE code does not use parser.feed() for response parsing.\n\
         BLE must use the same F3Parser as serial to decode f3/f4 frames.\n\
         \n\
         See: docs/contracts/feature_ble_transport.yml (BLE-001)\n"
    );
}

#[test]
fn ble_001_uses_online_mode_frame() {
    let source = read_source("crates/mbot-companion/src/transport.rs");

    assert!(
        source.contains("protocol::online_mode_frame"),
        "\n\nCONTRACT VIOLATION: BLE-001\n\
         File: crates/mbot-companion/src/transport.rs\n\
         Issue: BLE init does not use protocol::online_mode_frame().\n\
         BLE must send the same online_mode frame as serial to initialize CyberPi.\n\
         \n\
         See: docs/contracts/feature_ble_transport.yml (BLE-001)\n"
    );
}

// ============================================================
// BLE-002: BLE uses Makeblock standard GATT UUIDs
// ============================================================

#[test]
fn ble_002_service_uuid_ffe1() {
    let source = read_source("crates/mbot-companion/src/transport.rs");

    assert!(
        source.contains("ffe1"),
        "\n\nCONTRACT VIOLATION: BLE-002\n\
         File: crates/mbot-companion/src/transport.rs\n\
         Issue: BLE service UUID ffe1 not found.\n\
         Makeblock devices use GATT service 0000ffe1-0000-1000-8000-00805f9b34fb.\n\
         \n\
         See: docs/contracts/feature_ble_transport.yml (BLE-002)\n"
    );
}

#[test]
fn ble_002_notify_uuid_ffe2() {
    let source = read_source("crates/mbot-companion/src/transport.rs");

    assert!(
        source.contains("ffe2"),
        "\n\nCONTRACT VIOLATION: BLE-002\n\
         File: crates/mbot-companion/src/transport.rs\n\
         Issue: BLE notify characteristic UUID ffe2 not found.\n\
         Makeblock devices use notify char 0000ffe2-0000-1000-8000-00805f9b34fb.\n\
         \n\
         See: docs/contracts/feature_ble_transport.yml (BLE-002)\n"
    );
}

#[test]
fn ble_002_write_uuid_ffe3() {
    let source = read_source("crates/mbot-companion/src/transport.rs");

    assert!(
        source.contains("ffe3"),
        "\n\nCONTRACT VIOLATION: BLE-002\n\
         File: crates/mbot-companion/src/transport.rs\n\
         Issue: BLE write characteristic UUID ffe3 not found.\n\
         Makeblock devices use write char 0000ffe3-0000-1000-8000-00805f9b34fb.\n\
         \n\
         See: docs/contracts/feature_ble_transport.yml (BLE-002)\n"
    );
}

// ============================================================
// BLE-003: BLE read_sensors must NOT call read_simulated()
// ============================================================

#[test]
fn ble_003_read_sensors_dispatches_to_ble() {
    let source = read_source("crates/mbot-companion/src/transport.rs");

    // The read_sensors function must dispatch Bluetooth to read_sensors_ble
    assert!(
        source.contains("read_sensors_ble"),
        "\n\nCONTRACT VIOLATION: BLE-003\n\
         File: crates/mbot-companion/src/transport.rs\n\
         Issue: read_sensors_ble() function not found.\n\
         Bluetooth must dispatch to a dedicated BLE sensor read function.\n\
         \n\
         See: docs/contracts/feature_ble_transport.yml (BLE-003)\n"
    );
}

// ============================================================
// BLE-004: BLE sensor reads use notification stream
// ============================================================

#[test]
fn ble_004_uses_notification_stream() {
    let source = read_source("crates/mbot-companion/src/transport.rs");

    assert!(
        source.contains("notifications()"),
        "\n\nCONTRACT VIOLATION: BLE-004\n\
         File: crates/mbot-companion/src/transport.rs\n\
         Issue: BLE code does not call .notifications() to read from notify stream.\n\
         BLE sensor responses arrive as ffe2 notifications, not synchronous reads.\n\
         \n\
         See: docs/contracts/feature_ble_transport.yml (BLE-004)\n"
    );
}

#[test]
fn ble_004_has_timeouts() {
    let source = read_source("crates/mbot-companion/src/transport.rs");

    assert!(
        source.contains("timeout_at"),
        "\n\nCONTRACT VIOLATION: BLE-004\n\
         File: crates/mbot-companion/src/transport.rs\n\
         Issue: BLE reads do not use timeout_at() to prevent indefinite hangs.\n\
         All BLE notification reads must have a bounded timeout.\n\
         \n\
         See: docs/contracts/feature_ble_transport.yml (BLE-004)\n"
    );
}

// ============================================================
// BLE-005: All BLE code feature-gated under bluetooth
// ============================================================

#[test]
fn ble_005_feature_gated() {
    let source = read_source("crates/mbot-companion/src/transport.rs");

    assert!(
        source.contains(r#"cfg(feature = "bluetooth")"#),
        "\n\nCONTRACT VIOLATION: BLE-005\n\
         File: crates/mbot-companion/src/transport.rs\n\
         Issue: No #[cfg(feature = \"bluetooth\")] found in transport.rs.\n\
         All BLE code must be feature-gated to avoid mandatory libdbus dependency.\n\
         \n\
         See: docs/contracts/feature_ble_transport.yml (BLE-005)\n"
    );
}

#[test]
fn ble_005_bluetooth_in_cargo_features() {
    let source = read_source("crates/mbot-companion/Cargo.toml");

    assert!(
        source.contains("bluetooth"),
        "\n\nCONTRACT VIOLATION: BLE-005\n\
         File: crates/mbot-companion/Cargo.toml\n\
         Issue: 'bluetooth' feature not defined in Cargo.toml.\n\
         BLE transport must be an opt-in feature.\n\
         \n\
         See: docs/contracts/feature_ble_transport.yml (BLE-005)\n"
    );
}
