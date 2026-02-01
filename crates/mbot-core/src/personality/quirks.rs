//! Quirks System (#26)
//!
//! Quirks are unique, surprising behaviors that make each personality distinct.
//! They trigger based on specific conditions and respect cooldown periods.
//!
//! # Invariants
//! - **I-PERS-016:** Quirks must not interfere with safety behaviors
//! - **I-PERS-017:** Quirk activation rates must be configurable
//! - **I-PERS-018:** Quirks must respect cooldown periods
//! - **I-PERS-019:** Multiple quirks can coexist without conflict

#[cfg(feature = "no_std")]
extern crate alloc;

#[cfg(feature = "no_std")]
use alloc::vec::Vec;
#[cfg(feature = "no_std")]
use alloc::string::String;

#[cfg(not(feature = "no_std"))]
use std::vec::Vec;
#[cfg(not(feature = "no_std"))]
use std::string::String;

use core::fmt;

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

/// All available quirks in the system
///
/// Each quirk has a unique triggering condition and behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum Quirk {
    /// Occasional sigh sound when idle
    RandomSigh,
    /// Spin in place when coherence is high
    SpinWhenHappy,
    /// Always reverse first when startled
    BackUpWhenScared,
    /// Sometimes chases own "tail" when bored
    ChaseTail,
    /// Stops near objects, doesn't want to leave
    CollectorInstinct,
    /// More active in darker environments
    NightOwl,
    /// More active in brighter environments
    EarlyBird,
    /// Seeks out movement/sound sources
    SocialButterfly,
    /// Avoids movement/sound sources
    Hermit,
}

impl Quirk {
    /// Returns all available quirks
    pub const fn all() -> &'static [Quirk] {
        &[
            Quirk::RandomSigh,
            Quirk::SpinWhenHappy,
            Quirk::BackUpWhenScared,
            Quirk::ChaseTail,
            Quirk::CollectorInstinct,
            Quirk::NightOwl,
            Quirk::EarlyBird,
            Quirk::SocialButterfly,
            Quirk::Hermit,
        ]
    }

    /// Returns the string representation of this quirk
    pub const fn to_str(self) -> &'static str {
        match self {
            Quirk::RandomSigh => "random_sigh",
            Quirk::SpinWhenHappy => "spin_when_happy",
            Quirk::BackUpWhenScared => "back_up_when_scared",
            Quirk::ChaseTail => "chase_tail",
            Quirk::CollectorInstinct => "collector_instinct",
            Quirk::NightOwl => "night_owl",
            Quirk::EarlyBird => "early_bird",
            Quirk::SocialButterfly => "social_butterfly",
            Quirk::Hermit => "hermit",
        }
    }

    /// Parses a quirk from a string
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "random_sigh" => Some(Quirk::RandomSigh),
            "spin_when_happy" => Some(Quirk::SpinWhenHappy),
            "back_up_when_scared" => Some(Quirk::BackUpWhenScared),
            "chase_tail" => Some(Quirk::ChaseTail),
            "collector_instinct" => Some(Quirk::CollectorInstinct),
            "night_owl" => Some(Quirk::NightOwl),
            "early_bird" => Some(Quirk::EarlyBird),
            "social_butterfly" => Some(Quirk::SocialButterfly),
            "hermit" => Some(Quirk::Hermit),
            _ => None,
        }
    }

    /// Returns a human-readable description of this quirk
    pub const fn description(self) -> &'static str {
        match self {
            Quirk::RandomSigh => "Occasionally emits a sigh sound when idle, as if contemplating",
            Quirk::SpinWhenHappy => "Performs a celebration spin when coherence is very high",
            Quirk::BackUpWhenScared => "Always backs up first before reacting to sudden stimuli",
            Quirk::ChaseTail => "Sometimes spins in circles chasing its own tail when bored",
            Quirk::CollectorInstinct => "Stops near interesting objects and is reluctant to leave",
            Quirk::NightOwl => "Becomes more energetic and active in low-light environments",
            Quirk::EarlyBird => "Becomes more energetic and active in bright environments",
            Quirk::SocialButterfly => "Actively seeks out and approaches movement or sound sources",
            Quirk::Hermit => "Tends to avoid and retreat from movement or sound sources",
        }
    }

    /// Returns the default cooldown in milliseconds
    ///
    /// Safety-related quirks (BackUpWhenScared) have no cooldown.
    /// Continuous modifiers (NightOwl, EarlyBird, SocialButterfly, Hermit) have no cooldown.
    pub const fn default_cooldown_ms(self) -> u64 {
        match self {
            // Safety - no cooldown
            Quirk::BackUpWhenScared => 0,
            // Continuous modifiers - no cooldown
            Quirk::NightOwl => 0,
            Quirk::EarlyBird => 0,
            Quirk::SocialButterfly => 0,
            Quirk::Hermit => 0,
            // Rare behaviors - long cooldown
            Quirk::ChaseTail => 30_000,
            Quirk::CollectorInstinct => 20_000,
            // Common behaviors - medium cooldown
            Quirk::RandomSigh => 15_000,
            Quirk::SpinWhenHappy => 10_000,
        }
    }

    /// Returns the default activation chance (0.0-1.0)
    pub fn default_activation_chance(self) -> f32 {
        match self {
            // Continuous modifiers - always active
            Quirk::NightOwl => 1.0,
            Quirk::EarlyBird => 1.0,
            // Safety behaviors - very high chance
            Quirk::BackUpWhenScared => 0.9,
            // Social behaviors - high chance
            Quirk::SocialButterfly => 0.7,
            Quirk::Hermit => 0.7,
            // Fun behaviors - medium chance
            Quirk::SpinWhenHappy => 0.3,
            Quirk::CollectorInstinct => 0.4,
            // Ambient behaviors - low chance
            Quirk::RandomSigh => 0.15,
            Quirk::ChaseTail => 0.1,
        }
    }
}

impl fmt::Display for Quirk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_str())
    }
}

/// Trigger condition for a quirk
#[derive(Debug, Clone)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum QuirkTrigger {
    /// Triggers when robot has been idle for a duration
    Idle {
        /// Minimum idle duration in milliseconds
        duration_ms: u64,
    },
    /// Triggers when a nervous system state crosses a threshold
    StateThreshold {
        /// Which state to check (tension, coherence, energy)
        state_key: StateKey,
        /// Threshold value (0.0-1.0)
        threshold: f32,
        /// True if trigger above threshold, false if below
        above: bool,
    },
    /// Triggers on sudden stimulus (sound, touch, etc.)
    Stimulus {
        /// Minimum stimulus intensity to trigger
        min_intensity: f32,
    },
    /// Triggers based on environment conditions
    Environment {
        /// Type of environmental condition
        condition: EnvironmentCondition,
    },
}

/// Nervous system state keys
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum StateKey {
    Tension,
    Coherence,
    Energy,
}

/// Environmental conditions that can trigger quirks
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum EnvironmentCondition {
    /// Light level below threshold
    LowLight,
    /// Light level above threshold
    BrightLight,
    /// Movement detected in environment
    MovementDetected,
    /// Sound detected in environment
    SoundDetected,
    /// Near an interesting object
    NearObject,
}

/// Behavior that executes when a quirk triggers
#[derive(Debug, Clone)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum QuirkBehavior {
    /// Simple movement pattern
    Movement {
        /// Movement type (spin, backup, etc.)
        pattern: MovementPattern,
    },
    /// Sound emission
    Sound {
        /// Sound type
        sound: SoundType,
    },
    /// Light pattern
    Light {
        /// Light pattern type
        pattern: LightPattern,
    },
    /// Modifies personality parameters temporarily
    ParameterModifier {
        /// Which parameter to modify
        parameter: String,
        /// Modifier value (added to baseline)
        modifier: f32,
    },
    /// Combination of multiple behaviors
    Compound {
        /// List of behaviors to execute
        behaviors: Vec<QuirkBehavior>,
    },
}

/// Movement patterns for quirk behaviors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum MovementPattern {
    /// Spin in place
    Spin,
    /// Back up
    Backup,
    /// Chase tail (spin rapidly)
    ChaseTail,
    /// Stay in place (refuse to move)
    Stay,
    /// Move toward stimulus
    Approach,
    /// Move away from stimulus
    Retreat,
}

/// Sound types for quirk behaviors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum SoundType {
    Sigh,
    Chirp,
    Beep,
    Silence,
}

/// Light patterns for quirk behaviors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum LightPattern {
    Pulse,
    Flash,
    Dim,
    Bright,
}

/// Configuration for a specific quirk instance
#[derive(Debug, Clone)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct QuirkConfig {
    /// The quirk this config is for
    pub quirk: Quirk,
    /// When this quirk should trigger
    pub trigger: QuirkTrigger,
    /// Probability of activation when trigger condition is met (0.0-1.0)
    pub activation_chance: f32,
    /// Cooldown period in milliseconds
    pub cooldown_ms: u64,
    /// What behavior to execute
    pub behavior: QuirkBehavior,
}

impl QuirkConfig {
    /// Creates a default configuration for a quirk
    pub fn default_for(quirk: Quirk) -> Self {
        let trigger = match quirk {
            Quirk::RandomSigh | Quirk::ChaseTail => QuirkTrigger::Idle {
                duration_ms: if matches!(quirk, Quirk::RandomSigh) {
                    10_000
                } else {
                    30_000
                },
            },
            Quirk::SpinWhenHappy => QuirkTrigger::StateThreshold {
                state_key: StateKey::Coherence,
                threshold: 0.8,
                above: true,
            },
            Quirk::CollectorInstinct => QuirkTrigger::Environment {
                condition: EnvironmentCondition::NearObject,
            },
            Quirk::BackUpWhenScared => QuirkTrigger::Stimulus { min_intensity: 0.5 },
            Quirk::NightOwl => QuirkTrigger::Environment {
                condition: EnvironmentCondition::LowLight,
            },
            Quirk::EarlyBird => QuirkTrigger::Environment {
                condition: EnvironmentCondition::BrightLight,
            },
            Quirk::SocialButterfly => QuirkTrigger::Environment {
                condition: EnvironmentCondition::MovementDetected,
            },
            Quirk::Hermit => QuirkTrigger::Environment {
                condition: EnvironmentCondition::MovementDetected,
            },
        };

        let behavior = match quirk {
            Quirk::RandomSigh => QuirkBehavior::Sound {
                sound: SoundType::Sigh,
            },
            Quirk::SpinWhenHappy => QuirkBehavior::Movement {
                pattern: MovementPattern::Spin,
            },
            Quirk::BackUpWhenScared => QuirkBehavior::Movement {
                pattern: MovementPattern::Backup,
            },
            Quirk::ChaseTail => QuirkBehavior::Movement {
                pattern: MovementPattern::ChaseTail,
            },
            Quirk::CollectorInstinct => QuirkBehavior::Movement {
                pattern: MovementPattern::Stay,
            },
            Quirk::NightOwl => QuirkBehavior::ParameterModifier {
                parameter: String::from("energy_baseline"),
                modifier: 0.2,
            },
            Quirk::EarlyBird => QuirkBehavior::ParameterModifier {
                parameter: String::from("energy_baseline"),
                modifier: 0.2,
            },
            Quirk::SocialButterfly => QuirkBehavior::Movement {
                pattern: MovementPattern::Approach,
            },
            Quirk::Hermit => QuirkBehavior::Movement {
                pattern: MovementPattern::Retreat,
            },
        };

        Self {
            quirk,
            trigger,
            activation_chance: quirk.default_activation_chance(),
            cooldown_ms: quirk.default_cooldown_ms(),
            behavior,
        }
    }
}

/// Manages active quirks and their cooldown states
#[derive(Debug, Clone)]
pub struct QuirkEngine {
    /// Currently active quirks
    active_quirks: Vec<QuirkConfig>,
    /// Cooldown tracking (quirk -> last activation time in ms)
    #[cfg(feature = "std")]
    cooldowns: Vec<(Quirk, u64)>,
    #[cfg(not(feature = "std"))]
    cooldowns: Vec<(Quirk, u64)>,
}

impl QuirkEngine {
    /// Creates a new quirk engine with no active quirks
    pub fn new() -> Self {
        Self {
            active_quirks: Vec::new(),
            #[cfg(feature = "std")]
            cooldowns: Vec::new(),
            #[cfg(not(feature = "std"))]
            cooldowns: Vec::new(),
        }
    }

    /// Adds a quirk to the engine (replaces existing config if duplicate)
    pub fn add_quirk(&mut self, config: QuirkConfig) {
        // Remove existing config for this quirk
        self.active_quirks.retain(|q| q.quirk != config.quirk);
        // Add new config
        self.active_quirks.push(config);
    }

    /// Adds a quirk by string name with default configuration
    pub fn add_quirk_from_str(&mut self, quirk_str: &str) {
        if let Some(quirk) = Quirk::from_str(quirk_str) {
            self.add_quirk(QuirkConfig::default_for(quirk));
        }
    }

    /// Removes a quirk from the engine
    pub fn remove_quirk(&mut self, quirk: Quirk) {
        self.active_quirks.retain(|q| q.quirk != quirk);
        self.cooldowns.retain(|(q, _)| *q != quirk);
    }

    /// Clears all active quirks
    pub fn clear(&mut self) {
        self.active_quirks.clear();
        self.cooldowns.clear();
    }

    /// Returns the list of active quirks
    pub fn active_quirks(&self) -> &[QuirkConfig] {
        &self.active_quirks
    }

    /// Checks if a quirk is currently on cooldown
    pub fn is_on_cooldown(&self, quirk: Quirk, current_time_ms: u64) -> bool {
        for (q, last_activation) in &self.cooldowns {
            if *q == quirk {
                let elapsed = current_time_ms.saturating_sub(*last_activation);
                let cooldown = quirk.default_cooldown_ms();
                return elapsed < cooldown;
            }
        }
        false
    }

    /// Records that a quirk has been activated
    pub fn record_activation(&mut self, quirk: Quirk, current_time_ms: u64) {
        // Remove existing entry
        self.cooldowns.retain(|(q, _)| *q != quirk);
        // Add new entry
        self.cooldowns.push((quirk, current_time_ms));
    }

    /// Checks which quirks should trigger based on current state
    ///
    /// This does NOT check activation chance or cooldowns - it only checks
    /// if the trigger conditions are met.
    pub fn check_triggers(&self, state: &QuirkCheckState) -> Vec<Quirk> {
        let mut triggered = Vec::new();

        for config in &self.active_quirks {
            let should_trigger = match &config.trigger {
                QuirkTrigger::Idle { duration_ms } => {
                    state.idle_duration_ms >= *duration_ms
                }
                QuirkTrigger::StateThreshold {
                    state_key,
                    threshold,
                    above,
                } => {
                    let value = match state_key {
                        StateKey::Tension => state.tension,
                        StateKey::Coherence => state.coherence,
                        StateKey::Energy => state.energy,
                    };
                    if *above {
                        value >= *threshold
                    } else {
                        value <= *threshold
                    }
                }
                QuirkTrigger::Stimulus { min_intensity } => {
                    state.stimulus_intensity >= *min_intensity
                }
                QuirkTrigger::Environment { condition } => match condition {
                    EnvironmentCondition::LowLight => state.light_level < 0.3,
                    EnvironmentCondition::BrightLight => state.light_level > 0.7,
                    EnvironmentCondition::MovementDetected => state.movement_detected,
                    EnvironmentCondition::SoundDetected => state.sound_detected,
                    EnvironmentCondition::NearObject => state.near_object,
                },
            };

            if should_trigger {
                triggered.push(config.quirk);
            }
        }

        triggered
    }

    /// Determines which quirks should actually activate
    ///
    /// This checks triggers, activation chances, and cooldowns.
    /// Returns list of quirks that should execute their behaviors.
    pub fn evaluate(
        &mut self,
        state: &QuirkCheckState,
        current_time_ms: u64,
        rng: &mut dyn FnMut() -> f32,
    ) -> Vec<&QuirkConfig> {
        let triggered = self.check_triggers(state);
        let mut quirks_to_activate = Vec::new();

        for quirk in triggered {
            // Check cooldown
            if self.is_on_cooldown(quirk, current_time_ms) {
                continue;
            }

            // Find config and check activation chance
            let should_activate = self.active_quirks
                .iter()
                .find(|c| c.quirk == quirk)
                .map(|config| {
                    let roll = rng();
                    roll < config.activation_chance
                })
                .unwrap_or(false);

            if should_activate {
                // Record activation
                self.record_activation(quirk, current_time_ms);
                quirks_to_activate.push(quirk);
            }
        }

        // Return references to configs
        let mut to_activate = Vec::new();
        for quirk in quirks_to_activate {
            if let Some(config) = self.active_quirks.iter().find(|c| c.quirk == quirk) {
                to_activate.push(config);
            }
        }

        to_activate
    }
}

impl Default for QuirkEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// State snapshot for checking quirk triggers
#[derive(Debug, Clone)]
pub struct QuirkCheckState {
    /// Current tension value (0.0-1.0)
    pub tension: f32,
    /// Current coherence value (0.0-1.0)
    pub coherence: f32,
    /// Current energy value (0.0-1.0)
    pub energy: f32,
    /// How long robot has been idle (ms)
    pub idle_duration_ms: u64,
    /// Intensity of recent stimulus (0.0-1.0)
    pub stimulus_intensity: f32,
    /// Current light level (0.0-1.0)
    pub light_level: f32,
    /// Movement detected in environment
    pub movement_detected: bool,
    /// Sound detected in environment
    pub sound_detected: bool,
    /// Robot is near an interesting object
    pub near_object: bool,
}

impl Default for QuirkCheckState {
    fn default() -> Self {
        Self {
            tension: 0.5,
            coherence: 0.5,
            energy: 0.5,
            idle_duration_ms: 0,
            stimulus_intensity: 0.0,
            light_level: 0.5,
            movement_detected: false,
            sound_detected: false,
            near_object: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quirk_all_returns_9_quirks() {
        assert_eq!(Quirk::all().len(), 9);
    }

    #[test]
    fn test_quirk_to_str_and_from_str_roundtrip() {
        for quirk in Quirk::all() {
            let s = quirk.to_str();
            let parsed = Quirk::from_str(s);
            assert_eq!(parsed, Some(*quirk));
        }
    }

    #[test]
    fn test_quirk_from_str_invalid() {
        assert_eq!(Quirk::from_str("invalid_quirk"), None);
    }

    #[test]
    fn test_quirk_descriptions_non_empty() {
        for quirk in Quirk::all() {
            let desc = quirk.description();
            assert!(!desc.is_empty());
        }
    }

    #[test]
    fn test_default_config_for_all_quirks() {
        for quirk in Quirk::all() {
            let config = QuirkConfig::default_for(*quirk);
            assert_eq!(config.quirk, *quirk);
            assert!(config.activation_chance >= 0.0);
            assert!(config.activation_chance <= 1.0);
        }
    }

    #[test]
    fn test_safety_quirks_have_no_cooldown() {
        let quirk = Quirk::BackUpWhenScared;
        assert_eq!(quirk.default_cooldown_ms(), 0);
    }

    #[test]
    fn test_continuous_quirks_have_no_cooldown() {
        assert_eq!(Quirk::NightOwl.default_cooldown_ms(), 0);
        assert_eq!(Quirk::EarlyBird.default_cooldown_ms(), 0);
        assert_eq!(Quirk::SocialButterfly.default_cooldown_ms(), 0);
        assert_eq!(Quirk::Hermit.default_cooldown_ms(), 0);
    }

    #[test]
    fn test_quirk_engine_new() {
        let engine = QuirkEngine::new();
        assert_eq!(engine.active_quirks().len(), 0);
    }

    #[test]
    fn test_add_quirk() {
        let mut engine = QuirkEngine::new();
        engine.add_quirk(QuirkConfig::default_for(Quirk::RandomSigh));
        assert_eq!(engine.active_quirks().len(), 1);
    }

    #[test]
    fn test_add_duplicate_quirk_replaces() {
        let mut engine = QuirkEngine::new();
        engine.add_quirk(QuirkConfig::default_for(Quirk::RandomSigh));
        engine.add_quirk(QuirkConfig::default_for(Quirk::RandomSigh));
        assert_eq!(engine.active_quirks().len(), 1);
    }

    #[test]
    fn test_remove_quirk() {
        let mut engine = QuirkEngine::new();
        engine.add_quirk(QuirkConfig::default_for(Quirk::RandomSigh));
        engine.remove_quirk(Quirk::RandomSigh);
        assert_eq!(engine.active_quirks().len(), 0);
    }

    #[test]
    fn test_clear_quirks() {
        let mut engine = QuirkEngine::new();
        engine.add_quirk(QuirkConfig::default_for(Quirk::RandomSigh));
        engine.add_quirk(QuirkConfig::default_for(Quirk::SpinWhenHappy));
        engine.clear();
        assert_eq!(engine.active_quirks().len(), 0);
    }

    #[test]
    fn test_cooldown_tracking() {
        let mut engine = QuirkEngine::new();
        let quirk = Quirk::RandomSigh;

        assert!(!engine.is_on_cooldown(quirk, 1000));
        engine.record_activation(quirk, 1000);
        assert!(engine.is_on_cooldown(quirk, 1001));
        assert!(engine.is_on_cooldown(quirk, 10_000));

        let cooldown = quirk.default_cooldown_ms();
        assert!(!engine.is_on_cooldown(quirk, 1000 + cooldown + 1));
    }

    #[test]
    fn test_check_triggers_idle() {
        let mut engine = QuirkEngine::new();
        engine.add_quirk(QuirkConfig::default_for(Quirk::RandomSigh));

        let mut state = QuirkCheckState::default();
        state.idle_duration_ms = 5000;

        let triggered = engine.check_triggers(&state);
        assert_eq!(triggered.len(), 0); // Not idle long enough

        state.idle_duration_ms = 10_000;
        let triggered = engine.check_triggers(&state);
        assert_eq!(triggered.len(), 1);
        assert_eq!(triggered[0], Quirk::RandomSigh);
    }

    #[test]
    fn test_check_triggers_state_threshold() {
        let mut engine = QuirkEngine::new();
        engine.add_quirk(QuirkConfig::default_for(Quirk::SpinWhenHappy));

        let mut state = QuirkCheckState::default();
        state.coherence = 0.7;

        let triggered = engine.check_triggers(&state);
        assert_eq!(triggered.len(), 0); // Below threshold

        state.coherence = 0.85;
        let triggered = engine.check_triggers(&state);
        assert_eq!(triggered.len(), 1);
        assert_eq!(triggered[0], Quirk::SpinWhenHappy);
    }

    #[test]
    fn test_check_triggers_environment() {
        let mut engine = QuirkEngine::new();
        engine.add_quirk(QuirkConfig::default_for(Quirk::NightOwl));

        let mut state = QuirkCheckState::default();
        state.light_level = 0.5;

        let triggered = engine.check_triggers(&state);
        assert_eq!(triggered.len(), 0); // Not dark enough

        state.light_level = 0.2;
        let triggered = engine.check_triggers(&state);
        assert_eq!(triggered.len(), 1);
        assert_eq!(triggered[0], Quirk::NightOwl);
    }

    #[test]
    fn test_evaluate_respects_activation_chance() {
        let mut engine = QuirkEngine::new();
        let mut config = QuirkConfig::default_for(Quirk::RandomSigh);
        config.activation_chance = 0.0; // Never activate
        engine.add_quirk(config);

        let mut state = QuirkCheckState::default();
        state.idle_duration_ms = 10_000;

        let mut always_one = || 1.0f32;
        let activated = engine.evaluate(&state, 1000, &mut always_one);
        assert_eq!(activated.len(), 0);
    }

    #[test]
    fn test_evaluate_respects_cooldown() {
        let mut engine = QuirkEngine::new();
        let mut config = QuirkConfig::default_for(Quirk::RandomSigh);
        config.activation_chance = 1.0; // Always activate
        engine.add_quirk(config);

        let mut state = QuirkCheckState::default();
        state.idle_duration_ms = 10_000;

        let mut always_zero = || 0.0f32;
        let activated = engine.evaluate(&state, 1000, &mut always_zero);
        assert_eq!(activated.len(), 1);

        // Try again immediately - should be on cooldown
        let activated = engine.evaluate(&state, 1001, &mut always_zero);
        assert_eq!(activated.len(), 0);
    }

    #[test]
    fn test_add_quirk_from_str() {
        let mut engine = QuirkEngine::new();
        engine.add_quirk_from_str("random_sigh");
        engine.add_quirk_from_str("spin_when_happy");
        assert_eq!(engine.active_quirks().len(), 2);

        // Invalid quirk should be ignored
        engine.add_quirk_from_str("invalid_quirk");
        assert_eq!(engine.active_quirks().len(), 2);
    }
}
