//! Reinforcement Learning System
//!
//! Q-learning implementation for mBot to learn from game outcomes and user feedback.
//! Supports multiple games with separate Q-tables and policy persistence.

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use alloc::{string::String, vec::Vec};
#[cfg(not(feature = "std"))]
use hashbrown::HashMap;

#[cfg(feature = "std")]
use std::{collections::HashMap, string::String, vec::Vec};

mod q_learning;
mod policy;
mod reward;
mod metrics;
pub mod prediction;  // Issue #87: Predictive Behavior Engine

pub use q_learning::QLearner;
pub use policy::{Policy, PolicyStorage};
pub use reward::{RewardFunction, UserFeedback, FeedbackRating};
pub use metrics::{LearningMetrics, LearningConfig};

// Re-export prediction types for convenience
pub use prediction::{
    PredictiveEngine,
    UserActivity,
    ActivityType,
    Context as PredictionContext,
    Pattern,
    PatternType,
    Prediction,
    PredictionSettings,
    Action as PredictedAction,
};

/// State representation for learning
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct State {
    /// Game type (e.g., "tictactoe", "connect-four")
    pub game_type: String,
    /// Board state serialized as string
    pub board_state: String,
    /// Extracted features for learning
    pub features: Vec<i32>,
}

impl State {
    pub fn new(game_type: String, board_state: String) -> Self {
        Self {
            game_type,
            board_state,
            features: Vec::new(),
        }
    }

    pub fn with_features(mut self, features: Vec<i32>) -> Self {
        self.features = features;
        self
    }

    /// Serialize state to string for use as Q-table key
    pub fn to_key(&self) -> String {
        #[cfg(not(feature = "std"))]
        {
            use alloc::format;
            format!("{}:{}", self.board_state, self.features.len())
        }
        #[cfg(feature = "std")]
        {
            format!("{}:{:?}", self.board_state, self.features)
        }
    }
}

/// Action representation
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Action {
    /// Action type (e.g., "place_marker", "move")
    pub action_type: String,
    /// Position or move identifier
    pub position: (i32, i32),
    /// Additional parameters
    pub params: Vec<i32>,
}

impl Action {
    pub fn new(action_type: String, position: (i32, i32)) -> Self {
        Self {
            action_type,
            position,
            params: Vec::new(),
        }
    }

    pub fn with_params(mut self, params: Vec<i32>) -> Self {
        self.params = params;
        self
    }

    /// Serialize action to string for use as Q-table key
    pub fn to_key(&self) -> String {
        #[cfg(not(feature = "std"))]
        {
            use alloc::format;
            format!("{}_{}_{}", self.action_type, self.position.0, self.position.1)
        }
        #[cfg(feature = "std")]
        {
            format!("{}_({},{})", self.action_type, self.position.0, self.position.1)
        }
    }
}

/// Main reinforcement learning interface
pub trait ReinforcementLearner {
    /// Learn from a state transition
    fn learn(&mut self, state: &State, action: &Action, reward: f32, next_state: &State);

    /// Select best action for current state (with optional exploration)
    fn select_action(&mut self, state: &State, available_actions: &[Action], explore: bool) -> Option<Action>;

    /// Update policy based on user feedback
    fn update_from_feedback(&mut self, behavior_id: &str, feedback: UserFeedback);

    /// Get current learning metrics
    fn get_metrics(&self) -> &LearningMetrics;

    /// Save policy to storage
    fn save_policy(&self, game_type: &str) -> Result<Vec<u8>, &'static str>;

    /// Load policy from storage
    fn load_policy(&mut self, game_type: &str, data: &[u8]) -> Result<(), &'static str>;

    /// Reset learning for a specific game
    fn reset_learning(&mut self, game_type: &str);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_creation() {
        let state = State::new("tictactoe".into(), "X__O__X__".into());
        assert_eq!(state.game_type, "tictactoe");
        assert_eq!(state.board_state, "X__O__X__");
    }

    #[test]
    fn test_action_creation() {
        let action = Action::new("place_marker".into(), (1, 1));
        assert_eq!(action.action_type, "place_marker");
        assert_eq!(action.position, (1, 1));
    }

    #[test]
    fn test_state_key_generation() {
        let state = State::new("test".into(), "board".into())
            .with_features(vec![1, 2, 3]);
        let key = state.to_key();
        assert!(!key.is_empty());
    }

    #[test]
    fn test_action_key_generation() {
        let action = Action::new("move".into(), (2, 3));
        let key = action.to_key();
        assert!(!key.is_empty());
    }
}
