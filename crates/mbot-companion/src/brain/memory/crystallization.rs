//! Foundry-Style Pattern Crystallization
//!
//! When the robot repeatedly uses the same navigation strategy successfully,
//! the pattern is "crystallized" into a permanent behavior that skips Q-learning.
//!
//! 1. Record every (state, action, outcome) triple
//! 2. After 5+ identical (state→action) with 70%+ success → candidate pattern
//! 3. Promote to permanent behavior (skip Q-learning lookup)
//! 4. Narrate: "I've figured something out!"

#[cfg(feature = "brain")]
use std::collections::HashMap;

/// A recorded state-action-outcome triple.
#[cfg(feature = "brain")]
#[derive(Clone, Debug)]
struct Trial {
    success: bool,
}

/// A crystallized pattern — a learned rule the robot uses automatically.
#[cfg(feature = "brain")]
#[derive(Clone, Debug)]
pub struct CrystalPattern {
    /// State key that triggers this pattern.
    pub state_key: String,
    /// Action key to execute.
    pub action_key: String,
    /// Success rate when this pattern was crystallized.
    pub success_rate: f32,
    /// How many trials led to crystallization.
    pub trial_count: u32,
    /// Human-readable description for narration.
    pub description: String,
    /// Tick when crystallized.
    pub crystallized_at: u64,
}

/// Crystallization engine that detects and promotes patterns.
#[cfg(feature = "brain")]
pub struct CrystallizationEngine {
    /// Recorded trials: (state_key, action_key) → list of outcomes.
    trials: HashMap<(String, String), Vec<Trial>>,
    /// Crystallized patterns: state_key → CrystalPattern.
    patterns: HashMap<String, CrystalPattern>,
    /// Minimum trials before considering crystallization.
    min_trials: usize,
    /// Minimum success rate for crystallization (0.0–1.0).
    min_success_rate: f32,
    /// Newly crystallized patterns awaiting narration.
    pending_narrations: Vec<String>,
}

#[cfg(feature = "brain")]
impl CrystallizationEngine {
    pub fn new() -> Self {
        Self {
            trials: HashMap::new(),
            patterns: HashMap::new(),
            min_trials: 5,
            min_success_rate: 0.70,
            pending_narrations: Vec::new(),
        }
    }

    /// Record a trial: the robot tried `action_key` in `state_key` and it was `success`.
    pub fn record_trial(
        &mut self,
        state_key: &str,
        action_key: &str,
        success: bool,
        current_tick: u64,
    ) {
        let key = (state_key.to_string(), action_key.to_string());
        let trials = self.trials.entry(key.clone()).or_insert_with(Vec::new);
        trials.push(Trial { success });

        // Check if this pair should be crystallized
        if trials.len() >= self.min_trials && !self.patterns.contains_key(state_key) {
            let successes = trials.iter().filter(|t| t.success).count();
            let rate = successes as f32 / trials.len() as f32;

            if rate >= self.min_success_rate {
                let description = Self::describe_pattern(&key.0, &key.1);
                let pattern = CrystalPattern {
                    state_key: key.0.clone(),
                    action_key: key.1.clone(),
                    success_rate: rate,
                    trial_count: trials.len() as u32,
                    description: description.clone(),
                    crystallized_at: current_tick,
                };
                self.patterns.insert(key.0.clone(), pattern);
                self.pending_narrations.push(description);
            }
        }
    }

    /// Check if there's a crystallized pattern for this state.
    /// Returns the action_key to use directly (bypassing Q-learning).
    pub fn get_crystallized_action(&self, state_key: &str) -> Option<&str> {
        self.patterns.get(state_key).map(|p| p.action_key.as_str())
    }

    /// Take pending narration texts (drain queue).
    pub fn take_narrations(&mut self) -> Vec<String> {
        std::mem::take(&mut self.pending_narrations)
    }

    /// Number of crystallized patterns.
    pub fn pattern_count(&self) -> usize {
        self.patterns.len()
    }

    /// Get all crystallized patterns for display.
    pub fn patterns(&self) -> impl Iterator<Item = &CrystalPattern> {
        self.patterns.values()
    }

    /// Generate a human-readable description of a discovered pattern.
    fn describe_pattern(state_key: &str, action_key: &str) -> String {
        // Parse the nav state format: "sec:N|obs:X|nrg:Y|rfx:Z"
        let action_name = match action_key {
            k if k.contains("turn_left") => "turn left",
            k if k.contains("turn_right") => "turn right",
            k if k.contains("forward") => "go forward",
            k if k.contains("backup") => "back up",
            k if k.contains("scan") => "scan around",
            _ => "act",
        };

        let obstacle = if state_key.contains("obs:near") || state_key.contains("obs:touch") {
            "an obstacle nearby"
        } else if state_key.contains("obs:far") {
            "something in the distance"
        } else {
            "open space"
        };

        format!(
            "I figured something out! When I sense {}, I should {}. It works every time!",
            obstacle, action_name
        )
    }
}

#[cfg(feature = "brain")]
impl Default for CrystallizationEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "brain")]
    use super::*;

    #[cfg(feature = "brain")]
    #[test]
    fn test_crystallization_after_threshold() {
        let mut engine = CrystallizationEngine::new();
        let state = "sec:3|obs:near|nrg:hi|rfx:active";
        let action = "nav_turn_left_30_(0,0)";

        // Record 5 successes
        for i in 0..5 {
            engine.record_trial(state, action, true, i as u64);
        }

        assert!(engine.get_crystallized_action(state).is_some());
        assert_eq!(engine.pattern_count(), 1);
        let narrations = engine.take_narrations();
        assert_eq!(narrations.len(), 1);
        assert!(narrations[0].contains("figured something out"));
    }

    #[cfg(feature = "brain")]
    #[test]
    fn test_no_crystallization_below_threshold() {
        let mut engine = CrystallizationEngine::new();
        let state = "sec:3|obs:near|nrg:hi|rfx:active";
        let action = "nav_forward_(0,0)";

        // Record 3 successes, 2 failures (60% < 70%)
        for _ in 0..3 {
            engine.record_trial(state, action, true, 0);
        }
        for _ in 0..2 {
            engine.record_trial(state, action, false, 0);
        }

        assert!(engine.get_crystallized_action(state).is_none());
        assert_eq!(engine.pattern_count(), 0);
    }
}
