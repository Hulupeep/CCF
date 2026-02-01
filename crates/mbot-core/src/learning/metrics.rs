//! Learning Metrics and Configuration
//!
//! Tracks learning progress and hyperparameters.

#![cfg_attr(feature = "no_std", no_std)]

/// Learning configuration parameters
#[derive(Clone, Debug)]
pub struct LearningConfig {
    /// Learning rate (alpha) - how much to update Q-values
    pub learning_rate: f32,

    /// Discount factor (gamma) - importance of future rewards
    pub discount_factor: f32,

    /// Initial exploration rate
    pub epsilon_start: f32,

    /// Final exploration rate
    pub epsilon_end: f32,

    /// Exploration decay rate per episode
    pub epsilon_decay: f32,

    /// Maximum training episodes
    pub max_episodes: u32,
}

impl Default for LearningConfig {
    fn default() -> Self {
        Self {
            learning_rate: 0.1,
            discount_factor: 0.9,
            epsilon_start: 1.0,
            epsilon_end: 0.1,
            epsilon_decay: 0.995,
            max_episodes: 1000,
        }
    }
}

impl LearningConfig {
    /// Configuration optimized for fast convergence
    pub fn fast_convergence() -> Self {
        Self {
            learning_rate: 0.2,
            discount_factor: 0.95,
            epsilon_start: 0.8,
            epsilon_end: 0.05,
            epsilon_decay: 0.99,
            max_episodes: 500,
        }
    }

    /// Configuration for careful exploration
    pub fn careful_exploration() -> Self {
        Self {
            learning_rate: 0.05,
            discount_factor: 0.9,
            epsilon_start: 1.0,
            epsilon_end: 0.2,
            epsilon_decay: 0.998,
            max_episodes: 2000,
        }
    }
}

/// Learning metrics for monitoring progress
#[derive(Clone, Debug)]
pub struct LearningMetrics {
    /// Total episodes completed
    pub episode_count: u32,

    /// Average reward over recent episodes
    pub average_reward: f32,

    /// Win rate (last 100 games)
    pub win_rate: f32,

    /// Loss rate (last 100 games)
    pub loss_rate: f32,

    /// Draw rate (last 100 games)
    pub draw_rate: f32,

    /// Current exploration rate
    pub epsilon_current: f32,

    /// Current learning rate
    pub learning_rate_current: f32,

    /// Convergence score (0-1, 1 = fully converged)
    pub convergence_score: f32,
}

impl Default for LearningMetrics {
    fn default() -> Self {
        Self {
            episode_count: 0,
            average_reward: 0.0,
            win_rate: 0.0,
            loss_rate: 0.0,
            draw_rate: 0.0,
            epsilon_current: 1.0,
            learning_rate_current: 0.1,
            convergence_score: 0.0,
        }
    }
}

impl LearningMetrics {
    /// Check if learning has converged
    pub fn is_converged(&self) -> bool {
        self.convergence_score > 0.9
    }

    /// Check if ready for deployment (good performance)
    pub fn is_ready(&self) -> bool {
        self.episode_count >= 100 && self.win_rate > 0.6
    }

    /// Get overall performance score (0-1)
    pub fn performance_score(&self) -> f32 {
        // Weight win rate more heavily than draw rate
        let outcome_score = self.win_rate * 1.0 + self.draw_rate * 0.3;
        // Combine with convergence
        (outcome_score + self.convergence_score) / 2.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = LearningConfig::default();
        assert_eq!(config.learning_rate, 0.1);
        assert_eq!(config.discount_factor, 0.9);
        assert_eq!(config.epsilon_start, 1.0);
    }

    #[test]
    fn test_fast_convergence_config() {
        let config = LearningConfig::fast_convergence();
        assert!(config.learning_rate > 0.1);
        assert!(config.max_episodes < 1000);
    }

    #[test]
    fn test_careful_exploration_config() {
        let config = LearningConfig::careful_exploration();
        assert!(config.learning_rate < 0.1);
        assert!(config.max_episodes > 1000);
    }

    #[test]
    fn test_metrics_default() {
        let metrics = LearningMetrics::default();
        assert_eq!(metrics.episode_count, 0);
        assert_eq!(metrics.win_rate, 0.0);
        assert!(!metrics.is_converged());
    }

    #[test]
    fn test_convergence_check() {
        let mut metrics = LearningMetrics::default();
        assert!(!metrics.is_converged());

        metrics.convergence_score = 0.95;
        assert!(metrics.is_converged());
    }

    #[test]
    fn test_ready_check() {
        let mut metrics = LearningMetrics::default();
        assert!(!metrics.is_ready());

        metrics.episode_count = 150;
        metrics.win_rate = 0.7;
        assert!(metrics.is_ready());
    }

    #[test]
    fn test_performance_score() {
        let mut metrics = LearningMetrics::default();
        metrics.win_rate = 0.8;
        metrics.convergence_score = 0.9;

        let score = metrics.performance_score();
        assert!(score > 0.5 && score <= 1.0);
    }
}
