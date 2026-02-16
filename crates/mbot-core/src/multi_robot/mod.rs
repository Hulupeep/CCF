//! Multi-Robot Coordination
//!
//! Coordination protocol for 2-4 robots to synchronize actions, share state,
//! and execute coordinated behaviors.
//!
//! Implements invariants I-MULTI-001 through I-MULTI-006 from feature_multi_robot.yml
//!
//! ## Modules
//!
//! - `swarm` - Swarm play modes (Issue #83)
//! - `collision` - Collision avoidance system (Issue #83)

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use alloc::{string::String, vec::Vec};

#[cfg(feature = "std")]
use std::{string::String, vec::Vec};

// Re-export swarm and collision modules
pub mod swarm;
pub mod collision;

// Contract: I-MULTI-004 - Maximum 4 robots
pub const MAX_ROBOTS: usize = 4;

// Contract: I-MULTI-001 - Discovery timeout 5 seconds
pub const DISCOVERY_TIMEOUT_MS: u32 = 5000;

// Contract: I-MULTI-003 - Election timeout 3 seconds
pub const ELECTION_TIMEOUT_MS: u32 = 3000;

// Contract: I-MULTI-005 - Sync interval 100ms for <100ms latency
pub const SYNC_INTERVAL_MS: u32 = 100;

// Contract: I-MULTI-002 - Heartbeat for consistency
pub const HEARTBEAT_INTERVAL_MS: u32 = 1000;

// Contract: I-MULTI-006 - Disconnect detection timeout
pub const DISCONNECT_TIMEOUT_MS: u32 = 3000;

/// Unique identifier for a robot in the coordination mesh
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RobotId(pub String);

impl RobotId {
    pub fn new(id: String) -> Self {
        Self(id)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Robot role in coordination
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum RobotRole {
    Leader,
    Follower,
}

/// Robot status
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum RobotStatus {
    Idle,
    Moving,
    Executing,
}

/// 2D position
#[derive(Clone, Copy, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

/// Robot state in the coordination mesh
/// Contract: I-MULTI-002 - Includes sequence and timestamp for ordering
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RobotState {
    pub id: RobotId,
    pub role: RobotRole,
    pub position: Position,
    pub status: RobotStatus,
    pub last_heartbeat: u64,
    pub sequence: u64,
}

impl RobotState {
    pub fn new(id: RobotId) -> Self {
        Self {
            id,
            role: RobotRole::Follower,
            position: Position { x: 0.0, y: 0.0 },
            status: RobotStatus::Idle,
            last_heartbeat: 0,
            sequence: 0,
        }
    }

    /// Check if robot is disconnected based on heartbeat
    /// Contract: I-MULTI-006 - Disconnect detection
    pub fn is_disconnected(&self, current_time: u64) -> bool {
        current_time.saturating_sub(self.last_heartbeat) > DISCONNECT_TIMEOUT_MS as u64
    }
}

/// Message action types
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum MessageAction {
    Sync,
    Command,
    State,
    Heartbeat,
    Election,
    ElectionAck,
}

/// Coordination message
/// Contract: I-MULTI-002 - Includes sequence and timestamp for ordering
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CoordinationMessage {
    pub from_robot: RobotId,
    pub to_robots: Vec<RobotId>,
    pub action: MessageAction,
    pub payload: MessagePayload,
    pub timestamp: u64,
    pub sequence: u64,
}

/// Message payload variants
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum MessagePayload {
    Heartbeat,
    StateUpdate { position: Position, status: RobotStatus },
    Command { command_type: String, params: Vec<f32> },
    Election { priority: u64 },
    ElectionAck { elected_id: RobotId },
}

/// Coordination configuration
#[derive(Clone, Debug)]
pub struct CoordinationConfig {
    pub max_robots: usize,
    pub heartbeat_interval: u32,
    pub discovery_timeout: u32,
    pub sync_interval: u32,
    pub election_timeout: u32,
}

impl Default for CoordinationConfig {
    fn default() -> Self {
        Self {
            max_robots: MAX_ROBOTS,
            heartbeat_interval: HEARTBEAT_INTERVAL_MS,
            discovery_timeout: DISCOVERY_TIMEOUT_MS,
            sync_interval: SYNC_INTERVAL_MS,
            election_timeout: ELECTION_TIMEOUT_MS,
        }
    }
}

/// Coordination errors
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CoordinationError {
    TooManyRobots,
    RobotNotFound,
    ElectionTimeout,
    DiscoveryTimeout,
    InvalidMessage,
    NetworkError,
}

/// Multi-robot coordination manager
/// Contract: I-MULTI-002 - Maintains consistent state across robots
pub struct CoordinationManager {
    pub local_id: RobotId,
    pub robots: Vec<RobotState>,
    pub config: CoordinationConfig,
    pub sequence: u64,
    pub role: RobotRole,
    pub election_in_progress: bool,
    pub election_start_time: u64,
}

impl CoordinationManager {
    pub fn new(local_id: RobotId, config: CoordinationConfig) -> Self {
        let mut robots = Vec::new();
        robots.push(RobotState::new(local_id.clone()));

        Self {
            local_id,
            robots,
            config,
            sequence: 0,
            role: RobotRole::Follower,
            election_in_progress: false,
            election_start_time: 0,
        }
    }

    /// Add a discovered robot to the mesh
    /// Contract: I-MULTI-004 - Enforces max 4 robots
    pub fn add_robot(&mut self, robot_id: RobotId) -> Result<(), CoordinationError> {
        if self.robots.len() >= self.config.max_robots {
            return Err(CoordinationError::TooManyRobots);
        }

        if !self.robots.iter().any(|r| r.id == robot_id) {
            self.robots.push(RobotState::new(robot_id));
        }

        Ok(())
    }

    /// Remove a robot from the mesh
    /// Contract: I-MULTI-006 - Handle disconnects gracefully
    pub fn remove_robot(&mut self, robot_id: &RobotId) -> Result<(), CoordinationError> {
        let was_leader = self.robots.iter()
            .find(|r| r.id == *robot_id)
            .map(|r| r.role == RobotRole::Leader)
            .unwrap_or(false);

        self.robots.retain(|r| r.id != *robot_id);

        // If leader disconnected, trigger election
        if was_leader {
            self.start_election();
        }

        Ok(())
    }

    /// Get robot state by ID
    pub fn get_robot(&self, robot_id: &RobotId) -> Option<&RobotState> {
        self.robots.iter().find(|r| r.id == *robot_id)
    }

    /// Get mutable robot state by ID
    fn get_robot_mut(&mut self, robot_id: &RobotId) -> Option<&mut RobotState> {
        self.robots.iter_mut().find(|r| r.id == *robot_id)
    }

    /// Update robot state
    /// Contract: I-MULTI-002 - State changes use sequence numbers
    pub fn update_robot_state(
        &mut self,
        robot_id: &RobotId,
        position: Position,
        status: RobotStatus,
        sequence: u64,
        timestamp: u64,
    ) -> Result<(), CoordinationError> {
        let robot = self.get_robot_mut(robot_id)
            .ok_or(CoordinationError::RobotNotFound)?;

        // Only update if sequence is newer (prevents out-of-order updates)
        if sequence > robot.sequence {
            robot.position = position;
            robot.status = status;
            robot.sequence = sequence;
            robot.last_heartbeat = timestamp;
        }

        Ok(())
    }

    /// Process heartbeat
    /// Contract: I-MULTI-002 - Heartbeat maintains consistency
    pub fn process_heartbeat(
        &mut self,
        robot_id: &RobotId,
        timestamp: u64,
    ) -> Result<(), CoordinationError> {
        let robot = self.get_robot_mut(robot_id)
            .ok_or(CoordinationError::RobotNotFound)?;

        robot.last_heartbeat = timestamp;
        Ok(())
    }

    /// Check for disconnected robots
    /// Contract: I-MULTI-006 - Detect disconnects via timeout
    pub fn detect_disconnects(&mut self, current_time: u64) -> Vec<RobotId> {
        let disconnected: Vec<RobotId> = self.robots.iter()
            .filter(|r| r.id != self.local_id && r.is_disconnected(current_time))
            .map(|r| r.id.clone())
            .collect();

        // Remove disconnected robots
        for robot_id in &disconnected {
            let _ = self.remove_robot(robot_id);
        }

        disconnected
    }

    /// Start leader election
    /// Contract: I-MULTI-003 - Bully algorithm, completes in 3s
    pub fn start_election(&mut self) {
        self.election_in_progress = true;
        self.election_start_time = self.current_time();
    }

    /// Get election priority (based on robot ID lexicographic order)
    /// Contract: I-MULTI-003 - Deterministic election
    fn get_priority(&self) -> u64 {
        // Use robot ID as priority (could be enhanced with uptime, capabilities, etc.)
        self.local_id.as_str().bytes().fold(0u64, |acc, b| acc.wrapping_add(b as u64))
    }

    /// Process election message
    /// Contract: I-MULTI-003 - Bully algorithm
    pub fn process_election(
        &mut self,
        from_robot: &RobotId,
        priority: u64,
    ) -> Option<CoordinationMessage> {
        let my_priority = self.get_priority();

        if priority > my_priority {
            // Higher priority robot found, acknowledge it
            #[cfg(not(feature = "std"))]
            use alloc::vec;

            Some(CoordinationMessage {
                from_robot: self.local_id.clone(),
                to_robots: vec![from_robot.clone()],
                action: MessageAction::ElectionAck,
                payload: MessagePayload::ElectionAck {
                    elected_id: from_robot.clone(),
                },
                timestamp: self.current_time(),
                sequence: self.next_sequence(),
            })
        } else {
            // I have higher priority, ignore and continue election
            None
        }
    }

    /// Check if election has timed out and declare self as leader
    /// Contract: I-MULTI-003 - Election completes within 3 seconds
    pub fn check_election_timeout(&mut self, current_time: u64) -> bool {
        if !self.election_in_progress {
            return false;
        }

        let elapsed = current_time.saturating_sub(self.election_start_time);
        if elapsed >= ELECTION_TIMEOUT_MS as u64 {
            // Election timeout - become leader
            self.role = RobotRole::Leader;
            self.election_in_progress = false;

            // Update own state
            let local_id = self.local_id.clone();
            if let Some(robot) = self.get_robot_mut(&local_id) {
                robot.role = RobotRole::Leader;
            }

            return true;
        }

        false
    }

    /// Create a coordination message
    pub fn create_message(
        &mut self,
        to_robots: Vec<RobotId>,
        action: MessageAction,
        payload: MessagePayload,
    ) -> CoordinationMessage {
        CoordinationMessage {
            from_robot: self.local_id.clone(),
            to_robots,
            action,
            payload,
            timestamp: self.current_time(),
            sequence: self.next_sequence(),
        }
    }

    /// Get next sequence number
    /// Contract: I-MULTI-002 - Sequence numbers for ordering
    fn next_sequence(&mut self) -> u64 {
        self.sequence = self.sequence.wrapping_add(1);
        self.sequence
    }

    /// Get current time (milliseconds)
    /// Note: In no_std, this should be provided by platform
    fn current_time(&self) -> u64 {
        #[cfg(feature = "std")]
        {
            use std::time::{SystemTime, UNIX_EPOCH};
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64
        }

        #[cfg(not(feature = "std"))]
        {
            // In no_std, time must be provided externally
            0
        }
    }

    /// Get current leader
    pub fn get_leader(&self) -> Option<&RobotState> {
        self.robots.iter().find(|r| r.role == RobotRole::Leader)
    }

    /// Check if this robot is the leader
    pub fn is_leader(&self) -> bool {
        self.role == RobotRole::Leader
    }

    /// Get all connected robots
    pub fn get_connected_robots(&self) -> Vec<&RobotState> {
        self.robots.iter().collect()
    }

    /// Get robot count
    pub fn robot_count(&self) -> usize {
        self.robots.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_max_robots_enforced() {
        let mut manager = CoordinationManager::new(
            RobotId::new("robot1".into()),
            CoordinationConfig::default(),
        );

        // Add robots up to limit
        assert!(manager.add_robot(RobotId::new("robot2".into())).is_ok());
        assert!(manager.add_robot(RobotId::new("robot3".into())).is_ok());
        assert!(manager.add_robot(RobotId::new("robot4".into())).is_ok());

        // Should fail on 5th robot
        assert_eq!(
            manager.add_robot(RobotId::new("robot5".into())),
            Err(CoordinationError::TooManyRobots)
        );
    }

    #[test]
    fn test_disconnect_detection() {
        let mut manager = CoordinationManager::new(
            RobotId::new("robot1".into()),
            CoordinationConfig::default(),
        );

        let robot2_id = RobotId::new("robot2".into());
        manager.add_robot(robot2_id.clone()).unwrap();

        // Update heartbeat to old time
        manager.process_heartbeat(&robot2_id, 1000).unwrap();

        // Check for disconnects with current time far in future
        let current_time = 1000 + DISCONNECT_TIMEOUT_MS as u64 + 100;
        let disconnected = manager.detect_disconnects(current_time);

        assert_eq!(disconnected.len(), 1);
        assert_eq!(disconnected[0], robot2_id);
        assert_eq!(manager.robot_count(), 1); // Only local robot remains
    }

    #[test]
    fn test_state_sequence_ordering() {
        let mut manager = CoordinationManager::new(
            RobotId::new("robot1".into()),
            CoordinationConfig::default(),
        );

        let robot2_id = RobotId::new("robot2".into());
        manager.add_robot(robot2_id.clone()).unwrap();

        // Update with sequence 10
        manager.update_robot_state(
            &robot2_id,
            Position { x: 10.0, y: 10.0 },
            RobotStatus::Moving,
            10,
            2000,
        ).unwrap();

        let robot = manager.get_robot(&robot2_id).unwrap();
        assert_eq!(robot.position.x, 10.0);
        assert_eq!(robot.sequence, 10);

        // Try to update with older sequence (should be ignored)
        manager.update_robot_state(
            &robot2_id,
            Position { x: 5.0, y: 5.0 },
            RobotStatus::Idle,
            5,
            2100,
        ).unwrap();

        let robot = manager.get_robot(&robot2_id).unwrap();
        assert_eq!(robot.position.x, 10.0); // Should not change
        assert_eq!(robot.sequence, 10); // Should not change
    }

    #[test]
    fn test_leader_election_trigger_on_disconnect() {
        let mut manager = CoordinationManager::new(
            RobotId::new("robot1".into()),
            CoordinationConfig::default(),
        );

        let leader_id = RobotId::new("leader".into());
        manager.add_robot(leader_id.clone()).unwrap();

        // Make the other robot the leader
        if let Some(robot) = manager.get_robot_mut(&leader_id) {
            robot.role = RobotRole::Leader;
        }

        assert!(!manager.election_in_progress);

        // Remove leader
        manager.remove_robot(&leader_id).unwrap();

        // Election should be triggered
        assert!(manager.election_in_progress);
    }

    #[test]
    fn test_sequence_numbers_increment() {
        let mut manager = CoordinationManager::new(
            RobotId::new("robot1".into()),
            CoordinationConfig::default(),
        );

        let seq1 = manager.next_sequence();
        let seq2 = manager.next_sequence();
        let seq3 = manager.next_sequence();

        assert_eq!(seq1, 1);
        assert_eq!(seq2, 2);
        assert_eq!(seq3, 3);
    }
}
