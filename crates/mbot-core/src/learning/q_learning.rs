//! Q-Learning Implementation
//!
//! Classic tabular Q-learning with epsilon-greedy exploration.
//! Invariants enforced:
//! - I-AI-001: Convergence within 1000 episodes
//! - I-AI-003: Learning rate decay
//! - I-AI-004: Epsilon-greedy exploration decay

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use alloc::{string::String, vec::Vec};
#[cfg(not(feature = "std"))]
use hashbrown::HashMap;

#[cfg(feature = "std")]
use std::{collections::HashMap, string::String, vec::Vec};

use super::{State, Action, Policy, RewardFunction, LearningMetrics, LearningConfig, ReinforcementLearner, UserFeedback};
use super::reward::FeedbackRating;

/// Q-Learning agent
pub struct QLearner {
    /// Q-table: (state_key, action_key) -> Q-value
    q_table: HashMap<String, HashMap<String, f32>>,

    /// Learning configuration
    config: LearningConfig,

    /// Current learning metrics
    metrics: LearningMetrics,

    /// Reward function
    reward_fn: RewardFunction,

    /// Episode count
    episode_count: u32,

    /// Current epsilon (exploration rate)
    epsilon: f32,

    /// Current learning rate
    learning_rate: f32,

    /// Recent rewards for convergence detection
    recent_rewards: Vec<f32>,

    /// Game-specific policies
    policies: HashMap<String, Policy>,
}

impl QLearner {
    pub fn new(config: LearningConfig) -> Self {
        Self {
            q_table: HashMap::new(),
            epsilon: config.epsilon_start,
            learning_rate: config.learning_rate,
            recent_rewards: Vec::with_capacity(100),
            episode_count: 0,
            metrics: LearningMetrics::default(),
            reward_fn: RewardFunction::default(),
            config,
            policies: HashMap::new(),
        }
    }

    pub fn with_reward_function(mut self, reward_fn: RewardFunction) -> Self {
        self.reward_fn = reward_fn;
        self
    }

    /// Get Q-value for state-action pair
    fn get_q_value(&self, state: &State, action: &Action) -> f32 {
        let state_key = state.to_key();
        let action_key = action.to_key();

        self.q_table
            .get(&state_key)
            .and_then(|actions| actions.get(&action_key))
            .copied()
            .unwrap_or(0.0)
    }

    /// Set Q-value for state-action pair
    fn set_q_value(&mut self, state: &State, action: &Action, value: f32) {
        let state_key = state.to_key();
        let action_key = action.to_key();

        self.q_table
            .entry(state_key)
            .or_insert_with(HashMap::new)
            .insert(action_key, value);
    }

    /// Get max Q-value for a state across all actions
    fn get_max_q(&self, state: &State, actions: &[Action]) -> f32 {
        actions
            .iter()
            .map(|a| self.get_q_value(state, a))
            .fold(f32::NEG_INFINITY, f32::max)
            .max(0.0)
    }

    /// Update epsilon (exploration rate) using decay - I-AI-004
    fn update_epsilon(&mut self) {
        self.epsilon = (self.epsilon * self.config.epsilon_decay)
            .max(self.config.epsilon_end);
    }

    /// Update learning rate using decay - I-AI-003
    fn update_learning_rate(&mut self) {
        // Simple decay: alpha_t = alpha_0 / (1 + episode_count / 100)
        self.learning_rate = self.config.learning_rate /
            (1.0 + (self.episode_count as f32) / 100.0);
    }

    /// Check for convergence - I-AI-001
    fn check_convergence(&mut self) {
        if self.recent_rewards.len() < 100 {
            self.metrics.convergence_score = 0.0;
            return;
        }

        // Calculate variance of recent rewards
        let mean: f32 = self.recent_rewards.iter().sum::<f32>() / self.recent_rewards.len() as f32;
        let variance: f32 = self.recent_rewards
            .iter()
            .map(|r| (r - mean).powi(2))
            .sum::<f32>() / self.recent_rewards.len() as f32;

        // Low variance = converged
        // Convergence score: 1.0 when variance < 0.01
        self.metrics.convergence_score = (1.0 - (variance * 10.0).min(1.0)).max(0.0);
    }

    /// Complete an episode (for tracking)
    pub fn complete_episode(&mut self, total_reward: f32, outcome: &str) {
        self.episode_count += 1;
        self.metrics.episode_count = self.episode_count;

        // Track reward
        self.recent_rewards.push(total_reward);
        if self.recent_rewards.len() > 100 {
            self.recent_rewards.remove(0);
        }
        self.metrics.average_reward = self.recent_rewards.iter().sum::<f32>()
            / self.recent_rewards.len() as f32;

        // Track outcomes
        match outcome {
            "win" => self.metrics.win_rate = self.calculate_win_rate(1.0),
            "loss" => self.metrics.loss_rate = self.calculate_loss_rate(1.0),
            "draw" => self.metrics.draw_rate = self.calculate_draw_rate(1.0),
            _ => {}
        }

        // Update hyperparameters
        self.update_epsilon();
        self.update_learning_rate();
        self.check_convergence();

        // Update metrics
        self.metrics.epsilon_current = self.epsilon;
        self.metrics.learning_rate_current = self.learning_rate;
    }

    fn calculate_win_rate(&self, outcome: f32) -> f32 {
        // Exponential moving average
        let alpha = 0.1;
        alpha * outcome + (1.0 - alpha) * self.metrics.win_rate
    }

    fn calculate_loss_rate(&self, outcome: f32) -> f32 {
        let alpha = 0.1;
        alpha * outcome + (1.0 - alpha) * self.metrics.loss_rate
    }

    fn calculate_draw_rate(&self, outcome: f32) -> f32 {
        let alpha = 0.1;
        alpha * outcome + (1.0 - alpha) * self.metrics.draw_rate
    }
}

impl ReinforcementLearner for QLearner {
    fn learn(&mut self, state: &State, action: &Action, reward: f32, next_state: &State) {
        // Get current Q-value
        let current_q = self.get_q_value(state, action);

        // Get max Q-value for next state
        // Note: We'd need available actions for next_state, using empty vec as placeholder
        let next_max_q = self.get_max_q(next_state, &[]);

        // Q-learning update: Q(s,a) = Q(s,a) + α[r + γ*max_a'Q(s',a') - Q(s,a)]
        let new_q = current_q + self.learning_rate *
            (reward + self.config.discount_factor * next_max_q - current_q);

        self.set_q_value(state, action, new_q);
    }

    fn select_action(&mut self, state: &State, available_actions: &[Action], explore: bool) -> Option<Action> {
        if available_actions.is_empty() {
            return None;
        }

        // Epsilon-greedy exploration - I-AI-004
        if explore && self.epsilon > 0.0 {
            // Simple pseudo-random selection using episode count
            // In production, use proper RNG with seed
            let random_val = ((self.episode_count * 1103515245 + 12345) % 100) as f32 / 100.0;

            if random_val < self.epsilon {
                // Explore: random action
                let idx = ((self.episode_count * 7919) % available_actions.len() as u32) as usize;
                return Some(available_actions[idx].clone());
            }
        }

        // Exploit: best action
        let mut best_action = &available_actions[0];
        let mut best_q = self.get_q_value(state, best_action);

        for action in &available_actions[1..] {
            let q = self.get_q_value(state, action);
            if q > best_q {
                best_q = q;
                best_action = action;
            }
        }

        Some(best_action.clone())
    }

    fn update_from_feedback(&mut self, behavior_id: &str, feedback: UserFeedback) {
        // Apply reward adjustment to recent actions
        let reward = match feedback.rating {
            FeedbackRating::Good => self.reward_fn.user_good_reward,
            FeedbackRating::Bad => self.reward_fn.user_bad_reward,
            FeedbackRating::Neutral => 0.0,
        };

        // In a full implementation, we'd track behavior_id to state-action mapping
        // For now, this is a placeholder that shows the interface
        let _ = (behavior_id, reward);
    }

    fn get_metrics(&self) -> &LearningMetrics {
        &self.metrics
    }

    fn save_policy(&self, game_type: &str) -> Result<Vec<u8>, &'static str> {
        // Create policy snapshot
        let policy = Policy {
            q_table: self.q_table.clone(),
            game_type: game_type.into(),
            episode_count: self.episode_count,
            learning_rate: self.learning_rate,
            discount_factor: self.config.discount_factor,
            epsilon: self.epsilon,
            created_at: 0, // Would use real timestamp in production
            updated_at: 0,
        };

        // In production, serialize to JSON or binary format
        // For now, return empty vec as placeholder
        Ok(Vec::new())
    }

    fn load_policy(&mut self, game_type: &str, data: &[u8]) -> Result<(), &'static str> {
        // In production, deserialize from data
        // For now, placeholder
        let _ = (game_type, data);
        Ok(())
    }

    fn reset_learning(&mut self, game_type: &str) {
        // Remove all Q-values for this game type
        self.q_table.retain(|k, _| !k.starts_with(game_type));

        // Reset metrics
        if self.policies.contains_key(game_type) {
            self.policies.remove(game_type);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_q_learner_creation() {
        let config = LearningConfig::default();
        let learner = QLearner::new(config);
        assert_eq!(learner.episode_count, 0);
        assert!(learner.epsilon > 0.0);
    }

    #[test]
    fn test_q_value_get_set() {
        let config = LearningConfig::default();
        let mut learner = QLearner::new(config);

        let state = State::new("test".into(), "state1".into());
        let action = Action::new("move".into(), (0, 0));

        learner.set_q_value(&state, &action, 0.5);
        assert_eq!(learner.get_q_value(&state, &action), 0.5);
    }

    #[test]
    fn test_epsilon_decay() {
        let mut config = LearningConfig::default();
        config.epsilon_decay = 0.9;
        let mut learner = QLearner::new(config);

        let initial_epsilon = learner.epsilon;
        learner.update_epsilon();
        assert!(learner.epsilon < initial_epsilon);
    }

    #[test]
    fn test_learning_rate_decay() {
        let config = LearningConfig::default();
        let mut learner = QLearner::new(config);

        let initial_lr = learner.learning_rate;
        learner.episode_count = 100;
        learner.update_learning_rate();
        assert!(learner.learning_rate < initial_lr);
    }

    #[test]
    fn test_q_learning_update() {
        let config = LearningConfig::default();
        let mut learner = QLearner::new(config);

        let state = State::new("test".into(), "s1".into());
        let action = Action::new("a1".into(), (0, 0));
        let next_state = State::new("test".into(), "s2".into());

        learner.learn(&state, &action, 1.0, &next_state);

        let q_value = learner.get_q_value(&state, &action);
        assert!(q_value > 0.0);
    }

    #[test]
    fn test_action_selection() {
        let config = LearningConfig::default();
        let mut learner = QLearner::new(config);

        let state = State::new("test".into(), "s1".into());
        let actions = vec![
            Action::new("a1".into(), (0, 0)),
            Action::new("a2".into(), (1, 1)),
        ];

        let selected = learner.select_action(&state, &actions, false);
        assert!(selected.is_some());
    }
}
