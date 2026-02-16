//! Collision Avoidance System
//!
//! Implements safety buffer and collision prevention for swarm movements.
//!
//! **Contract Compliance:**
//! - I-MULTI-004: 20cm safety buffer between robots
//! - ARCH-003: Kitchen Table Test - no harmful behaviors

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use alloc::{vec::Vec, string::String};

#[cfg(feature = "std")]
use std::{vec::Vec, string::String};

use super::{Position, RobotState};

/// Safety buffer distance - robots must maintain 20cm separation
/// **Contract**: I-MULTI-004
pub const SAFETY_BUFFER_CM: f32 = 20.0;

/// Warning buffer - start adjusting trajectory at this distance
pub const WARNING_BUFFER_CM: f32 = 30.0;

/// Collision risk assessment
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CollisionRisk {
    /// No collision risk
    Safe,
    /// Potential collision - adjust trajectory
    Warning,
    /// Imminent collision - stop immediately
    Critical,
}

/// Result of collision check
#[derive(Clone, Debug)]
pub struct CollisionCheck {
    /// Overall risk level
    pub risk: CollisionRisk,
    /// Robots that are too close
    pub conflicting_robots: Vec<String>,
    /// Recommended avoidance vector (if needed)
    pub avoidance_vector: Option<(f32, f32)>,
}

/// Collision avoidance system
pub struct CollisionAvoidance {
    /// Safety buffer distance
    safety_buffer: f32,
    /// Warning buffer distance
    warning_buffer: f32,
}

impl CollisionAvoidance {
    pub fn new() -> Self {
        Self {
            safety_buffer: SAFETY_BUFFER_CM,
            warning_buffer: WARNING_BUFFER_CM,
        }
    }

    /// Check if a position would cause collision with other robots
    pub fn check_position(&self, position: &Position, robots: &[RobotState]) -> CollisionCheck {
        let mut risk = CollisionRisk::Safe;
        let mut conflicting_robots = Vec::new();
        let mut avoidance_x = 0.0;
        let mut avoidance_y = 0.0;

        for robot in robots {
            let dx = position.x - robot.position.x;
            let dy = position.y - robot.position.y;

            #[cfg(not(feature = "std"))]
            let distance = libm::sqrtf(dx * dx + dy * dy);
            #[cfg(feature = "std")]
            let distance = (dx * dx + dy * dy).sqrt();

            if distance < self.safety_buffer {
                risk = CollisionRisk::Critical;
                conflicting_robots.push(robot.id.as_str().into());

                // Calculate avoidance vector (away from robot)
                #[cfg(not(feature = "std"))]
                let norm = libm::sqrtf(dx * dx + dy * dy);
                #[cfg(feature = "std")]
                let norm = (dx * dx + dy * dy).sqrt();

                if norm > 0.001 {
                    avoidance_x += dx / norm;
                    avoidance_y += dy / norm;
                }
            } else if distance < self.warning_buffer && risk != CollisionRisk::Critical {
                risk = CollisionRisk::Warning;
                conflicting_robots.push(robot.id.as_str().into());

                // Calculate gentle avoidance vector
                #[cfg(not(feature = "std"))]
                let norm = libm::sqrtf(dx * dx + dy * dy);
                #[cfg(feature = "std")]
                let norm = (dx * dx + dy * dy).sqrt();

                if norm > 0.001 {
                    avoidance_x += 0.5 * dx / norm;
                    avoidance_y += 0.5 * dy / norm;
                }
            }
        }

        let avoidance_vector = if risk != CollisionRisk::Safe {
            Some((avoidance_x, avoidance_y))
        } else {
            None
        };

        CollisionCheck {
            risk,
            conflicting_robots,
            avoidance_vector,
        }
    }

    /// Apply collision avoidance to a target position
    pub fn apply_avoidance(&self, target: &Position, robots: &[RobotState]) -> Position {
        let check = self.check_position(target, robots);

        match check.risk {
            CollisionRisk::Safe => *target,
            CollisionRisk::Warning | CollisionRisk::Critical => {
                if let Some((avoid_x, avoid_y)) = check.avoidance_vector {
                    // Scale avoidance based on risk
                    let scale = match check.risk {
                        CollisionRisk::Critical => 2.0,
                        CollisionRisk::Warning => 1.0,
                        CollisionRisk::Safe => 0.0,
                    };

                    Position {
                        x: target.x + avoid_x * scale,
                        y: target.y + avoid_y * scale,
                    }
                } else {
                    *target
                }
            }
        }
    }

    /// Check if a trajectory would cause collision
    pub fn check_trajectory(&self, start: &Position, end: &Position, robots: &[RobotState]) -> bool {
        // Sample points along trajectory
        const SAMPLES: usize = 10;

        for i in 0..=SAMPLES {
            let t = (i as f32) / (SAMPLES as f32);
            let x = start.x + t * (end.x - start.x);
            let y = start.y + t * (end.y - start.y);
            let pos = Position { x, y };

            let check = self.check_position(&pos, robots);
            if check.risk == CollisionRisk::Critical {
                return true;
            }
        }

        false
    }

    /// Verify all robots maintain safety buffer
    pub fn verify_swarm_safety(&self, robots: &[RobotState]) -> Result<(), String> {
        for i in 0..robots.len() {
            for j in (i + 1)..robots.len() {
                let dx = robots[i].position.x - robots[j].position.x;
                let dy = robots[i].position.y - robots[j].position.y;

                #[cfg(not(feature = "std"))]
                let distance = libm::sqrtf(dx * dx + dy * dy);
                #[cfg(feature = "std")]
                let distance = (dx * dx + dy * dy).sqrt();

                if distance < self.safety_buffer {
                    #[cfg(not(feature = "std"))]
                    use alloc::format;

                    return Err(format!(
                        "Safety violation: {} and {} are {:.1}cm apart (min: {:.1}cm)",
                        robots[i].id.as_str(), robots[j].id.as_str(), distance, self.safety_buffer
                    ));
                }
            }
        }

        Ok(())
    }
}

impl Default for CollisionAvoidance {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::RobotId;

    fn make_test_robot(id: &str, x: f32, y: f32) -> RobotState {
        let mut robot = RobotState::new(RobotId::new(id.into()));
        robot.position = Position { x, y };
        robot
    }

    #[test]
    fn test_safe_distance() {
        let collision = CollisionAvoidance::new();
        let target = Position { x: 0.0, y: 0.0 };
        let robots = vec![
            make_test_robot("robot1", 50.0, 0.0),
        ];

        let check = collision.check_position(&target, &robots);
        assert_eq!(check.risk, CollisionRisk::Safe);
    }

    #[test]
    fn test_warning_distance() {
        let collision = CollisionAvoidance::new();
        let target = Position { x: 0.0, y: 0.0 };
        let robots = vec![
            make_test_robot("robot1", 25.0, 0.0),
        ];

        let check = collision.check_position(&target, &robots);
        assert_eq!(check.risk, CollisionRisk::Warning);
    }

    #[test]
    fn test_critical_distance() {
        let collision = CollisionAvoidance::new();
        let target = Position { x: 0.0, y: 0.0 };
        let robots = vec![
            make_test_robot("robot1", 15.0, 0.0),
        ];

        let check = collision.check_position(&target, &robots);
        assert_eq!(check.risk, CollisionRisk::Critical);
    }

    #[test]
    fn test_avoidance_vector() {
        let collision = CollisionAvoidance::new();
        let target = Position { x: 0.0, y: 0.0 };
        let robots = vec![
            make_test_robot("robot1", 15.0, 0.0),
        ];

        let adjusted = collision.apply_avoidance(&target, &robots);

        // Should move away from robot (negative X direction)
        assert!(adjusted.x < target.x);
    }

    #[test]
    fn test_trajectory_collision() {
        let collision = CollisionAvoidance::new();
        let start = Position { x: 0.0, y: 0.0 };
        let end = Position { x: 100.0, y: 0.0 };
        let robots = vec![
            make_test_robot("robot1", 50.0, 5.0), // Close to trajectory
        ];

        assert!(collision.check_trajectory(&start, &end, &robots));
    }

    #[test]
    fn test_verify_swarm_safety() {
        let collision = CollisionAvoidance::new();

        // Safe configuration
        let safe_robots = vec![
            make_test_robot("robot1", 0.0, 0.0),
            make_test_robot("robot2", 50.0, 0.0),
        ];
        assert!(collision.verify_swarm_safety(&safe_robots).is_ok());

        // Unsafe configuration
        let unsafe_robots = vec![
            make_test_robot("robot1", 0.0, 0.0),
            make_test_robot("robot2", 10.0, 0.0), // Too close!
        ];
        assert!(collision.verify_swarm_safety(&unsafe_robots).is_err());
    }
}
