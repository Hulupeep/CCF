//! `CognitumSensors` — 6-dimensional `SensorVocabulary` for the Cognitum v0
//! appliance sensor expansion port.
//!
//! # Sensor dimensions
//!
//! | Dim | Field | Hardware | States |
//! |-----|-------|----------|--------|
//! | 0 | `presence` | PIR / ultrasonic | 4 (Absent / Static / Approaching / Retreating) |
//! | 1 | `light` | Photodiode | 3 (Dark / Dim / Bright) |
//! | 2 | `sound` | Microphone envelope | 3 (Quiet / Moderate / Loud) |
//! | 3 | `touch` | Capacitive pad | 2 (None / Contact) |
//! | 4 | `attention` | Camera / face detect | 3 (None / Glance / Sustained) |
//! | 5 | `time_period` | Host RTC | 4 (Night / Morning / Day / Evening) |
//!
//! # Invariants
//!
//! - **I-COG-010** — compiles to `wasm32-wasip1` with no heap allocator
//! - **I-COG-011** — compiles to `thumbv7em-none-eabihf` with `no_std`
//! - **I-COG-014** — all 6 feature vector components are in [0.0, 1.0]
//! - **I-COG-015** — `context_hash_u32()` is deterministic across restarts

use ccf_core::vocabulary::SensorVocabulary;

/// 6-dimensional sensor vocabulary for a Cognitum v0 appliance with attached
/// sensors on the expansion port.
///
/// Plug any combination of the supported sensor categories into the 33-byte
/// Cognitum protocol port and the `SensorWindow` will produce a valid snapshot
/// of this type.
///
/// ```rust
/// use ccf_cognitum::sensors::{CognitumSensors, Presence, LightBand, SoundBand,
///     Touch, Attention, TimePeriod};
/// use ccf_core::vocabulary::{ContextKey, SensorVocabulary};
///
/// let sensors = CognitumSensors {
///     presence:    Presence::Approaching,
///     light:       LightBand::Bright,
///     sound:       SoundBand::Quiet,
///     touch:       Touch::None,
///     attention:   Attention::Sustained,
///     time_period: TimePeriod::Day,
/// };
/// let key = ContextKey::<CognitumSensors, 6>::new(sensors);
/// let hash = key.context_hash_u32();
/// let vec  = key.vocabulary.to_feature_vec();
/// assert!(vec.iter().all(|&v| v >= 0.0 && v <= 1.0));
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CognitumSensors {
    /// Human or object proximity detected by PIR / ultrasonic sensor.
    pub presence: Presence,
    /// Ambient light level from photodiode.
    pub light: LightBand,
    /// Ambient sound level from microphone envelope detector.
    pub sound: SoundBand,
    /// Physical contact on the capacitive touch pad.
    pub touch: Touch,
    /// Face / gaze detection from attached camera module.
    pub attention: Attention,
    /// Time of day period from host RTC or system clock.
    pub time_period: TimePeriod,
}

impl CognitumSensors {
    /// Ambient baseline — used when no frames have been received yet.
    ///
    /// Represents a quiet desktop environment during the day: nobody present,
    /// dim indoor lighting, background sound, no touch, no face detected.
    pub const AMBIENT_BASELINE: Self = Self {
        presence: Presence::Absent,
        light: LightBand::Dim,
        sound: SoundBand::Quiet,
        touch: Touch::None,
        attention: Attention::None,
        time_period: TimePeriod::Day,
    };
}

impl SensorVocabulary<6> for CognitumSensors {
    /// Encode the 6-dimensional sensor state as a float vector in [0.0, 1.0].
    ///
    /// | Index | Field       | Normalisation |
    /// |-------|-------------|---------------|
    /// | 0     | presence    | ordinal / 3.0 |
    /// | 1     | light       | ordinal / 2.0 |
    /// | 2     | sound       | ordinal / 2.0 |
    /// | 3     | touch       | ordinal / 1.0 |
    /// | 4     | attention   | ordinal / 2.0 |
    /// | 5     | time_period | ordinal / 3.0 |
    fn to_feature_vec(&self) -> [f32; 6] {
        [
            self.presence.ordinal()    as f32 / 3.0,
            self.light.ordinal()       as f32 / 2.0,
            self.sound.ordinal()       as f32 / 2.0,
            self.touch.ordinal()       as f32 / 1.0,
            self.attention.ordinal()   as f32 / 2.0,
            self.time_period.ordinal() as f32 / 3.0,
        ]
    }
}

// ── Presence ─────────────────────────────────────────────────────────────────

/// Human or object proximity signature from PIR / ultrasonic sensor.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Presence {
    /// No person or object detected within sensor range.
    Absent = 0,
    /// Person or object present and not moving.
    Static = 1,
    /// Person or object moving toward the appliance.
    Approaching = 2,
    /// Person or object moving away from the appliance.
    Retreating = 3,
}

impl Presence {
    pub fn ordinal(self) -> u8 { self as u8 }

    /// Derive from a normalised sensor value (0.0–3.0 range from protocol frame).
    pub fn from_value(v: f64) -> Self {
        match v as u8 {
            0 => Self::Absent,
            1 => Self::Static,
            2 => Self::Approaching,
            _ => Self::Retreating,
        }
    }
}

// ── Light ─────────────────────────────────────────────────────────────────────

/// Ambient light level from photodiode.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum LightBand {
    /// Very low ambient light — night, blackout curtains, dark room.
    Dark = 0,
    /// Moderate light — typical indoor daytime, desk lamp.
    Dim = 1,
    /// High ambient light — bright room, near a window.
    Bright = 2,
}

impl LightBand {
    pub fn ordinal(self) -> u8 { self as u8 }

    /// Derive from a normalised lux value (0.0 = dark, 1.0 = bright).
    pub fn from_normalised(v: f64) -> Self {
        if v < 0.33 { Self::Dark }
        else if v < 0.67 { Self::Dim }
        else { Self::Bright }
    }
}

// ── Sound ─────────────────────────────────────────────────────────────────────

/// Ambient sound level from microphone envelope detector.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SoundBand {
    /// Quiet environment — library, empty room, background hum.
    Quiet = 0,
    /// Moderate sound — conversation, background music.
    Moderate = 1,
    /// Loud environment — crowd, machinery, alerts.
    Loud = 2,
}

impl SoundBand {
    pub fn ordinal(self) -> u8 { self as u8 }

    /// Derive from a normalised dB value (0.0 = quiet, 1.0 = loud).
    pub fn from_normalised(v: f64) -> Self {
        if v < 0.33 { Self::Quiet }
        else if v < 0.67 { Self::Moderate }
        else { Self::Loud }
    }
}

// ── Touch ─────────────────────────────────────────────────────────────────────

/// Physical contact on the capacitive touch pad.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Touch {
    /// No contact detected.
    None = 0,
    /// Contact detected on the touch pad.
    Contact = 1,
}

impl Touch {
    pub fn ordinal(self) -> u8 { self as u8 }

    /// Derive from a binary sensor value (0.0 = no contact, ≥0.5 = contact).
    pub fn from_value(v: f64) -> Self {
        if v >= 0.5 { Self::Contact } else { Self::None }
    }
}

// ── Attention ─────────────────────────────────────────────────────────────────

/// Face / gaze detection quality from attached camera module.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Attention {
    /// No face detected in camera frame.
    None = 0,
    /// Face detected briefly — passing glance, quick look.
    Glance = 1,
    /// Face detected and held for >1 second — sustained engagement.
    Sustained = 2,
}

impl Attention {
    pub fn ordinal(self) -> u8 { self as u8 }

    /// Derive from a normalised confidence × duration value (0.0–2.0).
    pub fn from_value(v: f64) -> Self {
        match v as u8 {
            0 => Self::None,
            1 => Self::Glance,
            _ => Self::Sustained,
        }
    }
}

// ── TimePeriod ────────────────────────────────────────────────────────────────

/// Time of day period — derived from host RTC.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TimePeriod {
    /// 21:00–05:59 — late evening through early morning.
    Night = 0,
    /// 06:00–11:59 — morning.
    Morning = 1,
    /// 12:00–16:59 — afternoon / daytime.
    Day = 2,
    /// 17:00–20:59 — early evening.
    Evening = 3,
}

impl TimePeriod {
    pub fn ordinal(self) -> u8 { self as u8 }

    /// Derive from a 24-hour clock value (0.0–23.99).
    pub fn from_hour(hour: u8) -> Self {
        match hour {
            6..=11  => Self::Morning,
            12..=16 => Self::Day,
            17..=20 => Self::Evening,
            _       => Self::Night,
        }
    }
}
