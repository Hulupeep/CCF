//! mBot Core - Minimal AI nervous system for mBot2
//!
//! This crate provides the core AI logic that can run on:
//! - Laptop companion (full features)
//! - ESP32/CyberPi (no_std, minimal)
//!
//! Based on RuVector's DAG nervous system architecture.

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

#[cfg(feature = "std")]
use std::vec::Vec;

// Math functions - use libm for no_std, std for normal builds
#[cfg(not(feature = "std"))]
use libm::{sinf, cosf, sqrtf, atan2f, fabsf, powf};

#[cfg(feature = "std")]
mod math {
    #[inline] pub fn sinf(x: f32) -> f32 { x.sin() }
    #[inline] pub fn cosf(x: f32) -> f32 { x.cos() }
    #[inline] pub fn sqrtf(x: f32) -> f32 { x.sqrt() }
    #[inline] pub fn atan2f(y: f32, x: f32) -> f32 { y.atan2(x) }
    #[inline] pub fn fabsf(x: f32) -> f32 { x.abs() }
    #[inline] pub fn powf(x: f32, y: f32) -> f32 { x.powf(y) }
}

#[cfg(feature = "std")]
use math::*;

/// Sensor frame from mBot2 hardware
#[derive(Clone, Debug, Default)]
pub struct MBotSensors {
    /// Timestamp in microseconds
    pub timestamp_us: u64,
    /// Ultrasonic distance in cm (0-400)
    pub ultrasonic_cm: f32,
    /// Left encoder ticks
    pub encoder_left: i32,
    /// Right encoder ticks
    pub encoder_right: i32,
    /// Quad RGB sensor readings [front_left, front_right, back_left, back_right]
    pub quad_rgb: [[u8; 3]; 4],
    /// Gyroscope Z-axis (rotation rate)
    pub gyro_z: f32,
    /// Accelerometer readings [x, y, z]
    pub accel: [f32; 3],
    /// Sound level from microphone (0.0-1.0)
    pub sound_level: f32,
    /// Light sensor reading (0.0-1.0)
    pub light_level: f32,
}

/// Motor command output
#[derive(Clone, Debug, Default)]
pub struct MotorCommand {
    /// Left motor power (-100 to 100)
    pub left: i8,
    /// Right motor power (-100 to 100)
    pub right: i8,
    /// Pen servo position (0 = up, 90 = down)
    pub pen_angle: u8,
    /// LED color [R, G, B]
    pub led_color: [u8; 3],
    /// Buzzer frequency (0 = off)
    pub buzzer_hz: u16,
}

/// Residual sensor overrides for preventing double-counting of stimulus spikes.
///
/// When a stimulus event fires (e.g., LoudnessSpike), the same spike value would
/// also inflate the variance-based tension in `compute_homeostasis()`. To prevent
/// this double-counting, the main loop can run stimulus detection FIRST, then
/// replace spiked sensor channels with their pre-spike values (from the detector's
/// previous readings) for the homeostasis calculation.
///
/// Each field, when `Some`, overrides the corresponding sensor value in the
/// `MBotSensors` passed to `compute_homeostasis()`.
#[derive(Clone, Debug, Default)]
pub struct ResidualOverrides {
    /// Override sound_level with pre-spike value (0.0-1.0 scale)
    pub sound_level: Option<f32>,
    /// Override light_level with pre-spike value (0.0-1.0 scale)
    pub light_level: Option<f32>,
    /// Override ultrasonic_cm with pre-spike value
    pub ultrasonic_cm: Option<f32>,
    /// Override accelerometer magnitude (pre-computed, not raw accel array)
    pub accel_magnitude: Option<f32>,
}

/// ArtBot - Drawing and artistic expression
pub mod artbot;

/// Reflex modes based on DAG tension levels
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReflexMode {
    /// Tension < 0.20: Relaxed wandering, learning allowed
    Calm,
    /// Tension 0.20-0.55: Active exploration
    Active,
    /// Tension 0.55-0.85: Heightened response, exciting stimulus
    Spike,
    /// Tension > 0.85: Protective mode, back away from danger
    Protect,
}

impl ReflexMode {
    pub fn from_tension(tension: f32) -> Self {
        if tension > 0.85 {
            ReflexMode::Protect
        } else if tension > 0.55 {
            ReflexMode::Spike
        } else if tension > 0.20 {
            ReflexMode::Active
        } else {
            ReflexMode::Calm
        }
    }

    pub fn led_color(&self) -> [u8; 3] {
        match self {
            ReflexMode::Calm => [0, 100, 255],      // Cool blue
            ReflexMode::Active => [0, 255, 100],    // Green
            ReflexMode::Spike => [255, 200, 0],     // Yellow/Orange
            ReflexMode::Protect => [255, 0, 0],     // Red
        }
    }
}

/// Homeostasis state - the robot's "feeling"
#[derive(Clone, Debug)]
pub struct HomeostasisState {
    /// Deviation from equilibrium (0.0-1.0)
    pub tension: f32,
    /// Internal consistency/stability (0.0-1.0)
    pub coherence: f32,
    /// Current reflex mode
    pub reflex: ReflexMode,
    /// Energy level (0.0-1.0)
    pub energy: f32,
    /// Curiosity level (0.0-1.0)
    pub curiosity: f32,
    /// Social phase from contextual coherence field
    pub social_phase: coherence::SocialPhase,
    /// Accumulated coherence for the current context [0.0, 1.0]
    pub context_coherence: f32,
}

impl Default for HomeostasisState {
    fn default() -> Self {
        Self {
            tension: 0.0,
            coherence: 1.0,
            reflex: ReflexMode::Calm,
            energy: 0.5,
            curiosity: 0.5,
            social_phase: coherence::SocialPhase::ShyObserver,
            context_coherence: 0.0,
        }
    }
}

/// The core nervous system for mBot2
pub struct MBotBrain {
    // EMA smoothing
    tension_ema: f32,
    coherence_ema: f32,
    alpha: f32,

    // State tracking
    last_distance: f32,
    last_encoder_left: i32,
    last_encoder_right: i32,

    // Behavior parameters
    base_speed: f32,
    turn_gain: f32,
    approach_distance: f32,
    danger_distance: f32,

    // Drawing state
    pen_down: bool,
    position: (f32, f32),  // Estimated X, Y position
    heading: f32,          // Heading in radians

    // Energy management
    energy: f32,

    // Tick counter
    tick_count: u64,
}

impl MBotBrain {
    pub fn new() -> Self {
        Self {
            tension_ema: 0.0,
            coherence_ema: 1.0,
            alpha: 0.15,

            last_distance: 100.0,
            last_encoder_left: 0,
            last_encoder_right: 0,

            base_speed: 50.0,
            turn_gain: 30.0,
            approach_distance: 50.0,
            danger_distance: 15.0,

            pen_down: false,
            position: (0.0, 0.0),
            heading: 0.0,

            energy: 1.0,

            tick_count: 0,
        }
    }

    /// Configure behavior parameters
    pub fn configure(&mut self, base_speed: f32, turn_gain: f32, danger_dist: f32) {
        self.base_speed = base_speed;
        self.turn_gain = turn_gain;
        self.danger_distance = danger_dist;
    }

    /// Main processing tick - takes sensors, returns motor commands.
    ///
    /// This uses the raw sensor values for both tension calculation and
    /// motor command generation. For the residual sensor stream pattern
    /// (which prevents double-counting tension from stimulus spikes),
    /// use `tick_with_residual()` instead.
    pub fn tick(&mut self, sensors: &MBotSensors) -> (HomeostasisState, MotorCommand) {
        self.tick_with_residual(sensors, &ResidualOverrides::default())
    }

    /// Main processing tick with residual sensor overrides.
    ///
    /// When a stimulus event fires (e.g., LoudnessSpike), the same spike
    /// would also inflate the variance-based tension. To prevent this
    /// double-counting, the caller runs stimulus detection FIRST, then
    /// passes the pre-spike values as residual overrides. The startle
    /// pipeline uses the original raw values; the variance calculation
    /// uses these "residual" values instead.
    pub fn tick_with_residual(
        &mut self,
        sensors: &MBotSensors,
        residual: &ResidualOverrides,
    ) -> (HomeostasisState, MotorCommand) {
        self.tick_count += 1;

        // Update position estimate from encoders
        self.update_odometry(sensors);

        // Compute homeostasis using residual-adjusted sensor values
        let state = self.compute_homeostasis(sensors, residual);

        // Generate motor command based on state (uses raw sensors for reactive behavior)
        let cmd = self.generate_command(sensors, &state);

        // Update last values
        self.last_distance = sensors.ultrasonic_cm;
        self.last_encoder_left = sensors.encoder_left;
        self.last_encoder_right = sensors.encoder_right;

        (state, cmd)
    }

    fn compute_homeostasis(
        &mut self,
        sensors: &MBotSensors,
        residual: &ResidualOverrides,
    ) -> HomeostasisState {
        // === TENSION CALCULATION ===
        // Apply residual overrides: when a stimulus spike was detected, use the
        // pre-spike values for variance-based tension so the spike only counts
        // once (through the startle pipeline, not here).

        let eff_distance = residual.ultrasonic_cm.unwrap_or(sensors.ultrasonic_cm);
        let eff_sound = residual.sound_level.unwrap_or(sensors.sound_level);

        // Proximity tension (closer = more tense)
        let proximity = if eff_distance < 100.0 {
            1.0 - (eff_distance / 100.0)
        } else {
            0.0
        };

        // Sudden change tension
        let distance_delta = fabsf(eff_distance - self.last_distance);
        let change_tension = (distance_delta / 50.0).min(1.0);

        // Sound tension
        let sound_tension = eff_sound * 0.5;

        // Movement tension (from accelerometer)
        let accel_magnitude = if let Some(am) = residual.accel_magnitude {
            am
        } else {
            sqrtf(powf(sensors.accel[0], 2.0) +
                  powf(sensors.accel[1], 2.0) +
                  powf(sensors.accel[2], 2.0))
        };
        let movement_tension = (accel_magnitude / 20.0).min(1.0);

        // Combined raw tension
        let raw_tension = (proximity * 0.5 +
                          change_tension * 0.2 +
                          sound_tension * 0.15 +
                          movement_tension * 0.15).min(1.0);

        // EMA smoothing
        self.tension_ema = self.alpha * raw_tension + (1.0 - self.alpha) * self.tension_ema;

        // === COHERENCE CALCULATION ===

        // Coherence drops with high/unstable tension
        let tension_instability = fabsf(raw_tension - self.tension_ema);
        let raw_coherence = 1.0 - (self.tension_ema * 0.4 + tension_instability * 0.6);
        self.coherence_ema = self.alpha * raw_coherence + (1.0 - self.alpha) * self.coherence_ema;

        // === ENERGY ===

        // Energy depletes with high tension, recovers when calm
        if self.tension_ema > 0.5 {
            self.energy = (self.energy - 0.001).max(0.1);
        } else {
            self.energy = (self.energy + 0.0005).min(1.0);
        }

        // === CURIOSITY ===

        // Curiosity increases when things are novel but not threatening
        let curiosity = if self.tension_ema > 0.2 && self.tension_ema < 0.6 {
            (self.coherence_ema * 0.7 + change_tension * 0.3).min(1.0)
        } else {
            0.2
        };

        HomeostasisState {
            tension: self.tension_ema.clamp(0.0, 1.0),
            coherence: self.coherence_ema.clamp(0.0, 1.0),
            reflex: ReflexMode::from_tension(self.tension_ema),
            energy: self.energy,
            curiosity,
            // These are set by the companion's main loop after CCF processing
            social_phase: coherence::SocialPhase::ShyObserver,
            context_coherence: 0.0,
        }
    }

    fn generate_command(&self, sensors: &MBotSensors, state: &HomeostasisState) -> MotorCommand {
        let (left, right) = match state.reflex {
            ReflexMode::Calm => {
                // Gentle wandering with occasional turns
                let wander = sinf((self.tick_count as f32) * 0.05) * 10.0;
                let speed = self.base_speed * 0.6 * self.energy;
                ((speed + wander) as i8, (speed - wander) as i8)
            }

            ReflexMode::Active => {
                // Active exploration - follow interesting stimuli
                let turn = if sensors.ultrasonic_cm < self.approach_distance {
                    // Something ahead - turn slightly to investigate
                    (state.curiosity * self.turn_gain) as i8
                } else {
                    // Wander more actively
                    let wander = sinf((self.tick_count as f32) * 0.1) * 20.0;
                    wander as i8
                };

                let speed = (self.base_speed * 0.8 * self.energy) as i8;
                (speed - turn, speed + turn)
            }

            ReflexMode::Spike => {
                // Excited! Move faster, more erratic
                let excitement = state.tension * 30.0;
                let speed = (self.base_speed * self.energy + excitement) as i8;

                // Sharp turns based on stimuli
                let turn = if sensors.ultrasonic_cm < 30.0 {
                    40_i8  // Sharp turn toward interesting thing
                } else {
                    (sinf(self.tick_count as f32 * 0.2) * 25.0) as i8
                };

                ((speed - turn).clamp(-100, 100), (speed + turn).clamp(-100, 100))
            }

            ReflexMode::Protect => {
                // DANGER! Back away
                if sensors.ultrasonic_cm < self.danger_distance {
                    (-60, -60)  // Back up
                } else {
                    // Turn away from threat
                    (-30, 50)
                }
            }
        };

        MotorCommand {
            left,
            right,
            pen_angle: if self.pen_down { 90 } else { 45 },
            led_color: state.reflex.led_color(),
            buzzer_hz: if state.reflex == ReflexMode::Protect { 440 } else { 0 },
        }
    }

    fn update_odometry(&mut self, sensors: &MBotSensors) {
        // Calculate wheel movement
        let left_delta = sensors.encoder_left - self.last_encoder_left;
        let right_delta = sensors.encoder_right - self.last_encoder_right;

        // Convert to distance (approximate)
        const TICKS_PER_CM: f32 = 10.0;  // Calibrate this!
        const WHEEL_BASE: f32 = 10.0;    // Distance between wheels in cm

        let left_dist = left_delta as f32 / TICKS_PER_CM;
        let right_dist = right_delta as f32 / TICKS_PER_CM;

        // Calculate movement
        let forward = (left_dist + right_dist) / 2.0;
        let rotation = (right_dist - left_dist) / WHEEL_BASE;

        // Update heading
        self.heading += rotation;

        // Update position
        self.position.0 += forward * cosf(self.heading);
        self.position.1 += forward * sinf(self.heading);
    }

    // === DRAWING METHODS ===

    /// Set pen state
    pub fn set_pen(&mut self, down: bool) {
        self.pen_down = down;
    }

    /// Get current estimated position
    pub fn position(&self) -> (f32, f32) {
        self.position
    }

    /// Get current heading in radians
    pub fn heading(&self) -> f32 {
        self.heading
    }

    /// Reset position tracking
    pub fn reset_position(&mut self) {
        self.position = (0.0, 0.0);
        self.heading = 0.0;
    }

    /// Get tick count
    pub fn tick_count(&self) -> u64 {
        self.tick_count
    }
}

impl Default for MBotBrain {
    fn default() -> Self {
        Self::new()
    }
}

// === PERSONALITY MODULE ===
pub mod personality;

// === LEARNINGLAB MODULE ===
pub mod learninglab;

// === GAMEBOT MODULE ===
pub mod gamebot;

// === HELPERBOT MODULE ===
pub mod helperbot;

// === MULTI-ROBOT COORDINATION MODULE ===
pub mod multi_robot;

// === PERFORMANCE PROFILING AND OPTIMIZATION MODULE ===
#[cfg(feature = "std")]
pub mod performance;

// === NERVOUS SYSTEM — STIMULUS DETECTION & STARTLE SUPPRESSION ===
pub mod nervous_system;

// === CONTEXTUAL COHERENCE FIELDS MODULE ===
pub mod coherence;

// === SPATIAL EXPLORATION MODULE ===
pub mod exploration;

// === REINFORCEMENT LEARNING MODULE ===
pub mod learning;

// === DRAWING HELPERS ===

/// Calculate motor powers to drive to a target position
pub fn drive_to_point(
    current: (f32, f32),
    heading: f32,
    target: (f32, f32),
    base_speed: f32,
) -> (i8, i8) {
    let dx = target.0 - current.0;
    let dy = target.1 - current.1;
    let distance = sqrtf(dx * dx + dy * dy);

    if distance < 1.0 {
        return (0, 0);  // Close enough
    }

    let target_angle = atan2f(dy, dx);
    let angle_diff = normalize_angle(target_angle - heading);

    // Proportional control
    let turn = (angle_diff * 50.0).clamp(-base_speed, base_speed);
    let speed = base_speed * (1.0 - (fabsf(angle_diff) / core::f32::consts::PI));

    (
        (speed - turn) as i8,
        (speed + turn) as i8,
    )
}

/// Normalize angle to [-PI, PI]
pub fn normalize_angle(angle: f32) -> f32 {
    let mut a = angle;
    while a > core::f32::consts::PI {
        a -= 2.0 * core::f32::consts::PI;
    }
    while a < -core::f32::consts::PI {
        a += 2.0 * core::f32::consts::PI;
    }
    a
}

/// Generate points for a circle
/// Generate points for a circle
pub fn circle_points_vec(center: (f32, f32), radius: f32, segments: usize) -> Vec<(f32, f32)> {
    let mut points = Vec::with_capacity(segments + 1);
    for i in 0..=segments {
        let angle = (i as f32 / segments as f32) * 2.0 * core::f32::consts::PI;
        points.push((
            center.0 + radius * cosf(angle),
            center.1 + radius * sinf(angle),
        ));
    }
    points
}

#[cfg(feature = "std")]
pub fn circle_points(center: (f32, f32), radius: f32, segments: usize) -> impl Iterator<Item = (f32, f32)> {
    (0..=segments).map(move |i| {
        let angle = (i as f32 / segments as f32) * 2.0 * core::f32::consts::PI;
        (
            center.0 + radius * cosf(angle),
            center.1 + radius * sinf(angle),
        )
    })
}

/// Generate points for an X
pub fn x_points(center: (f32, f32), size: f32) -> [(f32, f32); 5] {
    let half = size / 2.0;
    [
        (center.0 - half, center.1 - half),  // Top-left
        (center.0 + half, center.1 + half),  // Bottom-right
        (center.0, center.1),                 // Center (pen up point)
        (center.0 + half, center.1 - half),  // Top-right
        (center.0 - half, center.1 + half),  // Bottom-left
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reflex_modes() {
        assert_eq!(ReflexMode::from_tension(0.1), ReflexMode::Calm);
        assert_eq!(ReflexMode::from_tension(0.3), ReflexMode::Active);
        assert_eq!(ReflexMode::from_tension(0.7), ReflexMode::Spike);
        assert_eq!(ReflexMode::from_tension(0.9), ReflexMode::Protect);
    }

    #[test]
    fn test_brain_tick() {
        let mut brain = MBotBrain::new();
        let sensors = MBotSensors {
            ultrasonic_cm: 50.0,
            ..Default::default()
        };

        let (state, cmd) = brain.tick(&sensors);

        assert!(state.tension >= 0.0 && state.tension <= 1.0);
        assert!(state.coherence >= 0.0 && state.coherence <= 1.0);
        assert!(cmd.left >= -100 && cmd.left <= 100);
        assert!(cmd.right >= -100 && cmd.right <= 100);
    }

    #[test]
    fn test_protect_mode_backs_up() {
        let mut brain = MBotBrain::new();

        // Override EMA to force high tension directly (simulates accumulated danger)
        brain.tension_ema = 0.90;  // Force Protect mode

        // Test that Protect mode backs up when object is within danger zone
        let sensors = MBotSensors {
            ultrasonic_cm: 10.0,  // Within danger_distance (15.0)
            ..Default::default()
        };

        let (state, cmd) = brain.tick(&sensors);

        // Should be in Protect mode due to high tension
        assert_eq!(state.reflex, ReflexMode::Protect, "Should be in Protect mode with tension=0.90");

        // Should back up (both motors negative)
        assert!(cmd.left < 0 && cmd.right < 0,
                "Should back up in Protect mode when within danger distance, got left={} right={}",
                cmd.left, cmd.right);
    }

    #[test]
    fn test_normalize_angle() {
        use core::f32::consts::PI;

        assert!(fabsf(normalize_angle(0.0) - 0.0) < 0.001);
        assert!(fabsf(normalize_angle(PI) - PI) < 0.001);
        assert!(fabsf(normalize_angle(3.0 * PI) - PI) < 0.001);
        assert!(fabsf(normalize_angle(-3.0 * PI) - (-PI)) < 0.001);
    }

    // === Residual Sensor Stream Tests (Issue #11) ===

    #[test]
    fn test_tick_with_residual_backward_compatible() {
        // tick_with_residual with default overrides should produce the same
        // result as tick() -- both use raw sensor values.
        let mut brain_a = MBotBrain::new();
        let mut brain_b = MBotBrain::new();
        let sensors = MBotSensors {
            ultrasonic_cm: 50.0,
            sound_level: 0.3,
            light_level: 0.5,
            ..Default::default()
        };

        let (state_a, _) = brain_a.tick(&sensors);
        let (state_b, _) = brain_b.tick_with_residual(&sensors, &ResidualOverrides::default());

        assert!(
            fabsf(state_a.tension - state_b.tension) < 0.001,
            "Default residual overrides should match plain tick: {} vs {}",
            state_a.tension, state_b.tension,
        );
        assert!(
            fabsf(state_a.coherence - state_b.coherence) < 0.001,
            "Default residual overrides should match plain tick coherence: {} vs {}",
            state_a.coherence, state_b.coherence,
        );
    }

    #[test]
    fn test_residual_loudness_reduces_tension() {
        // A loud spike (sound_level = 0.9) causes high sound tension.
        // With residual override replacing sound_level with the quiet pre-spike
        // value (0.1), the tension should be lower.
        let spike_sensors = MBotSensors {
            ultrasonic_cm: 100.0, // far away, no proximity tension
            sound_level: 0.9,     // loud spike
            ..Default::default()
        };

        // Brain A: no residual (spike hits tension directly)
        let mut brain_a = MBotBrain::new();
        let (state_raw, _) = brain_a.tick(&spike_sensors);

        // Brain B: residual replaces sound with pre-spike quiet value
        let mut brain_b = MBotBrain::new();
        let residual = ResidualOverrides {
            sound_level: Some(0.1), // pre-spike: quiet
            ..Default::default()
        };
        let (state_residual, _) = brain_b.tick_with_residual(&spike_sensors, &residual);

        assert!(
            state_residual.tension < state_raw.tension,
            "Residual sound override should produce less tension: residual={} vs raw={}",
            state_residual.tension, state_raw.tension,
        );
    }

    #[test]
    fn test_residual_proximity_reduces_tension() {
        // Object suddenly very close (5cm) causes high proximity tension.
        // With residual override replacing distance with the pre-spike value
        // (100cm = far away), the tension should be lower.
        let spike_sensors = MBotSensors {
            ultrasonic_cm: 5.0,    // suddenly very close
            sound_level: 0.0,
            ..Default::default()
        };

        // Brain A: no residual
        let mut brain_a = MBotBrain::new();
        let (state_raw, _) = brain_a.tick(&spike_sensors);

        // Brain B: residual says it was far away before
        let mut brain_b = MBotBrain::new();
        let residual = ResidualOverrides {
            ultrasonic_cm: Some(100.0), // pre-spike: far away
            ..Default::default()
        };
        let (state_residual, _) = brain_b.tick_with_residual(&spike_sensors, &residual);

        assert!(
            state_residual.tension < state_raw.tension,
            "Residual distance override should produce less tension: residual={} vs raw={}",
            state_residual.tension, state_raw.tension,
        );
    }

    #[test]
    fn test_residual_accel_reduces_tension() {
        // Impact shock: high accelerometer spike.
        // With residual override using a calm baseline, tension should be lower.
        let spike_sensors = MBotSensors {
            ultrasonic_cm: 100.0,
            accel: [0.0, 0.0, 15.0], // high G-force spike
            ..Default::default()
        };

        // Brain A: no residual
        let mut brain_a = MBotBrain::new();
        let (state_raw, _) = brain_a.tick(&spike_sensors);

        // Brain B: residual uses calm 1G baseline
        let mut brain_b = MBotBrain::new();
        let residual = ResidualOverrides {
            accel_magnitude: Some(1.0), // pre-spike: calm 1G
            ..Default::default()
        };
        let (state_residual, _) = brain_b.tick_with_residual(&spike_sensors, &residual);

        assert!(
            state_residual.tension < state_raw.tension,
            "Residual accel override should produce less tension: residual={} vs raw={}",
            state_residual.tension, state_raw.tension,
        );
    }

    #[test]
    fn test_residual_combined_prevents_double_counting() {
        // Simulate the full double-counting scenario:
        // A loud clap at sound_level=0.8 with previous quiet at 0.1.
        // Without residual: the spike adds to both startle AND variance tension.
        // With residual: the spike only adds to startle tension; variance uses 0.1.
        //
        // We test that the difference between raw and residual tension is
        // proportional to the sound_tension contribution.

        let spike_sensors = MBotSensors {
            ultrasonic_cm: 100.0,
            sound_level: 0.8, // loud clap
            ..Default::default()
        };

        // Raw: sound_tension = 0.8 * 0.5 = 0.4
        let mut brain_raw = MBotBrain::new();
        let (state_raw, _) = brain_raw.tick(&spike_sensors);

        // Residual: sound_tension = 0.1 * 0.5 = 0.05
        let mut brain_res = MBotBrain::new();
        let residual = ResidualOverrides {
            sound_level: Some(0.1),
            ..Default::default()
        };
        let (state_res, _) = brain_res.tick_with_residual(&spike_sensors, &residual);

        // The raw tension should be higher by roughly the sound contribution difference.
        // sound_tension weight = 0.15, difference = (0.4 - 0.05) * 0.15 = 0.0525
        // After EMA smoothing (alpha=0.15), first tick delta ~ 0.0525 * 0.15 ~ 0.008
        let tension_diff = state_raw.tension - state_res.tension;
        assert!(
            tension_diff > 0.005,
            "Residual should meaningfully reduce tension. Raw={}, Residual={}, diff={}",
            state_raw.tension, state_res.tension, tension_diff,
        );
        assert!(
            tension_diff < 0.1,
            "Tension difference should be bounded (not overly large). diff={}",
            tension_diff,
        );
    }

    #[test]
    fn test_residual_motor_commands_use_raw_sensors() {
        // The generate_command function should use the raw sensor values for
        // reactive behavior, not the residual values. We verify this by
        // applying a sound residual override (which only affects tension)
        // and confirming the motor commands still react to the raw distance.
        //
        // Two brains see different raw distances (5cm vs 80cm) but both
        // have the same sound residual override. If generate_command used
        // residual values the commands would not differ by distance, but
        // since it uses raw sensors, the close brain backs up while the
        // far brain turns.

        // Brain A: close object (5cm), loud sound with residual
        let close_sensors = MBotSensors {
            ultrasonic_cm: 5.0,     // very close -> Protect backs up
            sound_level: 0.9,       // loud
            ..Default::default()
        };
        let mut brain_a = MBotBrain::new();
        brain_a.tension_ema = 0.98;  // High enough to stay in Protect after EMA blend

        let residual = ResidualOverrides {
            sound_level: Some(0.1), // Only override sound for tension calc
            ..Default::default()
        };
        let (state_a, cmd_a) = brain_a.tick_with_residual(&close_sensors, &residual);

        // Brain B: far object (80cm), same loud sound with same residual
        let far_sensors = MBotSensors {
            ultrasonic_cm: 80.0,    // far -> Protect turns away
            sound_level: 0.9,
            ..Default::default()
        };
        let mut brain_b = MBotBrain::new();
        brain_b.tension_ema = 0.98;

        let (state_b, cmd_b) = brain_b.tick_with_residual(&far_sensors, &residual);

        // Both should be in Protect mode (close sensor keeps raw tension high)
        assert_eq!(state_a.reflex, ReflexMode::Protect,
            "Brain A should be in Protect mode, tension={}",
            state_a.tension);
        assert_eq!(state_b.reflex, ReflexMode::Protect,
            "Brain B should be in Protect mode, tension={}",
            state_b.tension);

        // Motor commands should differ because generate_command sees the real
        // ultrasonic_cm (5cm vs 80cm).
        // At 5cm (within danger_distance=15), both motors = -60 (back up).
        // At 80cm (outside danger_distance), motors = (-30, 50) (turn away).
        assert!(
            cmd_a.left != cmd_b.left || cmd_a.right != cmd_b.right,
            "Motor commands should differ based on raw sensor distance. \
             Close: ({}, {}), Far: ({}, {})",
            cmd_a.left, cmd_a.right, cmd_b.left, cmd_b.right,
        );

        // The close brain should back up (both motors negative and equal)
        assert!(
            cmd_a.left < 0 && cmd_a.right < 0,
            "Close object should cause both motors to reverse: left={} right={}",
            cmd_a.left, cmd_a.right,
        );
    }

    #[test]
    fn test_residual_overrides_default_is_noop() {
        // Verify that ResidualOverrides::default() has all fields as None
        let r = ResidualOverrides::default();
        assert!(r.sound_level.is_none());
        assert!(r.light_level.is_none());
        assert!(r.ultrasonic_cm.is_none());
        assert!(r.accel_magnitude.is_none());
    }
}
