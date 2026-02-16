//! Swarm Play Modes
//!
//! Implements coordinated behaviors for 2-4 robots:
//! - Follow Leader
//! - Circle Formation
//! - Wave Pattern
//! - Random Walk (coordinated chaos)
//!
//! **Contract Compliance:**
//! - I-MULTI-004: 20cm safety buffer (via collision avoidance)
//! - I-MULTI-006: Formation accuracy ±5cm
//! - I-MULTI-007: Movement sync within 100ms

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use alloc::{vec::Vec, string::String, boxed::Box};

#[cfg(feature = "std")]
use std::{vec::Vec, string::String, boxed::Box};

use super::{RobotId, RobotState, Position};

/// Formation tolerance - robots must stay within ±5cm of target position
/// **Contract**: I-MULTI-006
pub const FORMATION_TOLERANCE_CM: f32 = 5.0;

/// Synchronization tolerance - movements must start within 100ms
/// **Contract**: I-MULTI-007
pub const SYNC_TOLERANCE_MS: u64 = 100;

/// Type of swarm play mode
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SwarmModeType {
    /// Robots follow a leader in formation
    FollowLeader,
    /// Robots form a circle and move together
    Circle,
    /// Robots move in a wave pattern
    Wave,
    /// Robots perform coordinated random walk
    RandomWalk,
    /// Robots perform synchronized dance routine
    Dance,
    /// Robots take turns drawing (collaborative art)
    CollaborativeDraw,
    /// Robots play a tag team game
    TagTeam,
    /// Robots patrol in formation
    Patrol,
}

/// Status of a swarm mode
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlayStatus {
    /// Mode is initialized but not started
    Idle,
    /// Mode is actively running
    Active,
    /// Mode is paused
    Paused,
    /// Mode completed successfully
    Completed,
    /// Mode failed (robot dropout, safety violation, etc.)
    Failed,
}

/// Configuration for swarm play mode
#[derive(Clone, Debug)]
pub struct SwarmConfig {
    /// Type of swarm mode
    pub mode_type: SwarmModeType,
    /// Participating robot IDs
    pub participants: Vec<RobotId>,
    /// Current status
    pub status: PlayStatus,
    /// Start time (milliseconds)
    pub start_time: u64,
    /// Current step in choreography
    pub current_step: u32,
    /// Mode-specific parameters
    pub params: SwarmParams,
}

/// Mode-specific parameters
#[derive(Clone, Debug)]
pub enum SwarmParams {
    FollowLeader {
        /// ID of leader robot
        leader_id: RobotId,
        /// Spacing between robots (cm)
        spacing: f32,
    },
    Circle {
        /// Center of circle
        center: Position,
        /// Radius of circle (cm)
        radius: f32,
        /// Rotation speed (radians/second)
        rotation_speed: f32,
    },
    Wave {
        /// Wave amplitude (cm)
        amplitude: f32,
        /// Wave frequency (Hz)
        frequency: f32,
        /// Phase offset per robot
        phase_offset: f32,
    },
    RandomWalk {
        /// Boundary for random walk
        bounds: (Position, Position),
        /// Movement duration (seconds)
        duration: f32,
    },
    Dance {
        /// Choreography steps
        choreography: Vec<DanceStep>,
        /// Sync tolerance (ms)
        sync_tolerance: u64,
    },
    CollaborativeDraw {
        /// Turn duration per robot (seconds)
        turn_duration: f32,
        /// Canvas size
        canvas: (f32, f32),
    },
    TagTeam {
        /// Game type
        game: String,
        /// Strategy
        strategy: String,
    },
    Patrol {
        /// Formation type
        formation: FormationType,
        /// Spacing between robots (cm)
        spacing: f32,
        /// Movement speed (cm/s)
        speed: f32,
    },
}

/// Dance choreography step
#[derive(Clone, Debug)]
pub struct DanceStep {
    /// Step number
    pub step: u32,
    /// Actions for each robot (robot_id -> action)
    pub robot_actions: Vec<(RobotId, RobotAction)>,
    /// Duration of this step (ms)
    pub duration: u64,
    /// Whether this is a synchronization point
    pub sync_point: bool,
}

/// Action a robot can perform in choreography
#[derive(Clone, Debug)]
pub enum RobotAction {
    /// Move to position
    Move { target: Position, speed: f32 },
    /// Rotate by angle (radians)
    Rotate { angle: f32 },
    /// Wait for duration (ms)
    Wait { duration: u64 },
    /// Play sound
    Beep { frequency: u16, duration: u64 },
    /// Flash LEDs
    Flash { color: [u8; 3], duration: u64 },
}

/// Formation type for patrol mode
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FormationType {
    /// Robots in a line
    Line,
    /// Robots in a diamond shape
    Diamond,
    /// Robots in a circle
    Circle,
    /// Robots in a square
    Square,
}

/// Trait for swarm mode behaviors
pub trait SwarmMode {
    /// Initialize the swarm mode
    fn init(&mut self, robots: &[RobotState]) -> Result<(), SwarmError>;

    /// Update positions for next step
    fn update(&mut self, delta_time: f32, robots: &[RobotState]) -> Result<Vec<TargetPosition>, SwarmError>;

    /// Check if mode is complete
    fn is_complete(&self) -> bool;

    /// Handle robot dropout
    fn handle_dropout(&mut self, robot_id: &RobotId) -> Result<(), SwarmError>;

    /// Get mode type
    fn mode_type(&self) -> SwarmModeType;
}

/// Target position for a robot
#[derive(Clone, Debug)]
pub struct TargetPosition {
    pub robot_id: RobotId,
    pub position: Position,
    pub heading: f32,
    pub speed: f32,
}

/// Swarm mode errors
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SwarmError {
    /// Not enough robots for this mode
    InsufficientRobots,
    /// Too many robots for this mode
    TooManyRobots,
    /// Robot not found in swarm
    RobotNotFound,
    /// Safety violation (collision risk)
    SafetyViolation,
    /// Formation cannot be maintained
    FormationFailure,
    /// Synchronization timeout
    SyncTimeout,
    /// Invalid configuration
    InvalidConfig,
}

// ============================================================================
// Follow Leader Mode
// ============================================================================

/// Follow leader swarm mode
pub struct FollowLeaderMode {
    leader_id: RobotId,
    spacing: f32,
    status: PlayStatus,
}

impl FollowLeaderMode {
    pub fn new(leader_id: RobotId, spacing: f32) -> Self {
        Self {
            leader_id,
            spacing,
            status: PlayStatus::Idle,
        }
    }

    /// Calculate follower positions based on leader
    fn calculate_positions(&self, leader_pos: Position, leader_heading: f32, follower_count: usize) -> Vec<Position> {
        let mut positions = Vec::new();

        #[cfg(not(feature = "std"))]
        use libm::{cosf, sinf};

        for i in 0..follower_count {
            let offset = (i + 1) as f32 * self.spacing;

            #[cfg(not(feature = "std"))]
            let x = leader_pos.x - offset * cosf(leader_heading);
            #[cfg(feature = "std")]
            let x = leader_pos.x - offset * leader_heading.cos();

            #[cfg(not(feature = "std"))]
            let y = leader_pos.y - offset * sinf(leader_heading);
            #[cfg(feature = "std")]
            let y = leader_pos.y - offset * leader_heading.sin();

            positions.push(Position { x, y });
        }

        positions
    }
}

impl SwarmMode for FollowLeaderMode {
    fn init(&mut self, robots: &[RobotState]) -> Result<(), SwarmError> {
        if robots.len() < 2 {
            return Err(SwarmError::InsufficientRobots);
        }
        if robots.len() > 4 {
            return Err(SwarmError::TooManyRobots);
        }

        // Verify leader exists
        if !robots.iter().any(|r| r.id == self.leader_id) {
            return Err(SwarmError::RobotNotFound);
        }

        self.status = PlayStatus::Active;
        Ok(())
    }

    fn update(&mut self, _delta_time: f32, robots: &[RobotState]) -> Result<Vec<TargetPosition>, SwarmError> {
        // Find leader
        let leader = robots.iter()
            .find(|r| r.id == self.leader_id)
            .ok_or(SwarmError::RobotNotFound)?;

        // TODO (#82): Get actual heading from robot state
        let leader_heading = 0.0; // Placeholder

        // Calculate follower positions
        let follower_positions = self.calculate_positions(leader.position, leader_heading, robots.len() - 1);

        // Create target positions for followers
        let mut targets = Vec::new();
        let mut follower_idx = 0;

        for robot in robots {
            if robot.id != self.leader_id {
                targets.push(TargetPosition {
                    robot_id: robot.id.clone(),
                    position: follower_positions[follower_idx],
                    heading: leader_heading,
                    speed: 30.0, // Base speed
                });
                follower_idx += 1;
            }
        }

        Ok(targets)
    }

    fn is_complete(&self) -> bool {
        self.status == PlayStatus::Completed
    }

    fn handle_dropout(&mut self, robot_id: &RobotId) -> Result<(), SwarmError> {
        if robot_id == &self.leader_id {
            // Leader dropped out - mode fails
            self.status = PlayStatus::Failed;
            return Err(SwarmError::FormationFailure);
        }

        // Follower dropped out - can continue
        Ok(())
    }

    fn mode_type(&self) -> SwarmModeType {
        SwarmModeType::FollowLeader
    }
}

// ============================================================================
// Circle Formation Mode
// ============================================================================

/// Circle formation mode
pub struct CircleMode {
    center: Position,
    radius: f32,
    rotation_speed: f32,
    current_angle: f32,
    status: PlayStatus,
}

impl CircleMode {
    pub fn new(center: Position, radius: f32, rotation_speed: f32) -> Self {
        Self {
            center,
            radius,
            rotation_speed,
            current_angle: 0.0,
            status: PlayStatus::Idle,
        }
    }
}

impl SwarmMode for CircleMode {
    fn init(&mut self, robots: &[RobotState]) -> Result<(), SwarmError> {
        if robots.len() < 2 {
            return Err(SwarmError::InsufficientRobots);
        }
        if robots.len() > 4 {
            return Err(SwarmError::TooManyRobots);
        }

        self.status = PlayStatus::Active;
        Ok(())
    }

    fn update(&mut self, delta_time: f32, robots: &[RobotState]) -> Result<Vec<TargetPosition>, SwarmError> {
        #[cfg(not(feature = "std"))]
        use libm::{cosf, sinf};

        // Update rotation angle
        self.current_angle += self.rotation_speed * delta_time;

        // Calculate positions evenly spaced around circle
        let mut targets = Vec::new();
        let angle_step = 2.0 * core::f32::consts::PI / robots.len() as f32;

        for (i, robot) in robots.iter().enumerate() {
            let angle = self.current_angle + (i as f32) * angle_step;

            #[cfg(not(feature = "std"))]
            let x = self.center.x + self.radius * cosf(angle);
            #[cfg(feature = "std")]
            let x = self.center.x + self.radius * angle.cos();

            #[cfg(not(feature = "std"))]
            let y = self.center.y + self.radius * sinf(angle);
            #[cfg(feature = "std")]
            let y = self.center.y + self.radius * angle.sin();

            targets.push(TargetPosition {
                robot_id: robot.id.clone(),
                position: Position { x, y },
                heading: angle + core::f32::consts::PI / 2.0, // Face tangent to circle
                speed: 25.0,
            });
        }

        Ok(targets)
    }

    fn is_complete(&self) -> bool {
        self.status == PlayStatus::Completed
    }

    fn handle_dropout(&mut self, _robot_id: &RobotId) -> Result<(), SwarmError> {
        // Recalculate positions in next update
        Ok(())
    }

    fn mode_type(&self) -> SwarmModeType {
        SwarmModeType::Circle
    }
}

// ============================================================================
// Wave Pattern Mode
// ============================================================================

/// Wave pattern mode
pub struct WaveMode {
    amplitude: f32,
    frequency: f32,
    phase_offset: f32,
    current_time: f32,
    status: PlayStatus,
}

impl WaveMode {
    pub fn new(amplitude: f32, frequency: f32, phase_offset: f32) -> Self {
        Self {
            amplitude,
            frequency,
            phase_offset,
            current_time: 0.0,
            status: PlayStatus::Idle,
        }
    }
}

impl SwarmMode for WaveMode {
    fn init(&mut self, robots: &[RobotState]) -> Result<(), SwarmError> {
        if robots.len() < 2 {
            return Err(SwarmError::InsufficientRobots);
        }
        if robots.len() > 4 {
            return Err(SwarmError::TooManyRobots);
        }

        self.status = PlayStatus::Active;
        Ok(())
    }

    fn update(&mut self, delta_time: f32, robots: &[RobotState]) -> Result<Vec<TargetPosition>, SwarmError> {
        #[cfg(not(feature = "std"))]
        use libm::sinf;

        self.current_time += delta_time;

        let mut targets = Vec::new();
        let base_y = 50.0; // Base Y position

        for (i, robot) in robots.iter().enumerate() {
            let phase = (i as f32) * self.phase_offset;
            let wave_value = {
                let arg = 2.0 * core::f32::consts::PI * self.frequency * self.current_time + phase;
                #[cfg(not(feature = "std"))]
                let result = sinf(arg);
                #[cfg(feature = "std")]
                let result = arg.sin();
                result
            };

            let y = base_y + self.amplitude * wave_value;
            let x = (i as f32) * 30.0; // Space robots along X axis

            targets.push(TargetPosition {
                robot_id: robot.id.clone(),
                position: Position { x, y },
                heading: 0.0,
                speed: 20.0,
            });
        }

        Ok(targets)
    }

    fn is_complete(&self) -> bool {
        self.status == PlayStatus::Completed
    }

    fn handle_dropout(&mut self, _robot_id: &RobotId) -> Result<(), SwarmError> {
        Ok(())
    }

    fn mode_type(&self) -> SwarmModeType {
        SwarmModeType::Wave
    }
}

// ============================================================================
// Random Walk Mode
// ============================================================================

/// Coordinated random walk mode
pub struct RandomWalkMode {
    bounds: (Position, Position),
    duration: f32,
    elapsed: f32,
    status: PlayStatus,
}

impl RandomWalkMode {
    pub fn new(bounds: (Position, Position), duration: f32) -> Self {
        Self {
            bounds,
            duration,
            elapsed: 0.0,
            status: PlayStatus::Idle,
        }
    }

    // TODO: Implement proper random number generation for no_std
    // For now, use a simple pseudorandom based on time
    fn pseudo_random(&self, seed: u32) -> f32 {
        let a = 1103515245_u32;
        let c = 12345_u32;
        let m = 2147483648_u32;
        let x = (a.wrapping_mul(seed).wrapping_add(c)) % m;
        (x as f32) / (m as f32)
    }
}

impl SwarmMode for RandomWalkMode {
    fn init(&mut self, robots: &[RobotState]) -> Result<(), SwarmError> {
        if robots.len() < 2 {
            return Err(SwarmError::InsufficientRobots);
        }
        if robots.len() > 4 {
            return Err(SwarmError::TooManyRobots);
        }

        self.status = PlayStatus::Active;
        Ok(())
    }

    fn update(&mut self, delta_time: f32, robots: &[RobotState]) -> Result<Vec<TargetPosition>, SwarmError> {
        self.elapsed += delta_time;

        if self.elapsed >= self.duration {
            self.status = PlayStatus::Completed;
        }

        let mut targets = Vec::new();

        for (i, robot) in robots.iter().enumerate() {
            // Generate pseudorandom position within bounds
            let seed = (self.elapsed * 1000.0) as u32 + (i as u32) * 1000;
            let rand_x = self.pseudo_random(seed);
            let rand_y = self.pseudo_random(seed + 1);

            let x = self.bounds.0.x + rand_x * (self.bounds.1.x - self.bounds.0.x);
            let y = self.bounds.0.y + rand_y * (self.bounds.1.y - self.bounds.0.y);

            targets.push(TargetPosition {
                robot_id: robot.id.clone(),
                position: Position { x, y },
                heading: 0.0,
                speed: 15.0,
            });
        }

        Ok(targets)
    }

    fn is_complete(&self) -> bool {
        self.status == PlayStatus::Completed
    }

    fn handle_dropout(&mut self, _robot_id: &RobotId) -> Result<(), SwarmError> {
        Ok(())
    }

    fn mode_type(&self) -> SwarmModeType {
        SwarmModeType::RandomWalk
    }
}

#[cfg(all(test, not(feature = "no_std")))]
mod tests {
    use super::*;

    fn make_test_robot(id: &str, x: f32, y: f32) -> RobotState {
        let mut robot = RobotState::new(RobotId::new(id.into()));
        robot.position = Position { x, y };
        robot
    }

    #[test]
    fn test_follow_leader_init() {
        let mut mode = FollowLeaderMode::new(RobotId::new("leader".into()), 30.0);
        let robots = vec![
            make_test_robot("leader", 0.0, 0.0),
            make_test_robot("follower1", 10.0, 10.0),
        ];

        assert!(mode.init(&robots).is_ok());
        assert_eq!(mode.status, PlayStatus::Active);
    }

    #[test]
    fn test_follow_leader_insufficient_robots() {
        let mut mode = FollowLeaderMode::new(RobotId::new("leader".into()), 30.0);
        let robots = vec![
            make_test_robot("leader", 0.0, 0.0),
        ];

        assert_eq!(mode.init(&robots), Err(SwarmError::InsufficientRobots));
    }

    #[test]
    fn test_circle_mode() {
        let mut mode = CircleMode::new(Position { x: 0.0, y: 0.0 }, 50.0, 0.1);
        let robots = vec![
            make_test_robot("robot1", 50.0, 0.0),
            make_test_robot("robot2", -50.0, 0.0),
        ];

        assert!(mode.init(&robots).is_ok());

        let targets = mode.update(0.1, &robots).unwrap();
        assert_eq!(targets.len(), 2);

        // Verify robots maintain formation tolerance
        for target in &targets {
            let dx = target.position.x - mode.center.x;
            let dy = target.position.y - mode.center.y;
            #[cfg(not(feature = "std"))]
            let dist_to_center = libm::sqrtf(dx * dx + dy * dy);
            #[cfg(feature = "std")]
            let dist_to_center = (dx * dx + dy * dy).sqrt();

            assert!((dist_to_center - mode.radius).abs() < FORMATION_TOLERANCE_CM);
        }
    }

    #[test]
    fn test_wave_mode() {
        let mut mode = WaveMode::new(20.0, 0.5, core::f32::consts::PI / 2.0);
        let robots = vec![
            make_test_robot("robot1", 0.0, 50.0),
            make_test_robot("robot2", 30.0, 50.0),
        ];

        assert!(mode.init(&robots).is_ok());

        let targets = mode.update(0.1, &robots).unwrap();
        assert_eq!(targets.len(), 2);
    }
}
