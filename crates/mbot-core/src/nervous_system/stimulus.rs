//! Stimulus detection from raw sensor deltas.
//!
//! Invariants:
//! - I-STRT-001: no_std compatible, no heap allocation
//! - I-STRT-008: Detection uses delta-per-tick, not absolute values
//! - I-STRT-MAG: Magnitude always clamped to [0.0, 1.0]

/// The kind of stimulus detected from sensor deltas.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum StimulusKind {
    /// Loudness delta > threshold in one tick
    LoudnessSpike,
    /// Brightness delta > threshold in one tick
    BrightnessSpike,
    /// Ultrasonic distance drops rapidly (something approaching fast)
    ProximityRush,
    /// Accelerometer spike (collision or being grabbed)
    ImpactShock,
    /// Gyroscope detects rapid tilt/flip (picked up, knocked over)
    OrientationFlip,
}

/// A discrete, above-threshold sensor change within a single tick.
#[derive(Clone, Copy, Debug)]
pub struct StimulusEvent {
    pub kind: StimulusKind,
    /// Normalized intensity [0.0, 1.0]
    pub magnitude: f32,
    pub tick: u64,
}

/// Thresholds that trigger stimulus event creation from raw sensor deltas.
/// All values are delta-per-tick, not absolute.
#[derive(Clone, Copy, Debug)]
pub struct StimulusThresholds {
    /// Raw loudness delta per tick (default: 40.0)
    pub loudness_spike: f32,
    /// Raw brightness delta per tick (default: 80.0)
    pub brightness_spike: f32,
    /// Distance drop in cm per tick (default: 15.0)
    pub proximity_rush: f32,
    /// G-force from accelerometer (default: 1.5)
    pub impact_g: f32,
    /// Degrees per second from gyro (default: 120.0)
    pub orientation_dps: f32,
}

impl Default for StimulusThresholds {
    fn default() -> Self {
        Self {
            loudness_spike: 40.0,
            brightness_spike: 80.0,
            proximity_rush: 15.0,
            impact_g: 1.5,
            orientation_dps: 120.0,
        }
    }
}

/// Max expected values for magnitude normalization.
struct MaxExpected;
impl MaxExpected {
    const LOUDNESS: f32 = 100.0;
    const BRIGHTNESS: f32 = 255.0;
    const PROXIMITY: f32 = 50.0;
    const IMPACT: f32 = 8.0;
    const ORIENTATION: f32 = 500.0;
}

/// Fixed-capacity container for up to 5 stimulus events per tick.
/// No heap allocation (STRT-001).
#[derive(Clone, Debug)]
pub struct StimulusEvents {
    buf: [StimulusEvent; 5],
    len: usize,
}

impl StimulusEvents {
    fn new() -> Self {
        Self {
            buf: [StimulusEvent {
                kind: StimulusKind::LoudnessSpike,
                magnitude: 0.0,
                tick: 0,
            }; 5],
            len: 0,
        }
    }

    fn push(&mut self, event: StimulusEvent) {
        if self.len < 5 {
            self.buf[self.len] = event;
            self.len += 1;
        }
    }

    /// Number of events detected this tick.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Whether no events were detected.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Iterate over detected events.
    pub fn iter(&self) -> impl Iterator<Item = &StimulusEvent> {
        self.buf[..self.len].iter()
    }

    /// Get event by index.
    pub fn get(&self, idx: usize) -> Option<&StimulusEvent> {
        if idx < self.len { Some(&self.buf[idx]) } else { None }
    }
}

/// Detects stimulus events from raw sensor deltas each tick.
///
/// Call `detect()` once per tick with current sensor readings.
/// The detector tracks previous values to compute deltas.
pub struct StimulusDetector {
    prev_loudness: f32,
    prev_brightness: f32,
    prev_distance: f32,
    thresholds: StimulusThresholds,
    initialized: bool,
}

impl StimulusDetector {
    pub fn new() -> Self {
        Self::with_thresholds(StimulusThresholds::default())
    }

    pub fn with_thresholds(thresholds: StimulusThresholds) -> Self {
        Self {
            prev_loudness: 0.0,
            prev_brightness: 0.0,
            prev_distance: 100.0,
            thresholds,
            initialized: false,
        }
    }

    /// Previous loudness value used for delta calculation.
    /// Returns the value from the tick before the most recent `detect()` call.
    /// Useful for residual sensor streams that need pre-spike values.
    pub fn prev_loudness(&self) -> f32 {
        self.prev_loudness
    }

    /// Previous brightness value used for delta calculation.
    pub fn prev_brightness(&self) -> f32 {
        self.prev_brightness
    }

    /// Previous distance value used for delta calculation.
    pub fn prev_distance(&self) -> f32 {
        self.prev_distance
    }

    /// Whether the detector has been initialized (has seen at least one tick).
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// Detect stimulus events from current sensor readings.
    ///
    /// Returns up to 5 events (one per sensor channel). In practice,
    /// rarely more than one fires simultaneously.
    pub fn detect(
        &mut self,
        loudness: f32,
        brightness: f32,
        distance_cm: f32,
        accel_magnitude: f32,
        gyro_magnitude: f32,
        tick: u64,
    ) -> StimulusEvents {
        let mut events = StimulusEvents::new();

        // Skip the first tick — we need a previous reading to compute deltas
        if !self.initialized {
            self.prev_loudness = loudness;
            self.prev_brightness = brightness;
            self.prev_distance = distance_cm;
            self.initialized = true;
            return events;
        }

        // Loudness spike (positive delta only — getting louder)
        let loud_delta = loudness - self.prev_loudness;
        if loud_delta > self.thresholds.loudness_spike {
            events.push(StimulusEvent {
                kind: StimulusKind::LoudnessSpike,
                magnitude: normalize(loud_delta, self.thresholds.loudness_spike, MaxExpected::LOUDNESS),
                tick,
            });
        }

        // Brightness spike (absolute delta — either direction)
        let bright_delta = if brightness > self.prev_brightness {
            brightness - self.prev_brightness
        } else {
            self.prev_brightness - brightness
        };
        if bright_delta > self.thresholds.brightness_spike {
            events.push(StimulusEvent {
                kind: StimulusKind::BrightnessSpike,
                magnitude: normalize(bright_delta, self.thresholds.brightness_spike, MaxExpected::BRIGHTNESS),
                tick,
            });
        }

        // Proximity rush (distance decreasing rapidly — something approaching)
        let dist_delta = self.prev_distance - distance_cm; // positive = approaching
        if dist_delta > self.thresholds.proximity_rush {
            events.push(StimulusEvent {
                kind: StimulusKind::ProximityRush,
                magnitude: normalize(dist_delta, self.thresholds.proximity_rush, MaxExpected::PROXIMITY),
                tick,
            });
        }

        // Impact shock (absolute accelerometer magnitude)
        if accel_magnitude > self.thresholds.impact_g {
            events.push(StimulusEvent {
                kind: StimulusKind::ImpactShock,
                magnitude: normalize(accel_magnitude, self.thresholds.impact_g, MaxExpected::IMPACT),
                tick,
            });
        }

        // Orientation flip (absolute gyro magnitude)
        if gyro_magnitude > self.thresholds.orientation_dps {
            events.push(StimulusEvent {
                kind: StimulusKind::OrientationFlip,
                magnitude: normalize(gyro_magnitude, self.thresholds.orientation_dps, MaxExpected::ORIENTATION),
                tick,
            });
        }

        // Update previous values for next tick
        self.prev_loudness = loudness;
        self.prev_brightness = brightness;
        self.prev_distance = distance_cm;

        events
    }
}

impl Default for StimulusDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// Normalize a value to [0.0, 1.0] based on threshold and max expected.
/// I-STRT-MAG: result always clamped.
fn normalize(value: f32, threshold: f32, max_expected: f32) -> f32 {
    let raw = (value - threshold) / (max_expected - threshold);
    if raw < 0.0 {
        0.0
    } else if raw > 1.0 {
        1.0
    } else {
        raw
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_loudness_spike_detection() {
        let mut det = StimulusDetector::new();
        // First tick initializes
        let _ = det.detect(20.0, 0.0, 100.0, 0.0, 0.0, 0);
        // Second tick: delta = 60, threshold = 40 → fires
        let events = det.detect(80.0, 0.0, 100.0, 0.0, 0.0, 1);
        assert_eq!(events.len(), 1);
        assert_eq!(events.get(0).unwrap().kind, StimulusKind::LoudnessSpike);
        // magnitude = (60-40)/(100-40) = 20/60 ≈ 0.333
        let mag = events.get(0).unwrap().magnitude;
        assert!((mag - 0.333).abs() < 0.01, "expected ~0.333, got {}", mag);
    }

    #[test]
    fn test_below_threshold_ignored() {
        let mut det = StimulusDetector::new();
        let _ = det.detect(20.0, 0.0, 100.0, 0.0, 0.0, 0);
        // Delta = 30 < threshold 40
        let events = det.detect(50.0, 0.0, 100.0, 0.0, 0.0, 1);
        assert!(events.is_empty());
    }

    #[test]
    fn test_multiple_simultaneous_stimuli() {
        let mut det = StimulusDetector::new();
        let _ = det.detect(20.0, 50.0, 100.0, 0.0, 0.0, 0);
        // Loudness delta=60 (>40), brightness delta=100 (>80)
        let events = det.detect(80.0, 150.0, 100.0, 0.0, 0.0, 1);
        assert_eq!(events.len(), 2);
        let kinds: Vec<_> = events.iter().map(|e| e.kind).collect();
        assert!(kinds.contains(&StimulusKind::LoudnessSpike));
        assert!(kinds.contains(&StimulusKind::BrightnessSpike));
    }

    #[test]
    fn test_proximity_rush_detection() {
        let mut det = StimulusDetector::new();
        let _ = det.detect(0.0, 0.0, 30.0, 0.0, 0.0, 0);
        // Distance drops from 30 to 10 = delta 20 > threshold 15
        let events = det.detect(0.0, 0.0, 10.0, 0.0, 0.0, 1);
        assert_eq!(events.len(), 1);
        assert_eq!(events.get(0).unwrap().kind, StimulusKind::ProximityRush);
    }

    #[test]
    fn test_impact_shock_detection() {
        let mut det = StimulusDetector::new();
        let _ = det.detect(0.0, 0.0, 100.0, 0.0, 0.0, 0);
        // Accel = 3.0g > threshold 1.5g
        // magnitude = (3.0-1.5)/(8.0-1.5) = 1.5/6.5 ≈ 0.231
        let events = det.detect(0.0, 0.0, 100.0, 3.0, 0.0, 1);
        assert_eq!(events.len(), 1);
        assert_eq!(events.get(0).unwrap().kind, StimulusKind::ImpactShock);
        let mag = events.get(0).unwrap().magnitude;
        assert!((mag - 0.231).abs() < 0.01, "expected ~0.231, got {}", mag);
    }

    #[test]
    fn test_orientation_flip_detection() {
        let mut det = StimulusDetector::new();
        let _ = det.detect(0.0, 0.0, 100.0, 0.0, 0.0, 0);
        // Gyro = 200 dps > threshold 120
        let events = det.detect(0.0, 0.0, 100.0, 0.0, 200.0, 1);
        assert_eq!(events.len(), 1);
        assert_eq!(events.get(0).unwrap().kind, StimulusKind::OrientationFlip);
    }

    #[test]
    fn test_first_tick_no_events() {
        let mut det = StimulusDetector::new();
        // First tick should never produce events (no previous values)
        let events = det.detect(100.0, 200.0, 5.0, 5.0, 500.0, 0);
        assert!(events.is_empty());
    }

    #[test]
    fn test_magnitude_clamped_to_max() {
        let mut det = StimulusDetector::new();
        let _ = det.detect(0.0, 0.0, 100.0, 0.0, 0.0, 0);
        // Extreme loudness delta = 200 >> max_expected 100
        let events = det.detect(200.0, 0.0, 100.0, 0.0, 0.0, 1);
        assert_eq!(events.len(), 1);
        assert_eq!(events.get(0).unwrap().magnitude, 1.0, "magnitude should clamp to 1.0");
    }

    #[test]
    fn test_normalize_function() {
        assert_eq!(normalize(40.0, 40.0, 100.0), 0.0); // exactly at threshold
        assert_eq!(normalize(100.0, 40.0, 100.0), 1.0); // at max
        assert_eq!(normalize(200.0, 40.0, 100.0), 1.0); // above max, clamped
        let mid = normalize(70.0, 40.0, 100.0);
        assert!((mid - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_brightness_spike_both_directions() {
        let mut det = StimulusDetector::new();
        let _ = det.detect(0.0, 200.0, 100.0, 0.0, 0.0, 0);
        // Brightness DROPS from 200 to 50 = abs delta 150 > threshold 80
        let events = det.detect(0.0, 50.0, 100.0, 0.0, 0.0, 1);
        assert_eq!(events.len(), 1);
        assert_eq!(events.get(0).unwrap().kind, StimulusKind::BrightnessSpike);
    }

    #[test]
    fn test_proximity_increasing_no_event() {
        let mut det = StimulusDetector::new();
        let _ = det.detect(0.0, 0.0, 10.0, 0.0, 0.0, 0);
        // Distance increases (object moving away) — no rush
        let events = det.detect(0.0, 0.0, 50.0, 0.0, 0.0, 1);
        assert!(events.is_empty());
    }

    #[test]
    fn test_all_five_stimuli_at_once() {
        let mut det = StimulusDetector::new();
        let _ = det.detect(0.0, 0.0, 100.0, 0.0, 0.0, 0);
        // All channels spike simultaneously
        let events = det.detect(100.0, 200.0, 10.0, 5.0, 300.0, 1);
        assert_eq!(events.len(), 5);
    }

    #[test]
    fn test_custom_thresholds() {
        let thresholds = StimulusThresholds {
            loudness_spike: 10.0, // much lower threshold
            ..Default::default()
        };
        let mut det = StimulusDetector::with_thresholds(thresholds);
        let _ = det.detect(0.0, 0.0, 100.0, 0.0, 0.0, 0);
        // Delta = 15, which exceeds custom threshold 10 but not default 40
        let events = det.detect(15.0, 0.0, 100.0, 0.0, 0.0, 1);
        assert_eq!(events.len(), 1);
    }

    #[test]
    fn test_event_tick_recorded() {
        let mut det = StimulusDetector::new();
        let _ = det.detect(0.0, 0.0, 100.0, 0.0, 0.0, 0);
        let events = det.detect(100.0, 0.0, 100.0, 0.0, 0.0, 42);
        assert_eq!(events.get(0).unwrap().tick, 42);
    }

    // === Previous Value Getters (Issue #11: Residual Sensor Stream) ===

    #[test]
    fn test_prev_values_before_init() {
        let det = StimulusDetector::new();
        // Before any detect() call, previous values are at their defaults
        assert_eq!(det.prev_loudness(), 0.0);
        assert_eq!(det.prev_brightness(), 0.0);
        assert_eq!(det.prev_distance(), 100.0);
        assert!(!det.is_initialized());
    }

    #[test]
    fn test_prev_values_after_first_tick() {
        let mut det = StimulusDetector::new();
        let _ = det.detect(50.0, 120.0, 30.0, 0.0, 0.0, 0);

        // After first tick, prev values are set to the first reading
        assert_eq!(det.prev_loudness(), 50.0);
        assert_eq!(det.prev_brightness(), 120.0);
        assert_eq!(det.prev_distance(), 30.0);
        assert!(det.is_initialized());
    }

    #[test]
    fn test_prev_values_update_after_each_tick() {
        let mut det = StimulusDetector::new();
        let _ = det.detect(20.0, 50.0, 100.0, 0.0, 0.0, 0);
        let _ = det.detect(80.0, 150.0, 40.0, 0.0, 0.0, 1);

        // After second tick, prev values are set to the second reading
        assert_eq!(det.prev_loudness(), 80.0);
        assert_eq!(det.prev_brightness(), 150.0);
        assert_eq!(det.prev_distance(), 40.0);
    }

    #[test]
    fn test_prev_values_captured_before_detect_updates_them() {
        // This test verifies the pattern used by the residual sensor stream:
        // capture prev values BEFORE calling detect(), then detect() updates them.
        let mut det = StimulusDetector::new();
        let _ = det.detect(20.0, 50.0, 100.0, 0.0, 0.0, 0);

        // Capture pre-spike values before second detect
        let pre_loudness = det.prev_loudness();
        let pre_brightness = det.prev_brightness();
        let pre_distance = det.prev_distance();

        assert_eq!(pre_loudness, 20.0);
        assert_eq!(pre_brightness, 50.0);
        assert_eq!(pre_distance, 100.0);

        // Now detect with a spike
        let events = det.detect(80.0, 200.0, 10.0, 0.0, 0.0, 1);
        assert!(!events.is_empty(), "Should detect stimuli from the spike");

        // After detect, prev values have been updated to the spike values
        assert_eq!(det.prev_loudness(), 80.0);
        assert_eq!(det.prev_brightness(), 200.0);
        assert_eq!(det.prev_distance(), 10.0);

        // But our captured values are still the pre-spike values
        assert_eq!(pre_loudness, 20.0);
        assert_eq!(pre_brightness, 50.0);
        assert_eq!(pre_distance, 100.0);
    }
}
