//! 33-byte Cognitum sensor protocol frame parser.
//!
//! The Cognitum sensor expansion port speaks a compact binary protocol:
//! each frame is exactly 33 bytes and carries one sensor reading.
//!
//! # Frame layout
//!
//! ```text
//! Offset  Size  Field
//! ──────  ────  ─────────────────────────────────────────────
//!  0       1    sensor_type   — sensor category (0x01–0x06)
//!  1       4    sensor_id     — hardware sensor ID (little-endian u32)
//!  5       4    timestamp_ms  — frame timestamp in ms (little-endian u32)
//!  9       8    value         — sensor reading (little-endian f64)
//! 17       4    flags         — status flags (little-endian u32)
//! 21      12    reserved      — reserved / metadata bytes
//! ──────  ────
//! total  33
//! ```
//!
//! # Invariant
//!
//! - **I-COG-012** — `SensorFrame::parse()` returns `Err` for unknown sensor_type;
//!   never panics on malformed input.

use crate::sensors::{Attention, LightBand, Presence, SoundBand, TimePeriod, Touch};

/// All sensor categories supported on the Cognitum v0 expansion port.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SensorType {
    /// PIR or ultrasonic distance sensor. Value = 0–3 (Absent/Static/Approaching/Retreating).
    Presence   = 0x01,
    /// Photodiode ambient light sensor. Value = normalised lux in [0.0, 1.0].
    Light      = 0x02,
    /// Microphone envelope detector. Value = normalised dB in [0.0, 1.0].
    Sound      = 0x03,
    /// Capacitive touch pad. Value = 0.0 (no contact) or 1.0 (contact).
    Touch      = 0x04,
    /// Camera face/gaze detector. Value = 0–2 (None/Glance/Sustained).
    Attention  = 0x05,
    /// Host RTC time-of-day. Value = hour in [0.0, 23.0].
    TimePeriod = 0x06,
}

impl SensorType {
    fn from_byte(b: u8) -> Option<Self> {
        match b {
            0x01 => Some(Self::Presence),
            0x02 => Some(Self::Light),
            0x03 => Some(Self::Sound),
            0x04 => Some(Self::Touch),
            0x05 => Some(Self::Attention),
            0x06 => Some(Self::TimePeriod),
            _    => None,
        }
    }
}

/// Error variants for frame parsing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrameError {
    /// `sensor_type` byte did not match any known category.
    UnknownSensorType(u8),
}

/// A parsed, validated Cognitum sensor frame.
#[derive(Clone, Debug)]
pub struct ParsedFrame {
    pub sensor_type:  SensorType,
    pub sensor_id:    u32,
    pub timestamp_ms: u32,
    pub value:        f64,
    pub flags:        u32,
}

/// Parse a raw 33-byte Cognitum sensor frame.
///
/// Returns `Err(FrameError::UnknownSensorType)` if `bytes[0]` is not a
/// recognised sensor category. Does not panic.
///
/// # Example
///
/// ```rust
/// use ccf_cognitum::frame::{parse_frame, SensorType};
///
/// let mut raw = [0u8; 33];
/// raw[0] = 0x01;                       // Presence
/// raw[9..17].copy_from_slice(&2.0f64.to_le_bytes()); // Approaching
///
/// let frame = parse_frame(&raw).unwrap();
/// assert_eq!(frame.sensor_type, SensorType::Presence);
/// assert!((frame.value - 2.0).abs() < 1e-9);
/// ```
pub fn parse_frame(bytes: &[u8; 33]) -> Result<ParsedFrame, FrameError> {
    let sensor_type = SensorType::from_byte(bytes[0])
        .ok_or(FrameError::UnknownSensorType(bytes[0]))?;

    let sensor_id    = u32::from_le_bytes([bytes[1], bytes[2], bytes[3], bytes[4]]);
    let timestamp_ms = u32::from_le_bytes([bytes[5], bytes[6], bytes[7], bytes[8]]);
    let value        = f64::from_le_bytes([
        bytes[9],  bytes[10], bytes[11], bytes[12],
        bytes[13], bytes[14], bytes[15], bytes[16],
    ]);
    let flags = u32::from_le_bytes([bytes[17], bytes[18], bytes[19], bytes[20]]);

    Ok(ParsedFrame { sensor_type, sensor_id, timestamp_ms, value, flags })
}

// ── Dimension classifiers ─────────────────────────────────────────────────────
// Convenience helpers used by SensorWindow to update individual fields.

pub(crate) fn classify_presence(value: f64)    -> Presence    { Presence::from_value(value) }
pub(crate) fn classify_light(value: f64)        -> LightBand   { LightBand::from_normalised(value) }
pub(crate) fn classify_sound(value: f64)        -> SoundBand   { SoundBand::from_normalised(value) }
pub(crate) fn classify_touch(value: f64)        -> Touch       { Touch::from_value(value) }
pub(crate) fn classify_attention(value: f64)    -> Attention   { Attention::from_value(value) }
pub(crate) fn classify_time_period(value: f64)  -> TimePeriod  { TimePeriod::from_hour(value as u8) }
