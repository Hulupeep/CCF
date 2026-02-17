//! Exploration State Machine
//!
//! Implements `ProactiveAction` to plug into the existing AutonomyEngine.
//! Drives autonomous room exploration using SectorMap + GridMap from mbot-core.
//!
//! State machine:
//!   IDLE → SCAN → CHOOSE_TARGET → MOVE_TO → ARRIVED → (SCAN or IDLE)
//!
//! Contract Compliance:
//! - I-AUTO-001: Respects cooldown via AutonomyEngine
//! - I-BRAIN-004: Motor commands pass through SafetyFilter
//! - I-BRAIN-005: Motor speeds clamped [-100, 100]
//! - ARCH-001: Spatial data in mbot-core (no_std), logic here in mbot-companion

#[cfg(feature = "brain")]
use async_trait::async_trait;

#[cfg(feature = "brain")]
use super::context::ContextMonitor;
#[cfg(feature = "brain")]
use crate::brain::error::BrainResult;
#[cfg(feature = "brain")]
use crate::brain::planner::BrainAction;
#[cfg(feature = "brain")]
use super::actions::ProactiveAction;

#[cfg(feature = "brain")]
use mbot_core::exploration::{
    SectorMap, GridMap, ExplorationEvent, Occupancy, GRID_SIZE,
};
#[cfg(feature = "brain")]
use mbot_core::learning::{
    QLearner, LearningConfig, NavAction, ReinforcementLearner,
    nav_state, classify_obstacle, classify_energy,
    NAV_REWARD_NEW_CELL, NAV_REWARD_REVISIT, NAV_REWARD_COLLISION,
    NAV_REWARD_SCAN,
};
#[cfg(feature = "brain")]
use mbot_core::MotorCommand;

#[cfg(feature = "brain")]
use std::time::Instant;

/// Exploration state machine phases.
#[cfg(feature = "brain")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExplorePhase {
    /// Default. Waiting for curiosity + energy to be sufficient.
    Idle,
    /// Rotating in place, reading ultrasonic at known yaw angles.
    Scanning { steps_remaining: u8 },
    /// Picked a target sector, about to move.
    ChoosingTarget,
    /// Driving toward chosen sector.
    MovingTo { target_sector: usize, ticks_remaining: u16 },
    /// Arrived at target position.
    Arrived,
}

/// Exploration action that plugs into the AutonomyEngine.
#[cfg(feature = "brain")]
pub struct ExploreAction {
    pub sector_map: SectorMap,
    pub grid_map: GridMap,
    pub phase: ExplorePhase,
    pub q_learner: QLearner,
    /// Recent events for narration / dashboard.
    pub events: Vec<ExplorationEvent>,
    /// Minimum curiosity level to start exploring.
    curiosity_threshold: f32,
    /// Minimum energy level to start exploring.
    energy_threshold: f32,
    /// How many ticks to drive per movement command.
    move_ticks: u16,
    /// Last ultrasonic reading (for state transitions).
    last_distance_cm: f32,
    /// Current target sector heading.
    target_heading_deg: f32,
    /// Tick counter for scan timing.
    scan_tick: u64,
    /// Total discoveries made.
    pub discovery_count: u32,
    /// Total exploration episodes.
    pub episode_count: u32,
    /// Cumulative reward for current episode.
    episode_reward: f32,
}

#[cfg(feature = "brain")]
impl ExploreAction {
    pub fn new() -> Self {
        let config = LearningConfig {
            learning_rate: 0.1,
            discount_factor: 0.9,
            epsilon_start: 0.3,
            epsilon_end: 0.05,
            epsilon_decay: 0.995,
            ..LearningConfig::default()
        };
        Self {
            sector_map: SectorMap::new(),
            grid_map: GridMap::new(),
            phase: ExplorePhase::Idle,
            q_learner: QLearner::new(config),
            events: Vec::new(),
            curiosity_threshold: 0.3,
            energy_threshold: 0.3,
            move_ticks: 8, // ~8 ticks of movement per segment
            last_distance_cm: 999.0,
            target_heading_deg: 0.0,
            scan_tick: 0,
            discovery_count: 0,
            episode_count: 0,
            episode_reward: 0.0,
        }
    }

    /// Advance the state machine by one tick. Returns a motor command if exploration
    /// wants to move, or None if idle / scan-in-progress.
    pub fn tick(
        &mut self,
        curiosity: f32,
        energy: f32,
        tension: f32,
        distance_cm: f32,
        heading_deg: f32,
        tick: u64,
    ) -> Option<MotorCommand> {
        self.last_distance_cm = distance_cm;
        self.scan_tick = tick;

        // Update grid map robot position
        self.grid_map.robot_heading_deg = heading_deg;

        // Only update sector map with valid ultrasonic readings.
        // distance_cm == 0.0 means sensor error/not connected.
        if distance_cm > 2.0 {
            self.sector_map.update_sector(heading_deg, distance_cm, tick);
        }

        match self.phase.clone() {
            ExplorePhase::Idle => {
                // Start exploring when energetic enough and not in high tension.
                // Curiosity threshold is low (0.15) because exploration itself generates
                // the stimulation that drives curiosity up — without moving, the robot
                // stays at curiosity ~0.2 (chicken-and-egg problem).
                if energy >= self.energy_threshold && tension < 0.7 {
                    self.phase = ExplorePhase::Scanning { steps_remaining: 6 };
                    self.events.push(ExplorationEvent::ScanComplete { sectors_updated: 0 });
                    // Start scan: turn left slowly
                    Some(MotorCommand { left: -25, right: 25, ..Default::default() })
                } else {
                    None
                }
            }

            ExplorePhase::Scanning { steps_remaining } => {
                if steps_remaining == 0 {
                    // Scan done, choose target
                    self.sector_map.recompute_interest(curiosity, tick);
                    self.phase = ExplorePhase::ChoosingTarget;
                    self.events.push(ExplorationEvent::ScanComplete {
                        sectors_updated: self.sector_map.mapped_count(),
                    });
                    // Stop motors during target selection
                    Some(MotorCommand::default())
                } else {
                    // Continue scanning: rotate slowly
                    self.phase = ExplorePhase::Scanning { steps_remaining: steps_remaining - 1 };
                    Some(MotorCommand { left: -20, right: 20, ..Default::default() })
                }
            }

            ExplorePhase::ChoosingTarget => {
                // Use Q-learning to pick action, or fallback to most interesting sector
                let target = self.sector_map.most_interesting_sector();
                self.target_heading_deg = SectorMap::sector_to_heading(target);

                let obs = classify_obstacle(distance_cm);
                let nrg = classify_energy(energy);
                let reflex = if tension > 0.85 { "protect" }
                    else if tension > 0.55 { "spike" }
                    else if tension > 0.20 { "active" }
                    else { "calm" };

                let state = nav_state(target, obs, nrg, reflex);
                let actions = NavAction::all_actions();

                let chosen = self.q_learner.select_action(&state, &actions, true);

                self.events.push(ExplorationEvent::TargetChosen {
                    sector: target,
                    heading_deg: self.target_heading_deg,
                });

                // Translate Q-learning action to motor command
                match chosen {
                    Some(a) if a.action_type == "nav_scan" => {
                        self.phase = ExplorePhase::Scanning { steps_remaining: 4 };
                        self.episode_reward += NAV_REWARD_SCAN;
                        Some(MotorCommand { left: -20, right: 20, ..Default::default() })
                    }
                    Some(a) if a.action_type == "nav_backup" => {
                        self.phase = ExplorePhase::MovingTo { target_sector: target, ticks_remaining: 3 };
                        Some(MotorCommand { left: -40, right: -40, ..Default::default() })
                    }
                    Some(a) if a.action_type == "nav_turn_left_30" => {
                        self.phase = ExplorePhase::MovingTo { target_sector: target, ticks_remaining: self.move_ticks };
                        Some(MotorCommand { left: -30, right: 30, ..Default::default() })
                    }
                    Some(a) if a.action_type == "nav_turn_right_30" => {
                        self.phase = ExplorePhase::MovingTo { target_sector: target, ticks_remaining: self.move_ticks };
                        Some(MotorCommand { left: 30, right: -30, ..Default::default() })
                    }
                    _ => {
                        // Default: drive forward toward target
                        self.phase = ExplorePhase::MovingTo { target_sector: target, ticks_remaining: self.move_ticks };
                        Some(MotorCommand { left: 40, right: 40, ..Default::default() })
                    }
                }
            }

            ExplorePhase::MovingTo { target_sector, ticks_remaining } => {
                // Check for obstacle — but only if ultrasonic reading is valid.
                // distance_cm == 0.0 means sensor error/not connected (CyberPi returns
                // AttributeError when no external ultrasonic module is plugged in).
                // Values < 2.0 are below the sensor's minimum range and unreliable.
                // Safety note: even without ultrasonic, the homeostasis Protect mode
                // (in mbot-core) detects collisions via IMU acceleration spikes.
                let ultrasonic_valid = distance_cm > 2.0;
                if ultrasonic_valid && distance_cm < 15.0 {
                    // Obstacle! Update grid, back up, re-plan
                    self.grid_map.mark_obstacle_ahead(distance_cm, heading_deg, tick);
                    self.events.push(ExplorationEvent::ObstacleFound {
                        distance_cm,
                        heading_deg,
                    });
                    self.episode_reward += NAV_REWARD_COLLISION;
                    // Back up briefly, then re-scan
                    self.phase = ExplorePhase::Scanning { steps_remaining: 4 };
                    return Some(MotorCommand { left: -40, right: -40, ..Default::default() });
                }

                if ticks_remaining == 0 {
                    // Arrived at target area — advance grid position by dead reckoning
                    self.grid_map.robot_heading_deg = heading_deg;
                    self.grid_map.advance_one_cell();
                    self.grid_map.mark_current_visited(tick);
                    let was_new = self.grid_map.cells[self.grid_map.robot_y][self.grid_map.robot_x].visit_count == 1;
                    if was_new {
                        self.discovery_count += 1;
                        self.episode_reward += NAV_REWARD_NEW_CELL;
                        self.events.push(ExplorationEvent::CellDiscovered {
                            x: self.grid_map.robot_x,
                            y: self.grid_map.robot_y,
                            occupancy: Occupancy::Free,
                        });
                    } else {
                        self.episode_reward += NAV_REWARD_REVISIT;
                    }
                    self.events.push(ExplorationEvent::Arrived {
                        x: self.grid_map.robot_x,
                        y: self.grid_map.robot_y,
                    });
                    self.phase = ExplorePhase::Arrived;
                    Some(MotorCommand::default()) // stop
                } else {
                    // Keep moving
                    self.phase = ExplorePhase::MovingTo {
                        target_sector,
                        ticks_remaining: ticks_remaining - 1,
                    };
                    Some(MotorCommand { left: 40, right: 40, ..Default::default() })
                }
            }

            ExplorePhase::Arrived => {
                // Complete episode
                self.episode_count += 1;
                self.q_learner.complete_episode(self.episode_reward, "explore");
                self.episode_reward = 0.0;

                // Decide: continue scanning from new position or pause
                if energy < self.energy_threshold {
                    self.phase = ExplorePhase::Idle;
                    None
                } else {
                    self.phase = ExplorePhase::Scanning { steps_remaining: 6 };
                    Some(MotorCommand { left: -25, right: 25, ..Default::default() })
                }
            }
        }
    }

    /// Take pending events (drains the queue).
    pub fn take_events(&mut self) -> Vec<ExplorationEvent> {
        std::mem::take(&mut self.events)
    }

    /// Whether exploration is currently active (not idle).
    pub fn is_active(&self) -> bool {
        self.phase != ExplorePhase::Idle
    }

    /// Get the current phase name for display.
    pub fn phase_name(&self) -> &'static str {
        match self.phase {
            ExplorePhase::Idle => "Idle",
            ExplorePhase::Scanning { .. } => "Scanning",
            ExplorePhase::ChoosingTarget => "Choosing Target",
            ExplorePhase::MovingTo { .. } => "Moving",
            ExplorePhase::Arrived => "Arrived",
        }
    }

    /// Get Q-learning metrics for dashboard.
    pub fn learning_metrics(&self) -> &mbot_core::learning::LearningMetrics {
        use mbot_core::learning::ReinforcementLearner;
        self.q_learner.get_metrics()
    }
}

#[cfg(feature = "brain")]
impl Default for ExploreAction {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "brain")]
#[async_trait]
impl ProactiveAction for ExploreAction {
    fn name(&self) -> &str {
        "explore_room"
    }

    fn should_trigger(&self, context: &ContextMonitor) -> bool {
        // Trigger when curious enough, energetic enough, and not too tense
        context.energy_level >= 0.3
            && context.tension_level < 0.7
            && context.is_idle(30) // idle for 30+ seconds
    }

    async fn execute(&self, _context: &ContextMonitor) -> BrainResult<BrainAction> {
        // The actual exploration is driven by tick() in the main loop.
        // This just signals the brain layer to speak an exploration intent.
        Ok(BrainAction::Speak("I feel curious... let me look around!".into()))
    }
}
