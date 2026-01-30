//! ArtBot - Drawing and artistic expression system for mBot2
//!
//! This module provides shape drawing, pen control, and artistic features.
//!
//! # Features
//! - **Pen Control (ART-001)**: Servo-based pen lift/lower mechanism
//! - **Shape Drawing (ART-003)**: Circle, spiral, line, arc, scribble
//! - **Mood Drawing (ART-002)**: Map personality to drawing style
//! - **Session Management (ART-004)**: Drawing session lifecycle and history
//!
//! # Invariants
//! - I-ART-001: Pen servo control MUST be abstracted from drawing logic
//! - I-ART-002: Line styles MUST derive deterministically from tension
//! - I-ART-003: Closed shapes must close within 5mm of start point
//! - I-ART-004: Session state must follow valid state machine transitions
//! - I-ART-004a: All drawing commands must be reversible via undo
//! - I-ART-004b: Session timing must exclude pause durations
//! - I-ART-005: Pen MUST be in up position when not actively drawing
//! - I-ART-006: Servo MUST respond within 100ms of command

pub mod pen_control;
pub mod session;
pub mod shapes;
pub mod styles;

// Re-export public API
pub use pen_control::{LocalPenControl, PenConfig, PenControl, PenError, PenPosition};
pub use session::{DrawingSession, SessionError, SessionManager, SessionState, SessionStats};
pub use shapes::{DrawCommand, CLOSURE_TOLERANCE_MM, VARIANCE_TOLERANCE};
pub use styles::{interpolate_styles, LineStyle, StyleError};
