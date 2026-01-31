//! Verification System for LEGO Sorter Operations
//!
//! Implements STORY-SORT-005: Verification & Error Handling
//! Verifies pick and drop operations to ensure pieces are actually grasped and released.
//!
//! Contracts:
//! - SORT-002: Deterministic verification logic
//! - SORT-003: Safety - no pinch events during recovery

use crate::sorter::{PieceObservation, VisionAnalysisResult, VisionDetector};
use serde::{Deserialize, Serialize};

// ============================================
// Verification Types
// ============================================

/// Result of a verification operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    /// The operation being verified
    pub operation: Operation,
    /// Whether the operation succeeded
    pub success: bool,
    /// Confidence level (0.0-1.0)
    pub confidence: f32,
    /// Reason for failure (if any)
    pub failure_reason: Option<FailureReason>,
    /// Whether a retry is recommended
    pub retry_recommended: bool,
    /// Timestamp of verification
    pub timestamp: u64,
}

impl VerificationResult {
    /// Create a successful verification result
    pub fn success(operation: Operation, confidence: f32, timestamp: u64) -> Self {
        Self {
            operation,
            success: true,
            confidence: confidence.clamp(0.0, 1.0),
            failure_reason: None,
            retry_recommended: false,
            timestamp,
        }
    }

    /// Create a failed verification result
    pub fn failure(
        operation: Operation,
        reason: FailureReason,
        confidence: f32,
        retry: bool,
        timestamp: u64,
    ) -> Self {
        Self {
            operation,
            success: false,
            confidence: confidence.clamp(0.0, 1.0),
            failure_reason: Some(reason),
            retry_recommended: retry,
            timestamp,
        }
    }
}

/// Type of operation being verified
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Operation {
    /// Pick operation (grasping piece)
    Pick,
    /// Drop operation (releasing piece)
    Drop,
}

/// Reasons for operation failure
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FailureReason {
    /// Piece was not grabbed during pick
    PieceNotGrabbed,
    /// Piece was dropped during transport
    PieceDroppedDuringTransport,
    /// Piece is stuck in gripper after drop
    PieceStuckInGripper,
    /// Piece missed the target bin
    BinMissed,
    /// Jam detected (multiple consecutive failures)
    JamDetected,
    /// Servo timeout (no response)
    ServoTimeout,
    /// Vision system lost tracking
    VisionLost,
}

impl FailureReason {
    /// Get human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            FailureReason::PieceNotGrabbed => "Piece was not grasped by gripper",
            FailureReason::PieceDroppedDuringTransport => "Piece fell during transport",
            FailureReason::PieceStuckInGripper => "Piece stuck in gripper after release",
            FailureReason::BinMissed => "Piece missed target bin",
            FailureReason::JamDetected => "Jam detected - manual intervention needed",
            FailureReason::ServoTimeout => "Servo did not respond in time",
            FailureReason::VisionLost => "Vision system lost piece tracking",
        }
    }

    /// Check if this failure should trigger a retry
    pub fn should_retry(&self) -> bool {
        matches!(
            self,
            FailureReason::PieceNotGrabbed | FailureReason::PieceStuckInGripper
        )
    }
}

// ============================================
// Retry Policy
// ============================================

/// Policy for retry behavior
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct RetryPolicy {
    /// Maximum pick retry attempts
    pub max_pick_retries: u32,
    /// Maximum drop retry attempts
    pub max_drop_retries: u32,
    /// Delay between retries in milliseconds
    pub retry_delay_ms: u64,
    /// Position adjustment on retry in millimeters
    pub micro_adjust_mm: f32,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_pick_retries: 2,
            max_drop_retries: 1,
            retry_delay_ms: 500,
            micro_adjust_mm: 2.0,
        }
    }
}

impl RetryPolicy {
    /// Check if retry is allowed for this operation and attempt count
    pub fn can_retry(&self, operation: Operation, attempt_count: u32) -> bool {
        match operation {
            Operation::Pick => attempt_count < self.max_pick_retries,
            Operation::Drop => attempt_count < self.max_drop_retries,
        }
    }

    /// Get the maximum attempts for an operation
    pub fn max_attempts(&self, operation: Operation) -> u32 {
        match operation {
            Operation::Pick => self.max_pick_retries,
            Operation::Drop => self.max_drop_retries,
        }
    }
}

// ============================================
// Jam Detection
// ============================================

/// Jam detection state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JamDetection {
    /// Whether piece is detected in gripper zone
    pub piece_in_gripper_zone: bool,
    /// Expected motion delta (mm)
    pub expected_motion_delta: f32,
    /// Actual motion delta (mm)
    pub actual_motion_delta: f32,
    /// Consecutive failure count
    pub consecutive_failures: u32,
    /// Threshold for jam detection
    pub jam_threshold: u32,
}

impl Default for JamDetection {
    fn default() -> Self {
        Self {
            piece_in_gripper_zone: false,
            expected_motion_delta: 0.0,
            actual_motion_delta: 0.0,
            consecutive_failures: 0,
            jam_threshold: 3,
        }
    }
}

impl JamDetection {
    /// Create a new jam detection instance
    pub fn new(jam_threshold: u32) -> Self {
        Self {
            jam_threshold,
            ..Default::default()
        }
    }

    /// Record a failure
    pub fn record_failure(&mut self) {
        self.consecutive_failures += 1;
    }

    /// Reset failure counter (after success)
    pub fn reset(&mut self) {
        self.consecutive_failures = 0;
        self.piece_in_gripper_zone = false;
        self.expected_motion_delta = 0.0;
        self.actual_motion_delta = 0.0;
    }

    /// Check if jam condition is met
    pub fn is_jammed(&self) -> bool {
        self.consecutive_failures >= self.jam_threshold
    }

    /// Get jam severity (0.0-1.0)
    pub fn severity(&self) -> f32 {
        (self.consecutive_failures as f32 / self.jam_threshold as f32).min(1.0)
    }
}

// ============================================
// Verification System
// ============================================

/// Main verification system for pick/drop operations
pub struct VerificationSystem {
    /// Vision detector for piece presence checks
    vision: VisionDetector,
    /// Retry policy
    retry_policy: RetryPolicy,
    /// Jam detection state
    jam_detection: JamDetection,
    /// Minimum confidence for valid verification
    min_confidence: f32,
}

impl VerificationSystem {
    /// Create a new verification system
    pub fn new(vision: VisionDetector, retry_policy: RetryPolicy) -> Self {
        Self {
            vision,
            retry_policy,
            jam_detection: JamDetection::default(),
            min_confidence: 0.75,
        }
    }

    /// Get the retry policy
    pub fn retry_policy(&self) -> &RetryPolicy {
        &self.retry_policy
    }

    /// Get jam detection state
    pub fn jam_detection(&self) -> &JamDetection {
        &self.jam_detection
    }

    /// Reset jam detection
    pub fn reset_jam_detection(&mut self) {
        self.jam_detection.reset();
    }

    /// Verify a pick operation
    ///
    /// Checks that:
    /// 1. Gripper sensor confirms grip (if available)
    /// 2. Vision confirms piece is no longer in tray
    ///
    /// # Arguments
    /// * `piece` - The piece that was picked
    /// * `rgb_data` - Current camera frame for verification
    /// * `width` - Frame width
    /// * `height` - Frame height
    /// * `timestamp` - Current timestamp
    /// * `gripper_has_pressure` - Whether gripper sensor detects pressure
    pub fn verify_pick(
        &mut self,
        piece: &PieceObservation,
        rgb_data: &[u8],
        width: u32,
        height: u32,
        timestamp: u64,
        gripper_has_pressure: bool,
    ) -> VerificationResult {
        // Method 1: Check gripper pressure sensor (if available)
        if gripper_has_pressure {
            self.jam_detection.reset();
            return VerificationResult::success(Operation::Pick, 0.95, timestamp);
        }

        // Method 2: Use vision to check if piece is still in tray
        let analysis = self.vision.analyze_frame(rgb_data, width, height, timestamp);

        // Look for the same piece in the tray
        let piece_still_present = analysis
            .observations
            .iter()
            .any(|obs| self.is_same_piece(obs, piece));

        if piece_still_present {
            // Piece is still in tray - pick failed
            self.jam_detection.record_failure();

            let reason = FailureReason::PieceNotGrabbed;
            let retry = self.retry_policy.can_retry(Operation::Pick, self.jam_detection.consecutive_failures);

            VerificationResult::failure(Operation::Pick, reason, 0.80, retry, timestamp)
        } else {
            // Piece is gone from tray - pick succeeded
            self.jam_detection.reset();
            VerificationResult::success(Operation::Pick, 0.85, timestamp)
        }
    }

    /// Verify a drop operation
    ///
    /// Checks that:
    /// 1. Gripper is open
    /// 2. Piece is not in gripper zone (vision)
    ///
    /// # Arguments
    /// * `rgb_data` - Current camera frame for verification
    /// * `width` - Frame width
    /// * `height` - Frame height
    /// * `timestamp` - Current timestamp
    /// * `gripper_is_open` - Whether gripper is in open position
    pub fn verify_drop(
        &mut self,
        rgb_data: &[u8],
        width: u32,
        height: u32,
        timestamp: u64,
        gripper_is_open: bool,
    ) -> VerificationResult {
        if !gripper_is_open {
            // Gripper not open - something is wrong
            return VerificationResult::failure(
                Operation::Drop,
                FailureReason::ServoTimeout,
                0.90,
                false,
                timestamp,
            );
        }

        // Use vision to check if piece is still in gripper zone
        let analysis = self.vision.analyze_frame(rgb_data, width, height, timestamp);

        // Check for piece in gripper zone (would be in center-top of frame)
        let piece_in_gripper = self.detect_piece_in_gripper_zone(&analysis);

        if piece_in_gripper {
            // Piece still in gripper - drop failed
            self.jam_detection.piece_in_gripper_zone = true;
            self.jam_detection.record_failure();

            let reason = FailureReason::PieceStuckInGripper;
            let retry = self.retry_policy.can_retry(Operation::Drop, self.jam_detection.consecutive_failures);

            VerificationResult::failure(Operation::Drop, reason, 0.85, retry, timestamp)
        } else {
            // Piece is gone - drop succeeded
            self.jam_detection.reset();
            VerificationResult::success(Operation::Drop, 0.90, timestamp)
        }
    }

    /// Check if a jam condition exists
    pub fn check_jam(&self) -> bool {
        self.jam_detection.is_jammed()
    }

    /// Get jam severity (0.0-1.0)
    pub fn jam_severity(&self) -> f32 {
        self.jam_detection.severity()
    }

    /// Check if two piece observations are likely the same piece
    fn is_same_piece(&self, obs1: &PieceObservation, obs2: &PieceObservation) -> bool {
        // Same color
        if obs1.color != obs2.color {
            return false;
        }

        // Same size category
        if obs1.estimated_size != obs2.estimated_size {
            return false;
        }

        // Position within tolerance (20mm)
        let distance = obs1.center_position.distance_to(&obs2.center_position);
        if distance > 20.0 {
            return false;
        }

        true
    }

    /// Detect if a piece is in the gripper zone
    fn detect_piece_in_gripper_zone(&self, analysis: &VisionAnalysisResult) -> bool {
        // Gripper zone is typically center-top of frame
        // For simplicity, check if any piece is in top 30% of frame
        for obs in &analysis.observations {
            if obs.center_position.y < self.vision.config().tray_region.height() * 0.3 {
                if obs.is_valid(self.min_confidence) {
                    return true;
                }
            }
        }
        false
    }
}

// ============================================
// Tests
// ============================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sorter::{
        BoundingBox, LegoColor, PieceSize, Position2D, TrayRegion, VisionConfig,
    };

    fn create_test_piece() -> PieceObservation {
        PieceObservation::new(
            "test_1".to_string(),
            0,
            LegoColor::Red,
            BoundingBox::new(50, 50, 40, 40),
            0.90,
            Position2D::new(70.0, 70.0),
            PieceSize::Medium,
        )
    }

    fn create_test_vision() -> VisionDetector {
        VisionDetector::with_defaults()
    }

    #[test]
    fn test_verification_result_creation() {
        let success = VerificationResult::success(Operation::Pick, 0.95, 1000);
        assert!(success.success);
        assert_eq!(success.confidence, 0.95);
        assert!(success.failure_reason.is_none());
        assert!(!success.retry_recommended);

        let failure = VerificationResult::failure(
            Operation::Pick,
            FailureReason::PieceNotGrabbed,
            0.80,
            true,
            2000,
        );
        assert!(!failure.success);
        assert_eq!(failure.confidence, 0.80);
        assert_eq!(
            failure.failure_reason,
            Some(FailureReason::PieceNotGrabbed)
        );
        assert!(failure.retry_recommended);
    }

    #[test]
    fn test_failure_reason_descriptions() {
        assert_eq!(
            FailureReason::PieceNotGrabbed.description(),
            "Piece was not grasped by gripper"
        );
        assert_eq!(
            FailureReason::JamDetected.description(),
            "Jam detected - manual intervention needed"
        );
    }

    #[test]
    fn test_failure_reason_retry_logic() {
        assert!(FailureReason::PieceNotGrabbed.should_retry());
        assert!(FailureReason::PieceStuckInGripper.should_retry());
        assert!(!FailureReason::JamDetected.should_retry());
        assert!(!FailureReason::ServoTimeout.should_retry());
    }

    #[test]
    fn test_retry_policy_defaults() {
        let policy = RetryPolicy::default();
        assert_eq!(policy.max_pick_retries, 2);
        assert_eq!(policy.max_drop_retries, 1);
        assert_eq!(policy.retry_delay_ms, 500);
        assert_eq!(policy.micro_adjust_mm, 2.0);
    }

    #[test]
    fn test_retry_policy_can_retry() {
        let policy = RetryPolicy::default();

        // Pick retries
        assert!(policy.can_retry(Operation::Pick, 0));
        assert!(policy.can_retry(Operation::Pick, 1));
        assert!(!policy.can_retry(Operation::Pick, 2));
        assert!(!policy.can_retry(Operation::Pick, 3));

        // Drop retries
        assert!(policy.can_retry(Operation::Drop, 0));
        assert!(!policy.can_retry(Operation::Drop, 1));
        assert!(!policy.can_retry(Operation::Drop, 2));
    }

    #[test]
    fn test_jam_detection_basics() {
        let mut jam = JamDetection::default();

        assert_eq!(jam.consecutive_failures, 0);
        assert!(!jam.is_jammed());
        assert_eq!(jam.severity(), 0.0);

        jam.record_failure();
        assert_eq!(jam.consecutive_failures, 1);
        assert!(!jam.is_jammed());
        assert!((jam.severity() - 0.333).abs() < 0.01);

        jam.record_failure();
        jam.record_failure();
        assert_eq!(jam.consecutive_failures, 3);
        assert!(jam.is_jammed());
        assert_eq!(jam.severity(), 1.0);

        jam.reset();
        assert_eq!(jam.consecutive_failures, 0);
        assert!(!jam.is_jammed());
    }

    #[test]
    fn test_jam_detection_custom_threshold() {
        let mut jam = JamDetection::new(5);
        assert_eq!(jam.jam_threshold, 5);

        for _ in 0..4 {
            jam.record_failure();
        }
        assert!(!jam.is_jammed());

        jam.record_failure();
        assert!(jam.is_jammed());
    }

    #[test]
    fn test_verification_system_creation() {
        let vision = create_test_vision();
        let policy = RetryPolicy::default();
        let system = VerificationSystem::new(vision, policy);

        assert_eq!(system.retry_policy().max_pick_retries, 2);
        assert!(!system.check_jam());
    }

    #[test]
    fn test_verify_pick_with_pressure() {
        let vision = create_test_vision();
        let policy = RetryPolicy::default();
        let mut system = VerificationSystem::new(vision, policy);

        let piece = create_test_piece();
        let empty_frame = vec![0u8; 640 * 480 * 3];

        // With gripper pressure: should succeed immediately
        let result = system.verify_pick(&piece, &empty_frame, 640, 480, 1000, true);

        assert!(result.success);
        assert_eq!(result.operation, Operation::Pick);
        assert!(result.confidence >= 0.90);
        assert!(!system.check_jam());
    }

    #[test]
    fn test_verify_pick_without_pressure_success() {
        let vision = create_test_vision();
        let policy = RetryPolicy::default();
        let mut system = VerificationSystem::new(vision, policy);

        let piece = create_test_piece();
        // Empty frame (piece gone from tray)
        let empty_frame = vec![128u8; 640 * 480 * 3];

        // Without pressure but piece gone: should succeed
        let result = system.verify_pick(&piece, &empty_frame, 640, 480, 1000, false);

        assert!(result.success);
        assert!(!system.check_jam());
    }

    #[test]
    fn test_verify_drop_success() {
        let vision = create_test_vision();
        let policy = RetryPolicy::default();
        let mut system = VerificationSystem::new(vision, policy);

        // Empty frame (no piece in gripper)
        let empty_frame = vec![128u8; 640 * 480 * 3];

        let result = system.verify_drop(&empty_frame, 640, 480, 1000, true);

        assert!(result.success);
        assert_eq!(result.operation, Operation::Drop);
        assert!(!system.check_jam());
    }

    #[test]
    fn test_verify_drop_gripper_not_open() {
        let vision = create_test_vision();
        let policy = RetryPolicy::default();
        let mut system = VerificationSystem::new(vision, policy);

        let empty_frame = vec![128u8; 640 * 480 * 3];

        // Gripper not open
        let result = system.verify_drop(&empty_frame, 640, 480, 1000, false);

        assert!(!result.success);
        assert_eq!(result.failure_reason, Some(FailureReason::ServoTimeout));
        assert!(!result.retry_recommended);
    }

    #[test]
    fn test_jam_detection_after_multiple_failures() {
        let vision = create_test_vision();
        let policy = RetryPolicy::default();
        let mut system = VerificationSystem::new(vision, policy);

        let piece = create_test_piece();
        let empty_frame = vec![128u8; 640 * 480 * 3];

        // Manually record failures to test jam detection threshold
        for _ in 0..3 {
            system.jam_detection.record_failure();
        }

        assert!(system.check_jam());
        assert_eq!(system.jam_severity(), 1.0);
    }

    #[test]
    fn test_jam_reset_on_success() {
        let vision = create_test_vision();
        let policy = RetryPolicy::default();
        let mut system = VerificationSystem::new(vision, policy);

        let piece = create_test_piece();
        let empty_frame = vec![128u8; 640 * 480 * 3];

        // Manually fail twice to test reset
        system.jam_detection.record_failure();
        system.jam_detection.record_failure();

        assert!(!system.check_jam());
        assert_eq!(system.jam_detection().consecutive_failures, 2);

        // Success resets (using gripper pressure for guaranteed success)
        system.verify_pick(&piece, &empty_frame, 640, 480, 2000, true);

        assert!(!system.check_jam());
        assert_eq!(system.jam_detection().consecutive_failures, 0);
    }

    #[test]
    fn test_is_same_piece() {
        let vision = create_test_vision();
        let policy = RetryPolicy::default();
        let system = VerificationSystem::new(vision, policy);

        let piece1 = PieceObservation::new(
            "p1".to_string(),
            0,
            LegoColor::Red,
            BoundingBox::new(50, 50, 40, 40),
            0.90,
            Position2D::new(70.0, 70.0),
            PieceSize::Medium,
        );

        // Same position, color, size
        let piece2 = PieceObservation::new(
            "p2".to_string(),
            100,
            LegoColor::Red,
            BoundingBox::new(55, 55, 40, 40),
            0.85,
            Position2D::new(72.0, 71.0),
            PieceSize::Medium,
        );

        assert!(system.is_same_piece(&piece1, &piece2));

        // Different color
        let piece3 = PieceObservation::new(
            "p3".to_string(),
            100,
            LegoColor::Blue,
            BoundingBox::new(50, 50, 40, 40),
            0.90,
            Position2D::new(70.0, 70.0),
            PieceSize::Medium,
        );

        assert!(!system.is_same_piece(&piece1, &piece3));

        // Different position (too far)
        let piece4 = PieceObservation::new(
            "p4".to_string(),
            100,
            LegoColor::Red,
            BoundingBox::new(150, 150, 40, 40),
            0.90,
            Position2D::new(170.0, 170.0),
            PieceSize::Medium,
        );

        assert!(!system.is_same_piece(&piece1, &piece4));
    }
}
