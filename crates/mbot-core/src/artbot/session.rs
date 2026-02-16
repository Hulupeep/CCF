//! Drawing Session Manager - STORY-ART-004 Implementation
//!
//! Manages drawing sessions with state tracking, history, and persistence.
//! Implements ART-004 contract: session lifecycle and undo support
//!
//! # Invariants enforced:
//! - I-ART-004: Session state must be one of defined states
//! - I-ART-004a: Drawing history must be reversible (undo support)
//! - I-ART-004b: Sessions must track elapsed time accurately
//! - ARCH-001: no_std compatible
//! - ARCH-002: Deterministic state transitions

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use alloc::{string::String, vec::Vec};

#[cfg(feature = "std")]
use std::{string::String, vec::Vec};

use core::fmt;

use crate::artbot::shapes::DrawCommand;

/// Error type for session operations
#[derive(Debug, Clone, PartialEq)]
pub enum SessionError {
    /// Invalid state transition attempted
    InvalidStateTransition {
        from: &'static str,
        to: &'static str,
    },
    /// Session history limit exceeded
    HistoryFull,
    /// Cannot undo - no commands in history
    CannotUndo,
    /// Session not found or invalid ID
    SessionNotFound { id: u32 },
    /// Invalid session duration
    InvalidDuration { duration_ms: u64 },
}

impl fmt::Display for SessionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SessionError::InvalidStateTransition { from, to } => {
                write!(f, "cannot transition from '{}' to '{}'", from, to)
            }
            SessionError::HistoryFull => {
                write!(f, "session history limit exceeded")
            }
            SessionError::CannotUndo => {
                write!(f, "nothing to undo")
            }
            SessionError::SessionNotFound { id } => {
                write!(f, "session {} not found", id)
            }
            SessionError::InvalidDuration { duration_ms } => {
                write!(f, "invalid duration: {} ms", duration_ms)
            }
        }
    }
}

// ============================================
// Session Constants
// ============================================

/// Maximum commands that can be stored in history
pub const MAX_HISTORY_SIZE: usize = 1000;

/// Maximum sessions that can be active simultaneously
pub const MAX_SESSIONS: usize = 10;

/// Milliseconds per second (for time calculations)
pub const MS_PER_SECOND: u64 = 1000;

/// Maximum session duration (1 hour)
pub const MAX_SESSION_DURATION_MS: u64 = 60 * 60 * 1000;

// ============================================
// Session State Machine
// ============================================

/// Drawing session state (I-ART-004)
///
/// State transitions:
/// ```text
/// New → Idle ↔ Drawing → Paused → Drawing → Complete
/// Any state → Error
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionState {
    /// Session created but not started
    Idle,
    /// Currently drawing
    Drawing,
    /// Temporarily paused (can resume)
    Paused,
    /// Session ended successfully
    Complete,
    /// Session encountered an error
    Error,
}

impl SessionState {
    /// Returns the string representation of the state
    pub fn as_str(&self) -> &'static str {
        match self {
            SessionState::Idle => "idle",
            SessionState::Drawing => "drawing",
            SessionState::Paused => "paused",
            SessionState::Complete => "complete",
            SessionState::Error => "error",
        }
    }

    /// Checks if a state transition is valid
    fn can_transition_to(&self, next: SessionState) -> bool {
        match (self, next) {
            // From Idle
            (SessionState::Idle, SessionState::Drawing) => true,
            (SessionState::Idle, SessionState::Error) => true,

            // From Drawing
            (SessionState::Drawing, SessionState::Paused) => true,
            (SessionState::Drawing, SessionState::Complete) => true,
            (SessionState::Drawing, SessionState::Error) => true,

            // From Paused
            (SessionState::Paused, SessionState::Drawing) => true,
            (SessionState::Paused, SessionState::Error) => true,
            (SessionState::Paused, SessionState::Complete) => true,

            // From Complete
            (SessionState::Complete, SessionState::Error) => true,

            // From Error
            (SessionState::Error, _) => false,

            // Any other transition invalid
            _ => false,
        }
    }
}

impl fmt::Display for SessionState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

// ============================================
// Drawing History Entry
// ============================================

/// A single entry in the drawing history (I-ART-004a)
#[derive(Debug, Clone, PartialEq)]
pub struct HistoryEntry {
    /// Index in the session history
    pub index: usize,
    /// The drawing command executed
    pub command: DrawCommand,
    /// Timestamp when command was executed (milliseconds since session start)
    pub timestamp_ms: u64,
}

// ============================================
// Drawing Session
// ============================================

/// Manages a single drawing session with state and history (I-ART-004, I-ART-004a, I-ART-004b)
///
/// Tracks:
/// - Current state (idle/drawing/paused/complete)
/// - Drawing history for undo support
/// - Timing information
/// - Session metadata
#[derive(Debug, Clone)]
pub struct DrawingSession {
    /// Unique session identifier
    pub id: u32,
    /// Current session state
    state: SessionState,
    /// All drawing commands in this session (I-ART-004a)
    history: Vec<HistoryEntry>,
    /// Current position in history (for undo/redo)
    history_index: usize,
    /// Session start time (milliseconds since epoch)
    start_time_ms: u64,
    /// Total elapsed time (includes paused time)
    total_elapsed_ms: u64,
    /// Time when session was paused (for calculating pause duration)
    pause_start_ms: Option<u64>,
    /// Unique drawing identifier (used for persistence)
    artwork_name: String,
    /// Whether the session has been persisted
    persisted: bool,
}

impl DrawingSession {
    /// Creates a new drawing session
    ///
    /// # Arguments
    /// * `id` - Unique session identifier
    /// * `start_time_ms` - Session start time in milliseconds
    /// * `artwork_name` - Descriptive name for this artwork
    ///
    /// # Returns
    /// A new session in `Idle` state
    pub fn new(id: u32, start_time_ms: u64, artwork_name: impl Into<String>) -> Self {
        Self {
            id,
            state: SessionState::Idle,
            history: Vec::new(),
            history_index: 0,
            start_time_ms,
            total_elapsed_ms: 0,
            pause_start_ms: None,
            artwork_name: artwork_name.into(),
            persisted: false,
        }
    }

    /// Returns the current session state
    pub fn state(&self) -> SessionState {
        self.state
    }

    /// Transitions to a new state
    ///
    /// # Errors
    /// Returns `SessionError::InvalidStateTransition` if the transition is not allowed
    pub fn set_state(&mut self, next: SessionState) -> Result<(), SessionError> {
        if !self.state.can_transition_to(next) {
            return Err(SessionError::InvalidStateTransition {
                from: self.state.as_str(),
                to: next.as_str(),
            });
        }
        self.state = next;
        Ok(())
    }

    /// Starts the drawing session (transitions from Idle to Drawing)
    ///
    /// # Errors
    /// Returns `SessionError::InvalidStateTransition` if not in Idle state
    pub fn start(&mut self) -> Result<(), SessionError> {
        self.set_state(SessionState::Drawing)
    }

    /// Pauses the drawing session
    ///
    /// # Errors
    /// Returns `SessionError::InvalidStateTransition` if not in Drawing state
    pub fn pause(&mut self, current_time_ms: u64) -> Result<(), SessionError> {
        if self.state != SessionState::Drawing {
            return Err(SessionError::InvalidStateTransition {
                from: self.state.as_str(),
                to: "paused",
            });
        }
        self.pause_start_ms = Some(current_time_ms);
        self.state = SessionState::Paused;
        Ok(())
    }

    /// Resumes the drawing session
    ///
    /// # Errors
    /// Returns `SessionError::InvalidStateTransition` if not in Paused state
    pub fn resume(&mut self, current_time_ms: u64) -> Result<(), SessionError> {
        if self.state != SessionState::Paused {
            return Err(SessionError::InvalidStateTransition {
                from: self.state.as_str(),
                to: "drawing",
            });
        }

        // Add pause duration to total elapsed time
        if let Some(pause_start) = self.pause_start_ms {
            let pause_duration = current_time_ms.saturating_sub(pause_start);
            self.total_elapsed_ms = self.total_elapsed_ms.saturating_add(pause_duration);
        }
        self.pause_start_ms = None;
        self.state = SessionState::Drawing;
        Ok(())
    }

    /// Completes the drawing session
    ///
    /// # Errors
    /// Returns `SessionError::InvalidStateTransition` if not in Drawing or Paused state
    pub fn complete(&mut self, current_time_ms: u64) -> Result<(), SessionError> {
        // Handle pause time if session was paused
        if self.state == SessionState::Paused {
            if let Some(pause_start) = self.pause_start_ms {
                let pause_duration = current_time_ms.saturating_sub(pause_start);
                self.total_elapsed_ms = self.total_elapsed_ms.saturating_add(pause_duration);
            }
        }

        if self.state == SessionState::Drawing || self.state == SessionState::Paused {
            self.state = SessionState::Complete;
            Ok(())
        } else {
            Err(SessionError::InvalidStateTransition {
                from: self.state.as_str(),
                to: "complete",
            })
        }
    }

    /// Adds a drawing command to the history
    ///
    /// # Arguments
    /// * `command` - The drawing command to record
    /// * `timestamp_ms` - When the command was executed (relative to session start)
    ///
    /// # Errors
    /// Returns `SessionError::HistoryFull` if history size limit reached
    pub fn add_command(
        &mut self,
        command: DrawCommand,
        timestamp_ms: u64,
    ) -> Result<(), SessionError> {
        if self.history.len() >= MAX_HISTORY_SIZE {
            return Err(SessionError::HistoryFull);
        }

        // Truncate any redo history
        self.history.truncate(self.history_index);

        let entry = HistoryEntry {
            index: self.history.len(),
            command,
            timestamp_ms,
        };

        self.history.push(entry);
        self.history_index = self.history.len();

        Ok(())
    }

    /// Returns the total number of commands in history
    pub fn history_length(&self) -> usize {
        self.history.len()
    }

    /// Returns the current position in the history
    pub fn history_position(&self) -> usize {
        self.history_index
    }

    /// Gets a command from history by index
    pub fn get_command(&self, index: usize) -> Option<&DrawCommand> {
        if index < self.history.len() {
            Some(&self.history[index].command)
        } else {
            None
        }
    }

    /// Returns all commands from the current history position backward
    pub fn get_all_commands(&self) -> Vec<DrawCommand> {
        self.history
            .iter()
            .take(self.history_index)
            .map(|entry| entry.command.clone())
            .collect()
    }

    /// Undoes the last command (I-ART-004a)
    ///
    /// # Errors
    /// Returns `SessionError::CannotUndo` if at the beginning of history
    pub fn undo(&mut self) -> Result<DrawCommand, SessionError> {
        if self.history_index == 0 {
            return Err(SessionError::CannotUndo);
        }

        self.history_index -= 1;
        let command = self.history[self.history_index].command.clone();
        Ok(command)
    }

    /// Redoes the previously undone command
    ///
    /// # Errors
    /// Returns an error if nothing to redo
    pub fn redo(&mut self) -> Result<DrawCommand, SessionError> {
        if self.history_index >= self.history.len() {
            return Err(SessionError::CannotUndo);  // Reuse undo error for "nothing to redo"
        }

        let command = self.history[self.history_index].command.clone();
        self.history_index += 1;
        Ok(command)
    }

    /// Calculates elapsed drawing time (I-ART-004b)
    ///
    /// Returns the time actually spent drawing (excluding pauses)
    ///
    /// # Arguments
    /// * `current_time_ms` - Current time in milliseconds
    pub fn elapsed_drawing_time_ms(&self, current_time_ms: u64) -> u64 {
        let mut elapsed = current_time_ms.saturating_sub(self.start_time_ms);

        // Subtract pause duration if currently paused
        if let Some(pause_start) = self.pause_start_ms {
            let pause_duration = current_time_ms.saturating_sub(pause_start);
            elapsed = elapsed.saturating_sub(pause_duration);
        }

        elapsed
    }

    /// Returns the time in seconds since session started
    pub fn elapsed_seconds(&self, current_time_ms: u64) -> f32 {
        self.elapsed_drawing_time_ms(current_time_ms) as f32 / MS_PER_SECOND as f32
    }

    /// Marks the session as persisted
    pub fn mark_persisted(&mut self) {
        self.persisted = true;
    }

    /// Returns whether the session has been persisted
    pub fn is_persisted(&self) -> bool {
        self.persisted
    }

    /// Returns the artwork name
    pub fn artwork_name(&self) -> &str {
        &self.artwork_name
    }

    /// Clears the drawing history (for restarting)
    pub fn clear_history(&mut self) {
        self.history.clear();
        self.history_index = 0;
    }

    /// Returns summary statistics about the session
    pub fn stats(&self, current_time_ms: u64) -> SessionStats {
        SessionStats {
            id: self.id,
            state: self.state,
            total_commands: self.history.len(),
            elapsed_ms: self.elapsed_drawing_time_ms(current_time_ms),
            artwork_name: self.artwork_name.clone(),
            persisted: self.persisted,
        }
    }
}

/// Session statistics for display and logging
#[derive(Debug, Clone)]
pub struct SessionStats {
    pub id: u32,
    pub state: SessionState,
    pub total_commands: usize,
    pub elapsed_ms: u64,
    pub artwork_name: String,
    pub persisted: bool,
}

// ============================================
// Session Manager (Optional for embedded use)
// ============================================

/// Manages multiple drawing sessions
/// Useful for applications that support multiple concurrent sessions
#[derive(Debug)]
pub struct SessionManager {
    sessions: Vec<DrawingSession>,
}

impl SessionManager {
    /// Creates a new session manager
    pub fn new() -> Self {
        Self {
            sessions: Vec::new(),
        }
    }

    /// Creates and registers a new session
    pub fn create_session(
        &mut self,
        artwork_name: impl Into<String>,
        start_time_ms: u64,
    ) -> Result<u32, SessionError> {
        if self.sessions.len() >= MAX_SESSIONS {
            return Err(SessionError::HistoryFull);
        }

        let id = self.sessions.len() as u32;
        let session = DrawingSession::new(id, start_time_ms, artwork_name);
        self.sessions.push(session);

        Ok(id)
    }

    /// Gets a mutable reference to a session
    pub fn get_session(&mut self, id: u32) -> Result<&mut DrawingSession, SessionError> {
        self.sessions
            .iter_mut()
            .find(|s| s.id == id)
            .ok_or(SessionError::SessionNotFound { id })
    }

    /// Lists all active sessions
    pub fn list_sessions(&self) -> Vec<SessionStats> {
        let now = 0;  // Would be actual current time in real implementation
        self.sessions.iter().map(|s| s.stats(now)).collect()
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // === I-ART-004: Session State Machine ===

    #[test]
    fn test_session_initial_state() {
        let session = DrawingSession::new(1, 0, "test");
        assert_eq!(session.state(), SessionState::Idle);
    }

    #[test]
    fn test_state_transition_idle_to_drawing() {
        let mut session = DrawingSession::new(1, 0, "test");
        assert!(session.set_state(SessionState::Drawing).is_ok());
        assert_eq!(session.state(), SessionState::Drawing);
    }

    #[test]
    fn test_state_transition_drawing_to_paused() {
        let mut session = DrawingSession::new(1, 0, "test");
        let _ = session.start();
        assert!(session.pause(100).is_ok());
        assert_eq!(session.state(), SessionState::Paused);
    }

    #[test]
    fn test_state_transition_paused_to_drawing() {
        let mut session = DrawingSession::new(1, 0, "test");
        let _ = session.start();
        let _ = session.pause(100);
        assert!(session.resume(150).is_ok());
        assert_eq!(session.state(), SessionState::Drawing);
    }

    #[test]
    fn test_invalid_state_transition() {
        let mut session = DrawingSession::new(1, 0, "test");
        // Idle to Paused is not allowed
        let result = session.set_state(SessionState::Paused);
        assert!(result.is_err());
    }

    #[test]
    fn test_cannot_transition_from_error() {
        let mut session = DrawingSession::new(1, 0, "test");
        let _ = session.set_state(SessionState::Error);
        let result = session.set_state(SessionState::Idle);
        assert!(result.is_err());
    }

    // === I-ART-004a: Drawing History & Undo ===

    #[test]
    fn test_add_command() {
        let mut session = DrawingSession::new(1, 0, "test");
        let cmd = DrawCommand::Move { x: 10.0, y: 20.0 };

        assert!(session.add_command(cmd.clone(), 100).is_ok());
        assert_eq!(session.history_length(), 1);
    }

    #[test]
    fn test_get_command() {
        let mut session = DrawingSession::new(1, 0, "test");
        let cmd = DrawCommand::Move { x: 10.0, y: 20.0 };

        let _ = session.add_command(cmd.clone(), 100);
        assert_eq!(session.get_command(0), Some(&cmd));
    }

    #[test]
    fn test_undo() {
        let mut session = DrawingSession::new(1, 0, "test");
        let cmd1 = DrawCommand::Move { x: 10.0, y: 20.0 };
        let cmd2 = DrawCommand::PenDown;

        let _ = session.add_command(cmd1.clone(), 100);
        let _ = session.add_command(cmd2, 110);

        assert_eq!(session.history_position(), 2);
        let undone = session.undo().unwrap();
        assert_eq!(undone, DrawCommand::PenDown);
        assert_eq!(session.history_position(), 1);
    }

    #[test]
    fn test_cannot_undo_at_start() {
        let session = DrawingSession::new(1, 0, "test");
        let result = session.clone().undo();
        assert!(matches!(result, Err(SessionError::CannotUndo)));
    }

    #[test]
    fn test_redo() {
        let mut session = DrawingSession::new(1, 0, "test");
        let cmd = DrawCommand::Move { x: 10.0, y: 20.0 };

        let _ = session.add_command(cmd.clone(), 100);
        let _ = session.undo();

        let redone = session.redo().unwrap();
        assert_eq!(redone, cmd);
        assert_eq!(session.history_position(), 1);
    }

    #[test]
    fn test_history_limit() {
        let mut session = DrawingSession::new(1, 0, "test");
        let cmd = DrawCommand::Move { x: 10.0, y: 20.0 };

        // Fill history to limit
        for i in 0..MAX_HISTORY_SIZE {
            let _ = session.add_command(cmd.clone(), i as u64);
        }

        // Next add should fail
        let result = session.add_command(cmd, MAX_HISTORY_SIZE as u64);
        assert!(matches!(result, Err(SessionError::HistoryFull)));
    }

    #[test]
    fn test_undo_clears_redo() {
        let mut session = DrawingSession::new(1, 0, "test");
        let cmd1 = DrawCommand::Move { x: 10.0, y: 20.0 };
        let cmd2 = DrawCommand::Move { x: 30.0, y: 40.0 };

        let _ = session.add_command(cmd1, 100);
        let _ = session.add_command(cmd2, 110);
        let _ = session.undo();

        // Add new command should clear redo history
        let cmd3 = DrawCommand::PenDown;
        let _ = session.add_command(cmd3.clone(), 120);

        assert_eq!(session.history_length(), 2);
        assert_eq!(session.get_command(1), Some(&cmd3));
    }

    // === I-ART-004b: Time Tracking ===

    #[test]
    fn test_elapsed_time_calculation() {
        let session = DrawingSession::new(1, 0, "test");
        let elapsed = session.elapsed_drawing_time_ms(1000);
        assert_eq!(elapsed, 1000);
    }

    #[test]
    fn test_elapsed_time_with_pause() {
        let mut session = DrawingSession::new(1, 0, "test");
        let _ = session.start();
        let _ = session.pause(500);

        // Elapsed time should not include pause duration
        let elapsed = session.elapsed_drawing_time_ms(1000);
        assert!(elapsed < 1000);
    }

    #[test]
    fn test_elapsed_seconds() {
        let session = DrawingSession::new(1, 0, "test");
        let elapsed = session.elapsed_seconds(5000);
        assert!((elapsed - 5.0).abs() < 0.01);
    }

    #[test]
    fn test_session_persistence() {
        let mut session = DrawingSession::new(1, 0, "test");
        assert!(!session.is_persisted());

        session.mark_persisted();
        assert!(session.is_persisted());
    }

    #[test]
    fn test_session_stats() {
        let mut session = DrawingSession::new(42, 0, "artwork");
        let cmd = DrawCommand::Move { x: 10.0, y: 20.0 };
        let _ = session.add_command(cmd, 100);

        let stats = session.stats(1000);
        assert_eq!(stats.id, 42);
        assert_eq!(stats.total_commands, 1);
        assert_eq!(stats.state, SessionState::Idle);
    }

    #[test]
    fn test_session_manager_create() {
        let mut manager = SessionManager::new();
        let id = manager.create_session("test", 0).unwrap();
        assert_eq!(id, 0);
    }

    #[test]
    fn test_session_manager_get() {
        let mut manager = SessionManager::new();
        let id = manager.create_session("test", 0).unwrap();

        let session = manager.get_session(id).unwrap();
        assert_eq!(session.artwork_name(), "test");
    }

    #[test]
    fn test_session_manager_not_found() {
        let mut manager = SessionManager::new();
        let result = manager.get_session(99);
        assert!(matches!(result, Err(SessionError::SessionNotFound { id: 99 })));
    }

    #[test]
    fn test_complete_session_workflow() {
        let mut session = DrawingSession::new(1, 0, "artwork");

        // Start session
        assert!(session.start().is_ok());
        assert_eq!(session.state(), SessionState::Drawing);

        // Add some commands
        let _ = session.add_command(DrawCommand::Move { x: 0.0, y: 0.0 }, 100);
        let _ = session.add_command(DrawCommand::PenDown, 110);
        let _ = session.add_command(DrawCommand::Line { x: 10.0, y: 10.0 }, 120);

        // Pause and resume
        let _ = session.pause(130);
        assert_eq!(session.state(), SessionState::Paused);

        let _ = session.resume(200);
        assert_eq!(session.state(), SessionState::Drawing);

        // Complete
        assert!(session.complete(500).is_ok());
        assert_eq!(session.state(), SessionState::Complete);

        // Check stats
        let stats = session.stats(500);
        assert_eq!(stats.total_commands, 3);
    }
}
