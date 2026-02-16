//! Cron Scheduler for timed autonomy actions
//!
//! Invariant I-AUTO-001: Cron actions have cooldown (default 60s)

#[cfg(feature = "brain")]
use crate::brain::error::{BrainError, BrainResult};

#[cfg(feature = "brain")]
use std::collections::HashMap;
#[cfg(feature = "brain")]
use std::time::Instant;

/// A scheduled cron job
#[cfg(feature = "brain")]
#[derive(Debug, Clone)]
pub struct CronJob {
    pub name: String,
    /// Cron expression (e.g., "0 8 * * *" for 8 AM daily)
    pub schedule: String,
    /// Minimum seconds between executions (I-AUTO-001)
    pub cooldown_secs: u64,
    /// Whether this job is active
    pub enabled: bool,
}

/// Simple cron scheduler that checks if jobs should fire
#[cfg(feature = "brain")]
pub struct CronScheduler {
    jobs: Vec<CronJob>,
    last_fired: HashMap<String, Instant>,
}

#[cfg(feature = "brain")]
impl CronScheduler {
    pub fn new() -> Self {
        Self {
            jobs: Vec::new(),
            last_fired: HashMap::new(),
        }
    }

    pub fn add_job(&mut self, job: CronJob) {
        self.jobs.push(job);
    }

    /// Check which jobs should fire now
    pub fn check_due(&mut self) -> Vec<String> {
        let now = Instant::now();
        let mut due = Vec::new();

        for job in &self.jobs {
            if !job.enabled {
                continue;
            }

            // Check cooldown (I-AUTO-001)
            if let Some(last) = self.last_fired.get(&job.name) {
                if now.duration_since(*last).as_secs() < job.cooldown_secs {
                    continue;
                }
            }

            // Simple time-based check (hour matching)
            // Full cron parsing would use tokio-cron-scheduler
            if self.should_fire(&job.schedule) {
                self.last_fired.insert(job.name.clone(), now);
                due.push(job.name.clone());
            }
        }

        due
    }

    fn should_fire(&self, _schedule: &str) -> bool {
        // Placeholder - real implementation uses tokio-cron-scheduler
        // For now, this is driven by the AutonomyEngine's check_triggers
        false
    }

    pub fn jobs(&self) -> &[CronJob] {
        &self.jobs
    }
}

#[cfg(feature = "brain")]
impl Default for CronScheduler {
    fn default() -> Self {
        Self::new()
    }
}
