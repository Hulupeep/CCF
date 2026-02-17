//! Autonomy Engine - Proactive behaviors for mBot2
//!
//! Invariants:
//! - I-AUTO-001: Cron actions have cooldown (default 60s)
//! - I-AUTO-002: Max concurrent actions configurable (default 5)
//! - I-AUTO-003: Safe mode: require approval before executing

#[cfg(feature = "brain")]
pub mod event_bus;
#[cfg(feature = "brain")]
pub mod cron;
#[cfg(feature = "brain")]
pub mod context;
#[cfg(feature = "brain")]
pub mod actions;
#[cfg(feature = "brain")]
pub mod explore;

#[cfg(feature = "brain")]
use crate::brain::error::{BrainError, BrainResult};
#[cfg(feature = "brain")]
use crate::brain::planner::BrainAction;
#[cfg(feature = "brain")]
use mbot_core::{HomeostasisState, MBotSensors};

#[cfg(feature = "brain")]
use event_bus::EventBus;
#[cfg(feature = "brain")]
use context::ContextMonitor;
#[cfg(feature = "brain")]
use actions::ProactiveAction;

#[cfg(feature = "brain")]
use std::collections::HashMap;
#[cfg(feature = "brain")]
use std::sync::Arc;
#[cfg(feature = "brain")]
use std::time::Instant;
#[cfg(feature = "brain")]
use tokio::sync::Mutex;

/// Autonomy engine orchestrator
#[cfg(feature = "brain")]
pub struct AutonomyEngine {
    event_bus: EventBus,
    context: ContextMonitor,
    actions: Vec<Box<dyn ProactiveAction>>,
    /// Track last execution time per action (I-AUTO-001: cooldown)
    last_execution: HashMap<String, Instant>,
    /// Cooldown between repeated actions in seconds (I-AUTO-001)
    cooldown_secs: u64,
    /// Max concurrent actions (I-AUTO-002)
    max_concurrent: usize,
    /// Currently running action count
    running_count: Arc<Mutex<usize>>,
    /// Safe mode - require approval (I-AUTO-003)
    safe_mode: bool,
    /// Pending actions awaiting approval in safe mode
    pending_approval: Vec<(String, BrainAction)>,
}

#[cfg(feature = "brain")]
impl AutonomyEngine {
    pub async fn new(safe_mode: bool, max_concurrent: usize) -> BrainResult<Self> {
        Ok(Self {
            event_bus: EventBus::new(256),
            context: ContextMonitor::new(),
            actions: actions::default_actions(),
            last_execution: HashMap::new(),
            cooldown_secs: 60,
            max_concurrent,
            running_count: Arc::new(Mutex::new(0)),
            safe_mode,
            pending_approval: Vec::new(),
        })
    }

    /// Check all triggers and return actions to execute
    pub async fn check_triggers(
        &mut self,
        state: &HomeostasisState,
        sensors: &MBotSensors,
    ) -> BrainResult<Vec<BrainAction>> {
        self.context.update(state, sensors);

        let mut result_actions = Vec::new();
        let now = Instant::now();

        for action in &self.actions {
            let action_name = action.name().to_string();

            // I-AUTO-001: Check cooldown
            if let Some(last) = self.last_execution.get(&action_name) {
                if now.duration_since(*last).as_secs() < self.cooldown_secs {
                    continue;
                }
            }

            // I-AUTO-002: Check concurrency limit
            let running = *self.running_count.lock().await;
            if running >= self.max_concurrent {
                break;
            }

            // Check if action should trigger
            if action.should_trigger(&self.context) {
                let brain_action = action.execute(&self.context).await?;

                if self.safe_mode {
                    // I-AUTO-003: Queue for approval
                    self.pending_approval.push((action_name.clone(), brain_action));
                } else {
                    self.last_execution.insert(action_name, now);
                    result_actions.push(brain_action);
                }
            }
        }

        Ok(result_actions)
    }

    /// Approve a pending action (for safe mode, I-AUTO-003)
    pub fn approve_pending(&mut self, index: usize) -> Option<BrainAction> {
        if index < self.pending_approval.len() {
            let (name, action) = self.pending_approval.remove(index);
            self.last_execution.insert(name, Instant::now());
            Some(action)
        } else {
            None
        }
    }

    /// Reject a pending action
    pub fn reject_pending(&mut self, index: usize) -> bool {
        if index < self.pending_approval.len() {
            self.pending_approval.remove(index);
            true
        } else {
            false
        }
    }

    /// Get pending approval list
    pub fn pending_approvals(&self) -> &[(String, BrainAction)] {
        &self.pending_approval
    }

    /// Get the event bus for subscribing to events
    pub fn event_bus(&self) -> &EventBus {
        &self.event_bus
    }

    /// Set cooldown duration (I-AUTO-001)
    pub fn set_cooldown(&mut self, secs: u64) {
        self.cooldown_secs = secs;
    }
}
