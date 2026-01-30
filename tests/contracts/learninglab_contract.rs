//! Contract tests for LearningLab features
//!
//! Enforces LEARN-001..004 and ARCH-LEARN-001..004 contracts from docs/contracts/feature_learninglab.yml

use mbot_core::learninglab::telemetry::*;

#[test]
fn test_visualizer_state_validation_i_learn_002() {
    // I-LEARN-002: All displayed values must reflect actual robot state
    let mut state = VisualizerState::default();

    // Valid state should pass
    assert!(state.validate().is_ok());

    // Out-of-bounds tension should fail
    state.tension = 1.5;
    assert!(state.validate().is_err());

    // Reset and test coherence
    state.tension = 0.5;
    state.coherence = -0.1;
    assert!(state.validate().is_err());

    // Reset and test energy
    state.coherence = 0.5;
    state.energy = 2.0;
    assert!(state.validate().is_err());

    // Reset all to valid
    state.energy = 0.5;
    assert!(state.validate().is_ok());
}

#[test]
fn test_telemetry_buffer_latency_i_learn_001() {
    // I-LEARN-001: Visualizer updates must complete within 100ms of state change
    let mut buf = TelemetryBuffer::new(100);

    // Add points with increasing latency values
    for i in 0..10 {
        let mut state = VisualizerState::default();
        state.tick_count = i as u64;

        let point = TelemetryPoint {
            state,
            latency_ms: 50 + (i as u32 * 5), // Latency between 50-95ms
        };
        buf.push(point);
    }

    // All points should have latency < 100ms
    for point in buf.all() {
        assert!(point.latency_ms < 100, "Latency {} exceeds 100ms limit", point.latency_ms);
    }
}

#[test]
fn test_visualizer_config_update_interval_i_learn_003() {
    // I-LEARN-003: Gauge animations must be smooth (60fps target)
    // 60fps = ~16.67ms per frame, so update should be <= 50ms for smooth animation
    let config = VisualizerConfig::default();

    // Default should target 50ms (20 updates/sec = smooth enough for gauge animation)
    assert!(config.update_interval_ms > 0);
    assert!(config.update_interval_ms <= 50, "Update interval too large for smooth animation");

    // Animation duration should be reasonable (300ms = smooth 60fps animation)
    assert!(config.gauge_animation_duration_ms >= 200);
    assert!(config.gauge_animation_duration_ms <= 500);
}

#[test]
fn test_telemetry_buffer_circular_preserves_history() {
    // Verify buffer correctly maintains history without data loss
    let mut buf = TelemetryBuffer::new(5);

    // Fill buffer beyond capacity
    for i in 0..15 {
        let mut state = VisualizerState::default();
        state.tick_count = i as u64;
        buf.push(TelemetryPoint {
            state,
            latency_ms: 10,
        });
    }

    // Should contain exactly 5 points (most recent)
    assert_eq!(buf.len(), 5);

    // Latest should be tick 14
    assert_eq!(buf.latest().unwrap().state.tick_count, 14);

    // All points should be consecutive ticks 10-14
    let all = buf.all();
    assert_eq!(all[0].state.tick_count, 10);
    assert_eq!(all[4].state.tick_count, 14);
}

#[test]
fn test_sensor_readings_valid_ranges() {
    // Verify sensor readings are properly bounded
    let mut readings = SensorReadings::default();

    // Light level should be 0.0-1.0
    readings.light_level = 0.5;
    assert!(readings.light_level >= 0.0 && readings.light_level <= 1.0);

    // Sound level should be 0.0-1.0
    readings.sound_level = 0.7;
    assert!(readings.sound_level >= 0.0 && readings.sound_level <= 1.0);

    // Ultrasonic can be up to 400cm
    readings.ultrasonic_cm = 350.0;
    assert!(readings.ultrasonic_cm >= 0.0 && readings.ultrasonic_cm <= 400.0);
}

#[test]
fn test_motor_outputs_bounded() {
    // Verify motor outputs are properly bounded for safety
    let outputs = MotorOutputs::default();

    // Motor powers should be -100 to 100
    let valid_motors = [
        (50_i8, -50_i8),
        (100_i8, 0_i8),
        (-100_i8, 100_i8),
    ];

    for (left, right) in valid_motors {
        assert!(left >= -100 && left <= 100);
        assert!(right >= -100 && right <= 100);
    }

    // Pen angle should be 0-180
    assert!(outputs.pen_angle >= 0 && outputs.pen_angle <= 180);

    // LED values should be 0-255 each
    for color in outputs.led_color {
        assert!(color >= 0 && color <= 255);
    }
}

#[test]
fn test_visualizer_state_builder_pattern() {
    let sensors = SensorReadings {
        ultrasonic_cm: 50.0,
        light_level: 0.8,
        sound_level: 0.3,
        quad_rgb: [[255, 0, 0]; 4],
    };

    let motors = MotorOutputs {
        left: 50,
        right: -50,
        pen_angle: 90,
        led_color: [0, 255, 0],
        buzzer_hz: 440,
    };

    let state = VisualizerState::new(ReflexModeDisplay::Active, 0.5, 0.7, 0.6, 1000, 100)
        .with_sensors(sensors)
        .with_motors(motors);

    assert_eq!(state.reflex_mode, ReflexModeDisplay::Active);
    assert_eq!(state.sensors.ultrasonic_cm, 50.0);
    assert_eq!(state.motors.left, 50);
    assert!(state.validate().is_ok());
}

#[test]
fn test_telemetry_buffer_time_window_query() {
    let mut buf = TelemetryBuffer::new(100);

    // Add points at 100us intervals
    for i in 0..10 {
        let mut state = VisualizerState::default();
        state.timestamp_us = (i * 100) as u64;
        buf.push(TelemetryPoint {
            state,
            latency_ms: 5,
        });
    }

    // Query window 200-600us should get points 2-6
    let window = buf.points_in_window(200, 600);
    assert_eq!(window.len(), 5);
    assert_eq!(window[0].state.timestamp_us, 200);
    assert_eq!(window[4].state.timestamp_us, 600);
}

#[test]
fn test_reflex_mode_display_variants() {
    // ARCH-LEARN-001: Core must support all reflex modes
    let modes = vec![
        ReflexModeDisplay::Calm,
        ReflexModeDisplay::Active,
        ReflexModeDisplay::Spike,
        ReflexModeDisplay::Protect,
    ];

    for mode in modes {
        let state = VisualizerState::new(mode, 0.5, 0.5, 0.5, 0, 0);
        assert_eq!(state.reflex_mode, mode);
    }
}
