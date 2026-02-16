//! Event Bus - tokio::broadcast for autonomy events

#[cfg(feature = "brain")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "brain")]
use tokio::sync::broadcast;

/// Events that flow through the autonomy system
#[cfg(feature = "brain")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AutoEvent {
    /// Robot state changed significantly
    StateChange { field: String, old: f32, new: f32 },
    /// User interaction detected
    UserPresent { channel: String, user_id: String },
    /// Scheduled time trigger
    CronTrigger { schedule_name: String },
    /// Inactivity threshold reached
    InactivityAlert { idle_secs: u64 },
    /// Battery/energy low
    LowEnergy { level: f32 },
    /// Custom event
    Custom { name: String, data: String },
}

/// Broadcast event bus for autonomy events
#[cfg(feature = "brain")]
pub struct EventBus {
    sender: broadcast::Sender<AutoEvent>,
}

#[cfg(feature = "brain")]
impl EventBus {
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self { sender }
    }

    /// Publish an event to all subscribers
    pub fn publish(&self, event: AutoEvent) -> usize {
        self.sender.send(event).unwrap_or(0)
    }

    /// Subscribe to events
    pub fn subscribe(&self) -> broadcast::Receiver<AutoEvent> {
        self.sender.subscribe()
    }
}
