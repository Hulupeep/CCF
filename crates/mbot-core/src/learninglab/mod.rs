//! LearningLab Module - Educational visualization system for mBot2
//!
//! Provides real-time telemetry streaming and data structures for visualizing
//! the robot's nervous system state in real-time.
//!
//! # Invariants Enforced
//! - I-LEARN-001: Visualizer updates must complete within 100ms of state change
//! - I-LEARN-002: All displayed values must reflect actual robot state
//! - I-LEARN-003: Gauge animations must be smooth (60fps target)
//! - I-LEARN-004: WebSocket connection must auto-reconnect on disconnect

pub mod telemetry;

pub use telemetry::{
    VisualizerState,
    VisualizerConfig,
    TelemetryBuffer,
    TelemetryPoint,
    SensorReadings,
    MotorOutputs,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_learninglab_module_loads() {
        // Module should load without panic
        let _config = VisualizerConfig::default();
    }
}
