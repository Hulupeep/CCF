//! Reward Functions
//!
//! Defines reward structures for different games and user feedback.

#![cfg_attr(not(feature = "std"), no_std)]

/// User feedback rating
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FeedbackRating {
    Good,
    Bad,
    Neutral,
}

/// User feedback on robot behavior
#[derive(Clone, Debug)]
pub struct UserFeedback {
    /// Unique behavior identifier
    pub behavior_id: u64,

    /// User rating
    pub rating: FeedbackRating,

    /// Timestamp (microseconds)
    pub timestamp: u64,
}

impl UserFeedback {
    pub fn new(behavior_id: u64, rating: FeedbackRating) -> Self {
        Self {
            behavior_id,
            rating,
            timestamp: 0, // Would use real timestamp in production
        }
    }
}

/// Reward function configuration for a game
#[derive(Clone, Debug)]
pub struct RewardFunction {
    /// Game type
    pub game_type: &'static str,

    /// Reward for winning
    pub win_reward: f32,

    /// Reward for losing
    pub loss_reward: f32,

    /// Reward for draw
    pub draw_reward: f32,

    /// Reward for good move (intermediate)
    pub good_move_reward: f32,

    /// Penalty for bad move
    pub bad_move_reward: f32,

    /// Reward for positive user feedback
    pub user_good_reward: f32,

    /// Penalty for negative user feedback
    pub user_bad_reward: f32,
}

impl Default for RewardFunction {
    fn default() -> Self {
        Self {
            game_type: "default",
            win_reward: 1.0,
            loss_reward: -1.0,
            draw_reward: 0.0,
            good_move_reward: 0.1,
            bad_move_reward: -0.1,
            user_good_reward: 0.5,
            user_bad_reward: -0.5,
        }
    }
}

impl RewardFunction {
    /// Reward function for Tic-Tac-Toe
    pub fn tictactoe() -> Self {
        Self {
            game_type: "tictactoe",
            win_reward: 1.0,
            loss_reward: -1.0,
            draw_reward: 0.0,
            good_move_reward: 0.1,  // Block opponent or create threat
            bad_move_reward: -0.2,  // Miss obvious win/block
            user_good_reward: 0.5,
            user_bad_reward: -0.5,
        }
    }

    /// Reward function for Connect Four
    pub fn connect_four() -> Self {
        Self {
            game_type: "connect-four",
            win_reward: 1.0,
            loss_reward: -1.0,
            draw_reward: 0.0,
            good_move_reward: 0.15,  // Build toward winning line
            bad_move_reward: -0.15,  // Let opponent win
            user_good_reward: 0.5,
            user_bad_reward: -0.5,
        }
    }

    /// Calculate reward for game outcome
    pub fn outcome_reward(&self, outcome: &str) -> f32 {
        match outcome {
            "win" => self.win_reward,
            "loss" => self.loss_reward,
            "draw" => self.draw_reward,
            _ => 0.0,
        }
    }

    /// Calculate reward for move quality
    pub fn move_reward(&self, is_good: bool) -> f32 {
        if is_good {
            self.good_move_reward
        } else {
            self.bad_move_reward
        }
    }

    /// Calculate reward from user feedback
    pub fn feedback_reward(&self, feedback: &UserFeedback) -> f32 {
        match feedback.rating {
            FeedbackRating::Good => self.user_good_reward,
            FeedbackRating::Bad => self.user_bad_reward,
            FeedbackRating::Neutral => 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_rewards() {
        let rf = RewardFunction::default();
        assert_eq!(rf.win_reward, 1.0);
        assert_eq!(rf.loss_reward, -1.0);
        assert_eq!(rf.draw_reward, 0.0);
    }

    #[test]
    fn test_tictactoe_rewards() {
        let rf = RewardFunction::tictactoe();
        assert_eq!(rf.game_type, "tictactoe");
        assert_eq!(rf.outcome_reward("win"), 1.0);
        assert_eq!(rf.outcome_reward("loss"), -1.0);
    }

    #[test]
    fn test_connect_four_rewards() {
        let rf = RewardFunction::connect_four();
        assert_eq!(rf.game_type, "connect-four");
    }

    #[test]
    fn test_move_rewards() {
        let rf = RewardFunction::default();
        assert!(rf.move_reward(true) > 0.0);
        assert!(rf.move_reward(false) < 0.0);
    }

    #[test]
    fn test_feedback_rewards() {
        let rf = RewardFunction::default();
        let feedback_good = UserFeedback::new(1, FeedbackRating::Good);
        let feedback_bad = UserFeedback::new(2, FeedbackRating::Bad);

        assert!(rf.feedback_reward(&feedback_good) > 0.0);
        assert!(rf.feedback_reward(&feedback_bad) < 0.0);
    }

    #[test]
    fn test_user_feedback_creation() {
        let feedback = UserFeedback::new(123, FeedbackRating::Good);
        assert_eq!(feedback.behavior_id, 123);
        assert_eq!(feedback.rating, FeedbackRating::Good);
    }
}
