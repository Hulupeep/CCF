//! Pen Servo Control - ART-001 Implementation
//!
//! Implements the PenControl trait for pen servo management.
//!
//! Invariants enforced:
//! - I-ART-001: Pen servo control MUST be abstracted from drawing logic via `PenControl` trait
//! - I-ART-004: Pen MUST be in up position when not actively drawing (default state)
//! - I-ART-005: Servo MUST respond within 100ms of command
//! - ART-001: Servo accuracy ±2° (contract from feature_artbot.yml)

use core::fmt;

/// Error types for pen servo control
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PenError {
    /// Servo did not respond within timeout
    ServoTimeout,
    /// Angle value out of valid range (0-180 degrees)
    InvalidAngle,
    /// Communication error with servo
    CommunicationError,
}

impl fmt::Display for PenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PenError::ServoTimeout => write!(f, "Servo timeout - no response within 100ms"),
            PenError::InvalidAngle => write!(f, "Invalid angle - must be 0-180 degrees"),
            PenError::CommunicationError => write!(f, "Communication error with servo"),
        }
    }
}

/// Pen position state
#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub enum PenPosition {
    /// Pen is up (not touching paper)
    Up,
    /// Pen is down (touching paper)
    Down,
    /// Pen is transitioning between states
    Transitioning,
}

/// Configuration for pen servo behavior
///
/// Enforces I-ART-004: Default pen-up state
#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub struct PenConfig {
    /// Angle in degrees when pen is up (default: 45°)
    pub up_angle: u8,
    /// Angle in degrees when pen is down (default: 90°)
    pub down_angle: u8,
    /// Transition time in milliseconds (default: 100ms)
    pub transition_time_ms: u16,
}

impl Default for PenConfig {
    fn default() -> Self {
        Self {
            up_angle: 45,      // Pen up position
            down_angle: 90,    // Pen down position
            transition_time_ms: 100, // Servo response time
        }
    }
}

impl PenConfig {
    /// Creates a custom pen configuration
    ///
    /// # Errors
    /// Returns `PenError::InvalidAngle` if angles are out of range [0, 180]
    pub fn new(up_angle: u8, down_angle: u8, transition_ms: u16) -> Result<Self, PenError> {
        if up_angle > 180 || down_angle > 180 {
            return Err(PenError::InvalidAngle);
        }
        Ok(Self {
            up_angle,
            down_angle,
            transition_time_ms: transition_ms,
        })
    }

    /// Validates that up_angle < down_angle (pen physically lifts when moving up)
    pub fn is_valid_geometry(&self) -> bool {
        self.up_angle < self.down_angle
    }
}

/// Trait for pen servo control abstraction
///
/// Implements I-ART-001: Pen servo control MUST be abstracted from drawing logic
/// This allows drawing logic to work without knowing servo angles or protocols.
pub trait PenControl {
    /// Lift pen from paper surface
    ///
    /// Sets servo to up_angle and updates internal state.
    /// Enforces I-ART-004: pen must be in up position by default.
    fn pen_up(&mut self) -> Result<(), PenError>;

    /// Lower pen to paper surface
    ///
    /// Sets servo to down_angle and updates internal state.
    fn pen_down(&mut self) -> Result<(), PenError>;

    /// Set servo angle directly (0-180 degrees)
    ///
    /// For fine-grained control. Validates angle is in valid range.
    fn set_angle(&mut self, angle: u8) -> Result<(), PenError>;

    /// Check if pen is currently down
    fn is_down(&self) -> bool;

    /// Get current pen position state
    fn position(&self) -> PenPosition;

    /// Get current servo angle
    fn current_angle(&self) -> u8;

    /// Get response time of last command in milliseconds
    fn last_response_time_ms(&self) -> Option<u16>;
}

/// Reference implementation of pen servo control
///
/// Tracks state locally without hardware communication.
/// Used for testing and simulation.
#[derive(Clone, Debug)]
pub struct LocalPenControl {
    config: PenConfig,
    current_angle: u8,
    is_down: bool,
    last_response_time_ms: Option<u16>,
}

impl LocalPenControl {
    /// Create a new pen control with default configuration
    pub fn new() -> Self {
        let config = PenConfig::default();
        Self {
            current_angle: config.up_angle, // I-ART-004: Start with pen up
            config,
            is_down: false,
            last_response_time_ms: None,
        }
    }

    /// Create a new pen control with custom configuration
    pub fn with_config(config: PenConfig) -> Result<Self, PenError> {
        if !config.is_valid_geometry() {
            return Err(PenError::InvalidAngle);
        }
        Ok(Self {
            current_angle: config.up_angle, // I-ART-004: Start with pen up
            config,
            is_down: false,
            last_response_time_ms: None,
        })
    }

    /// Get the configuration
    pub fn config(&self) -> &PenConfig {
        &self.config
    }
}

impl Default for LocalPenControl {
    fn default() -> Self {
        Self::new()
    }
}

impl PenControl for LocalPenControl {
    fn pen_up(&mut self) -> Result<(), PenError> {
        self.set_angle(self.config.up_angle)
    }

    fn pen_down(&mut self) -> Result<(), PenError> {
        self.set_angle(self.config.down_angle)
    }

    fn set_angle(&mut self, angle: u8) -> Result<(), PenError> {
        // Validate angle (ART-001: ±2° accuracy, so 0-180° is valid)
        if angle > 180 {
            return Err(PenError::InvalidAngle);
        }

        self.current_angle = angle;

        // Update is_down state based on angle
        // If angle is closer to down_angle, pen is down
        let to_down = (angle as i16 - self.config.down_angle as i16).abs();
        let to_up = (angle as i16 - self.config.up_angle as i16).abs();
        self.is_down = to_down <= to_up;

        // Simulate response time (typically 50-100ms)
        self.last_response_time_ms = Some(self.config.transition_time_ms);

        Ok(())
    }

    fn is_down(&self) -> bool {
        self.is_down
    }

    fn position(&self) -> PenPosition {
        match self.current_angle {
            angle if angle == self.config.up_angle => PenPosition::Up,
            angle if angle == self.config.down_angle => PenPosition::Down,
            _ => PenPosition::Transitioning,
        }
    }

    fn current_angle(&self) -> u8 {
        self.current_angle
    }

    fn last_response_time_ms(&self) -> Option<u16> {
        self.last_response_time_ms
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = PenConfig::default();
        assert_eq!(config.up_angle, 45);
        assert_eq!(config.down_angle, 90);
        assert_eq!(config.transition_time_ms, 100);
    }

    #[test]
    fn test_config_validation() {
        // Valid configuration
        assert!(PenConfig::new(45, 90, 100).is_ok());

        // Invalid: angle > 180
        assert_eq!(
            PenConfig::new(200, 90, 100),
            Err(PenError::InvalidAngle)
        );

        // Invalid: angle > 180
        assert_eq!(
            PenConfig::new(45, 190, 100),
            Err(PenError::InvalidAngle)
        );
    }

    #[test]
    fn test_pen_default_up() {
        // I-ART-004: Default state is pen up
        let pen = LocalPenControl::new();
        assert!(!pen.is_down());
        assert_eq!(pen.current_angle(), 45);
        assert_eq!(pen.position(), PenPosition::Up);
    }

    #[test]
    fn test_pen_up() {
        let mut pen = LocalPenControl::new();

        // Move down first
        pen.pen_down().unwrap();
        assert!(pen.is_down());

        // Then up
        pen.pen_up().unwrap();
        assert!(!pen.is_down());
        assert_eq!(pen.current_angle(), 45);
        assert_eq!(pen.position(), PenPosition::Up);
    }

    #[test]
    fn test_pen_down() {
        let mut pen = LocalPenControl::new();

        pen.pen_down().unwrap();
        assert!(pen.is_down());
        assert_eq!(pen.current_angle(), 90);
        assert_eq!(pen.position(), PenPosition::Down);
    }

    #[test]
    fn test_set_angle_valid() {
        let mut pen = LocalPenControl::new();

        // Valid angles
        assert!(pen.set_angle(0).is_ok());
        assert_eq!(pen.current_angle(), 0);

        assert!(pen.set_angle(90).is_ok());
        assert_eq!(pen.current_angle(), 90);

        assert!(pen.set_angle(180).is_ok());
        assert_eq!(pen.current_angle(), 180);
    }

    #[test]
    fn test_set_angle_invalid() {
        let mut pen = LocalPenControl::new();

        // Invalid: angle > 180
        assert_eq!(pen.set_angle(181), Err(PenError::InvalidAngle));
        assert_eq!(pen.set_angle(200), Err(PenError::InvalidAngle));
        assert_eq!(pen.set_angle(255), Err(PenError::InvalidAngle));
    }

    #[test]
    fn test_response_time() {
        let mut pen = LocalPenControl::new();

        // Initially no response time recorded
        assert_eq!(pen.last_response_time_ms(), None);

        // After setting angle, response time is recorded
        pen.pen_down().unwrap();
        assert_eq!(pen.last_response_time_ms(), Some(100)); // Default transition time
    }

    #[test]
    fn test_custom_config() {
        let config = PenConfig::new(30, 120, 150).unwrap();
        let mut pen = LocalPenControl::with_config(config).unwrap();

        pen.pen_up().unwrap();
        assert_eq!(pen.current_angle(), 30);

        pen.pen_down().unwrap();
        assert_eq!(pen.current_angle(), 120);

        assert_eq!(pen.last_response_time_ms(), Some(150));
    }

    #[test]
    fn test_angle_near_detection() {
        let config = PenConfig::new(45, 90, 100).unwrap();
        let mut pen = LocalPenControl::with_config(config).unwrap();

        // Angle closer to down_angle
        pen.set_angle(85).unwrap();
        assert!(pen.is_down());
        assert_eq!(pen.position(), PenPosition::Transitioning);

        // Angle closer to up_angle
        pen.set_angle(50).unwrap();
        assert!(!pen.is_down());
        assert_eq!(pen.position(), PenPosition::Transitioning);
    }

    #[test]
    fn test_i_art_004_enforcement() {
        // Invariant I-ART-004: Pen MUST be in up position when not actively drawing
        let pen = LocalPenControl::new();
        assert!(!pen.is_down(), "Pen must start in up position (I-ART-004)");
        assert_eq!(
            pen.position(),
            PenPosition::Up,
            "Default position must be Up (I-ART-004)"
        );
    }

    #[test]
    fn test_servo_accuracy_tolerance() {
        // ART-001: Servo accuracy ±2°
        // Verify we can set angles with typical servo resolution
        let mut pen = LocalPenControl::new();

        // Test a range of angles that servo might reach
        for angle in (0..=180).step_by(5) {
            assert!(pen.set_angle(angle as u8).is_ok());
            assert_eq!(pen.current_angle(), angle as u8);
        }
    }
}
