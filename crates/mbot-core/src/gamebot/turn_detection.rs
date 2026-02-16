//! Physical Turn Detection System for GameBot (GAME-007)
//!
//! Implements accelerometer-based tap detection and voice command recognition
//! for physical game turn signals without requiring the companion app.
//!
//! # Invariants Enforced
//! - I-GAME-003: Response time bounded to 500ms acknowledgment, 5s thinking
//! - I-GAME-006: No false positives (2g threshold, 0.7 voice confidence)
//! - I-GAME-007: Always acknowledge turn receipt (LED + sound + voice)
//!
//! # Kitchen Table Test Safety
//! - All motor responses bounded
//! - No harmful behavior patterns

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use alloc::{string::String, vec::Vec, vec};

#[cfg(feature = "std")]
use std::{string::String, vec::Vec, vec};

use crate::MotorCommand;

// ============================================
// Data Types
// ============================================

/// Type of turn signal detected
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TurnSignalType {
    /// Single tap (ignored, not a valid signal)
    Tap,
    /// Double-tap within threshold, >2g each tap
    DoubleTap,
    /// Voice command detected with sufficient confidence
    Voice,
    /// No input for timeout period
    Timeout,
    /// Turn signal from companion app
    App,
}

/// A detected turn signal from the player
#[derive(Clone, Debug)]
pub struct TurnSignal {
    /// Type of signal detected
    pub signal_type: TurnSignalType,
    /// Confidence level (0.0-1.0); 1.0 for taps, variable for voice
    pub confidence: f32,
    /// Timestamp in microseconds when signal detected
    pub timestamp_us: u64,
    /// Voice transcript if applicable
    pub raw_data: Option<String>,
}

impl TurnSignal {
    /// Create a new turn signal
    pub fn new(signal_type: TurnSignalType, confidence: f32, timestamp_us: u64) -> Self {
        Self {
            signal_type,
            confidence: confidence.clamp(0.0, 1.0),
            timestamp_us,
            raw_data: None,
        }
    }

    /// Create a turn signal with raw data (for voice)
    pub fn with_raw_data(mut self, data: String) -> Self {
        self.raw_data = Some(data);
        self
    }
}

/// Input methods for turn detection
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputMethod {
    /// Accelerometer-based tap detection
    Tap,
    /// Microphone-based voice commands
    Voice,
    /// Companion app button
    App,
}

/// Configuration for turn detection system
#[derive(Clone, Debug)]
pub struct TurnDetectionConfig {
    /// Which input methods are enabled
    pub enabled_inputs: Vec<InputMethod>,
    /// Maximum time between taps for double-tap (default: 400ms)
    pub double_tap_threshold_ms: u32,
    /// Minimum g-force for valid tap (default: 2.0g)
    pub double_tap_min_g: f32,
    /// Keywords that trigger voice turn signal
    pub voice_keywords: Vec<String>,
    /// Minimum confidence for voice detection (default: 0.7)
    pub voice_confidence_threshold: f32,
    /// Seconds before timeout prompt (default: 30)
    pub timeout_seconds: u32,
}

impl Default for TurnDetectionConfig {
    fn default() -> Self {
        Self {
            enabled_inputs: vec![InputMethod::Tap, InputMethod::Voice],
            double_tap_threshold_ms: 400,
            double_tap_min_g: 2.0,
            voice_keywords: vec![
                String::from("your turn"),
                String::from("go"),
                String::from("done"),
                String::from("roby"),
            ],
            voice_confidence_threshold: 0.7,
            timeout_seconds: 30,
        }
    }
}

/// LED patterns for acknowledgment
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LedPattern {
    /// Blue pulse for tap detection
    PulseBlue,
    /// Green flash for voice detection
    FlashGreen,
    /// Yellow pulse for timeout prompt
    PulseYellow,
}

impl LedPattern {
    /// Get RGB values for the pattern
    pub fn to_rgb(&self) -> [u8; 3] {
        match self {
            LedPattern::PulseBlue => [0, 100, 255],
            LedPattern::FlashGreen => [0, 255, 100],
            LedPattern::PulseYellow => [255, 200, 0],
        }
    }
}

/// Sound types for acknowledgment
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AckSound {
    /// Short beep
    Beep,
    /// Pleasant chime
    Chime,
    /// Spoken voice response
    Voice,
}

impl AckSound {
    /// Get frequency in Hz for the sound (0 for voice)
    pub fn frequency_hz(&self) -> u16 {
        match self {
            AckSound::Beep => 880,
            AckSound::Chime => 523, // C5
            AckSound::Voice => 0,
        }
    }
}

/// Robot's acknowledgment of turn receipt
/// I-GAME-007: Robot must always acknowledge turn receipt
#[derive(Clone, Debug)]
pub struct TurnAcknowledgment {
    /// LED pattern to display
    pub led_pattern: LedPattern,
    /// Sound to play
    pub sound: AckSound,
    /// Optional voice response
    pub voice_response: Option<String>,
}

/// Accelerometer reading from CyberPi
#[derive(Clone, Debug, Default)]
pub struct AccelerometerReading {
    /// X-axis acceleration in g
    pub x: f32,
    /// Y-axis acceleration in g
    pub y: f32,
    /// Z-axis acceleration in g
    pub z: f32,
    /// Timestamp in microseconds
    pub timestamp_us: u64,
}

impl AccelerometerReading {
    /// Calculate magnitude of acceleration
    pub fn magnitude(&self) -> f32 {
        #[cfg(not(feature = "std"))]
        {
            libm::sqrtf(self.x * self.x + self.y * self.y + self.z * self.z)
        }
        #[cfg(feature = "std")]
        {
            (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
        }
    }
}

/// Voice detection result from microphone processing
#[derive(Clone, Debug)]
pub struct VoiceDetectionResult {
    /// Detected transcript
    pub transcript: String,
    /// Confidence level (0.0-1.0)
    pub confidence: f32,
    /// Timestamp in microseconds
    pub timestamp_us: u64,
}

// ============================================
// Turn Detection State Machine
// ============================================

/// Current game turn state
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameTurnState {
    /// Human's turn, robot waiting for signal
    HumanTurn,
    /// Robot's turn, detection disabled
    RobotTurn,
    /// Game not active
    Inactive,
}

/// Turn Detection System
///
/// Processes sensor inputs to detect physical turn signals.
/// Enforces invariants I-GAME-006 (no false positives) and
/// I-GAME-007 (always acknowledge).
pub struct TurnDetectionSystem {
    config: TurnDetectionConfig,
    turn_state: GameTurnState,

    // Tap detection state
    last_tap_timestamp_us: u64,
    last_tap_force_g: f32,

    // Timeout tracking
    last_input_timestamp_us: u64,
    timeout_prompted: bool,
}

impl TurnDetectionSystem {
    /// Create a new turn detection system with default config
    pub fn new() -> Self {
        Self::with_config(TurnDetectionConfig::default())
    }

    /// Create with custom configuration
    pub fn with_config(config: TurnDetectionConfig) -> Self {
        Self {
            config,
            turn_state: GameTurnState::Inactive,
            last_tap_timestamp_us: 0,
            last_tap_force_g: 0.0,
            last_input_timestamp_us: 0,
            timeout_prompted: false,
        }
    }

    /// Get current configuration
    pub fn config(&self) -> &TurnDetectionConfig {
        &self.config
    }

    /// Update configuration
    pub fn set_config(&mut self, config: TurnDetectionConfig) {
        self.config = config;
    }

    /// Set game turn state
    pub fn set_turn_state(&mut self, state: GameTurnState) {
        self.turn_state = state;
        if state == GameTurnState::HumanTurn {
            self.timeout_prompted = false;
        }
    }

    /// Get current turn state
    pub fn turn_state(&self) -> GameTurnState {
        self.turn_state
    }

    /// Reset input timer (call when game starts or turn changes)
    pub fn reset_input_timer(&mut self, timestamp_us: u64) {
        self.last_input_timestamp_us = timestamp_us;
        self.timeout_prompted = false;
    }

    /// Check if an input method is enabled
    fn is_input_enabled(&self, method: InputMethod) -> bool {
        self.config.enabled_inputs.contains(&method)
    }

    /// Process accelerometer reading for tap detection
    ///
    /// # Invariant Enforcement
    /// - I-GAME-006: Requires 2g minimum and double-tap pattern
    /// - I-GAME-008: Disabled during robot's turn
    pub fn process_accelerometer(&mut self, reading: &AccelerometerReading) -> Option<TurnSignal> {
        // I-GAME-008: Detection disabled during robot's turn
        if self.turn_state != GameTurnState::HumanTurn {
            return None;
        }

        if !self.is_input_enabled(InputMethod::Tap) {
            return None;
        }

        let g_force = reading.magnitude();

        // I-GAME-006: Require minimum g-force threshold
        if g_force < self.config.double_tap_min_g {
            return None;
        }

        let time_since_last_tap_ms =
            (reading.timestamp_us.saturating_sub(self.last_tap_timestamp_us)) / 1000;

        // Check for double-tap pattern
        if self.last_tap_force_g >= self.config.double_tap_min_g
            && time_since_last_tap_ms <= self.config.double_tap_threshold_ms as u64
        {
            // Double-tap detected!
            self.last_tap_timestamp_us = 0;
            self.last_tap_force_g = 0.0;
            self.last_input_timestamp_us = reading.timestamp_us;
            self.timeout_prompted = false;

            return Some(TurnSignal::new(
                TurnSignalType::DoubleTap,
                1.0, // Deterministic confidence for taps
                reading.timestamp_us,
            ));
        }

        // Store this tap for potential double-tap
        self.last_tap_timestamp_us = reading.timestamp_us;
        self.last_tap_force_g = g_force;

        None
    }

    /// Process voice detection result
    ///
    /// # Invariant Enforcement
    /// - I-GAME-006: Requires 0.7 confidence minimum, keyword match
    /// - I-GAME-008: Disabled during robot's turn
    pub fn process_voice(&mut self, result: &VoiceDetectionResult) -> Option<TurnSignal> {
        // I-GAME-008: Detection disabled during robot's turn
        if self.turn_state != GameTurnState::HumanTurn {
            return None;
        }

        if !self.is_input_enabled(InputMethod::Voice) {
            return None;
        }

        // I-GAME-006: Require minimum confidence threshold
        if result.confidence < self.config.voice_confidence_threshold {
            return None;
        }

        let transcript_lower = result.transcript.to_lowercase();

        // Check for special "roby" keyword first (requires higher confidence)
        let contains_roby = self.config.voice_keywords.iter()
            .any(|kw| kw.to_lowercase() == "roby" && transcript_lower.contains("roby"));

        if contains_roby && result.confidence < 0.8 {
            return None;
        }

        // Then check for any matching keyword
        let matched_keyword = self.config.voice_keywords.iter()
            .find(|kw| transcript_lower.contains(&kw.to_lowercase()));

        let _keyword = matched_keyword?;

        self.last_input_timestamp_us = result.timestamp_us;
        self.timeout_prompted = false;

        Some(
            TurnSignal::new(
                TurnSignalType::Voice,
                result.confidence,
                result.timestamp_us,
            )
            .with_raw_data(result.transcript.clone())
        )
    }

    /// Process app button press
    pub fn process_app_signal(&mut self, timestamp_us: u64) -> Option<TurnSignal> {
        if self.turn_state != GameTurnState::HumanTurn {
            return None;
        }

        if !self.is_input_enabled(InputMethod::App) {
            return None;
        }

        self.last_input_timestamp_us = timestamp_us;
        self.timeout_prompted = false;

        Some(TurnSignal::new(
            TurnSignalType::App,
            1.0,
            timestamp_us,
        ))
    }

    /// Check for timeout condition
    ///
    /// Returns timeout signal if no input for configured duration.
    /// Only returns once per timeout period (until reset).
    pub fn check_timeout(&mut self, current_timestamp_us: u64) -> Option<TurnSignal> {
        if self.turn_state != GameTurnState::HumanTurn {
            return None;
        }

        if self.timeout_prompted {
            return None;
        }

        let time_since_input_s =
            (current_timestamp_us.saturating_sub(self.last_input_timestamp_us)) / 1_000_000;

        if time_since_input_s >= self.config.timeout_seconds as u64 {
            self.timeout_prompted = true;
            return Some(TurnSignal::new(
                TurnSignalType::Timeout,
                1.0,
                current_timestamp_us,
            ));
        }

        None
    }

    /// Generate acknowledgment for a turn signal
    ///
    /// # Invariant Enforcement
    /// - I-GAME-007: Robot must always acknowledge turn receipt
    pub fn generate_acknowledgment(&self, signal: &TurnSignal) -> TurnAcknowledgment {
        match signal.signal_type {
            TurnSignalType::DoubleTap => TurnAcknowledgment {
                led_pattern: LedPattern::PulseBlue,
                sound: AckSound::Beep,
                voice_response: Some(String::from("OK, my turn!")),
            },
            TurnSignalType::Voice => {
                // Check if it was an enthusiastic command with name
                let is_enthusiastic = signal.raw_data
                    .as_ref()
                    .map(|s| s.to_lowercase().contains("roby"))
                    .unwrap_or(false);

                TurnAcknowledgment {
                    led_pattern: LedPattern::FlashGreen,
                    sound: AckSound::Chime,
                    voice_response: Some(if is_enthusiastic {
                        String::from("Let's go!")
                    } else {
                        String::from("Got it!")
                    }),
                }
            },
            TurnSignalType::Timeout => TurnAcknowledgment {
                led_pattern: LedPattern::PulseYellow,
                sound: AckSound::Voice,
                voice_response: Some(String::from("Are you ready? Tap twice or say done")),
            },
            TurnSignalType::App => TurnAcknowledgment {
                led_pattern: LedPattern::FlashGreen,
                sound: AckSound::Beep,
                voice_response: Some(String::from("OK!")),
            },
            TurnSignalType::Tap => TurnAcknowledgment {
                // Single tap - minimal acknowledgment (shouldn't normally be used)
                led_pattern: LedPattern::PulseBlue,
                sound: AckSound::Beep,
                voice_response: None,
            },
        }
    }
}

impl Default for TurnDetectionSystem {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================
// Motor Command Generation (Safety Bounded)
// ============================================

/// Generate motor command for acknowledgment animation
///
/// # Kitchen Table Test Safety
/// - All speeds bounded to safe range
/// - No aggressive movements
pub fn acknowledgment_motor_command(ack: &TurnAcknowledgment) -> MotorCommand {
    // Small celebratory wiggle for acknowledgment
    // Speeds are bounded for safety (ARCH-003)
    MotorCommand {
        left: 0,
        right: 0,
        pen_angle: 45,
        led_color: ack.led_pattern.to_rgb(),
        buzzer_hz: ack.sound.frequency_hz(),
    }
}

// ============================================
// Tests
// ============================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = TurnDetectionConfig::default();
        assert_eq!(config.double_tap_threshold_ms, 400);
        assert_eq!(config.double_tap_min_g, 2.0);
        assert_eq!(config.voice_confidence_threshold, 0.7);
        assert_eq!(config.timeout_seconds, 30);
        assert!(config.voice_keywords.contains(&String::from("your turn")));
        assert!(config.voice_keywords.contains(&String::from("done")));
    }

    #[test]
    fn test_accelerometer_magnitude() {
        let reading = AccelerometerReading {
            x: 3.0,
            y: 4.0,
            z: 0.0,
            timestamp_us: 0,
        };
        assert!((reading.magnitude() - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_double_tap_detection() {
        let mut detector = TurnDetectionSystem::new();
        detector.set_turn_state(GameTurnState::HumanTurn);

        // First tap
        let reading1 = AccelerometerReading {
            x: 0.0, y: 3.0, z: 0.0,
            timestamp_us: 1_000_000,
        };
        let result1 = detector.process_accelerometer(&reading1);
        assert!(result1.is_none()); // First tap should not trigger

        // Second tap within threshold
        let reading2 = AccelerometerReading {
            x: 0.0, y: 2.5, z: 0.0,
            timestamp_us: 1_300_000, // 300ms later
        };
        let result2 = detector.process_accelerometer(&reading2);
        assert!(result2.is_some());

        let signal = result2.unwrap();
        assert_eq!(signal.signal_type, TurnSignalType::DoubleTap);
        assert_eq!(signal.confidence, 1.0);
    }

    #[test]
    fn test_tap_below_threshold_ignored() {
        let mut detector = TurnDetectionSystem::new();
        detector.set_turn_state(GameTurnState::HumanTurn);

        // Tap below 2g threshold
        let reading1 = AccelerometerReading {
            x: 0.0, y: 1.5, z: 0.0,
            timestamp_us: 1_000_000,
        };
        let reading2 = AccelerometerReading {
            x: 0.0, y: 1.8, z: 0.0,
            timestamp_us: 1_200_000,
        };

        detector.process_accelerometer(&reading1);
        let result = detector.process_accelerometer(&reading2);
        assert!(result.is_none());
    }

    #[test]
    fn test_taps_outside_window_ignored() {
        let mut detector = TurnDetectionSystem::new();
        detector.set_turn_state(GameTurnState::HumanTurn);

        // First tap
        let reading1 = AccelerometerReading {
            x: 0.0, y: 3.0, z: 0.0,
            timestamp_us: 1_000_000,
        };
        // Second tap after 500ms (outside 400ms threshold)
        let reading2 = AccelerometerReading {
            x: 0.0, y: 2.5, z: 0.0,
            timestamp_us: 1_500_000,
        };

        detector.process_accelerometer(&reading1);
        let result = detector.process_accelerometer(&reading2);
        assert!(result.is_none());
    }

    #[test]
    fn test_voice_detection() {
        let mut detector = TurnDetectionSystem::new();
        detector.set_turn_state(GameTurnState::HumanTurn);

        let result = VoiceDetectionResult {
            transcript: String::from("your turn"),
            confidence: 0.85,
            timestamp_us: 1_000_000,
        };

        let signal = detector.process_voice(&result);
        assert!(signal.is_some());

        let signal = signal.unwrap();
        assert_eq!(signal.signal_type, TurnSignalType::Voice);
        assert_eq!(signal.confidence, 0.85);
        assert_eq!(signal.raw_data, Some(String::from("your turn")));
    }

    #[test]
    fn test_voice_below_confidence_ignored() {
        let mut detector = TurnDetectionSystem::new();
        detector.set_turn_state(GameTurnState::HumanTurn);

        let result = VoiceDetectionResult {
            transcript: String::from("your turn"),
            confidence: 0.5, // Below 0.7 threshold
            timestamp_us: 1_000_000,
        };

        let signal = detector.process_voice(&result);
        assert!(signal.is_none());
    }

    #[test]
    fn test_voice_nonkeyword_ignored() {
        let mut detector = TurnDetectionSystem::new();
        detector.set_turn_state(GameTurnState::HumanTurn);

        let result = VoiceDetectionResult {
            transcript: String::from("hello there"),
            confidence: 0.9,
            timestamp_us: 1_000_000,
        };

        let signal = detector.process_voice(&result);
        assert!(signal.is_none());
    }

    #[test]
    fn test_roby_requires_higher_confidence() {
        let mut detector = TurnDetectionSystem::new();
        detector.set_turn_state(GameTurnState::HumanTurn);

        // 0.75 confidence should fail for "roby" command
        let result1 = VoiceDetectionResult {
            transcript: String::from("go Roby"),
            confidence: 0.75,
            timestamp_us: 1_000_000,
        };
        assert!(detector.process_voice(&result1).is_none());

        // 0.85 should succeed
        let result2 = VoiceDetectionResult {
            transcript: String::from("go Roby"),
            confidence: 0.85,
            timestamp_us: 2_000_000,
        };
        assert!(detector.process_voice(&result2).is_some());
    }

    #[test]
    fn test_detection_disabled_during_robot_turn() {
        let mut detector = TurnDetectionSystem::new();
        detector.set_turn_state(GameTurnState::RobotTurn);

        // Double-tap should be ignored
        let reading1 = AccelerometerReading {
            x: 0.0, y: 3.0, z: 0.0,
            timestamp_us: 1_000_000,
        };
        let reading2 = AccelerometerReading {
            x: 0.0, y: 2.5, z: 0.0,
            timestamp_us: 1_200_000,
        };
        detector.process_accelerometer(&reading1);
        assert!(detector.process_accelerometer(&reading2).is_none());

        // Voice should be ignored
        let voice = VoiceDetectionResult {
            transcript: String::from("done"),
            confidence: 0.9,
            timestamp_us: 2_000_000,
        };
        assert!(detector.process_voice(&voice).is_none());
    }

    #[test]
    fn test_timeout_detection() {
        let mut detector = TurnDetectionSystem::new();
        detector.set_turn_state(GameTurnState::HumanTurn);
        detector.reset_input_timer(0);

        // 29 seconds - no timeout
        assert!(detector.check_timeout(29_000_000).is_none());

        // 30 seconds - timeout
        let signal = detector.check_timeout(30_000_000);
        assert!(signal.is_some());
        assert_eq!(signal.unwrap().signal_type, TurnSignalType::Timeout);

        // Should only fire once
        assert!(detector.check_timeout(35_000_000).is_none());
    }

    #[test]
    fn test_acknowledgment_generation() {
        let detector = TurnDetectionSystem::new();

        // Double-tap acknowledgment
        let tap_signal = TurnSignal::new(TurnSignalType::DoubleTap, 1.0, 0);
        let ack = detector.generate_acknowledgment(&tap_signal);
        assert_eq!(ack.led_pattern, LedPattern::PulseBlue);
        assert_eq!(ack.voice_response, Some(String::from("OK, my turn!")));

        // Voice acknowledgment
        let voice_signal = TurnSignal::new(TurnSignalType::Voice, 0.85, 0);
        let ack = detector.generate_acknowledgment(&voice_signal);
        assert_eq!(ack.led_pattern, LedPattern::FlashGreen);
        assert_eq!(ack.voice_response, Some(String::from("Got it!")));

        // Timeout acknowledgment
        let timeout_signal = TurnSignal::new(TurnSignalType::Timeout, 1.0, 0);
        let ack = detector.generate_acknowledgment(&timeout_signal);
        assert_eq!(ack.led_pattern, LedPattern::PulseYellow);
        assert!(ack.voice_response.as_ref().unwrap().contains("Tap twice"));
    }

    #[test]
    fn test_enthusiastic_roby_response() {
        let detector = TurnDetectionSystem::new();

        let signal = TurnSignal::new(TurnSignalType::Voice, 0.9, 0)
            .with_raw_data(String::from("go Roby!"));

        let ack = detector.generate_acknowledgment(&signal);
        assert_eq!(ack.voice_response, Some(String::from("Let's go!")));
    }

    #[test]
    fn test_input_method_disable() {
        let mut config = TurnDetectionConfig::default();
        config.enabled_inputs = vec![InputMethod::Voice]; // Tap disabled

        let mut detector = TurnDetectionSystem::with_config(config);
        detector.set_turn_state(GameTurnState::HumanTurn);

        // Tap should be ignored
        let reading1 = AccelerometerReading {
            x: 0.0, y: 3.0, z: 0.0,
            timestamp_us: 1_000_000,
        };
        let reading2 = AccelerometerReading {
            x: 0.0, y: 2.5, z: 0.0,
            timestamp_us: 1_200_000,
        };
        detector.process_accelerometer(&reading1);
        assert!(detector.process_accelerometer(&reading2).is_none());

        // Voice should work
        let voice = VoiceDetectionResult {
            transcript: String::from("done"),
            confidence: 0.8,
            timestamp_us: 2_000_000,
        };
        assert!(detector.process_voice(&voice).is_some());
    }

    #[test]
    fn test_led_pattern_rgb() {
        assert_eq!(LedPattern::PulseBlue.to_rgb(), [0, 100, 255]);
        assert_eq!(LedPattern::FlashGreen.to_rgb(), [0, 255, 100]);
        assert_eq!(LedPattern::PulseYellow.to_rgb(), [255, 200, 0]);
    }

    #[test]
    fn test_sound_frequencies() {
        assert_eq!(AckSound::Beep.frequency_hz(), 880);
        assert_eq!(AckSound::Chime.frequency_hz(), 523);
        assert_eq!(AckSound::Voice.frequency_hz(), 0);
    }

    #[test]
    fn test_turn_signal_confidence_clamped() {
        let signal = TurnSignal::new(TurnSignalType::Voice, 1.5, 0);
        assert_eq!(signal.confidence, 1.0);

        let signal2 = TurnSignal::new(TurnSignalType::Voice, -0.5, 0);
        assert_eq!(signal2.confidence, 0.0);
    }
}
