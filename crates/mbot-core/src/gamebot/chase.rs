//! Chase Game Mechanics for GameBot
//!
//! Implements chase and flee game modes where the robot either pursues or evades targets.
//! Uses ultrasonic sensors for target tracking and personality-based evasion patterns.
//!
//! # Invariants Enforced
//! - I-GAME-007: Tag is declared when ultrasonic distance reads <= 5cm for at least 100ms
//! - I-GAME-008: Chase speed must never exceed safe movement limits
//! - I-GAME-009: Flee mode must give human reasonable chance to catch (no impossible evasions)
//! - ARCH-GAME-002: All games must have timeout/forfeit mechanisms

#![cfg_attr(feature = "no_std", allow(unused_imports))]

#[cfg(feature = "no_std")]
extern crate alloc;

#[cfg(feature = "no_std")]
use alloc::{string::String, vec, vec::Vec};

#[cfg(not(feature = "no_std"))]
use std::{string::String, vec::Vec};

use core::fmt;

use crate::gamebot::emotions::{GameEmotionContext, GameOutcome, GameType};

/// Chase game modes
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChaseMode {
    /// Robot chases the target
    Chase,
    /// Robot flees from the pursuer
    Flee,
}

impl fmt::Display for ChaseMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ChaseMode::Chase => write!(f, "chase"),
            ChaseMode::Flee => write!(f, "flee"),
        }
    }
}

/// Chase game status
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChaseStatus {
    /// Waiting to start
    Ready,
    /// Game in progress
    Active,
    /// Tagged (robot caught target)
    Tagged,
    /// Caught (robot was caught)
    Caught,
    /// Timeout forfeit
    Timeout,
}

impl fmt::Display for ChaseStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ChaseStatus::Ready => write!(f, "ready"),
            ChaseStatus::Active => write!(f, "active"),
            ChaseStatus::Tagged => write!(f, "tagged"),
            ChaseStatus::Caught => write!(f, "caught"),
            ChaseStatus::Timeout => write!(f, "timeout"),
        }
    }
}

/// Personality-based evasion styles
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EvasionStyle {
    /// Aggressive high-speed evasion
    Aggressive,
    /// Playful moderate evasion
    Playful,
    /// Cautious slow evasion
    Cautious,
    /// Erratic unpredictable evasion (high anxiety)
    Erratic,
    /// Lazy minimal evasion (low anxiety)
    Lazy,
}

impl fmt::Display for EvasionStyle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EvasionStyle::Aggressive => write!(f, "aggressive"),
            EvasionStyle::Playful => write!(f, "playful"),
            EvasionStyle::Cautious => write!(f, "cautious"),
            EvasionStyle::Erratic => write!(f, "erratic"),
            EvasionStyle::Lazy => write!(f, "lazy"),
        }
    }
}

/// Chase game state
#[derive(Clone, Debug)]
pub struct ChaseState {
    /// Current mode (chase or flee)
    pub mode: ChaseMode,
    /// Current status
    pub status: ChaseStatus,
    /// Target distance in centimeters (from ultrasonic)
    pub target_distance: u16,
    /// Target angle in degrees (relative to robot front)
    pub target_angle: i16,
    /// Tag confirmation counter (milliseconds at tag distance)
    pub tag_confirmation_ms: u32,
    /// Number of successful evasions in flee mode
    pub evasion_count: u32,
    /// Chase duration in milliseconds
    pub chase_duration_ms: u32,
    /// Current evasion style based on personality
    pub evasion_style: EvasionStyle,
    /// Fairness degradation factor (increases with evasion count)
    pub fairness_factor: f32,
}

impl ChaseState {
    /// Create a new chase state
    pub fn new(mode: ChaseMode) -> Self {
        Self {
            mode,
            status: ChaseStatus::Ready,
            target_distance: 400, // Max ultrasonic range
            target_angle: 0,
            tag_confirmation_ms: 0,
            evasion_count: 0,
            chase_duration_ms: 0,
            evasion_style: EvasionStyle::Playful,
            fairness_factor: 1.0,
        }
    }

    /// Update target distance and check for tag
    pub fn update_target_distance(&mut self, distance_cm: u16, delta_ms: u32) {
        self.target_distance = distance_cm;

        // I-GAME-007: Tag at <= 5cm for at least 100ms
        if distance_cm <= 5 {
            self.tag_confirmation_ms += delta_ms;
            if self.tag_confirmation_ms >= 100 {
                match self.mode {
                    ChaseMode::Chase => self.status = ChaseStatus::Tagged,
                    ChaseMode::Flee => self.status = ChaseStatus::Caught,
                }
            }
        } else {
            // Reset confirmation if distance increases
            self.tag_confirmation_ms = 0;
        }
    }

    /// Update target angle
    pub fn update_target_angle(&mut self, angle_deg: i16) {
        self.target_angle = angle_deg;
    }

    /// Get current chase speed factor based on distance
    /// I-GAME-008: Speed scaling for chase mode
    pub fn get_chase_speed_factor(&self) -> f32 {
        if self.mode != ChaseMode::Chase {
            return 0.0;
        }

        match self.target_distance {
            0..=5 => 0.0,      // TAG! Stop
            6..=15 => 1.0,     // Full speed
            16..=30 => 0.8,    // Fast chase
            31..=50 => 0.6,    // Medium approach
            _ => 0.3,          // Slow stalk
        }
    }

    /// Get current flee speed factor with fairness degradation
    /// I-GAME-009: Flee mode must be beatable
    pub fn get_flee_speed_factor(&self) -> f32 {
        if self.mode != ChaseMode::Flee {
            return 0.0;
        }

        // Base speed depends on target distance
        let base_speed = match self.target_distance {
            0..=20 => 1.0,   // Full escape speed
            21..=40 => 0.8,  // Fast flee
            41..=60 => 0.6,  // Moderate flee
            _ => 0.3,        // Lazy flee
        };

        // Apply fairness degradation
        base_speed * self.fairness_factor
    }

    /// Update evasion count and fairness degradation
    /// I-GAME-009: After 10 evasions, robot slows down
    pub fn record_evasion(&mut self) {
        self.evasion_count += 1;

        // Fairness degradation kicks in after 5 evasions
        if self.evasion_count > 5 {
            // Reduce speed by 5% per additional evasion, minimum 40%
            let degradation = 1.0 - ((self.evasion_count - 5) as f32 * 0.05);
            self.fairness_factor = degradation.max(0.4);
        }
    }

    /// Update chase duration and check for timeout
    /// ARCH-GAME-002: Timeout forfeit mechanism
    pub fn update_duration(&mut self, delta_ms: u32, timeout_ms: u32) {
        self.chase_duration_ms += delta_ms;

        if self.chase_duration_ms >= timeout_ms {
            self.status = ChaseStatus::Timeout;
        }
    }

    /// Set evasion style based on personality
    pub fn set_evasion_style_from_personality(&mut self, anxiety: f32, competitiveness: f32) {
        self.evasion_style = if anxiety > 0.7 {
            EvasionStyle::Erratic
        } else if anxiety < 0.3 {
            EvasionStyle::Lazy
        } else if competitiveness > 0.7 {
            EvasionStyle::Aggressive
        } else if competitiveness < 0.3 {
            EvasionStyle::Cautious
        } else {
            EvasionStyle::Playful
        };
    }

    /// Switch mode between chase and flee
    pub fn switch_mode(&mut self) {
        self.mode = match self.mode {
            ChaseMode::Chase => ChaseMode::Flee,
            ChaseMode::Flee => ChaseMode::Chase,
        };
        // Reset state on mode switch
        self.tag_confirmation_ms = 0;
        self.evasion_count = 0;
        self.fairness_factor = 1.0;
    }

    /// Start the chase game
    pub fn start(&mut self) {
        self.status = ChaseStatus::Active;
        self.chase_duration_ms = 0;
        self.tag_confirmation_ms = 0;
        self.evasion_count = 0;
        self.fairness_factor = 1.0;
    }

    /// Check if game is over
    pub fn is_game_over(&self) -> bool {
        matches!(
            self.status,
            ChaseStatus::Tagged | ChaseStatus::Caught | ChaseStatus::Timeout
        )
    }

    /// Generate emotion context for current state
    pub fn generate_emotion_context(&self, intensity: f32) -> GameEmotionContext {
        let outcome = match self.status {
            ChaseStatus::Tagged => GameOutcome::Won,   // Robot tagged target
            ChaseStatus::Caught => GameOutcome::Lost,  // Robot was caught
            ChaseStatus::Timeout => GameOutcome::Draw, // Timeout = draw
            _ => GameOutcome::Thinking,                // Still playing
        };

        GameEmotionContext::new(GameType::Chase, outcome, intensity)
    }
}

/// Chase game configuration
#[derive(Clone, Debug)]
pub struct ChaseConfig {
    /// Tag threshold in centimeters (default 5)
    pub tag_threshold_cm: u16,
    /// Minimum chase speed (0-100)
    pub chase_speed_min: u8,
    /// Maximum chase speed (0-100)
    pub chase_speed_max: u8,
    /// Minimum flee speed (0-100)
    pub flee_speed_min: u8,
    /// Maximum flee speed (0-100)
    pub flee_speed_max: u8,
    /// Maximum ultrasonic detection range in cm
    pub detection_range_cm: u16,
    /// Timeout duration in milliseconds (default 120000 = 2 minutes)
    pub timeout_ms: u32,
    /// Tag confirmation time in milliseconds (default 100)
    pub tag_confirmation_ms: u32,
}

impl Default for ChaseConfig {
    fn default() -> Self {
        Self {
            tag_threshold_cm: 5,
            chase_speed_min: 30,
            chase_speed_max: 100,
            flee_speed_min: 30,
            flee_speed_max: 100,
            detection_range_cm: 400,
            timeout_ms: 120_000, // 2 minutes
            tag_confirmation_ms: 100,
        }
    }
}

impl ChaseConfig {
    /// Create a new chase config with defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// Set tag threshold
    pub fn with_tag_threshold(mut self, cm: u16) -> Self {
        self.tag_threshold_cm = cm;
        self
    }

    /// Set timeout duration
    pub fn with_timeout(mut self, ms: u32) -> Self {
        self.timeout_ms = ms;
        self
    }

    /// I-GAME-008: Validate speed limits are safe
    pub fn validate_speeds(&self) -> bool {
        self.chase_speed_min <= 100
            && self.chase_speed_max <= 100
            && self.flee_speed_min <= 100
            && self.flee_speed_max <= 100
            && self.chase_speed_min <= self.chase_speed_max
            && self.flee_speed_min <= self.flee_speed_max
    }
}

/// Movement command for evasion patterns
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MovementCommand {
    /// Movement type
    pub movement_type: MovementType,
    /// Speed (0-100)
    pub speed: u8,
    /// Duration in milliseconds
    pub duration_ms: u32,
    /// Angle for turns (degrees)
    pub angle: Option<i16>,
}

/// Movement types for chase/flee
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MovementType {
    Forward,
    Backward,
    TurnLeft,
    TurnRight,
    Spin,
}

impl fmt::Display for MovementType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MovementType::Forward => write!(f, "forward"),
            MovementType::Backward => write!(f, "backward"),
            MovementType::TurnLeft => write!(f, "turn_left"),
            MovementType::TurnRight => write!(f, "turn_right"),
            MovementType::Spin => write!(f, "spin"),
        }
    }
}

/// Evasion pattern definition
#[derive(Clone, Debug)]
pub struct EvasionPattern {
    /// Pattern name
    pub name: String,
    /// Movement commands in sequence
    pub movements: Vec<MovementCommand>,
    /// Which personalities use this pattern
    pub personality_match: Vec<EvasionStyle>,
    /// Historical success rate (0.0-1.0)
    pub success_rate: f32,
}

impl EvasionPattern {
    /// Create a new evasion pattern
    pub fn new(name: String, movements: Vec<MovementCommand>) -> Self {
        Self {
            name,
            movements,
            personality_match: Vec::new(),
            success_rate: 0.5,
        }
    }

    /// Add personality match
    pub fn with_personality(mut self, style: EvasionStyle) -> Self {
        self.personality_match.push(style);
        self
    }

    /// Get standard evasion patterns
    pub fn get_standard_patterns() -> Vec<EvasionPattern> {
        let mut patterns = Vec::new();

        // Erratic pattern for nervous personalities
        patterns.push(
            EvasionPattern::new(
                "erratic_zigzag".into(),
                vec![
                    MovementCommand {
                        movement_type: MovementType::Forward,
                        speed: 80,
                        duration_ms: 200,
                        angle: None,
                    },
                    MovementCommand {
                        movement_type: MovementType::TurnLeft,
                        speed: 70,
                        duration_ms: 150,
                        angle: Some(45),
                    },
                    MovementCommand {
                        movement_type: MovementType::Forward,
                        speed: 80,
                        duration_ms: 200,
                        angle: None,
                    },
                    MovementCommand {
                        movement_type: MovementType::TurnRight,
                        speed: 70,
                        duration_ms: 150,
                        angle: Some(45),
                    },
                ],
            )
            .with_personality(EvasionStyle::Erratic),
        );

        // Lazy pattern for chill personalities
        patterns.push(
            EvasionPattern::new(
                "lazy_turn".into(),
                vec![
                    MovementCommand {
                        movement_type: MovementType::Spin,
                        speed: 40,
                        duration_ms: 500,
                        angle: Some(180),
                    },
                    MovementCommand {
                        movement_type: MovementType::Forward,
                        speed: 50,
                        duration_ms: 400,
                        angle: None,
                    },
                ],
            )
            .with_personality(EvasionStyle::Lazy),
        );

        // Aggressive pattern
        patterns.push(
            EvasionPattern::new(
                "aggressive_escape".into(),
                vec![
                    MovementCommand {
                        movement_type: MovementType::Spin,
                        speed: 100,
                        duration_ms: 200,
                        angle: Some(180),
                    },
                    MovementCommand {
                        movement_type: MovementType::Forward,
                        speed: 100,
                        duration_ms: 500,
                        angle: None,
                    },
                ],
            )
            .with_personality(EvasionStyle::Aggressive),
        );

        // Playful pattern
        patterns.push(
            EvasionPattern::new(
                "playful_dodge".into(),
                vec![
                    MovementCommand {
                        movement_type: MovementType::Backward,
                        speed: 60,
                        duration_ms: 300,
                        angle: None,
                    },
                    MovementCommand {
                        movement_type: MovementType::TurnRight,
                        speed: 60,
                        duration_ms: 200,
                        angle: Some(90),
                    },
                    MovementCommand {
                        movement_type: MovementType::Forward,
                        speed: 70,
                        duration_ms: 300,
                        angle: None,
                    },
                ],
            )
            .with_personality(EvasionStyle::Playful),
        );

        patterns
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chase_state_creation() {
        let state = ChaseState::new(ChaseMode::Chase);
        assert_eq!(state.mode, ChaseMode::Chase);
        assert_eq!(state.status, ChaseStatus::Ready);
        assert_eq!(state.evasion_count, 0);
        assert_eq!(state.fairness_factor, 1.0);
    }

    #[test]
    fn test_tag_confirmation() {
        let mut state = ChaseState::new(ChaseMode::Chase);
        state.start();

        // Update distance to 5cm
        state.update_target_distance(5, 50);
        assert_eq!(state.status, ChaseStatus::Active); // Not yet confirmed

        // Update again with 50ms more (total 100ms)
        state.update_target_distance(5, 50);
        assert_eq!(state.status, ChaseStatus::Tagged); // Now confirmed (I-GAME-007)
    }

    #[test]
    fn test_tag_confirmation_reset() {
        let mut state = ChaseState::new(ChaseMode::Chase);
        state.start();

        // Brief distance dip
        state.update_target_distance(4, 50);
        assert_eq!(state.tag_confirmation_ms, 50);

        // Distance increases - reset
        state.update_target_distance(10, 10);
        assert_eq!(state.tag_confirmation_ms, 0);
        assert_eq!(state.status, ChaseStatus::Active);
    }

    #[test]
    fn test_chase_speed_scaling() {
        let state = ChaseState::new(ChaseMode::Chase);

        // Test speed factors at different distances
        let mut test_state = state.clone();
        test_state.target_distance = 60;
        assert!((test_state.get_chase_speed_factor() - 0.3).abs() < 0.01);

        test_state.target_distance = 40;
        assert!((test_state.get_chase_speed_factor() - 0.6).abs() < 0.01);

        test_state.target_distance = 20;
        assert!((test_state.get_chase_speed_factor() - 0.8).abs() < 0.01);

        test_state.target_distance = 10;
        assert!((test_state.get_chase_speed_factor() - 1.0).abs() < 0.01);

        test_state.target_distance = 3;
        assert!((test_state.get_chase_speed_factor() - 0.0).abs() < 0.01); // Tagged!
    }

    #[test]
    fn test_flee_fairness_degradation() {
        let mut state = ChaseState::new(ChaseMode::Flee);
        state.target_distance = 15;

        // First 5 evasions - full speed
        for _ in 0..5 {
            state.record_evasion();
            assert_eq!(state.fairness_factor, 1.0);
        }

        // 6th evasion - degradation starts (I-GAME-009)
        state.record_evasion();
        assert!(state.fairness_factor < 1.0);
        assert!(state.fairness_factor >= 0.4);

        // 10 more evasions
        for _ in 0..10 {
            state.record_evasion();
        }

        // Should be at minimum 40%
        assert!(state.fairness_factor >= 0.4);
    }

    #[test]
    fn test_timeout_forfeit() {
        let mut state = ChaseState::new(ChaseMode::Chase);
        state.start();

        // Update duration to exceed timeout
        state.update_duration(60_000, 120_000);
        assert_eq!(state.status, ChaseStatus::Active);

        state.update_duration(60_001, 120_000);
        assert_eq!(state.status, ChaseStatus::Timeout); // ARCH-GAME-002
    }

    #[test]
    fn test_mode_switching() {
        let mut state = ChaseState::new(ChaseMode::Chase);
        state.evasion_count = 5;

        state.switch_mode();
        assert_eq!(state.mode, ChaseMode::Flee);
        assert_eq!(state.evasion_count, 0); // Reset on switch

        state.switch_mode();
        assert_eq!(state.mode, ChaseMode::Chase);
    }

    #[test]
    fn test_evasion_style_from_personality() {
        let mut state = ChaseState::new(ChaseMode::Flee);

        // High anxiety -> erratic
        state.set_evasion_style_from_personality(0.9, 0.5);
        assert_eq!(state.evasion_style, EvasionStyle::Erratic);

        // Low anxiety -> lazy
        state.set_evasion_style_from_personality(0.2, 0.5);
        assert_eq!(state.evasion_style, EvasionStyle::Lazy);

        // High competitiveness -> aggressive
        state.set_evasion_style_from_personality(0.5, 0.9);
        assert_eq!(state.evasion_style, EvasionStyle::Aggressive);

        // Low competitiveness -> cautious
        state.set_evasion_style_from_personality(0.5, 0.2);
        assert_eq!(state.evasion_style, EvasionStyle::Cautious);

        // Middle values -> playful
        state.set_evasion_style_from_personality(0.5, 0.5);
        assert_eq!(state.evasion_style, EvasionStyle::Playful);
    }

    #[test]
    fn test_chase_config_validation() {
        let config = ChaseConfig::default();
        assert!(config.validate_speeds());

        let mut bad_config = config.clone();
        bad_config.chase_speed_max = 150; // Over 100
        assert!(!bad_config.validate_speeds());
    }

    #[test]
    fn test_evasion_patterns() {
        let patterns = EvasionPattern::get_standard_patterns();
        assert!(patterns.len() >= 4);

        // Check for required patterns
        let has_erratic = patterns.iter().any(|p| p.name.contains("erratic"));
        let has_lazy = patterns.iter().any(|p| p.name.contains("lazy"));
        let has_aggressive = patterns.iter().any(|p| p.name.contains("aggressive"));
        let has_playful = patterns.iter().any(|p| p.name.contains("playful"));

        assert!(has_erratic);
        assert!(has_lazy);
        assert!(has_aggressive);
        assert!(has_playful);
    }

    #[test]
    fn test_emotion_context_generation() {
        let mut state = ChaseState::new(ChaseMode::Chase);

        // Tagged -> Won
        state.status = ChaseStatus::Tagged;
        let context = state.generate_emotion_context(0.7);
        assert_eq!(context.outcome, GameOutcome::Won);
        assert_eq!(context.game_type, GameType::Chase);

        // Caught -> Lost
        state.status = ChaseStatus::Caught;
        let context = state.generate_emotion_context(0.7);
        assert_eq!(context.outcome, GameOutcome::Lost);

        // Timeout -> Draw
        state.status = ChaseStatus::Timeout;
        let context = state.generate_emotion_context(0.7);
        assert_eq!(context.outcome, GameOutcome::Draw);
    }

    #[test]
    fn test_game_over_detection() {
        let mut state = ChaseState::new(ChaseMode::Chase);
        assert!(!state.is_game_over());

        state.status = ChaseStatus::Tagged;
        assert!(state.is_game_over());

        state.status = ChaseStatus::Caught;
        assert!(state.is_game_over());

        state.status = ChaseStatus::Timeout;
        assert!(state.is_game_over());
    }
}
