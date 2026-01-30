//! Game Emotional Responses System for GameBot
//!
//! Maps game outcomes (thinking, victory, loss, draw) to emotional behaviors
//! (LED patterns, movements, sounds) influenced by personality.
//!
//! # Invariants Enforced
//! - I-GAME-004: Robot must NEVER become aggressive or refuse to play after losing
//! - I-GAME-005: Emotional responses must be proportional to game context
//! - I-GAME-006: Robot must always offer to play again after any game outcome

#![cfg_attr(feature = "no_std", allow(unused_imports))]

#[cfg(feature = "no_std")]
extern crate alloc;

#[cfg(feature = "no_std")]
use alloc::{string::String, vec::Vec};

#[cfg(not(feature = "no_std"))]
use std::{string::String, vec::Vec};

use core::fmt;

/// Game outcome types
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameOutcome {
    /// Robot is thinking/calculating its move
    Thinking,
    /// Robot won
    Won,
    /// Robot lost
    Lost,
    /// Game ended in a draw
    Draw,
}

impl fmt::Display for GameOutcome {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GameOutcome::Thinking => write!(f, "thinking"),
            GameOutcome::Won => write!(f, "won"),
            GameOutcome::Lost => write!(f, "lost"),
            GameOutcome::Draw => write!(f, "draw"),
        }
    }
}

/// LED pattern types for emotional expression
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LedPattern {
    /// Solid color
    Solid,
    /// Flashing on/off
    Flash,
    /// Smooth pulsing
    Pulse,
    /// Rainbow cycling
    Rainbow,
    /// Chase effect
    Chase,
}

impl fmt::Display for LedPattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LedPattern::Solid => write!(f, "solid"),
            LedPattern::Flash => write!(f, "flash"),
            LedPattern::Pulse => write!(f, "pulse"),
            LedPattern::Rainbow => write!(f, "rainbow"),
            LedPattern::Chase => write!(f, "chase"),
        }
    }
}

/// LED animation speed
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnimationSpeed {
    Slow,
    Medium,
    Fast,
}

impl fmt::Display for AnimationSpeed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AnimationSpeed::Slow => write!(f, "slow"),
            AnimationSpeed::Medium => write!(f, "medium"),
            AnimationSpeed::Fast => write!(f, "fast"),
        }
    }
}

/// Movement types for emotional expression
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MovementType {
    /// Subtle back-and-forth wiggle
    Wiggle,
    /// Full rotation spin
    Spin,
    /// Bouncing up and down
    Bounce,
    /// Dejected slump movement
    Slump,
    /// Shrug-like movement
    Shrug,
    /// Subtle pulse/breath
    Pulse,
}

impl fmt::Display for MovementType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MovementType::Wiggle => write!(f, "wiggle"),
            MovementType::Spin => write!(f, "spin"),
            MovementType::Bounce => write!(f, "bounce"),
            MovementType::Slump => write!(f, "slump"),
            MovementType::Shrug => write!(f, "shrug"),
            MovementType::Pulse => write!(f, "pulse"),
        }
    }
}

/// Sound types for emotional expression
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EmotionSound {
    /// Generic beep
    Beep,
    /// Melodic chime
    Chime,
    /// Celebration sound
    Celebration,
    /// Sad/disappointed tone
    Sad,
    /// Thinking hum
    Hum,
    /// Playful sound
    Playful,
}

impl fmt::Display for EmotionSound {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EmotionSound::Beep => write!(f, "beep"),
            EmotionSound::Chime => write!(f, "chime"),
            EmotionSound::Celebration => write!(f, "celebration"),
            EmotionSound::Sad => write!(f, "sad"),
            EmotionSound::Hum => write!(f, "hum"),
            EmotionSound::Playful => write!(f, "playful"),
        }
    }
}

/// LED specification for emotional display
#[derive(Clone, Debug)]
pub struct LedSpec {
    /// Primary color [R, G, B]
    pub primary_color: [u8; 3],
    /// Optional secondary color [R, G, B]
    pub secondary_color: Option<[u8; 3]>,
    /// LED pattern
    pub pattern: LedPattern,
    /// Animation speed
    pub speed: AnimationSpeed,
}

impl LedSpec {
    /// Create a new LED spec
    pub fn new(
        primary_color: [u8; 3],
        pattern: LedPattern,
        speed: AnimationSpeed,
    ) -> Self {
        Self {
            primary_color,
            secondary_color: None,
            pattern,
            speed,
        }
    }

    /// Add secondary color
    pub fn with_secondary(mut self, color: [u8; 3]) -> Self {
        self.secondary_color = Some(color);
        self
    }

    /// Blue pulse (thinking)
    pub fn thinking() -> Self {
        Self::new([0, 100, 200], LedPattern::Pulse, AnimationSpeed::Medium)
    }

    /// Green flash (victory)
    pub fn victory() -> Self {
        Self::new([0, 255, 0], LedPattern::Flash, AnimationSpeed::Fast)
    }

    /// Orange pulse (loss)
    pub fn loss() -> Self {
        Self::new([255, 165, 0], LedPattern::Pulse, AnimationSpeed::Slow)
    }

    /// Yellow solid (draw)
    pub fn draw() -> Self {
        Self::new([255, 255, 0], LedPattern::Solid, AnimationSpeed::Medium)
    }
}

/// Complete emotional behavior response
#[derive(Clone, Debug)]
pub struct EmotionBehavior {
    /// LED specification
    pub led: LedSpec,
    /// Movement type
    pub movement: MovementType,
    /// Optional sound to play
    pub sound: Option<EmotionSound>,
    /// Duration in milliseconds
    pub duration_ms: u32,
    /// Number of times to repeat (for repeating patterns)
    pub repeat_count: u32,
}

impl EmotionBehavior {
    /// Create a new emotion behavior
    pub fn new(
        led: LedSpec,
        movement: MovementType,
        duration_ms: u32,
    ) -> Self {
        Self {
            led,
            movement,
            sound: None,
            duration_ms,
            repeat_count: 1,
        }
    }

    /// Add sound to the behavior
    pub fn with_sound(mut self, sound: EmotionSound) -> Self {
        self.sound = Some(sound);
        self
    }

    /// Set repeat count
    pub fn with_repeat(mut self, count: u32) -> Self {
        self.repeat_count = count.max(1);
        self
    }

    /// Thinking behavior: blue pulse with subtle micro-movements
    pub fn thinking(duration_ms: u32, personality_anxiety: f32) -> Self {
        // Higher anxiety = longer thinking time
        let adjusted_duration = (duration_ms as f32 * (1.0 + personality_anxiety * 0.5)) as u32;

        let mut led = LedSpec::thinking();
        if personality_anxiety > 0.7 {
            led.speed = AnimationSpeed::Fast;
        } else if personality_anxiety < 0.3 {
            led.speed = AnimationSpeed::Slow;
        }

        Self::new(led, MovementType::Wiggle, adjusted_duration)
            .with_sound(EmotionSound::Hum)
    }

    /// Victory celebration: green flash with spin/bounce
    pub fn victory(intensity: f32, game_closeness: f32) -> Self {
        // Close games trigger stronger celebration
        let duration_base = 3000_u32;
        let duration = (duration_base as f32 * (1.0 + game_closeness * 0.5)) as u32;

        let mut behavior = Self::new(LedSpec::victory(), MovementType::Spin, duration)
            .with_sound(EmotionSound::Celebration);

        // Stronger intensity = more repeats
        if intensity > 0.7 {
            behavior = behavior.with_repeat(2);
        }

        behavior
    }

    /// Loss response: orange pulse with slump (graceful, non-aggressive I-GAME-004)
    pub fn loss() -> Self {
        Self::new(LedSpec::loss(), MovementType::Slump, 2500)
            .with_sound(EmotionSound::Sad)
    }

    /// Draw response: yellow solid with shrug
    pub fn draw() -> Self {
        Self::new(LedSpec::draw(), MovementType::Shrug, 1500)
            .with_sound(EmotionSound::Beep)
    }
}

/// Game emotion context for determining appropriate response
#[derive(Clone, Debug)]
pub struct GameEmotionContext {
    /// Type of game being played
    pub game_type: GameType,
    /// Current game outcome
    pub outcome: GameOutcome,
    /// Personality-based intensity (0.0-1.0)
    pub intensity: f32,
    /// How close was the game (0.0-1.0)
    pub game_closeness: f32,
    /// Consecutive win/loss streak
    pub streak: i32,
}

/// Game types for emotion customization
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameType {
    TicTacToe,
    Chase,
    Simon,
    Dance,
    HideSeek,
}

impl GameEmotionContext {
    /// Create a new game emotion context
    pub fn new(
        game_type: GameType,
        outcome: GameOutcome,
        intensity: f32,
    ) -> Self {
        Self {
            game_type,
            outcome,
            intensity: intensity.clamp(0.0, 1.0),
            game_closeness: 0.5,
            streak: 0,
        }
    }

    /// Set game closeness (0.0 = blowout, 1.0 = very close)
    pub fn with_closeness(mut self, closeness: f32) -> Self {
        self.game_closeness = closeness.clamp(0.0, 1.0);
        self
    }

    /// Set win/loss streak
    pub fn with_streak(mut self, streak: i32) -> Self {
        self.streak = streak;
        self
    }

    /// Generate the appropriate emotion behavior for this context
    pub fn generate_behavior(&self) -> EmotionBehavior {
        match self.outcome {
            GameOutcome::Thinking => {
                // Use intensity as anxiety factor (higher intensity = more anxious)
                EmotionBehavior::thinking(2000, self.intensity)
            }
            GameOutcome::Won => {
                EmotionBehavior::victory(self.intensity, self.game_closeness)
            }
            GameOutcome::Lost => {
                // Always graceful, never aggressive (I-GAME-004)
                EmotionBehavior::loss()
            }
            GameOutcome::Draw => {
                EmotionBehavior::draw()
            }
        }
    }

    /// Check if rematch should be offered (always true per I-GAME-006)
    pub fn should_offer_rematch(&self) -> bool {
        matches!(
            self.outcome,
            GameOutcome::Won | GameOutcome::Lost | GameOutcome::Draw
        )
    }

    /// Get rematch prompt text
    pub fn rematch_prompt(&self) -> &'static str {
        "Play again?"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_outcome_variants() {
        let _thinking = GameOutcome::Thinking;
        let _won = GameOutcome::Won;
        let _lost = GameOutcome::Lost;
        let _draw = GameOutcome::Draw;

        assert_eq!(GameOutcome::Thinking, GameOutcome::Thinking);
        assert_ne!(GameOutcome::Won, GameOutcome::Lost);
    }

    #[test]
    fn test_led_pattern_variants() {
        let _solid = LedPattern::Solid;
        let _flash = LedPattern::Flash;
        let _pulse = LedPattern::Pulse;
        let _rainbow = LedPattern::Rainbow;
        let _chase = LedPattern::Chase;

        assert_eq!(LedPattern::Solid, LedPattern::Solid);
        assert_ne!(LedPattern::Flash, LedPattern::Pulse);
    }

    #[test]
    fn test_animation_speed_variants() {
        let _slow = AnimationSpeed::Slow;
        let _medium = AnimationSpeed::Medium;
        let _fast = AnimationSpeed::Fast;

        assert_eq!(AnimationSpeed::Slow, AnimationSpeed::Slow);
        assert_ne!(AnimationSpeed::Medium, AnimationSpeed::Fast);
    }

    #[test]
    fn test_movement_type_variants() {
        let _wiggle = MovementType::Wiggle;
        let _spin = MovementType::Spin;
        let _bounce = MovementType::Bounce;
        let _slump = MovementType::Slump;
        let _shrug = MovementType::Shrug;
        let _pulse = MovementType::Pulse;

        assert_eq!(MovementType::Wiggle, MovementType::Wiggle);
        assert_ne!(MovementType::Spin, MovementType::Bounce);
    }

    #[test]
    fn test_emotion_sound_variants() {
        let _beep = EmotionSound::Beep;
        let _celebration = EmotionSound::Celebration;
        let _sad = EmotionSound::Sad;

        assert_eq!(EmotionSound::Beep, EmotionSound::Beep);
        assert_ne!(EmotionSound::Celebration, EmotionSound::Sad);
    }

    #[test]
    fn test_led_spec_presets() {
        let thinking = LedSpec::thinking();
        assert_eq!(thinking.pattern, LedPattern::Pulse);
        assert_eq!(thinking.primary_color, [0, 100, 200]);

        let victory = LedSpec::victory();
        assert_eq!(victory.pattern, LedPattern::Flash);
        assert_eq!(victory.primary_color, [0, 255, 0]);

        let loss = LedSpec::loss();
        assert_eq!(loss.pattern, LedPattern::Pulse);
        assert_eq!(loss.primary_color, [255, 165, 0]);

        let draw = LedSpec::draw();
        assert_eq!(draw.pattern, LedPattern::Solid);
        assert_eq!(draw.primary_color, [255, 255, 0]);
    }

    #[test]
    fn test_emotion_behavior_thinking() {
        let behavior = EmotionBehavior::thinking(2000, 0.5);
        assert_eq!(behavior.duration_ms, 3000); // 2000 * (1 + 0.5 * 0.5)
        assert_eq!(behavior.movement, MovementType::Wiggle);
        assert_eq!(behavior.sound, Some(EmotionSound::Hum));
    }

    #[test]
    fn test_emotion_behavior_thinking_anxious() {
        let behavior = EmotionBehavior::thinking(2000, 0.9);
        // Higher anxiety = faster animation
        assert_eq!(behavior.led.speed, AnimationSpeed::Fast);
        assert!(behavior.duration_ms > 2000);
    }

    #[test]
    fn test_emotion_behavior_victory() {
        let behavior = EmotionBehavior::victory(0.5, 0.5);
        assert_eq!(behavior.movement, MovementType::Spin);
        assert_eq!(behavior.sound, Some(EmotionSound::Celebration));
        assert!(behavior.duration_ms > 0);
    }

    #[test]
    fn test_emotion_behavior_loss() {
        let behavior = EmotionBehavior::loss();
        assert_eq!(behavior.movement, MovementType::Slump);
        assert_eq!(behavior.sound, Some(EmotionSound::Sad));
        assert_eq!(behavior.duration_ms, 2500);
    }

    #[test]
    fn test_emotion_behavior_draw() {
        let behavior = EmotionBehavior::draw();
        assert_eq!(behavior.movement, MovementType::Shrug);
        assert_eq!(behavior.sound, Some(EmotionSound::Beep));
        assert_eq!(behavior.duration_ms, 1500);
    }

    #[test]
    fn test_game_emotion_context_creation() {
        let context = GameEmotionContext::new(GameType::TicTacToe, GameOutcome::Won, 0.7);
        assert_eq!(context.game_type, GameType::TicTacToe);
        assert_eq!(context.outcome, GameOutcome::Won);
        assert_eq!(context.intensity, 0.7);
    }

    #[test]
    fn test_game_emotion_context_intensity_clamping() {
        let context = GameEmotionContext::new(GameType::TicTacToe, GameOutcome::Won, 1.5);
        assert_eq!(context.intensity, 1.0);

        let context = GameEmotionContext::new(GameType::TicTacToe, GameOutcome::Won, -0.5);
        assert_eq!(context.intensity, 0.0);
    }

    #[test]
    fn test_game_emotion_context_generate_victory() {
        let context = GameEmotionContext::new(GameType::TicTacToe, GameOutcome::Won, 0.8)
            .with_closeness(0.9);

        let behavior = context.generate_behavior();
        assert_eq!(behavior.movement, MovementType::Spin);
        assert_eq!(behavior.sound, Some(EmotionSound::Celebration));
    }

    #[test]
    fn test_game_emotion_context_generate_loss_non_aggressive() {
        // I-GAME-004: Loss must never be aggressive
        let context = GameEmotionContext::new(GameType::TicTacToe, GameOutcome::Lost, 0.9);
        let behavior = context.generate_behavior();

        assert_eq!(behavior.movement, MovementType::Slump);
        assert_ne!(behavior.led.primary_color, [255, 0, 0]); // Not red
    }

    #[test]
    fn test_game_emotion_context_rematch_always_offered() {
        // I-GAME-006: Must always offer rematch
        let won = GameEmotionContext::new(GameType::TicTacToe, GameOutcome::Won, 0.5);
        assert!(won.should_offer_rematch());

        let lost = GameEmotionContext::new(GameType::TicTacToe, GameOutcome::Lost, 0.5);
        assert!(lost.should_offer_rematch());

        let draw = GameEmotionContext::new(GameType::TicTacToe, GameOutcome::Draw, 0.5);
        assert!(draw.should_offer_rematch());

        let thinking = GameEmotionContext::new(GameType::TicTacToe, GameOutcome::Thinking, 0.5);
        assert!(!thinking.should_offer_rematch());
    }

    #[test]
    fn test_streak_tracking() {
        let context = GameEmotionContext::new(GameType::TicTacToe, GameOutcome::Won, 0.5)
            .with_streak(3);

        assert_eq!(context.streak, 3);
    }

    #[test]
    fn test_rematch_prompt() {
        let context = GameEmotionContext::new(GameType::TicTacToe, GameOutcome::Won, 0.5);
        assert_eq!(context.rematch_prompt(), "Play again?");
    }

    #[test]
    fn test_victory_intensity_scaling() {
        let weak = EmotionBehavior::victory(0.3, 0.5);
        let strong = EmotionBehavior::victory(0.8, 0.5);

        // Stronger intensity should lead to repeat
        assert!(strong.repeat_count >= weak.repeat_count);
    }

    #[test]
    fn test_game_closeness_affects_duration() {
        let blowout = EmotionBehavior::victory(0.5, 0.1);
        let close = EmotionBehavior::victory(0.5, 0.9);

        // Close game should have longer celebration
        assert!(close.duration_ms >= blowout.duration_ms);
    }
}
