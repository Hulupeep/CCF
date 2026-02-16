//! Real-Time Telemetry Streaming for LearningLab Visualizer
//!
//! Implements data structures and buffering for streaming the robot's nervous system
//! state to the web dashboard with < 100ms latency (I-LEARN-001).

#![cfg_attr(feature = "no_std", allow(unused_imports))]

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use alloc::{string::{String, ToString}, vec::Vec, format};

#[cfg(feature = "std")]
use std::{string::String, vec::Vec};

use core::fmt;

/// Reflex modes for display
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReflexModeDisplay {
    Calm,
    Active,
    Spike,
    Protect,
}

impl fmt::Display for ReflexModeDisplay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReflexModeDisplay::Calm => write!(f, "calm"),
            ReflexModeDisplay::Active => write!(f, "active"),
            ReflexModeDisplay::Spike => write!(f, "spike"),
            ReflexModeDisplay::Protect => write!(f, "protect"),
        }
    }
}

/// Sensor readings snapshot for visualization
#[derive(Clone, Debug)]
pub struct SensorReadings {
    /// Ultrasonic distance in cm (0-400)
    pub ultrasonic_cm: f32,
    /// Light sensor reading (0.0-1.0)
    pub light_level: f32,
    /// Sound level from microphone (0.0-1.0)
    pub sound_level: f32,
    /// Quad RGB sensor readings [front_left, front_right, back_left, back_right]
    pub quad_rgb: [[u8; 3]; 4],
}

impl Default for SensorReadings {
    fn default() -> Self {
        Self {
            ultrasonic_cm: 0.0,
            light_level: 0.0,
            sound_level: 0.0,
            quad_rgb: [[0, 0, 0]; 4],
        }
    }
}

/// Motor output values for visualization
#[derive(Clone, Debug)]
pub struct MotorOutputs {
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

impl Default for MotorOutputs {
    fn default() -> Self {
        Self {
            left: 0,
            right: 0,
            pen_angle: 45,
            led_color: [0, 0, 0],
            buzzer_hz: 0,
        }
    }
}

/// Real-time nervous system state for visualization
///
/// Captures the complete state of the robot's brain at a single point in time.
/// This is streamed to the web dashboard for real-time visualization.
#[derive(Clone, Debug)]
pub struct VisualizerState {
    /// Current reflex mode (calm/active/spike/protect)
    pub reflex_mode: ReflexModeDisplay,
    /// Tension level (0.0-1.0)
    pub tension: f32,
    /// Coherence level (0.0-1.0)
    pub coherence: f32,
    /// Energy level (0.0-1.0)
    pub energy: f32,
    /// Sensor readings
    pub sensors: SensorReadings,
    /// Motor outputs
    pub motors: MotorOutputs,
    /// Timestamp in microseconds
    pub timestamp_us: u64,
    /// Tick count from brain
    pub tick_count: u64,
}

impl VisualizerState {
    /// Create a new visualizer state
    pub fn new(
        reflex_mode: ReflexModeDisplay,
        tension: f32,
        coherence: f32,
        energy: f32,
        timestamp_us: u64,
        tick_count: u64,
    ) -> Self {
        Self {
            reflex_mode,
            tension: tension.clamp(0.0, 1.0),
            coherence: coherence.clamp(0.0, 1.0),
            energy: energy.clamp(0.0, 1.0),
            sensors: SensorReadings::default(),
            motors: MotorOutputs::default(),
            timestamp_us,
            tick_count,
        }
    }

    /// Add sensor readings to this state
    pub fn with_sensors(mut self, sensors: SensorReadings) -> Self {
        self.sensors = sensors;
        self
    }

    /// Add motor outputs to this state
    pub fn with_motors(mut self, motors: MotorOutputs) -> Self {
        self.motors = motors;
        self
    }

    /// Validate all values are within bounds (I-LEARN-002)
    pub fn validate(&self) -> Result<(), String> {
        if !(0.0..=1.0).contains(&self.tension) {
            return Err(String::from("tension out of bounds"));
        }
        if !(0.0..=1.0).contains(&self.coherence) {
            return Err(String::from("coherence out of bounds"));
        }
        if !(0.0..=1.0).contains(&self.energy) {
            return Err(String::from("energy out of bounds"));
        }
        if self.sensors.light_level < 0.0 || self.sensors.light_level > 1.0 {
            return Err(String::from("light_level out of bounds"));
        }
        if self.sensors.sound_level < 0.0 || self.sensors.sound_level > 1.0 {
            return Err(String::from("sound_level out of bounds"));
        }
        Ok(())
    }
}

impl Default for VisualizerState {
    fn default() -> Self {
        Self {
            reflex_mode: ReflexModeDisplay::Calm,
            tension: 0.0,
            coherence: 1.0,
            energy: 0.5,
            sensors: SensorReadings::default(),
            motors: MotorOutputs::default(),
            timestamp_us: 0,
            tick_count: 0,
        }
    }
}

/// Configuration for telemetry streaming
#[derive(Clone, Debug)]
pub struct VisualizerConfig {
    /// Target update interval in milliseconds (default: 50ms)
    pub update_interval_ms: u32,
    /// Gauge animation duration in milliseconds
    pub gauge_animation_duration_ms: u32,
    /// Whether to display DAG node indicators
    pub show_dag_nodes: bool,
    /// Sensor display mode
    pub sensor_display_mode: SensorDisplayMode,
    /// Maximum buffer size for telemetry history
    pub max_buffer_size: usize,
}

/// Sensor display mode options
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SensorDisplayMode {
    /// Only numeric values
    Numeric,
    /// Only graphical representation
    Graphical,
    /// Both numeric and graphical
    Both,
}

impl Default for VisualizerConfig {
    fn default() -> Self {
        Self {
            update_interval_ms: 50,        // 20 fps target
            gauge_animation_duration_ms: 300,
            show_dag_nodes: true,
            sensor_display_mode: SensorDisplayMode::Both,
            max_buffer_size: 1200,         // 60 seconds at 20fps
        }
    }
}

/// A single data point in the telemetry buffer
#[derive(Clone, Debug)]
pub struct TelemetryPoint {
    /// The visualizer state at this point
    pub state: VisualizerState,
    /// Latency in milliseconds from state update to buffer receipt
    pub latency_ms: u32,
}

/// Circular buffer for telemetry history (I-LEARN-001: < 100ms latency)
///
/// Stores the last N telemetry points for smooth gauge animations
/// and historical visualization. Default buffer holds last 60 seconds.
pub struct TelemetryBuffer {
    buffer: Vec<TelemetryPoint>,
    capacity: usize,
    write_index: usize,
    count: usize,
}

impl TelemetryBuffer {
    /// Create a new telemetry buffer with given capacity
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: Vec::with_capacity(capacity),
            capacity,
            write_index: 0,
            count: 0,
        }
    }

    /// Add a point to the buffer
    pub fn push(&mut self, point: TelemetryPoint) {
        if self.buffer.len() < self.capacity {
            self.buffer.push(point);
            self.count = self.buffer.len();
        } else {
            // Circular overwrite
            self.buffer[self.write_index] = point;
        }
        self.write_index = (self.write_index + 1) % self.capacity;
    }

    /// Get the most recent point
    pub fn latest(&self) -> Option<&TelemetryPoint> {
        if self.count == 0 {
            return None;
        }
        let idx = if self.write_index == 0 {
            self.count - 1
        } else {
            self.write_index - 1
        };
        self.buffer.get(idx)
    }

    /// Get all points in chronological order
    pub fn all(&self) -> Vec<&TelemetryPoint> {
        if self.count == 0 {
            return Vec::new();
        }

        let mut result = Vec::with_capacity(self.count);
        if self.count < self.capacity {
            // Buffer not full yet, return in order
            for i in 0..self.count {
                result.push(&self.buffer[i]);
            }
        } else {
            // Buffer is full, start from write_index
            for i in 0..self.capacity {
                let idx = (self.write_index + i) % self.capacity;
                result.push(&self.buffer[idx]);
            }
        }
        result
    }

    /// Get the number of points in buffer
    pub fn len(&self) -> usize {
        self.count
    }

    /// Check if buffer is empty
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Clear the buffer
    pub fn clear(&mut self) {
        self.buffer.clear();
        self.write_index = 0;
        self.count = 0;
    }

    /// Get points within a time window (in microseconds)
    pub fn points_in_window(&self, start_us: u64, end_us: u64) -> Vec<&TelemetryPoint> {
        self.all()
            .into_iter()
            .filter(|p| p.state.timestamp_us >= start_us && p.state.timestamp_us <= end_us)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reflex_mode_variants() {
        // Test that enum variants exist and can be created
        let _calm = ReflexModeDisplay::Calm;
        let _active = ReflexModeDisplay::Active;
        let _spike = ReflexModeDisplay::Spike;
        let _protect = ReflexModeDisplay::Protect;

        // Test equality
        assert_eq!(ReflexModeDisplay::Calm, ReflexModeDisplay::Calm);
        assert_ne!(ReflexModeDisplay::Calm, ReflexModeDisplay::Active);
    }

    #[test]
    fn test_visualizer_state_validation() {
        let mut state = VisualizerState::default();
        assert!(state.validate().is_ok());

        state.tension = 1.5;
        assert!(state.validate().is_err());

        state.tension = 0.5;
        state.coherence = -0.1;
        assert!(state.validate().is_err());

        state.coherence = 0.5;
        state.sensors.light_level = 1.5;
        assert!(state.validate().is_err());
    }

    #[test]
    fn test_visualizer_state_clamping() {
        let state = VisualizerState::new(
            ReflexModeDisplay::Calm,
            1.5,  // Will be clamped to 1.0
            -0.5, // Will be clamped to 0.0
            0.5,
            1000,
            100,
        );

        assert_eq!(state.tension, 1.0);
        assert_eq!(state.coherence, 0.0);
        assert_eq!(state.energy, 0.5);
    }

    #[test]
    fn test_visualizer_config_default() {
        let config = VisualizerConfig::default();
        assert_eq!(config.update_interval_ms, 50);
        assert_eq!(config.gauge_animation_duration_ms, 300);
        assert!(config.show_dag_nodes);
    }

    #[test]
    fn test_telemetry_buffer_basic() {
        let mut buf = TelemetryBuffer::new(10);
        assert!(buf.is_empty());
        assert_eq!(buf.len(), 0);

        let point = TelemetryPoint {
            state: VisualizerState::default(),
            latency_ms: 5,
        };

        buf.push(point);
        assert_eq!(buf.len(), 1);
        assert!(buf.latest().is_some());
    }

    #[test]
    fn test_telemetry_buffer_circular() {
        let mut buf = TelemetryBuffer::new(5);

        for i in 0..10 {
            let mut state = VisualizerState::default();
            state.tick_count = i as u64;
            buf.push(TelemetryPoint {
                state,
                latency_ms: 5,
            });
        }

        // Should only keep last 5 points
        assert_eq!(buf.len(), 5);

        // Latest should be tick 9
        assert_eq!(buf.latest().unwrap().state.tick_count, 9);

        // Buffer should contain ticks 5-9 in order
        let all = buf.all();
        assert_eq!(all[0].state.tick_count, 5);
        assert_eq!(all[4].state.tick_count, 9);
    }

    #[test]
    fn test_telemetry_buffer_window() {
        let mut buf = TelemetryBuffer::new(100);

        for i in 0..10 {
            let mut state = VisualizerState::default();
            state.timestamp_us = (i * 1000) as u64;
            buf.push(TelemetryPoint {
                state,
                latency_ms: 5,
            });
        }

        let window = buf.points_in_window(2000, 6000);
        assert_eq!(window.len(), 5); // points 2-6
    }

    #[test]
    fn test_sensor_readings_default() {
        let readings = SensorReadings::default();
        assert_eq!(readings.ultrasonic_cm, 0.0);
        assert_eq!(readings.light_level, 0.0);
        assert_eq!(readings.sound_level, 0.0);
    }

    #[test]
    fn test_motor_outputs_default() {
        let outputs = MotorOutputs::default();
        assert_eq!(outputs.left, 0);
        assert_eq!(outputs.right, 0);
        assert_eq!(outputs.pen_angle, 45);
    }

    #[test]
    fn test_visualizer_state_with_sensors() {
        let state = VisualizerState::default();
        let mut sensors = SensorReadings::default();
        sensors.ultrasonic_cm = 50.0;

        let state = state.with_sensors(sensors);
        assert_eq!(state.sensors.ultrasonic_cm, 50.0);
    }

    #[test]
    fn test_visualizer_state_with_motors() {
        let state = VisualizerState::default();
        let mut motors = MotorOutputs::default();
        motors.left = 50;
        motors.right = -50;

        let state = state.with_motors(motors);
        assert_eq!(state.motors.left, 50);
        assert_eq!(state.motors.right, -50);
    }
}
