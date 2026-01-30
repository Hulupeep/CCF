//! Simon Says Memory Pattern Game for GameBot
//!
//! Implements a color-based memory game where the robot displays LED patterns
//! that the human must repeat. Patterns increase in length each round.
//!
//! # Invariants Enforced
//! - I-GAME-010: Each round's pattern MUST include all previous colors plus exactly one new color
//! - I-GAME-011: Each color must be visible for at least 600ms to be perceivable
//! - I-GAME-012: Human must have at least 5 seconds to input each color in the pattern
//! - ARCH-GAME-002: All games must have timeout/forfeit mechanisms

#![cfg_attr(feature = "no_std", allow(unused_imports))]

#[cfg(feature = "no_std")]
extern crate alloc;

#[cfg(feature = "no_std")]
use alloc::{string::String, vec::Vec};

#[cfg(not(feature = "no_std"))]
use std::{string::String, vec::Vec};

use core::fmt;

use crate::gamebot::emotions::{GameEmotionContext, GameOutcome, GameType};

/// Simon Says game colors
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SimonColor {
    Red,
    Green,
    Blue,
    Yellow,
}

impl fmt::Display for SimonColor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SimonColor::Red => write!(f, "red"),
            SimonColor::Green => write!(f, "green"),
            SimonColor::Blue => write!(f, "blue"),
            SimonColor::Yellow => write!(f, "yellow"),
        }
    }
}

impl SimonColor {
    /// Get RGB values for LED display
    pub fn to_rgb(&self) -> [u8; 3] {
        match self {
            SimonColor::Red => [255, 0, 0],
            SimonColor::Green => [0, 255, 0],
            SimonColor::Blue => [0, 0, 255],
            SimonColor::Yellow => [255, 255, 0],
        }
    }

    /// Get sound frequency for this color
    pub fn to_frequency_hz(&self) -> u16 {
        match self {
            SimonColor::Red => 440,    // A4
            SimonColor::Green => 554,  // C#5
            SimonColor::Blue => 659,   // E5
            SimonColor::Yellow => 880, // A5
        }
    }

    /// Get all available colors
    pub fn all_colors() -> [SimonColor; 4] {
        [
            SimonColor::Red,
            SimonColor::Green,
            SimonColor::Blue,
            SimonColor::Yellow,
        ]
    }
}

/// Simon Says game status
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SimonStatus {
    /// Waiting to start
    Ready,
    /// Displaying pattern
    Displaying,
    /// Waiting for user input
    Input,
    /// Pattern completed correctly
    PatternComplete,
    /// Game won (max rounds reached)
    Won,
    /// Game lost (incorrect input)
    Lost,
    /// Timeout
    Timeout,
}

impl fmt::Display for SimonStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SimonStatus::Ready => write!(f, "ready"),
            SimonStatus::Displaying => write!(f, "displaying"),
            SimonStatus::Input => write!(f, "input"),
            SimonStatus::PatternComplete => write!(f, "pattern_complete"),
            SimonStatus::Won => write!(f, "won"),
            SimonStatus::Lost => write!(f, "lost"),
            SimonStatus::Timeout => write!(f, "timeout"),
        }
    }
}

/// Simon Says game state
#[derive(Clone, Debug)]
pub struct SimonState {
    /// Current game status
    pub status: SimonStatus,
    /// Current round number (starts at 1)
    pub current_round: u32,
    /// The pattern to be repeated (I-GAME-010)
    pub pattern: Vec<SimonColor>,
    /// Current position in pattern during input
    pub input_index: usize,
    /// High score (highest round reached)
    pub high_score: u32,
    /// Display speed in milliseconds per color
    pub display_speed_ms: u32,
    /// Time elapsed waiting for current input
    pub input_elapsed_ms: u32,
}

impl SimonState {
    /// Create a new Simon Says game state
    pub fn new() -> Self {
        Self {
            status: SimonStatus::Ready,
            current_round: 0,
            pattern: Vec::new(),
            input_index: 0,
            high_score: 0,
            display_speed_ms: 800,
            input_elapsed_ms: 0,
        }
    }

    /// Start a new game
    pub fn start(&mut self) {
        self.status = SimonStatus::Displaying;
        self.current_round = 1;
        self.pattern = Vec::new();
        self.input_index = 0;
        self.input_elapsed_ms = 0;

        // Generate first color
        self.add_random_color();
    }

    /// Add a random color to the pattern (I-GAME-010)
    pub fn add_random_color(&mut self) {
        // In a real implementation, use a proper RNG
        // For now, we cycle through colors deterministically for testing
        let colors = SimonColor::all_colors();
        let next_color = colors[self.pattern.len() % 4];
        self.pattern.push(next_color);
    }

    /// Process user input
    pub fn process_input(&mut self, color: SimonColor) -> bool {
        if self.status != SimonStatus::Input {
            return false;
        }

        // Check if input matches pattern
        if self.input_index >= self.pattern.len() {
            return false;
        }

        let expected = self.pattern[self.input_index];
        let correct = expected == color;

        if correct {
            self.input_index += 1;
            self.input_elapsed_ms = 0; // Reset timeout for next color

            // Check if full pattern completed
            if self.input_index >= self.pattern.len() {
                self.status = SimonStatus::PatternComplete;
            }
        } else {
            // Incorrect input - game over
            self.status = SimonStatus::Lost;
            self.update_high_score();
        }

        correct
    }

    /// Advance to next round (I-GAME-010: add exactly one new color)
    pub fn advance_round(&mut self) {
        if self.status != SimonStatus::PatternComplete {
            return;
        }

        self.current_round += 1;
        self.input_index = 0;
        self.input_elapsed_ms = 0;

        // Add one new color to pattern (I-GAME-010)
        self.add_random_color();

        self.status = SimonStatus::Displaying;
    }

    /// Update input timer and check for timeout
    /// I-GAME-012: 5 second timeout per color
    pub fn update_input_timer(&mut self, delta_ms: u32, timeout_ms: u32) {
        if self.status != SimonStatus::Input {
            return;
        }

        self.input_elapsed_ms += delta_ms;

        if self.input_elapsed_ms >= timeout_ms {
            self.status = SimonStatus::Timeout;
            self.update_high_score();
        }
    }

    /// Mark display phase as complete and ready for input
    pub fn display_complete(&mut self) {
        if self.status == SimonStatus::Displaying {
            self.status = SimonStatus::Input;
            self.input_index = 0;
            self.input_elapsed_ms = 0;
        }
    }

    /// Update high score if current round is higher
    fn update_high_score(&mut self) {
        // High score is the last successfully completed round
        let completed_round = if self.current_round > 0 {
            self.current_round - 1
        } else {
            0
        };

        if completed_round > self.high_score {
            self.high_score = completed_round;
        }
    }

    /// Check if game is over
    pub fn is_game_over(&self) -> bool {
        matches!(
            self.status,
            SimonStatus::Won | SimonStatus::Lost | SimonStatus::Timeout
        )
    }

    /// Generate emotion context for current state
    pub fn generate_emotion_context(&self, intensity: f32) -> GameEmotionContext {
        let outcome = match self.status {
            SimonStatus::Won => GameOutcome::Won,
            SimonStatus::Lost | SimonStatus::Timeout => GameOutcome::Lost,
            _ => GameOutcome::Thinking,
        };

        GameEmotionContext::new(GameType::Simon, outcome, intensity)
    }

    /// Get expected color for current input position
    pub fn get_expected_color(&self) -> Option<SimonColor> {
        if self.input_index < self.pattern.len() {
            Some(self.pattern[self.input_index])
        } else {
            None
        }
    }

    /// Get progress through current pattern (0.0-1.0)
    pub fn get_input_progress(&self) -> f32 {
        if self.pattern.is_empty() {
            return 0.0;
        }
        self.input_index as f32 / self.pattern.len() as f32
    }
}

impl Default for SimonState {
    fn default() -> Self {
        Self::new()
    }
}

/// Simon Says game configuration
#[derive(Clone, Debug)]
pub struct SimonConfig {
    /// Duration to display each color (ms) - I-GAME-011: minimum 600ms
    pub color_display_ms: u32,
    /// Pause between colors in pattern (ms)
    pub pause_between_ms: u32,
    /// Timeout for each color input (ms) - I-GAME-012: minimum 5000ms
    pub input_timeout_ms: u32,
    /// Starting pattern length (default 1)
    pub starting_length: u32,
    /// Maximum rounds before winning (default 20)
    pub max_rounds: u32,
}

impl Default for SimonConfig {
    fn default() -> Self {
        Self {
            color_display_ms: 800,   // I-GAME-011: > 600ms minimum
            pause_between_ms: 200,
            input_timeout_ms: 5000,  // I-GAME-012: 5 seconds per color
            starting_length: 1,
            max_rounds: 20,
        }
    }
}

impl SimonConfig {
    /// Create a new config with defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// Set color display duration (I-GAME-011: enforce minimum 600ms)
    pub fn with_display_duration(mut self, ms: u32) -> Self {
        self.color_display_ms = ms.max(600); // Enforce I-GAME-011
        self
    }

    /// Set input timeout (I-GAME-012: enforce minimum 5000ms)
    pub fn with_input_timeout(mut self, ms: u32) -> Self {
        self.input_timeout_ms = ms.max(5000); // Enforce I-GAME-012
        self
    }

    /// Set pause between colors
    pub fn with_pause(mut self, ms: u32) -> Self {
        self.pause_between_ms = ms;
        self
    }

    /// Set max rounds
    pub fn with_max_rounds(mut self, rounds: u32) -> Self {
        self.max_rounds = rounds;
        self
    }

    /// Validate configuration
    pub fn validate(&self) -> bool {
        self.color_display_ms >= 600     // I-GAME-011
            && self.input_timeout_ms >= 5000 // I-GAME-012
            && self.starting_length >= 1
            && self.max_rounds > 0
    }

    /// Calculate total display time for a pattern
    pub fn calculate_display_time(&self, pattern_length: usize) -> u32 {
        if pattern_length == 0 {
            return 0;
        }

        let total_display = pattern_length as u32 * self.color_display_ms;
        let total_pauses = (pattern_length as u32 - 1) * self.pause_between_ms;

        total_display + total_pauses
    }
}

/// Pattern display event for tracking
#[derive(Clone, Debug)]
pub struct PatternDisplayEvent {
    /// Color being displayed
    pub color: SimonColor,
    /// Index in pattern (0-based)
    pub index: usize,
    /// Total pattern length
    pub total: usize,
    /// Display duration in milliseconds
    pub duration_ms: u32,
}

impl PatternDisplayEvent {
    /// Create a new display event
    pub fn new(color: SimonColor, index: usize, total: usize, duration_ms: u32) -> Self {
        Self {
            color,
            index,
            total,
            duration_ms,
        }
    }
}

/// Input result tracking
#[derive(Clone, Debug)]
pub struct InputResult {
    /// Expected color
    pub expected: SimonColor,
    /// Received color (or timeout)
    pub received: Option<SimonColor>,
    /// Whether input was correct
    pub correct: bool,
    /// Response time in milliseconds
    pub response_time_ms: u32,
}

impl InputResult {
    /// Create a new input result
    pub fn new(
        expected: SimonColor,
        received: Option<SimonColor>,
        response_time_ms: u32,
    ) -> Self {
        let correct = received.map_or(false, |r| r == expected);
        Self {
            expected,
            received,
            correct,
            response_time_ms,
        }
    }

    /// Create timeout result
    pub fn timeout(expected: SimonColor) -> Self {
        Self {
            expected,
            received: None,
            correct: false,
            response_time_ms: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simon_color_variants() {
        let red = SimonColor::Red;
        let green = SimonColor::Green;
        let _blue = SimonColor::Blue;
        let _yellow = SimonColor::Yellow;

        assert_eq!(red, SimonColor::Red);
        assert_ne!(red, green);

        // Check all colors are available
        let all = SimonColor::all_colors();
        assert_eq!(all.len(), 4);
    }

    #[test]
    fn test_simon_color_rgb() {
        assert_eq!(SimonColor::Red.to_rgb(), [255, 0, 0]);
        assert_eq!(SimonColor::Green.to_rgb(), [0, 255, 0]);
        assert_eq!(SimonColor::Blue.to_rgb(), [0, 0, 255]);
        assert_eq!(SimonColor::Yellow.to_rgb(), [255, 255, 0]);
    }

    #[test]
    fn test_simon_color_frequency() {
        assert_eq!(SimonColor::Red.to_frequency_hz(), 440);
        assert_eq!(SimonColor::Green.to_frequency_hz(), 554);
        assert_eq!(SimonColor::Blue.to_frequency_hz(), 659);
        assert_eq!(SimonColor::Yellow.to_frequency_hz(), 880);
    }

    #[test]
    fn test_simon_state_creation() {
        let state = SimonState::new();
        assert_eq!(state.status, SimonStatus::Ready);
        assert_eq!(state.current_round, 0);
        assert_eq!(state.pattern.len(), 0);
        assert_eq!(state.high_score, 0);
    }

    #[test]
    fn test_game_start_round_1() {
        let mut state = SimonState::new();
        state.start();

        assert_eq!(state.status, SimonStatus::Displaying);
        assert_eq!(state.current_round, 1);
        assert_eq!(state.pattern.len(), 1); // Round 1 starts with 1 color
    }

    #[test]
    fn test_pattern_integrity() {
        let mut state = SimonState::new();
        state.start();

        let first_color = state.pattern[0];

        // Complete round 1
        state.display_complete();
        state.process_input(first_color);
        assert_eq!(state.status, SimonStatus::PatternComplete);

        // Advance to round 2
        state.advance_round();
        assert_eq!(state.current_round, 2);
        assert_eq!(state.pattern.len(), 2); // Exactly one new color added (I-GAME-010)
        assert_eq!(state.pattern[0], first_color); // First color preserved (I-GAME-010)

        // Advance to round 3
        let second_color = state.pattern[1];
        state.display_complete();
        state.process_input(first_color);
        state.process_input(second_color);
        state.advance_round();

        assert_eq!(state.current_round, 3);
        assert_eq!(state.pattern.len(), 3); // Exactly one new color added
        assert_eq!(state.pattern[0], first_color); // Pattern integrity maintained
        assert_eq!(state.pattern[1], second_color);
    }

    #[test]
    fn test_correct_input() {
        let mut state = SimonState::new();
        state.start();
        state.display_complete();

        let expected = state.pattern[0];
        let result = state.process_input(expected);

        assert!(result);
        assert_eq!(state.status, SimonStatus::PatternComplete);
    }

    #[test]
    fn test_incorrect_input() {
        let mut state = SimonState::new();
        state.start();
        state.display_complete();

        let expected = state.pattern[0];
        let wrong = if expected == SimonColor::Red {
            SimonColor::Blue
        } else {
            SimonColor::Red
        };

        let result = state.process_input(wrong);

        assert!(!result);
        assert_eq!(state.status, SimonStatus::Lost);
    }

    #[test]
    fn test_multi_color_input() {
        let mut state = SimonState::new();
        state.start();

        // Complete round 1
        state.display_complete();
        state.process_input(state.pattern[0]);
        state.advance_round();

        // Now in round 2 with 2 colors
        assert_eq!(state.pattern.len(), 2);
        state.display_complete();

        // Input first color
        assert!(state.process_input(state.pattern[0]));
        assert_eq!(state.status, SimonStatus::Input); // Still waiting for second

        // Input second color
        assert!(state.process_input(state.pattern[1]));
        assert_eq!(state.status, SimonStatus::PatternComplete); // Now complete
    }

    #[test]
    fn test_input_timeout() {
        let mut state = SimonState::new();
        state.start();
        state.display_complete();

        // Update timer to exceed timeout
        state.update_input_timer(3000, 5000);
        assert_eq!(state.status, SimonStatus::Input);

        state.update_input_timer(2001, 5000);
        assert_eq!(state.status, SimonStatus::Timeout); // I-GAME-012
    }

    #[test]
    fn test_high_score_tracking() {
        let mut state = SimonState::new();
        state.start();

        // Complete 3 rounds
        for _ in 0..3 {
            state.display_complete();
            for i in 0..state.pattern.len() {
                state.process_input(state.pattern[i]);
            }
            if state.status == SimonStatus::PatternComplete {
                state.advance_round();
            }
        }

        // Fail on round 4
        state.display_complete();
        let wrong = if state.pattern[0] == SimonColor::Red {
            SimonColor::Blue
        } else {
            SimonColor::Red
        };
        state.process_input(wrong);

        // High score should be round 3
        assert_eq!(state.high_score, 3);
    }

    #[test]
    fn test_simon_config_defaults() {
        let config = SimonConfig::default();
        assert!(config.validate());
        assert!(config.color_display_ms >= 600); // I-GAME-011
        assert!(config.input_timeout_ms >= 5000); // I-GAME-012
    }

    #[test]
    fn test_simon_config_minimum_enforced() {
        // Try to set display time below minimum
        let config = SimonConfig::new().with_display_duration(400);
        assert_eq!(config.color_display_ms, 600); // Enforced to minimum (I-GAME-011)

        // Try to set timeout below minimum
        let config = SimonConfig::new().with_input_timeout(3000);
        assert_eq!(config.input_timeout_ms, 5000); // Enforced to minimum (I-GAME-012)
    }

    #[test]
    fn test_display_time_calculation() {
        let config = SimonConfig::default();

        // 1 color: 800ms display, 0 pauses
        assert_eq!(config.calculate_display_time(1), 800);

        // 3 colors: 3*800ms display + 2*200ms pauses = 2800ms
        assert_eq!(config.calculate_display_time(3), 2800);

        // 5 colors: 5*800ms + 4*200ms = 4800ms
        assert_eq!(config.calculate_display_time(5), 4800);
    }

    #[test]
    fn test_pattern_display_event() {
        let event = PatternDisplayEvent::new(SimonColor::Red, 0, 3, 800);
        assert_eq!(event.color, SimonColor::Red);
        assert_eq!(event.index, 0);
        assert_eq!(event.total, 3);
        assert_eq!(event.duration_ms, 800);
    }

    #[test]
    fn test_input_result() {
        let result = InputResult::new(SimonColor::Red, Some(SimonColor::Red), 1200);
        assert!(result.correct);
        assert_eq!(result.response_time_ms, 1200);

        let wrong = InputResult::new(SimonColor::Red, Some(SimonColor::Blue), 800);
        assert!(!wrong.correct);

        let timeout = InputResult::timeout(SimonColor::Green);
        assert!(!timeout.correct);
        assert!(timeout.received.is_none());
    }

    #[test]
    fn test_emotion_context_generation() {
        let mut state = SimonState::new();

        // Won
        state.status = SimonStatus::Won;
        let context = state.generate_emotion_context(0.7);
        assert_eq!(context.outcome, GameOutcome::Won);
        assert_eq!(context.game_type, GameType::Simon);

        // Lost
        state.status = SimonStatus::Lost;
        let context = state.generate_emotion_context(0.7);
        assert_eq!(context.outcome, GameOutcome::Lost);

        // Timeout
        state.status = SimonStatus::Timeout;
        let context = state.generate_emotion_context(0.7);
        assert_eq!(context.outcome, GameOutcome::Lost);
    }

    #[test]
    fn test_get_expected_color() {
        let mut state = SimonState::new();
        state.start();
        state.display_complete();

        let expected = state.get_expected_color();
        assert!(expected.is_some());
        assert_eq!(expected.unwrap(), state.pattern[0]);

        // After correct input, expect None
        state.process_input(state.pattern[0]);
        assert!(state.get_expected_color().is_none());
    }

    #[test]
    fn test_input_progress() {
        let mut state = SimonState::new();
        state.start();

        // Round 1: pattern length 1
        state.display_complete();
        assert_eq!(state.get_input_progress(), 0.0);

        state.process_input(state.pattern[0]);
        assert_eq!(state.get_input_progress(), 1.0);

        // Round 2: pattern length 2
        state.advance_round();
        state.display_complete();
        assert_eq!(state.get_input_progress(), 0.0);

        state.process_input(state.pattern[0]);
        assert_eq!(state.get_input_progress(), 0.5);

        state.process_input(state.pattern[1]);
        assert_eq!(state.get_input_progress(), 1.0);
    }
}
