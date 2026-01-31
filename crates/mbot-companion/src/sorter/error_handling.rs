//! Error Handling and Recovery for LEGO Sorter
//!
//! Implements STORY-SORT-005: Error Handling & Recovery
//! Provides strategies for recovering from pick/drop failures, jams, and edge cases.
//!
//! Contracts:
//! - SORT-002: Deterministic recovery actions
//! - SORT-003: Safety - all recovery moves are safe

use crate::sorter::sorting_loop::LoopState;
use crate::sorter::verification::{FailureReason, VerificationResult};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::time::{SystemTime, UNIX_EPOCH};

// ============================================
// Error Event Tracking
// ============================================

/// An error event that occurred during sorting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorEvent {
    /// Unique event identifier
    pub event_id: String,
    /// Timestamp in microseconds
    pub timestamp: u64,
    /// Type of error
    pub error_type: FailureReason,
    /// Piece observation ID (if applicable)
    pub piece_id: Option<String>,
    /// Operation that failed
    pub operation: Operation,
    /// Number of retries attempted
    pub retry_count: u32,
    /// How the error was resolved
    pub resolution: Resolution,
    /// Additional context/notes
    pub context: String,
}

impl ErrorEvent {
    /// Create a new error event
    pub fn new(
        event_id: String,
        timestamp: u64,
        error_type: FailureReason,
        piece_id: Option<String>,
        operation: Operation,
        retry_count: u32,
    ) -> Self {
        Self {
            event_id,
            timestamp,
            error_type,
            piece_id,
            operation,
            retry_count,
            resolution: Resolution::Pending,
            context: String::new(),
        }
    }

    /// Mark as retried
    pub fn mark_retried(&mut self, context: String) {
        self.resolution = Resolution::Retried;
        self.context = context;
    }

    /// Mark as skipped
    pub fn mark_skipped(&mut self, context: String) {
        self.resolution = Resolution::Skipped;
        self.context = context;
    }

    /// Mark as requiring manual intervention
    pub fn mark_manual_intervention(&mut self, context: String) {
        self.resolution = Resolution::ManualIntervention;
        self.context = context;
    }
}

/// Type of operation that failed
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Operation {
    Pick,
    Drop,
    Transport,
}

/// How an error was resolved
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Resolution {
    /// Error is still being handled
    Pending,
    /// Operation was retried
    Retried,
    /// Piece was skipped
    Skipped,
    /// Manual intervention required
    ManualIntervention,
}

// ============================================
// Recovery Actions
// ============================================

/// Action to take to recover from an error
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecoveryAction {
    /// Type of action
    pub action: Action,
    /// Reason for this action
    pub reason: String,
    /// Next state to transition to
    pub next_state: LoopState,
}

impl RecoveryAction {
    /// Create a retry action
    pub fn retry(reason: String) -> Self {
        Self {
            action: Action::Retry,
            reason,
            next_state: LoopState::Picking, // Will be adjusted by caller
        }
    }

    /// Create a skip action
    pub fn skip(reason: String) -> Self {
        Self {
            action: Action::Skip,
            reason,
            next_state: LoopState::Detecting,
        }
    }

    /// Create a shake action
    pub fn shake(reason: String) -> Self {
        Self {
            action: Action::Shake,
            reason,
            next_state: LoopState::Dropping,
        }
    }

    /// Create an alert action
    pub fn alert(reason: String) -> Self {
        Self {
            action: Action::Alert,
            reason,
            next_state: LoopState::Error,
        }
    }

    /// Create a stop action
    pub fn stop(reason: String) -> Self {
        Self {
            action: Action::Stop,
            reason,
            next_state: LoopState::Error,
        }
    }
}

/// Type of recovery action
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Action {
    /// Retry the operation
    Retry,
    /// Skip the current piece
    Skip,
    /// Shake to dislodge stuck piece
    Shake,
    /// Alert user but continue
    Alert,
    /// Stop sorting and require intervention
    Stop,
}

impl Action {
    /// Get human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            Action::Retry => "Retry operation with micro-adjustment",
            Action::Skip => "Skip piece and move to next",
            Action::Shake => "Shake gripper to dislodge piece",
            Action::Alert => "Alert user of issue",
            Action::Stop => "Stop and await manual intervention",
        }
    }
}

// ============================================
// Error Handler
// ============================================

/// Main error handling system
pub struct ErrorHandler {
    /// Recent error events (rolling window)
    error_log: VecDeque<ErrorEvent>,
    /// Maximum log size
    max_log_size: usize,
    /// Error counter for event IDs
    event_counter: u64,
    /// High error rate threshold (errors per 10 operations)
    high_error_rate_threshold: f32,
    /// Recent operation count (for rate calculation)
    recent_operations: u32,
    /// Recent error count (for rate calculation)
    recent_errors: u32,
}

impl Default for ErrorHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl ErrorHandler {
    /// Create a new error handler
    pub fn new() -> Self {
        Self {
            error_log: VecDeque::with_capacity(100),
            max_log_size: 100,
            event_counter: 0,
            high_error_rate_threshold: 0.5, // 50% error rate = 5 errors in 10 ops
            recent_operations: 0,
            recent_errors: 0,
        }
    }

    /// Record an operation (for rate tracking)
    pub fn record_operation(&mut self, success: bool) {
        self.recent_operations += 1;
        if !success {
            self.recent_errors += 1;
        }

        // Reset window after 10 operations
        if self.recent_operations >= 10 {
            self.recent_operations = 0;
            self.recent_errors = 0;
        }
    }

    /// Check if error rate is high
    pub fn is_high_error_rate(&self) -> bool {
        if self.recent_operations == 0 {
            return false;
        }
        let rate = self.recent_errors as f32 / self.recent_operations as f32;
        rate >= self.high_error_rate_threshold
    }

    /// Get current error rate (0.0-1.0)
    pub fn error_rate(&self) -> f32 {
        if self.recent_operations == 0 {
            return 0.0;
        }
        self.recent_errors as f32 / self.recent_operations as f32
    }

    /// Determine recovery action from verification result
    ///
    /// Contract: SORT-002 - Deterministic recovery based on failure type
    /// Contract: SORT-003 - All actions are safe
    pub fn determine_recovery(
        &mut self,
        verification: &VerificationResult,
        piece_id: Option<String>,
        retry_count: u32,
    ) -> RecoveryAction {
        // Log the error
        let event = self.log_error(verification, piece_id, retry_count);

        // Check for high error rate first
        if self.is_high_error_rate() {
            return RecoveryAction::stop(
                "High error rate detected (>50%) - check system setup".to_string(),
            );
        }

        // Determine action based on failure reason
        match verification.failure_reason {
            Some(FailureReason::PieceNotGrabbed) => {
                if verification.retry_recommended {
                    RecoveryAction::retry(format!(
                        "Pick failed, attempt {} - adjusting position",
                        retry_count + 1
                    ))
                } else {
                    RecoveryAction::skip("Max pick attempts reached - skipping piece".to_string())
                }
            }

            Some(FailureReason::PieceStuckInGripper) => {
                if retry_count == 0 {
                    RecoveryAction::shake("Piece stuck - attempting shake recovery".to_string())
                } else if verification.retry_recommended {
                    RecoveryAction::retry(format!(
                        "Shake unsuccessful, retry {} - reopening gripper",
                        retry_count + 1
                    ))
                } else {
                    RecoveryAction::alert("Piece stuck after retries - manual intervention may be needed".to_string())
                }
            }

            Some(FailureReason::PieceDroppedDuringTransport) => {
                RecoveryAction::skip(
                    "Piece dropped during transport - will be re-queued for sorting".to_string(),
                )
            }

            Some(FailureReason::BinMissed) => {
                if verification.retry_recommended {
                    RecoveryAction::retry("Piece missed bin - retrying drop".to_string())
                } else {
                    RecoveryAction::alert("Piece missed bin after retries - check bin position".to_string())
                }
            }

            Some(FailureReason::JamDetected) => RecoveryAction::stop(
                "Jam detected (3+ consecutive failures) - manual intervention required".to_string(),
            ),

            Some(FailureReason::ServoTimeout) => RecoveryAction::stop(
                "Servo timeout - check hardware connections".to_string(),
            ),

            Some(FailureReason::VisionLost) => RecoveryAction::alert(
                "Vision lost tracking - check camera and lighting".to_string(),
            ),

            None => {
                // Should not happen, but handle gracefully
                RecoveryAction::skip("Unknown error - skipping operation".to_string())
            }
        }
    }

    /// Log an error event
    fn log_error(
        &mut self,
        verification: &VerificationResult,
        piece_id: Option<String>,
        retry_count: u32,
    ) -> ErrorEvent {
        self.event_counter += 1;
        let event_id = format!("err_{}", self.event_counter);

        let operation = match verification.operation {
            crate::sorter::verification::Operation::Pick => Operation::Pick,
            crate::sorter::verification::Operation::Drop => Operation::Drop,
        };

        let error_type = verification
            .failure_reason
            .unwrap_or(FailureReason::VisionLost);

        let event = ErrorEvent::new(
            event_id,
            verification.timestamp,
            error_type,
            piece_id,
            operation,
            retry_count,
        );

        self.error_log.push_back(event.clone());

        // Trim log if too large
        while self.error_log.len() > self.max_log_size {
            self.error_log.pop_front();
        }

        self.record_operation(false);

        event
    }

    /// Mark an event as resolved
    pub fn resolve_event(&mut self, event_id: &str, resolution: Resolution, context: String) {
        if let Some(event) = self.error_log.iter_mut().find(|e| e.event_id == event_id) {
            event.resolution = resolution;
            event.context = context;
        }
    }

    /// Get the error log
    pub fn error_log(&self) -> &VecDeque<ErrorEvent> {
        &self.error_log
    }

    /// Get recent errors (last N)
    pub fn recent_errors(&self, count: usize) -> Vec<&ErrorEvent> {
        self.error_log
            .iter()
            .rev()
            .take(count)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect()
    }

    /// Get error count by type
    pub fn error_count_by_type(&self, error_type: FailureReason) -> usize {
        self.error_log
            .iter()
            .filter(|e| e.error_type == error_type)
            .count()
    }

    /// Get unresolved error count
    pub fn unresolved_count(&self) -> usize {
        self.error_log
            .iter()
            .filter(|e| e.resolution == Resolution::Pending)
            .count()
    }

    /// Clear the error log
    pub fn clear_log(&mut self) {
        self.error_log.clear();
        self.recent_operations = 0;
        self.recent_errors = 0;
    }

    /// Get error statistics
    pub fn statistics(&self) -> ErrorStatistics {
        let total = self.error_log.len();
        let unresolved = self.unresolved_count();
        let retried = self
            .error_log
            .iter()
            .filter(|e| e.resolution == Resolution::Retried)
            .count();
        let skipped = self
            .error_log
            .iter()
            .filter(|e| e.resolution == Resolution::Skipped)
            .count();
        let manual = self
            .error_log
            .iter()
            .filter(|e| e.resolution == Resolution::ManualIntervention)
            .count();

        ErrorStatistics {
            total_errors: total,
            unresolved_errors: unresolved,
            retried_count: retried,
            skipped_count: skipped,
            manual_intervention_count: manual,
            current_error_rate: self.error_rate(),
            high_error_rate_threshold: self.high_error_rate_threshold,
        }
    }
}

/// Error statistics summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorStatistics {
    /// Total errors logged
    pub total_errors: usize,
    /// Unresolved errors
    pub unresolved_errors: usize,
    /// Count of retried operations
    pub retried_count: usize,
    /// Count of skipped pieces
    pub skipped_count: usize,
    /// Count requiring manual intervention
    pub manual_intervention_count: usize,
    /// Current error rate (0.0-1.0)
    pub current_error_rate: f32,
    /// Threshold for high error rate
    pub high_error_rate_threshold: f32,
}

// ============================================
// Tests
// ============================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sorter::verification::Operation as VerifyOp;

    fn create_test_verification(
        operation: VerifyOp,
        reason: FailureReason,
        retry: bool,
    ) -> VerificationResult {
        VerificationResult::failure(operation, reason, 0.80, retry, 1000)
    }

    #[test]
    fn test_error_event_creation() {
        let event = ErrorEvent::new(
            "err_1".to_string(),
            1000,
            FailureReason::PieceNotGrabbed,
            Some("piece_1".to_string()),
            Operation::Pick,
            0,
        );

        assert_eq!(event.event_id, "err_1");
        assert_eq!(event.error_type, FailureReason::PieceNotGrabbed);
        assert_eq!(event.operation, Operation::Pick);
        assert_eq!(event.resolution, Resolution::Pending);
    }

    #[test]
    fn test_error_event_resolution() {
        let mut event = ErrorEvent::new(
            "err_1".to_string(),
            1000,
            FailureReason::PieceNotGrabbed,
            None,
            Operation::Pick,
            0,
        );

        event.mark_retried("Adjusted position".to_string());
        assert_eq!(event.resolution, Resolution::Retried);
        assert_eq!(event.context, "Adjusted position");

        event.mark_skipped("Max attempts".to_string());
        assert_eq!(event.resolution, Resolution::Skipped);
    }

    #[test]
    fn test_recovery_action_creation() {
        let retry = RecoveryAction::retry("Test retry".to_string());
        assert_eq!(retry.action, Action::Retry);
        assert_eq!(retry.reason, "Test retry");

        let skip = RecoveryAction::skip("Test skip".to_string());
        assert_eq!(skip.action, Action::Skip);
        assert_eq!(skip.next_state, LoopState::Detecting);

        let stop = RecoveryAction::stop("Test stop".to_string());
        assert_eq!(stop.action, Action::Stop);
        assert_eq!(stop.next_state, LoopState::Error);
    }

    #[test]
    fn test_action_descriptions() {
        assert_eq!(
            Action::Retry.description(),
            "Retry operation with micro-adjustment"
        );
        assert_eq!(Action::Skip.description(), "Skip piece and move to next");
        assert_eq!(
            Action::Shake.description(),
            "Shake gripper to dislodge piece"
        );
    }

    #[test]
    fn test_error_handler_creation() {
        let handler = ErrorHandler::new();
        assert_eq!(handler.error_log().len(), 0);
        assert_eq!(handler.error_rate(), 0.0);
        assert!(!handler.is_high_error_rate());
    }

    #[test]
    fn test_error_rate_tracking() {
        let mut handler = ErrorHandler::new();

        // Record 9 operations: 3 failures, 6 successes (avoid triggering reset at 10)
        for i in 0..9 {
            handler.record_operation(i >= 3);
        }

        // Error rate calculation: 3 failures out of 9 operations
        let expected_rate = 3.0 / 9.0;
        assert!((handler.error_rate() - expected_rate).abs() < 0.01); // ~33% error rate
        assert!(!handler.is_high_error_rate()); // Below 50% threshold
    }

    #[test]
    fn test_high_error_rate_detection() {
        let mut handler = ErrorHandler::new();

        // Record operations: false = failure, true = success
        // 6 failures, 2 successes = 75% error rate
        handler.record_operation(false);
        handler.record_operation(false);
        handler.record_operation(false);
        handler.record_operation(false);
        handler.record_operation(false);
        handler.record_operation(false);
        handler.record_operation(true);
        handler.record_operation(true);

        assert_eq!(handler.recent_operations, 8);
        assert_eq!(handler.recent_errors, 6);
        assert!(handler.error_rate() >= 0.5); // Above 50% threshold
        assert!(handler.is_high_error_rate());
    }

    #[test]
    fn test_determine_recovery_piece_not_grabbed() {
        let mut handler = ErrorHandler::new();

        // Ensure we don't trigger high error rate by recording some successes first
        for _ in 0..5 {
            handler.record_operation(true);
        }

        let verification = create_test_verification(
            VerifyOp::Pick,
            FailureReason::PieceNotGrabbed,
            true,
        );

        let action = handler.determine_recovery(&verification, Some("p1".to_string()), 0);

        assert_eq!(action.action, Action::Retry);
        assert!(action.reason.contains("Pick failed"));
    }

    #[test]
    fn test_determine_recovery_max_retries() {
        let mut handler = ErrorHandler::new();

        // Record successes to avoid high error rate
        for _ in 0..5 {
            handler.record_operation(true);
        }

        let verification = create_test_verification(
            VerifyOp::Pick,
            FailureReason::PieceNotGrabbed,
            false, // No retry recommended
        );

        let action = handler.determine_recovery(&verification, Some("p1".to_string()), 2);

        assert_eq!(action.action, Action::Skip);
        assert!(action.reason.contains("Max pick attempts"));
    }

    #[test]
    fn test_determine_recovery_stuck_piece() {
        let mut handler = ErrorHandler::new();

        // Record successes to avoid high error rate
        for _ in 0..5 {
            handler.record_operation(true);
        }

        let verification = create_test_verification(
            VerifyOp::Drop,
            FailureReason::PieceStuckInGripper,
            true,
        );

        let action = handler.determine_recovery(&verification, Some("p1".to_string()), 0);

        assert_eq!(action.action, Action::Shake);
        assert!(action.reason.contains("shake"));
    }

    #[test]
    fn test_determine_recovery_jam_detected() {
        let mut handler = ErrorHandler::new();

        // Record successes to avoid triggering high error rate (which also returns Stop)
        for _ in 0..5 {
            handler.record_operation(true);
        }

        let verification = create_test_verification(
            VerifyOp::Pick,
            FailureReason::JamDetected,
            false,
        );

        let action = handler.determine_recovery(&verification, Some("p1".to_string()), 3);

        assert_eq!(action.action, Action::Stop);
        assert!(action.reason.contains("Jam detected"));
        assert_eq!(action.next_state, LoopState::Error);
    }

    #[test]
    fn test_determine_recovery_servo_timeout() {
        let mut handler = ErrorHandler::new();

        // Record successes to avoid high error rate
        for _ in 0..5 {
            handler.record_operation(true);
        }

        let verification = create_test_verification(
            VerifyOp::Pick,
            FailureReason::ServoTimeout,
            false,
        );

        let action = handler.determine_recovery(&verification, Some("p1".to_string()), 0);

        assert_eq!(action.action, Action::Stop);
        assert!(action.reason.contains("Servo timeout"));
    }

    #[test]
    fn test_determine_recovery_high_error_rate() {
        let mut handler = ErrorHandler::new();

        // Create high error rate condition
        for i in 0..10 {
            handler.record_operation(i >= 6);
        }

        let verification = create_test_verification(
            VerifyOp::Pick,
            FailureReason::PieceNotGrabbed,
            true,
        );

        let action = handler.determine_recovery(&verification, Some("p1".to_string()), 0);

        assert_eq!(action.action, Action::Stop);
        assert!(action.reason.contains("High error rate"));
    }

    #[test]
    fn test_error_log_management() {
        let mut handler = ErrorHandler::new();

        // Add some errors
        for i in 0..5 {
            let verification = create_test_verification(
                VerifyOp::Pick,
                FailureReason::PieceNotGrabbed,
                true,
            );
            handler.determine_recovery(&verification, Some(format!("p{}", i)), 0);
        }

        assert_eq!(handler.error_log().len(), 5);

        let recent = handler.recent_errors(3);
        assert_eq!(recent.len(), 3);
    }

    #[test]
    fn test_error_count_by_type() {
        let mut handler = ErrorHandler::new();

        // Add different types of errors
        let v1 = create_test_verification(
            VerifyOp::Pick,
            FailureReason::PieceNotGrabbed,
            true,
        );
        handler.determine_recovery(&v1, None, 0);

        let v2 = create_test_verification(
            VerifyOp::Pick,
            FailureReason::PieceNotGrabbed,
            true,
        );
        handler.determine_recovery(&v2, None, 0);

        let v3 = create_test_verification(
            VerifyOp::Drop,
            FailureReason::PieceStuckInGripper,
            true,
        );
        handler.determine_recovery(&v3, None, 0);

        assert_eq!(
            handler.error_count_by_type(FailureReason::PieceNotGrabbed),
            2
        );
        assert_eq!(
            handler.error_count_by_type(FailureReason::PieceStuckInGripper),
            1
        );
    }

    #[test]
    fn test_resolve_event() {
        let mut handler = ErrorHandler::new();

        let verification = create_test_verification(
            VerifyOp::Pick,
            FailureReason::PieceNotGrabbed,
            true,
        );
        handler.determine_recovery(&verification, Some("p1".to_string()), 0);

        let event_id = handler.error_log().back().unwrap().event_id.clone();

        handler.resolve_event(&event_id, Resolution::Retried, "Successfully retried".to_string());

        let event = handler.error_log().iter().find(|e| e.event_id == event_id).unwrap();
        assert_eq!(event.resolution, Resolution::Retried);
        assert_eq!(event.context, "Successfully retried");
    }

    #[test]
    fn test_unresolved_count() {
        let mut handler = ErrorHandler::new();

        // Add 3 errors
        for i in 0..3 {
            let verification = create_test_verification(
                VerifyOp::Pick,
                FailureReason::PieceNotGrabbed,
                true,
            );
            handler.determine_recovery(&verification, Some(format!("p{}", i)), 0);
        }

        assert_eq!(handler.unresolved_count(), 3);

        // Resolve one
        let event_id = handler.error_log().front().unwrap().event_id.clone();
        handler.resolve_event(&event_id, Resolution::Retried, "Fixed".to_string());

        assert_eq!(handler.unresolved_count(), 2);
    }

    #[test]
    fn test_clear_log() {
        let mut handler = ErrorHandler::new();

        // Add errors
        for _ in 0..5 {
            let verification = create_test_verification(
                VerifyOp::Pick,
                FailureReason::PieceNotGrabbed,
                true,
            );
            handler.determine_recovery(&verification, None, 0);
        }

        assert_eq!(handler.error_log().len(), 5);

        handler.clear_log();
        assert_eq!(handler.error_log().len(), 0);
        assert_eq!(handler.error_rate(), 0.0);
    }

    #[test]
    fn test_error_statistics() {
        let mut handler = ErrorHandler::new();

        // Add various errors
        let v1 = create_test_verification(
            VerifyOp::Pick,
            FailureReason::PieceNotGrabbed,
            true,
        );
        handler.determine_recovery(&v1, None, 0);

        let v2 = create_test_verification(
            VerifyOp::Pick,
            FailureReason::PieceNotGrabbed,
            false,
        );
        handler.determine_recovery(&v2, None, 2);

        // Resolve one as retried
        let event_id = handler.error_log().front().unwrap().event_id.clone();
        handler.resolve_event(&event_id, Resolution::Retried, "Done".to_string());

        let stats = handler.statistics();
        assert_eq!(stats.total_errors, 2);
        assert_eq!(stats.unresolved_errors, 1);
        assert_eq!(stats.retried_count, 1);
        assert!(stats.current_error_rate > 0.0);
    }
}
