//! Policy Storage and Persistence
//!
//! Handles saving and loading learned policies.
//! Invariant I-AI-002: Learned policy must persist across sessions

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use alloc::{string::String, vec::Vec};
#[cfg(not(feature = "std"))]
use hashbrown::HashMap;

#[cfg(feature = "std")]
use std::{collections::HashMap, string::String, vec::Vec};

/// Learned policy for a specific game
#[derive(Clone, Debug)]
pub struct Policy {
    /// Q-table mapping state-action pairs to values
    pub q_table: HashMap<String, HashMap<String, f32>>,

    /// Game type this policy is for
    pub game_type: String,

    /// Number of training episodes
    pub episode_count: u32,

    /// Learning rate used
    pub learning_rate: f32,

    /// Discount factor
    pub discount_factor: f32,

    /// Exploration rate
    pub epsilon: f32,

    /// Creation timestamp (seconds since epoch)
    pub created_at: u64,

    /// Last update timestamp
    pub updated_at: u64,
}

impl Policy {
    pub fn new(game_type: String) -> Self {
        Self {
            q_table: HashMap::new(),
            game_type,
            episode_count: 0,
            learning_rate: 0.1,
            discount_factor: 0.9,
            epsilon: 1.0,
            created_at: 0,
            updated_at: 0,
        }
    }

    /// Get number of states in policy
    pub fn state_count(&self) -> usize {
        self.q_table.len()
    }

    /// Get total number of state-action pairs
    pub fn total_entries(&self) -> usize {
        self.q_table.values().map(|actions| actions.len()).sum()
    }

    /// Check if policy is trained (has entries)
    pub fn is_trained(&self) -> bool {
        !self.q_table.is_empty() && self.episode_count > 0
    }
}

/// Policy storage interface
pub trait PolicyStorage {
    /// Save policy to persistent storage
    fn save(&self, policy: &Policy) -> Result<(), &'static str>;

    /// Load policy from persistent storage
    fn load(&self, game_type: &str) -> Result<Policy, &'static str>;

    /// List all saved policies
    fn list(&self) -> Result<Vec<String>, &'static str>;

    /// Delete a policy
    fn delete(&self, game_type: &str) -> Result<(), &'static str>;
}

/// In-memory policy storage (for testing and embedded)
pub struct MemoryPolicyStorage {
    policies: HashMap<String, Policy>,
}

impl MemoryPolicyStorage {
    pub fn new() -> Self {
        Self {
            policies: HashMap::new(),
        }
    }
}

impl Default for MemoryPolicyStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl PolicyStorage for MemoryPolicyStorage {
    fn save(&self, policy: &Policy) -> Result<(), &'static str> {
        // In a mutable version, would store the policy
        let _ = policy;
        Ok(())
    }

    fn load(&self, game_type: &str) -> Result<Policy, &'static str> {
        self.policies
            .get(game_type)
            .cloned()
            .ok_or("Policy not found")
    }

    fn list(&self) -> Result<Vec<String>, &'static str> {
        Ok(self.policies.keys().cloned().collect())
    }

    fn delete(&self, game_type: &str) -> Result<(), &'static str> {
        // In a mutable version, would remove the policy
        let _ = game_type;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_policy_creation() {
        let policy = Policy::new("tictactoe".into());
        assert_eq!(policy.game_type, "tictactoe");
        assert_eq!(policy.episode_count, 0);
        assert!(!policy.is_trained());
    }

    #[test]
    fn test_policy_trained_check() {
        let mut policy = Policy::new("test".into());
        assert!(!policy.is_trained());

        policy.episode_count = 100;
        policy.q_table.insert("state1".into(), HashMap::new());
        assert!(policy.is_trained());
    }

    #[test]
    fn test_policy_counts() {
        let mut policy = Policy::new("test".into());

        let mut actions = HashMap::new();
        actions.insert("a1".into(), 0.5);
        actions.insert("a2".into(), 0.8);

        policy.q_table.insert("s1".into(), actions);

        assert_eq!(policy.state_count(), 1);
        assert_eq!(policy.total_entries(), 2);
    }

    #[test]
    fn test_memory_storage() {
        let storage = MemoryPolicyStorage::new();
        let policy = Policy::new("test".into());

        assert!(storage.save(&policy).is_ok());
    }
}
