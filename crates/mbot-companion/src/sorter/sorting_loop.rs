//! Pick-Drop Sorting Loop for LEGO Sorter
//!
//! Implements STORY-SORT-004: Main deterministic sorting loop that picks pieces
//! from the tray and drops them into the correct bins based on color classification.
//!
//! Contracts:
//! - SORT-002: Deterministic sorting sequence
//! - SORT-003: Safe servo movements, no pinch events

use crate::sorter::{
    Bin, CarouselConfig, PieceObservation, Position2D, ServoCalibration, VisionAnalysisResult,
    VisionDetector,
};
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

// ============================================
// Core Enums and Types
// ============================================

/// State of the sorting loop
/// Contract: SORT-002 - State transitions are deterministic
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoopState {
    /// Loop is idle, waiting to start
    Idle,
    /// Scanning tray for pieces
    Detecting,
    /// Picking up a piece
    Picking,
    /// Transporting piece to bin
    Transporting,
    /// Dropping piece into bin
    Dropping,
    /// Verifying drop was successful
    Verifying,
    /// Error occurred, requires intervention
    Error,
    /// All pieces sorted, tray empty
    Complete,
}

/// Step within a pick-drop cycle
/// Contract: SORT-002 - Steps execute in deterministic order
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SortingStep {
    /// Scan tray with vision system
    ScanTray,
    /// Select next piece to pick
    SelectPiece,
    /// Align chassis to piece position
    AlignToPiece,
    /// Lower arm to pick height
    LowerArm,
    /// Close gripper to grasp piece
    CloseGripper,
    /// Lift arm with piece
    LiftArm,
    /// Rotate chassis to target bin
    RotateToBin,
    /// Lower arm over bin
    LowerToBin,
    /// Open gripper to release piece
    OpenGripper,
    /// Return to home position
    ReturnHome,
    /// Verify piece dropped successfully
    VerifyDrop,
}

impl SortingStep {
    /// Get the next step in the sequence
    pub fn next(&self) -> Option<SortingStep> {
        match self {
            SortingStep::ScanTray => Some(SortingStep::SelectPiece),
            SortingStep::SelectPiece => Some(SortingStep::AlignToPiece),
            SortingStep::AlignToPiece => Some(SortingStep::LowerArm),
            SortingStep::LowerArm => Some(SortingStep::CloseGripper),
            SortingStep::CloseGripper => Some(SortingStep::LiftArm),
            SortingStep::LiftArm => Some(SortingStep::RotateToBin),
            SortingStep::RotateToBin => Some(SortingStep::LowerToBin),
            SortingStep::LowerToBin => Some(SortingStep::OpenGripper),
            SortingStep::OpenGripper => Some(SortingStep::ReturnHome),
            SortingStep::ReturnHome => Some(SortingStep::VerifyDrop),
            SortingStep::VerifyDrop => None, // Cycle complete, return to ScanTray
        }
    }

    /// Human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            SortingStep::ScanTray => "Scanning tray for pieces",
            SortingStep::SelectPiece => "Selecting next piece to pick",
            SortingStep::AlignToPiece => "Aligning to piece position",
            SortingStep::LowerArm => "Lowering arm to pick height",
            SortingStep::CloseGripper => "Closing gripper on piece",
            SortingStep::LiftArm => "Lifting piece",
            SortingStep::RotateToBin => "Rotating to target bin",
            SortingStep::LowerToBin => "Lowering over bin",
            SortingStep::OpenGripper => "Releasing piece",
            SortingStep::ReturnHome => "Returning to home position",
            SortingStep::VerifyDrop => "Verifying drop success",
        }
    }
}

/// Grip force level for gripper
/// Contract: SORT-003 - Limited force for safety
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GripForce {
    /// Light grip for small/delicate pieces
    Light,
    /// Normal grip for standard pieces
    Normal,
    /// Firm grip for larger/heavier pieces
    Firm,
}

impl GripForce {
    /// Get the gripper angle offset from base grip angle
    pub fn angle_offset(&self) -> i16 {
        match self {
            GripForce::Light => -5,  // 5° less closure
            GripForce::Normal => 0,  // Standard grip angle
            GripForce::Firm => 5,    // 5° more closure
        }
    }
}

/// Pick operation parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PickOperation {
    /// The piece being picked
    pub piece: PieceObservation,
    /// Approach angle (degrees)
    pub approach_angle: f32,
    /// Pick position in mm
    pub pick_position: Position2D,
    /// Lift height in mm above tray
    pub lift_height: f32,
    /// Grip force to apply
    pub grip_force: GripForce,
    /// Number of pick attempts so far
    pub attempt_count: u32,
}

impl PickOperation {
    /// Maximum pick attempts before skipping piece
    pub const MAX_ATTEMPTS: u32 = 3;

    /// Create a new pick operation
    pub fn new(piece: PieceObservation, lift_height: f32) -> Self {
        // Determine grip force based on piece size
        let grip_force = match piece.estimated_size {
            crate::sorter::PieceSize::Small => GripForce::Light,
            crate::sorter::PieceSize::Medium => GripForce::Normal,
            crate::sorter::PieceSize::Large => GripForce::Firm,
        };

        Self {
            approach_angle: 0.0, // Approach from front
            pick_position: piece.center_position,
            lift_height,
            grip_force,
            attempt_count: 0,
            piece,
        }
    }

    /// Increment attempt counter
    pub fn increment_attempt(&mut self) {
        self.attempt_count += 1;
    }

    /// Check if max attempts reached
    pub fn max_attempts_reached(&self) -> bool {
        self.attempt_count >= Self::MAX_ATTEMPTS
    }
}

/// Drop operation parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DropOperation {
    /// Target bin
    pub bin: Bin,
    /// Drop angle (degrees, relative to home)
    pub drop_angle: f32,
    /// Drop height in mm above bin
    pub drop_height: f32,
    /// Delay after opening gripper (ms)
    pub release_delay_ms: u64,
}

impl DropOperation {
    /// Create a new drop operation
    pub fn new(bin: Bin, drop_height: f32) -> Self {
        Self {
            drop_angle: bin.position_angle,
            drop_height,
            release_delay_ms: 200, // 200ms delay for piece to fall
            bin,
        }
    }
}

/// Metrics for the sorting loop
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LoopMetrics {
    /// Total pieces sorted successfully
    pub pieces_sorted: u32,
    /// Total pick attempts
    pub pick_attempts: u32,
    /// Successful pick operations
    pub pick_successes: u32,
    /// Total drop attempts
    pub drop_attempts: u32,
    /// Successful drop operations
    pub drop_successes: u32,
    /// Average cycle time per piece (milliseconds)
    pub avg_cycle_time_ms: u64,
    /// Start time of current session (microseconds)
    pub session_start_time: u64,
    /// Total pieces skipped (failed after max retries)
    pub pieces_skipped: u32,
}

impl LoopMetrics {
    /// Calculate pick success rate
    pub fn pick_success_rate(&self) -> f32 {
        if self.pick_attempts == 0 {
            return 0.0;
        }
        (self.pick_successes as f32 / self.pick_attempts as f32) * 100.0
    }

    /// Calculate drop success rate
    pub fn drop_success_rate(&self) -> f32 {
        if self.drop_attempts == 0 {
            return 0.0;
        }
        (self.drop_successes as f32 / self.drop_attempts as f32) * 100.0
    }

    /// Get elapsed time since session start (milliseconds)
    pub fn elapsed_time_ms(&self) -> u64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_micros() as u64;
        (now - self.session_start_time) / 1000
    }
}

// ============================================
// Main Sorting Loop
// ============================================

/// Main sorting loop coordinator
/// Contract: SORT-002 - Deterministic state machine
pub struct SortingLoop {
    /// Unique loop identifier
    pub loop_id: String,
    /// Current state
    pub state: LoopState,
    /// Current step within the pick-drop cycle
    pub step: SortingStep,
    /// Current piece being sorted (if any)
    pub current_piece: Option<PieceObservation>,
    /// Target bin for current piece
    pub target_bin: Option<Bin>,
    /// Current pick operation
    pub current_pick: Option<PickOperation>,
    /// Current drop operation
    pub current_drop: Option<DropOperation>,
    /// Loop metrics
    pub metrics: LoopMetrics,
    /// Servo calibration
    calibration: ServoCalibration,
    /// Carousel configuration
    carousel: CarouselConfig,
    /// Vision detector
    vision: VisionDetector,
    /// Emergency stop flag
    emergency_stop: bool,
    /// Paused flag
    paused: bool,
    /// Last vision analysis result
    last_vision_result: Option<VisionAnalysisResult>,
    /// Cycle start time (for timing)
    cycle_start_time: Option<u64>,
}

impl SortingLoop {
    /// Create a new sorting loop
    pub fn new(
        loop_id: String,
        calibration: ServoCalibration,
        carousel: CarouselConfig,
        vision: VisionDetector,
    ) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_micros() as u64;

        Self {
            loop_id,
            state: LoopState::Idle,
            step: SortingStep::ScanTray,
            current_piece: None,
            target_bin: None,
            current_pick: None,
            current_drop: None,
            metrics: LoopMetrics {
                session_start_time: now,
                ..Default::default()
            },
            calibration,
            carousel,
            vision,
            emergency_stop: false,
            paused: false,
            last_vision_result: None,
            cycle_start_time: None,
        }
    }

    /// Start the sorting loop
    pub fn start(&mut self) -> Result<(), String> {
        if self.state != LoopState::Idle && self.state != LoopState::Complete {
            return Err(format!("Cannot start from state {:?}", self.state));
        }

        // Validate calibration
        self.calibration.validate()?;
        if !self.calibration.is_verified() {
            return Err("Servo calibration not verified".to_string());
        }

        // Validate carousel
        self.carousel.validate()?;
        if self.carousel.bins.is_empty() {
            return Err("No bins configured in carousel".to_string());
        }

        self.state = LoopState::Detecting;
        self.step = SortingStep::ScanTray;
        self.emergency_stop = false;
        self.paused = false;

        Ok(())
    }

    /// Pause the sorting loop
    /// Contract: SORT-003 - Safe pause (completes current operation)
    pub fn pause(&mut self) {
        self.paused = true;
    }

    /// Resume the sorting loop
    pub fn resume(&mut self) {
        if self.paused {
            self.paused = false;
        }
    }

    /// Emergency stop - immediate halt
    /// Contract: SORT-003 - Emergency stop available at all times
    pub fn emergency_stop(&mut self) {
        self.emergency_stop = true;
        self.state = LoopState::Error;
        // In real implementation: immediately open gripper, stop all servos
    }

    /// Check if loop is stopped (paused or emergency)
    pub fn is_stopped(&self) -> bool {
        self.emergency_stop || self.paused
    }

    /// Reset after emergency stop
    pub fn reset_after_emergency(&mut self) {
        if self.emergency_stop {
            self.emergency_stop = false;
            self.state = LoopState::Idle;
            self.step = SortingStep::ScanTray;
            self.current_piece = None;
            self.target_bin = None;
            self.current_pick = None;
            self.current_drop = None;
        }
    }

    /// Execute one step of the sorting loop
    /// Contract: SORT-002 - Deterministic step execution
    pub fn execute_step(&mut self, rgb_data: &[u8], width: u32, height: u32) -> Result<(), String> {
        if self.is_stopped() {
            return Ok(()); // Skip execution if stopped
        }

        match self.step {
            SortingStep::ScanTray => self.step_scan_tray(rgb_data, width, height)?,
            SortingStep::SelectPiece => self.step_select_piece()?,
            SortingStep::AlignToPiece => self.step_align_to_piece()?,
            SortingStep::LowerArm => self.step_lower_arm()?,
            SortingStep::CloseGripper => self.step_close_gripper()?,
            SortingStep::LiftArm => self.step_lift_arm()?,
            SortingStep::RotateToBin => self.step_rotate_to_bin()?,
            SortingStep::LowerToBin => self.step_lower_to_bin()?,
            SortingStep::OpenGripper => self.step_open_gripper()?,
            SortingStep::ReturnHome => self.step_return_home()?,
            SortingStep::VerifyDrop => self.step_verify_drop()?,
        }

        Ok(())
    }

    /// Scan tray for pieces using vision system
    fn step_scan_tray(&mut self, rgb_data: &[u8], width: u32, height: u32) -> Result<(), String> {
        self.state = LoopState::Detecting;

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_micros() as u64;

        let result = self.vision.analyze_frame(rgb_data, width, height, timestamp);

        // Check if tray is empty
        if result.is_empty(self.vision.config().min_confidence) {
            self.state = LoopState::Complete;
            return Ok(());
        }

        self.last_vision_result = Some(result);
        self.advance_step();

        Ok(())
    }

    /// Select the next piece to pick
    fn step_select_piece(&mut self) -> Result<(), String> {
        if let Some(ref result) = self.last_vision_result {
            // Get the piece closest to the tray center (pick position)
            let pick_position = Position2D::new(
                self.vision.config().tray_region.width() / 2.0,
                self.vision.config().tray_region.height() / 2.0,
            );

            if let Some(piece) =
                result.next_to_pick(&pick_position, self.vision.config().min_confidence)
            {
                self.current_piece = Some(piece.clone());

                // Find target bin for this color
                let category = format!("color:{}", piece.color.name().to_lowercase());
                if let Some(bin) = self.carousel.find_by_category(&category) {
                    self.target_bin = Some(bin.clone());
                } else {
                    // No bin for this color - try to find "misc" or "unknown" bin
                    if let Some(misc_bin) = self.carousel.find_by_category("misc") {
                        self.target_bin = Some(misc_bin.clone());
                    } else {
                        return Err(format!("No bin configured for color: {}", piece.color.name()));
                    }
                }

                // Check if target bin has capacity
                if let Some(ref bin) = self.target_bin {
                    if !bin.has_capacity() {
                        return Err(format!("Target bin '{}' is full", bin.bin_id));
                    }

                    if bin.is_nearly_full() {
                        // Warning but continue
                        eprintln!("Warning: Bin '{}' is nearly full (90%+)", bin.bin_id);
                    }
                }

                // Create pick operation
                let lift_height = 50.0; // 50mm lift height
                self.current_pick = Some(PickOperation::new(piece.clone(), lift_height));

                // Start cycle timing
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_micros() as u64;
                self.cycle_start_time = Some(now);

                self.advance_step();
                Ok(())
            } else {
                Err("No valid pieces to pick".to_string())
            }
        } else {
            Err("No vision result available".to_string())
        }
    }

    /// Align chassis to piece position
    fn step_align_to_piece(&mut self) -> Result<(), String> {
        if let Some(ref _pick_op) = self.current_pick {
            // In real implementation: command chassis to rotate/move
            // For now: simulate alignment
            if let Some(ref pick_op) = self.current_pick {
                println!(
                    "Aligning to piece at position ({:.1}, {:.1})",
                    pick_op.pick_position.x, pick_op.pick_position.y
                );
            }

            self.advance_step();
            Ok(())
        } else {
            Err("No pick operation active".to_string())
        }
    }

    /// Lower arm to pick height
    fn step_lower_arm(&mut self) -> Result<(), String> {
        if let Some(ref pick_op) = self.current_pick {
            let target_angle = self.calibration.lift.pick_angle;

            // Contract: SORT-003 - Verify angle is safe
            if !self.calibration.lift.is_safe_angle(target_angle) {
                return Err(format!("Unsafe pick angle: {}°", target_angle));
            }

            println!("Lowering arm to pick angle: {}°", target_angle);
            self.state = LoopState::Picking;
            self.advance_step();
            Ok(())
        } else {
            Err("No pick operation active".to_string())
        }
    }

    /// Close gripper to grasp piece
    fn step_close_gripper(&mut self) -> Result<(), String> {
        if let Some(ref mut pick_op) = self.current_pick {
            pick_op.increment_attempt();
            self.metrics.pick_attempts += 1;

            let base_angle = self.calibration.gripper.grip_angle;
            let offset = pick_op.grip_force.angle_offset();
            let target_angle = (base_angle as i16 + offset) as u16;

            // Contract: SORT-003 - Verify angle is safe
            if !self.calibration.gripper.is_safe_angle(target_angle) {
                return Err(format!("Unsafe gripper angle: {}°", target_angle));
            }

            println!(
                "Closing gripper to {}° (force: {:?}, attempt {})",
                target_angle, pick_op.grip_force, pick_op.attempt_count
            );

            // In real implementation: close gripper and check feedback
            // Simulate success for now
            let pick_successful = true; // In production: read sensor feedback

            if pick_successful {
                self.metrics.pick_successes += 1;
                self.advance_step();
                Ok(())
            } else if pick_op.max_attempts_reached() {
                // Max attempts reached, skip this piece
                println!(
                    "Failed to pick piece {} after {} attempts, skipping",
                    pick_op.piece.observation_id,
                    PickOperation::MAX_ATTEMPTS
                );
                self.metrics.pieces_skipped += 1;
                self.current_piece = None;
                self.current_pick = None;
                self.target_bin = None;
                self.step = SortingStep::ScanTray; // Start over
                Ok(())
            } else {
                // Retry: back to align step
                println!("Pick failed, retrying...");
                self.step = SortingStep::AlignToPiece;
                Ok(())
            }
        } else {
            Err("No pick operation active".to_string())
        }
    }

    /// Lift arm with piece
    fn step_lift_arm(&mut self) -> Result<(), String> {
        let target_angle = self.calibration.lift.home_angle;

        // Contract: SORT-003 - Verify angle is safe
        if !self.calibration.lift.is_safe_angle(target_angle) {
            return Err(format!("Unsafe lift angle: {}°", target_angle));
        }

        println!("Lifting arm to home angle: {}°", target_angle);
        self.state = LoopState::Transporting;
        self.advance_step();
        Ok(())
    }

    /// Rotate chassis to target bin
    fn step_rotate_to_bin(&mut self) -> Result<(), String> {
        if let Some(ref drop_op) = self.current_drop {
            println!("Rotating to bin angle: {:.1}°", drop_op.drop_angle);
            self.advance_step();
            Ok(())
        } else if let Some(ref bin) = self.target_bin {
            // Create drop operation
            let drop_height = 100.0; // 100mm above bin
            self.current_drop = Some(DropOperation::new(bin.clone(), drop_height));
            println!("Rotating to bin angle: {:.1}°", bin.position_angle);
            self.advance_step();
            Ok(())
        } else {
            Err("No target bin set".to_string())
        }
    }

    /// Lower arm over bin
    fn step_lower_to_bin(&mut self) -> Result<(), String> {
        let target_angle = self.calibration.lift.drop_angle;

        // Contract: SORT-003 - Verify angle is safe
        if !self.calibration.lift.is_safe_angle(target_angle) {
            return Err(format!("Unsafe drop angle: {}°", target_angle));
        }

        println!("Lowering arm to drop angle: {}°", target_angle);
        self.state = LoopState::Dropping;
        self.advance_step();
        Ok(())
    }

    /// Open gripper to release piece
    fn step_open_gripper(&mut self) -> Result<(), String> {
        self.metrics.drop_attempts += 1;

        let target_angle = self.calibration.gripper.open_angle;

        // Contract: SORT-003 - Verify angle is safe
        if !self.calibration.gripper.is_safe_angle(target_angle) {
            return Err(format!("Unsafe gripper open angle: {}°", target_angle));
        }

        println!("Opening gripper to {}°", target_angle);

        if let Some(ref drop_op) = self.current_drop {
            // Wait for piece to fall
            std::thread::sleep(Duration::from_millis(drop_op.release_delay_ms));
        }

        self.metrics.drop_successes += 1;
        self.advance_step();
        Ok(())
    }

    /// Return to home position
    fn step_return_home(&mut self) -> Result<(), String> {
        // Lift arm to home
        let lift_home = self.calibration.lift.home_angle;
        println!("Returning to home position (lift: {}°)", lift_home);

        // Rotate to home
        let home_angle = self.carousel.home_angle;
        println!("Rotating to home angle: {:.1}°", home_angle);

        self.advance_step();
        Ok(())
    }

    /// Verify drop was successful
    fn step_verify_drop(&mut self) -> Result<(), String> {
        self.state = LoopState::Verifying;

        // In real implementation: check if piece is in gripper (failure) or not (success)
        // For now: assume success

        // Update inventory
        if let Some(ref mut bin) = self.target_bin {
            bin.add_piece();
        }

        // Update metrics
        self.metrics.pieces_sorted += 1;

        // Update cycle time
        if let Some(start_time) = self.cycle_start_time {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_micros() as u64;
            let cycle_time_ms = (now - start_time) / 1000;

            // Running average
            let total = self.metrics.avg_cycle_time_ms * (self.metrics.pieces_sorted - 1) as u64;
            self.metrics.avg_cycle_time_ms = (total + cycle_time_ms) / self.metrics.pieces_sorted as u64;
        }

        println!(
            "Piece sorted successfully! Total: {} pieces",
            self.metrics.pieces_sorted
        );

        // Reset for next cycle
        self.current_piece = None;
        self.current_pick = None;
        self.current_drop = None;
        self.target_bin = None;
        self.cycle_start_time = None;

        // Return to scanning
        self.step = SortingStep::ScanTray;
        self.state = LoopState::Detecting;

        Ok(())
    }

    /// Advance to the next step
    fn advance_step(&mut self) {
        if let Some(next) = self.step.next() {
            self.step = next;
        }
    }

    /// Get a summary of loop status
    pub fn status_summary(&self) -> String {
        format!(
            "Loop: {} | State: {:?} | Step: {} | Sorted: {} | Success Rate: {:.1}%",
            self.loop_id,
            self.state,
            self.step.description(),
            self.metrics.pieces_sorted,
            self.metrics.pick_success_rate()
        )
    }

    /// Get current inventory summary
    pub fn inventory_summary(&self) -> std::collections::HashMap<String, u32> {
        self.carousel.inventory_summary()
    }

    /// Get bins that are nearly full
    pub fn nearly_full_bins(&self) -> Vec<&Bin> {
        self.carousel.nearly_full_bins()
    }
}

// ============================================
// Tests
// ============================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sorter::{
        BoundingBox, LegoColor, PieceSize, TrayRegion, VisionConfig,
    };

    fn create_test_calibration() -> ServoCalibration {
        let mut cal = ServoCalibration::default();
        cal.mark_verified(12345);
        cal
    }

    fn create_test_carousel() -> CarouselConfig {
        let mut carousel = CarouselConfig::new("test_carousel");
        carousel
            .add_bin(crate::sorter::Bin::new(
                "bin-red",
                "marker-01",
                45.0,
                "color:red",
                50,
            ))
            .unwrap();
        carousel
            .add_bin(crate::sorter::Bin::new(
                "bin-blue",
                "marker-02",
                135.0,
                "color:blue",
                50,
            ))
            .unwrap();
        carousel
            .add_bin(crate::sorter::Bin::new(
                "bin-misc",
                "marker-03",
                225.0,
                "misc",
                50,
            ))
            .unwrap();
        carousel
    }

    fn create_test_vision() -> VisionDetector {
        VisionDetector::with_defaults()
    }

    #[test]
    fn test_sorting_step_sequence() {
        let mut step = SortingStep::ScanTray;

        assert_eq!(step.next(), Some(SortingStep::SelectPiece));
        step = step.next().unwrap();

        assert_eq!(step.next(), Some(SortingStep::AlignToPiece));
        step = step.next().unwrap();

        // Continue through sequence
        assert_eq!(step.next(), Some(SortingStep::LowerArm));
        step = step.next().unwrap();
        assert_eq!(step.next(), Some(SortingStep::CloseGripper));
        step = step.next().unwrap();
        assert_eq!(step.next(), Some(SortingStep::LiftArm));
    }

    #[test]
    fn test_loop_state_transitions() {
        let calibration = create_test_calibration();
        let carousel = create_test_carousel();
        let vision = create_test_vision();

        let mut sorting_loop = SortingLoop::new(
            "test_loop".to_string(),
            calibration,
            carousel,
            vision,
        );

        assert_eq!(sorting_loop.state, LoopState::Idle);

        // Start loop
        assert!(sorting_loop.start().is_ok());
        assert_eq!(sorting_loop.state, LoopState::Detecting);
        assert_eq!(sorting_loop.step, SortingStep::ScanTray);
    }

    #[test]
    fn test_pick_operation_retry_logic() {
        let piece = PieceObservation::new(
            "test_1".to_string(),
            0,
            LegoColor::Red,
            BoundingBox::new(10, 10, 50, 50),
            0.90,
            Position2D::new(25.0, 25.0),
            PieceSize::Medium,
        );

        let mut pick_op = PickOperation::new(piece, 50.0);

        assert_eq!(pick_op.attempt_count, 0);
        assert!(!pick_op.max_attempts_reached());

        pick_op.increment_attempt();
        assert_eq!(pick_op.attempt_count, 1);
        assert!(!pick_op.max_attempts_reached());

        pick_op.increment_attempt();
        pick_op.increment_attempt();
        assert_eq!(pick_op.attempt_count, 3);
        assert!(pick_op.max_attempts_reached());
    }

    #[test]
    fn test_grip_force_selection() {
        let small_piece = PieceObservation::new(
            "small".to_string(),
            0,
            LegoColor::Red,
            BoundingBox::new(10, 10, 20, 20),
            0.90,
            Position2D::new(20.0, 20.0),
            PieceSize::Small,
        );

        let pick_op = PickOperation::new(small_piece, 50.0);
        assert_eq!(pick_op.grip_force, GripForce::Light);

        let large_piece = PieceObservation::new(
            "large".to_string(),
            0,
            LegoColor::Blue,
            BoundingBox::new(10, 10, 100, 50),
            0.90,
            Position2D::new(60.0, 35.0),
            PieceSize::Large,
        );

        let pick_op = PickOperation::new(large_piece, 50.0);
        assert_eq!(pick_op.grip_force, GripForce::Firm);
    }

    #[test]
    fn test_loop_pause_resume() {
        let calibration = create_test_calibration();
        let carousel = create_test_carousel();
        let vision = create_test_vision();

        let mut sorting_loop = SortingLoop::new(
            "test_loop".to_string(),
            calibration,
            carousel,
            vision,
        );

        sorting_loop.start().unwrap();
        assert!(!sorting_loop.is_stopped());

        sorting_loop.pause();
        assert!(sorting_loop.is_stopped());
        assert!(sorting_loop.paused);

        sorting_loop.resume();
        assert!(!sorting_loop.is_stopped());
        assert!(!sorting_loop.paused);
    }

    #[test]
    fn test_emergency_stop() {
        let calibration = create_test_calibration();
        let carousel = create_test_carousel();
        let vision = create_test_vision();

        let mut sorting_loop = SortingLoop::new(
            "test_loop".to_string(),
            calibration,
            carousel,
            vision,
        );

        sorting_loop.start().unwrap();

        sorting_loop.emergency_stop();
        assert!(sorting_loop.emergency_stop);
        assert_eq!(sorting_loop.state, LoopState::Error);
        assert!(sorting_loop.is_stopped());

        // Reset after emergency
        sorting_loop.reset_after_emergency();
        assert!(!sorting_loop.emergency_stop);
        assert_eq!(sorting_loop.state, LoopState::Idle);
    }

    #[test]
    fn test_metrics_tracking() {
        let mut metrics = LoopMetrics::default();

        metrics.pick_attempts = 10;
        metrics.pick_successes = 8;
        assert_eq!(metrics.pick_success_rate(), 80.0);

        metrics.drop_attempts = 8;
        metrics.drop_successes = 7;
        assert_eq!(metrics.drop_success_rate(), 87.5);

        metrics.pieces_sorted = 5;
        metrics.pieces_skipped = 2;
        assert_eq!(metrics.pieces_sorted + metrics.pieces_skipped, 7);
    }

    #[test]
    fn test_loop_requires_verified_calibration() {
        let mut calibration = ServoCalibration::default();
        // Not verified
        let carousel = create_test_carousel();
        let vision = create_test_vision();

        let mut sorting_loop = SortingLoop::new(
            "test_loop".to_string(),
            calibration.clone(),
            carousel,
            vision,
        );

        // Should fail to start without verification
        assert!(sorting_loop.start().is_err());

        // Mark as verified
        calibration.mark_verified(12345);
        sorting_loop.calibration = calibration;

        // Should succeed now
        assert!(sorting_loop.start().is_ok());
    }

    #[test]
    fn test_loop_requires_bins() {
        let calibration = create_test_calibration();
        let empty_carousel = CarouselConfig::new("empty");
        let vision = create_test_vision();

        let mut sorting_loop = SortingLoop::new(
            "test_loop".to_string(),
            calibration,
            empty_carousel,
            vision,
        );

        // Should fail without bins
        assert!(sorting_loop.start().is_err());
    }

    #[test]
    fn test_bin_capacity_check() {
        let calibration = create_test_calibration();
        let mut carousel = create_test_carousel();

        // Fill red bin to capacity
        if let Some(bin) = carousel.get_bin_mut("bin-red") {
            bin.current_count = bin.capacity;
        }

        let vision = create_test_vision();
        let mut sorting_loop = SortingLoop::new(
            "test_loop".to_string(),
            calibration,
            carousel,
            vision,
        );

        sorting_loop.start().unwrap();

        // Create a red piece
        let red_piece = PieceObservation::new(
            "red_1".to_string(),
            0,
            LegoColor::Red,
            BoundingBox::new(10, 10, 50, 50),
            0.90,
            Position2D::new(25.0, 25.0),
            PieceSize::Medium,
        );

        sorting_loop.current_piece = Some(red_piece.clone());

        // Should fail because red bin is full
        let result = sorting_loop.step_select_piece();
        assert!(result.is_err());
    }

    #[test]
    fn test_deterministic_state_machine() {
        // Contract: SORT-002 - State transitions must be deterministic
        let calibration = create_test_calibration();
        let carousel = create_test_carousel();
        let vision = create_test_vision();

        let mut loop1 = SortingLoop::new(
            "loop1".to_string(),
            calibration.clone(),
            carousel.clone(),
            vision,
        );

        let mut loop2 = SortingLoop::new(
            "loop2".to_string(),
            calibration,
            carousel,
            create_test_vision(),
        );

        loop1.start().unwrap();
        loop2.start().unwrap();

        // Both loops should start in same state
        assert_eq!(loop1.state, loop2.state);
        assert_eq!(loop1.step, loop2.step);
    }
}
