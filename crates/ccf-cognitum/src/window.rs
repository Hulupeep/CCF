//! Rolling sensor window — aggregates 33-byte frames into a `CognitumSensors` snapshot.
//!
//! The Cognitum sensor port delivers one frame per sensor per tick. The `SensorWindow`
//! retains the most recent value for each of the 6 sensor dimensions and produces a
//! `CognitumSensors` snapshot on demand.
//!
//! # Design
//!
//! Each sensor dimension is tracked independently. When a frame arrives, the
//! corresponding field is updated. If no frame has arrived for a dimension, the
//! ambient baseline value is used.
//!
//! This means `snapshot()` always returns a valid `CognitumSensors` regardless of
//! how many frames have been received — satisfying **I-COG-013**.

use crate::{
    frame::{
        classify_attention, classify_light, classify_presence, classify_sound,
        classify_time_period, classify_touch, ParsedFrame, SensorType,
    },
    sensors::{Attention, CognitumSensors, LightBand, Presence, SoundBand, TimePeriod, Touch},
};

/// Tracks the most recent reading for each sensor dimension.
///
/// Initialises all dimensions to `CognitumSensors::AMBIENT_BASELINE`.
/// Call `ingest()` with each `ParsedFrame` received from the sensor port,
/// then `snapshot()` to obtain the current `CognitumSensors`.
///
/// # Invariant
///
/// - **I-COG-013** — `snapshot()` always returns a valid `CognitumSensors`,
///   even when no frames have been ingested.
pub struct SensorWindow {
    presence:    Presence,
    light:       LightBand,
    sound:       SoundBand,
    touch:       Touch,
    attention:   Attention,
    time_period: TimePeriod,
}

impl SensorWindow {
    /// Create a new window, initialised to the ambient baseline.
    pub fn new() -> Self {
        let b = CognitumSensors::AMBIENT_BASELINE;
        Self {
            presence:    b.presence,
            light:       b.light,
            sound:       b.sound,
            touch:       b.touch,
            attention:   b.attention,
            time_period: b.time_period,
        }
    }

    /// Update the window with a newly parsed sensor frame.
    ///
    /// Only the dimension corresponding to `frame.sensor_type` is updated.
    /// All other dimensions retain their previous value.
    pub fn ingest(&mut self, frame: ParsedFrame) {
        match frame.sensor_type {
            SensorType::Presence   => self.presence    = classify_presence(frame.value),
            SensorType::Light      => self.light        = classify_light(frame.value),
            SensorType::Sound      => self.sound        = classify_sound(frame.value),
            SensorType::Touch      => self.touch        = classify_touch(frame.value),
            SensorType::Attention  => self.attention    = classify_attention(frame.value),
            SensorType::TimePeriod => self.time_period  = classify_time_period(frame.value),
        }
    }

    /// Produce a `CognitumSensors` snapshot from the current window state.
    ///
    /// Always returns a valid snapshot. Dimensions that have not yet received
    /// a frame use the ambient baseline value.
    pub fn snapshot(&self) -> CognitumSensors {
        CognitumSensors {
            presence:    self.presence,
            light:       self.light,
            sound:       self.sound,
            touch:       self.touch,
            attention:   self.attention,
            time_period: self.time_period,
        }
    }

    /// Reset all dimensions to the ambient baseline.
    pub fn reset(&mut self) {
        let b = CognitumSensors::AMBIENT_BASELINE;
        self.presence    = b.presence;
        self.light       = b.light;
        self.sound       = b.sound;
        self.touch       = b.touch;
        self.attention   = b.attention;
        self.time_period = b.time_period;
    }
}

impl Default for SensorWindow {
    fn default() -> Self { Self::new() }
}
