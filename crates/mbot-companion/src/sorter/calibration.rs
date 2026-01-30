//! Servo Calibration System for LEGO Sorter
//!
//! Implements SORT-001 (Repeatable Calibration) and SORT-003 (Safety)
//!
//! This module provides a servo calibration wizard that records safe angle ranges
//! for the lift and gripper servos, storing calibration profiles that persist
//! across power cycles.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

// ============================================
// Core Data Structures (Contract: SORT-001)
// ============================================

/// Lift servo configuration
/// Contract: SORT-001 ServoCalibration.lift
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LiftServo {
    /// GPIO pin number (default: 4)
    pub pin: u8,
    /// Safe minimum angle (up position, ~80°)
    pub min_angle: u16,
    /// Safe maximum angle (down position, ~160°)
    pub max_angle: u16,
    /// Rest/home position
    pub home_angle: u16,
    /// Position for picking up pieces
    pub pick_angle: u16,
    /// Position for dropping pieces
    pub drop_angle: u16,
}

impl Default for LiftServo {
    fn default() -> Self {
        Self {
            pin: 4,
            min_angle: 80,
            max_angle: 160,
            home_angle: 90,
            pick_angle: 150,
            drop_angle: 140,
        }
    }
}

impl LiftServo {
    /// Check if an angle is within safe limits
    pub fn is_safe_angle(&self, angle: u16) -> bool {
        angle >= self.min_angle && angle <= self.max_angle
    }

    /// Get the angle range (must be at least 30° per contract)
    pub fn angle_range(&self) -> u16 {
        self.max_angle.saturating_sub(self.min_angle)
    }

    /// Validate that the configuration meets safety requirements
    pub fn validate(&self) -> Result<(), String> {
        if self.angle_range() < 30 {
            return Err(format!(
                "Range too narrow for safe operation: {}° (need at least 30°)",
                self.angle_range()
            ));
        }

        if !self.is_safe_angle(self.home_angle) {
            return Err(format!(
                "Home angle {}° is outside safe range [{}, {}]",
                self.home_angle, self.min_angle, self.max_angle
            ));
        }

        if !self.is_safe_angle(self.pick_angle) {
            return Err(format!(
                "Pick angle {}° is outside safe range [{}, {}]",
                self.pick_angle, self.min_angle, self.max_angle
            ));
        }

        if !self.is_safe_angle(self.drop_angle) {
            return Err(format!(
                "Drop angle {}° is outside safe range [{}, {}]",
                self.drop_angle, self.min_angle, self.max_angle
            ));
        }

        Ok(())
    }
}

/// Gripper servo configuration
/// Contract: SORT-001 ServoCalibration.gripper
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GripperServo {
    /// GPIO pin number (default: 3)
    pub pin: u8,
    /// Fully open angle (~80°)
    pub open_angle: u16,
    /// Fully closed angle (~140°)
    pub closed_angle: u16,
    /// Holding pressure angle (between open and closed)
    pub grip_angle: u16,
}

impl Default for GripperServo {
    fn default() -> Self {
        Self {
            pin: 3,
            open_angle: 80,
            closed_angle: 140,
            grip_angle: 110,
        }
    }
}

impl GripperServo {
    /// Check if an angle is within safe limits
    pub fn is_safe_angle(&self, angle: u16) -> bool {
        let min = self.open_angle.min(self.closed_angle);
        let max = self.open_angle.max(self.closed_angle);
        angle >= min && angle <= max
    }

    /// Validate that the configuration meets safety requirements
    pub fn validate(&self) -> Result<(), String> {
        if !self.is_safe_angle(self.grip_angle) {
            return Err(format!(
                "Grip angle {}° is outside safe range [{}, {}]",
                self.grip_angle,
                self.open_angle.min(self.closed_angle),
                self.open_angle.max(self.closed_angle)
            ));
        }

        let range = self
            .closed_angle
            .abs_diff(self.open_angle);
        if range < 20 {
            return Err(format!(
                "Gripper range too narrow: {}° (need at least 20°)",
                range
            ));
        }

        Ok(())
    }
}

/// Complete servo calibration profile
/// Contract: SORT-001 ServoCalibration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ServoCalibration {
    /// Robot identifier
    pub robot_id: String,
    /// Creation timestamp (microseconds since epoch)
    pub created_at: u64,
    /// Lift servo configuration
    pub lift: LiftServo,
    /// Gripper servo configuration
    pub gripper: GripperServo,
    /// Whether calibration has been verified
    pub verified: bool,
    /// Verification timestamp (microseconds since epoch)
    pub verification_timestamp: Option<u64>,
}

impl Default for ServoCalibration {
    fn default() -> Self {
        Self {
            robot_id: "default_robot".to_string(),
            created_at: 0,
            lift: LiftServo::default(),
            gripper: GripperServo::default(),
            verified: false,
            verification_timestamp: None,
        }
    }
}

impl ServoCalibration {
    /// Create a new calibration profile
    pub fn new(robot_id: impl Into<String>, timestamp: u64) -> Self {
        Self {
            robot_id: robot_id.into(),
            created_at: timestamp,
            lift: LiftServo::default(),
            gripper: GripperServo::default(),
            verified: false,
            verification_timestamp: None,
        }
    }

    /// Validate the entire calibration profile
    /// Contract: SORT-001 - Must enforce safety constraints
    pub fn validate(&self) -> Result<(), String> {
        self.lift.validate()?;
        self.gripper.validate()?;
        Ok(())
    }

    /// Mark calibration as verified
    pub fn mark_verified(&mut self, timestamp: u64) {
        self.verified = true;
        self.verification_timestamp = Some(timestamp);
    }

    /// Check if calibration is verified and recent
    pub fn is_verified(&self) -> bool {
        self.verified && self.verification_timestamp.is_some()
    }

    /// Save to a JSON file
    /// Contract: SORT-001 - Persist calibration across power cycles
    pub fn save_to_file(&self, path: impl AsRef<Path>) -> Result<(), String> {
        self.validate()?;

        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize calibration: {}", e))?;

        fs::write(path, json).map_err(|e| format!("Failed to write calibration file: {}", e))?;

        Ok(())
    }

    /// Load from a JSON file
    /// Contract: SORT-001 - Restore calibration on startup
    pub fn load_from_file(path: impl AsRef<Path>) -> Result<Self, String> {
        let json = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read calibration file: {}", e))?;

        let calibration: ServoCalibration = serde_json::from_str(&json)
            .map_err(|e| format!("Failed to parse calibration: {}", e))?;

        calibration.validate()?;

        Ok(calibration)
    }

    /// Check if angles match recorded values within tolerance (±2°)
    /// Contract: SORT-001 - ±2° accuracy requirement
    pub fn verify_angle(&self, recorded: u16, actual: u16) -> bool {
        recorded.abs_diff(actual) <= 2
    }
}

/// Calibration wizard step
/// Contract: SORT-001 CalibrationStep
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CalibrationStep {
    /// Initial step - move to safe neutral position
    Start,
    /// Record lift minimum (up) position
    LiftMin,
    /// Record lift maximum (down) position
    LiftMax,
    /// Record lift home position
    LiftHome,
    /// Record lift pick position
    LiftPick,
    /// Record lift drop position
    LiftDrop,
    /// Record gripper open position
    GripperOpen,
    /// Record gripper closed position
    GripperClosed,
    /// Record gripper grip position
    GripperGrip,
    /// Verification test cycle
    Verification,
    /// Calibration complete
    Complete,
}

impl CalibrationStep {
    /// Get the human-readable instruction for this step
    pub fn instruction(&self) -> &'static str {
        match self {
            CalibrationStep::Start => "Press button to begin calibration. Robot will move to safe neutral position.",
            CalibrationStep::LiftMin => "Move lift to highest safe position, then press confirm.",
            CalibrationStep::LiftMax => "Move lift to lowest safe position, then press confirm.",
            CalibrationStep::LiftHome => "Move lift to rest/home position, then press confirm.",
            CalibrationStep::LiftPick => "Move lift to picking position (near ground), then press confirm.",
            CalibrationStep::LiftDrop => "Move lift to dropping position (over bins), then press confirm.",
            CalibrationStep::GripperOpen => "Open gripper fully, then press confirm.",
            CalibrationStep::GripperClosed => "Close gripper fully, then press confirm.",
            CalibrationStep::GripperGrip => "Set gripper to holding pressure (grip a piece), then press confirm.",
            CalibrationStep::Verification => "Press verify to test all positions with a cycle.",
            CalibrationStep::Complete => "Calibration complete! Press save to persist.",
        }
    }

    /// Get the servo being calibrated
    pub fn servo(&self) -> Option<&'static str> {
        match self {
            CalibrationStep::Start => None,
            CalibrationStep::LiftMin
            | CalibrationStep::LiftMax
            | CalibrationStep::LiftHome
            | CalibrationStep::LiftPick
            | CalibrationStep::LiftDrop => Some("lift"),
            CalibrationStep::GripperOpen
            | CalibrationStep::GripperClosed
            | CalibrationStep::GripperGrip => Some("gripper"),
            CalibrationStep::Verification | CalibrationStep::Complete => None,
        }
    }

    /// Get the next step
    pub fn next(&self) -> Option<CalibrationStep> {
        match self {
            CalibrationStep::Start => Some(CalibrationStep::LiftMin),
            CalibrationStep::LiftMin => Some(CalibrationStep::LiftMax),
            CalibrationStep::LiftMax => Some(CalibrationStep::LiftHome),
            CalibrationStep::LiftHome => Some(CalibrationStep::LiftPick),
            CalibrationStep::LiftPick => Some(CalibrationStep::LiftDrop),
            CalibrationStep::LiftDrop => Some(CalibrationStep::GripperOpen),
            CalibrationStep::GripperOpen => Some(CalibrationStep::GripperClosed),
            CalibrationStep::GripperClosed => Some(CalibrationStep::GripperGrip),
            CalibrationStep::GripperGrip => Some(CalibrationStep::Verification),
            CalibrationStep::Verification => Some(CalibrationStep::Complete),
            CalibrationStep::Complete => None,
        }
    }

    /// Get step number (0-indexed)
    pub fn step_number(&self) -> usize {
        match self {
            CalibrationStep::Start => 0,
            CalibrationStep::LiftMin => 1,
            CalibrationStep::LiftMax => 2,
            CalibrationStep::LiftHome => 3,
            CalibrationStep::LiftPick => 4,
            CalibrationStep::LiftDrop => 5,
            CalibrationStep::GripperOpen => 6,
            CalibrationStep::GripperClosed => 7,
            CalibrationStep::GripperGrip => 8,
            CalibrationStep::Verification => 9,
            CalibrationStep::Complete => 10,
        }
    }

    /// Total number of steps
    pub fn total_steps() -> usize {
        11
    }
}

/// Calibration profile for persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalibrationProfile {
    pub calibration: ServoCalibration,
    pub file_version: u32,
}

impl CalibrationProfile {
    const VERSION: u32 = 1;

    pub fn new(calibration: ServoCalibration) -> Self {
        Self {
            calibration,
            file_version: Self::VERSION,
        }
    }

    /// Get default calibration file path
    pub fn default_path() -> PathBuf {
        PathBuf::from("config/servo_calibration.json")
    }

    /// Load or create default profile
    pub fn load_or_default() -> Self {
        let path = Self::default_path();
        if path.exists() {
            ServoCalibration::load_from_file(&path)
                .map(Self::new)
                .unwrap_or_else(|_| Self::new(ServoCalibration::default()))
        } else {
            Self::new(ServoCalibration::default())
        }
    }

    /// Save profile to default location
    pub fn save(&self) -> Result<(), String> {
        let path = Self::default_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create config directory: {}", e))?;
        }
        self.calibration.save_to_file(path)
    }
}

// ============================================
// Calibration Wizard
// ============================================

/// Interactive calibration wizard
pub struct ServoCalibrationWizard {
    current_step: CalibrationStep,
    calibration: ServoCalibration,
    emergency_stop: bool,
}

impl ServoCalibrationWizard {
    /// Create a new calibration wizard
    pub fn new(robot_id: impl Into<String>) -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_micros() as u64;

        Self {
            current_step: CalibrationStep::Start,
            calibration: ServoCalibration::new(robot_id, timestamp),
            emergency_stop: false,
        }
    }

    /// Get the current step
    pub fn current_step(&self) -> &CalibrationStep {
        &self.current_step
    }

    /// Get the current instruction text
    pub fn instruction(&self) -> &'static str {
        self.current_step.instruction()
    }

    /// Get progress (current step / total steps)
    pub fn progress(&self) -> (usize, usize) {
        (
            self.current_step.step_number(),
            CalibrationStep::total_steps(),
        )
    }

    /// Record an angle for the current step
    pub fn record_angle(&mut self, angle: u16) -> Result<(), String> {
        if self.emergency_stop {
            return Err("Wizard paused - resume or cancel first".to_string());
        }

        match self.current_step {
            CalibrationStep::LiftMin => {
                self.calibration.lift.min_angle = angle;
            }
            CalibrationStep::LiftMax => {
                self.calibration.lift.max_angle = angle;
            }
            CalibrationStep::LiftHome => {
                self.calibration.lift.home_angle = angle;
            }
            CalibrationStep::LiftPick => {
                self.calibration.lift.pick_angle = angle;
            }
            CalibrationStep::LiftDrop => {
                self.calibration.lift.drop_angle = angle;
            }
            CalibrationStep::GripperOpen => {
                self.calibration.gripper.open_angle = angle;
            }
            CalibrationStep::GripperClosed => {
                self.calibration.gripper.closed_angle = angle;
            }
            CalibrationStep::GripperGrip => {
                self.calibration.gripper.grip_angle = angle;
            }
            _ => {
                return Err(format!("Cannot record angle at step {:?}", self.current_step));
            }
        }

        Ok(())
    }

    /// Advance to the next step
    pub fn next_step(&mut self) -> Result<(), String> {
        if self.emergency_stop {
            return Err("Wizard paused - resume or cancel first".to_string());
        }

        // Validate before advancing from certain steps
        if matches!(
            self.current_step,
            CalibrationStep::LiftDrop | CalibrationStep::GripperGrip
        ) {
            if let Err(e) = self.calibration.validate() {
                return Err(format!("Validation failed: {}", e));
            }
        }

        if let Some(next) = self.current_step.next() {
            self.current_step = next;
            Ok(())
        } else {
            Err("Already at final step".to_string())
        }
    }

    /// Emergency stop - pause wizard
    /// Contract: SORT-003 - Emergency stop must be available
    pub fn emergency_stop(&mut self) {
        self.emergency_stop = true;
    }

    /// Resume after emergency stop
    pub fn resume(&mut self) {
        self.emergency_stop = false;
    }

    /// Cancel calibration
    pub fn cancel(&mut self) {
        self.emergency_stop = false;
        self.current_step = CalibrationStep::Start;
        self.calibration = ServoCalibration::default();
    }

    /// Is wizard paused?
    pub fn is_paused(&self) -> bool {
        self.emergency_stop
    }

    /// Verify calibration with test cycle
    /// Contract: SORT-001 - Verification must confirm ±2° accuracy
    pub fn verify(&mut self, test_results: &[VerificationTest]) -> Result<(), String> {
        for test in test_results {
            if !self.calibration.verify_angle(test.expected, test.actual) {
                return Err(format!(
                    "Verification failed at {:?}: expected {}°, got {}° (±2° tolerance)",
                    test.position, test.expected, test.actual
                ));
            }
        }

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_micros() as u64;

        self.calibration.mark_verified(timestamp);
        Ok(())
    }

    /// Get the final calibration (only after verification)
    pub fn finalize(self) -> Result<ServoCalibration, String> {
        if !self.calibration.is_verified() {
            return Err("Calibration not verified".to_string());
        }

        self.calibration.validate()?;

        Ok(self.calibration)
    }

    /// Get current calibration (for preview)
    pub fn calibration(&self) -> &ServoCalibration {
        &self.calibration
    }
}

/// Verification test result
pub struct VerificationTest {
    pub position: String,
    pub expected: u16,
    pub actual: u16,
}

// ============================================
// Tests
// ============================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lift_servo_validation() {
        let mut lift = LiftServo::default();
        assert!(lift.validate().is_ok());

        // Test narrow range rejection
        lift.min_angle = 100;
        lift.max_angle = 120; // Only 20° range
        assert!(lift.validate().is_err());

        // Restore valid range
        lift.min_angle = 80;
        lift.max_angle = 160;
        assert!(lift.validate().is_ok());

        // Test out-of-range positions
        lift.home_angle = 200; // Outside min-max
        assert!(lift.validate().is_err());
    }

    #[test]
    fn test_gripper_servo_validation() {
        let mut gripper = GripperServo::default();
        assert!(gripper.validate().is_ok());

        // Test narrow range rejection
        gripper.open_angle = 90;
        gripper.closed_angle = 100; // Only 10° range
        assert!(gripper.validate().is_err());
    }

    #[test]
    fn test_angle_verification_tolerance() {
        let calibration = ServoCalibration::default();

        // Within tolerance (±2°)
        assert!(calibration.verify_angle(90, 90));
        assert!(calibration.verify_angle(90, 91));
        assert!(calibration.verify_angle(90, 92));
        assert!(calibration.verify_angle(90, 88));

        // Outside tolerance
        assert!(!calibration.verify_angle(90, 93));
        assert!(!calibration.verify_angle(90, 87));
    }

    #[test]
    fn test_calibration_wizard_flow() {
        let mut wizard = ServoCalibrationWizard::new("test_robot");

        assert_eq!(wizard.current_step(), &CalibrationStep::Start);

        // Advance through steps
        assert!(wizard.next_step().is_ok());
        assert_eq!(wizard.current_step(), &CalibrationStep::LiftMin);

        // Record angle
        assert!(wizard.record_angle(85).is_ok());
        assert_eq!(wizard.calibration.lift.min_angle, 85);

        // Progress tracking
        let (current, total) = wizard.progress();
        assert_eq!(total, 11);
        assert!(current < total);
    }

    #[test]
    fn test_emergency_stop() {
        let mut wizard = ServoCalibrationWizard::new("test_robot");

        assert!(!wizard.is_paused());

        wizard.emergency_stop();
        assert!(wizard.is_paused());

        // Cannot record while paused
        assert!(wizard.record_angle(90).is_err());
        assert!(wizard.next_step().is_err());

        wizard.resume();
        assert!(!wizard.is_paused());

        // Can continue after resume
        assert!(wizard.next_step().is_ok());
    }

    #[test]
    fn test_calibration_persistence() {
        use std::fs;
        use std::path::PathBuf;

        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_calibration.json");

        let mut calibration = ServoCalibration::new("test_robot", 12345);
        calibration.lift.home_angle = 95;
        calibration.mark_verified(67890);

        // Save
        assert!(calibration.save_to_file(&test_file).is_ok());
        assert!(test_file.exists());

        // Load
        let loaded = ServoCalibration::load_from_file(&test_file).unwrap();
        assert_eq!(loaded.robot_id, "test_robot");
        assert_eq!(loaded.lift.home_angle, 95);
        assert_eq!(loaded.verification_timestamp, Some(67890));

        // Cleanup
        let _ = fs::remove_file(test_file);
    }

    #[test]
    fn test_calibration_step_sequence() {
        let mut step = CalibrationStep::Start;

        // Test full sequence
        assert!(step.next().is_some());
        step = step.next().unwrap();
        assert_eq!(step, CalibrationStep::LiftMin);

        // Skip to near end
        let mut step = CalibrationStep::Verification;
        step = step.next().unwrap();
        assert_eq!(step, CalibrationStep::Complete);
        assert!(step.next().is_none());
    }

    #[test]
    fn test_wizard_requires_verification() {
        let mut wizard = ServoCalibrationWizard::new("test_robot");

        // Cannot finalize without verification
        assert!(wizard.finalize().is_err());
    }

    #[test]
    fn test_wizard_finalize_after_verification() {
        let mut wizard = ServoCalibrationWizard::new("test_robot");

        // Mark as verified
        wizard.calibration.mark_verified(12345);

        // Now can finalize
        assert!(wizard.finalize().is_ok());
    }
}
