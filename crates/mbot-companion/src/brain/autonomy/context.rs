//! Context Monitor - tracks time, idle duration, battery etc.

#[cfg(feature = "brain")]
use mbot_core::{HomeostasisState, MBotSensors};
#[cfg(feature = "brain")]
use std::time::Instant;

/// Current context information for autonomy decisions
#[cfg(feature = "brain")]
pub struct ContextMonitor {
    pub started_at: Instant,
    pub last_interaction: Instant,
    pub energy_level: f32,
    pub tension_level: f32,
    pub coherence_level: f32,
    pub sound_level: f32,
    pub light_level: f32,
    pub idle_secs: u64,
    pub tick_count: u64,
}

#[cfg(feature = "brain")]
impl ContextMonitor {
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            started_at: now,
            last_interaction: now,
            energy_level: 1.0,
            tension_level: 0.0,
            coherence_level: 1.0,
            sound_level: 0.0,
            light_level: 0.5,
            idle_secs: 0,
            tick_count: 0,
        }
    }

    /// Update context from current state and sensors
    pub fn update(&mut self, state: &HomeostasisState, sensors: &MBotSensors) {
        self.energy_level = state.energy;
        self.tension_level = state.tension;
        self.coherence_level = state.coherence;
        self.sound_level = sensors.sound_level;
        self.light_level = sensors.light_level;
        self.idle_secs = self.last_interaction.elapsed().as_secs();
        self.tick_count += 1;

        // Sound above threshold counts as interaction
        if sensors.sound_level > 0.3 {
            self.last_interaction = Instant::now();
        }
    }

    /// Record a user interaction
    pub fn record_interaction(&mut self) {
        self.last_interaction = Instant::now();
    }

    /// How long since startup
    pub fn uptime_secs(&self) -> u64 {
        self.started_at.elapsed().as_secs()
    }

    /// Whether the robot has been idle for a while
    pub fn is_idle(&self, threshold_secs: u64) -> bool {
        self.idle_secs >= threshold_secs
    }
}

#[cfg(feature = "brain")]
impl Default for ContextMonitor {
    fn default() -> Self {
        Self::new()
    }
}
