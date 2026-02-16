//! Chat Channels - Telegram, Discord, and more
//!
//! Invariants:
//! - I-CHAN-001: Channels implement ChatChannel trait
//! - I-CHAN-002: Rate limit: 20 msg/min per user
//! - I-CHAN-003: Bot tokens from env vars only

#[cfg(feature = "telegram")]
pub mod telegram;
#[cfg(feature = "discord")]
pub mod discord;

#[cfg(feature = "brain")]
use async_trait::async_trait;
#[cfg(feature = "brain")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "brain")]
use std::collections::HashMap;
#[cfg(feature = "brain")]
use std::sync::Mutex;
#[cfg(feature = "brain")]
use std::time::Instant;

#[cfg(feature = "brain")]
use crate::brain::error::BrainResult;

/// Channel types
#[cfg(feature = "brain")]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChannelType {
    Local,
    Telegram,
    Discord,
    Voice,
}

/// Inbound message from a chat channel
#[cfg(feature = "brain")]
#[derive(Debug, Clone)]
pub struct InboundMessage {
    pub channel: ChannelType,
    pub user_id: String,
    pub user_name: Option<String>,
    pub content: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Outbound message to a chat channel
#[cfg(feature = "brain")]
#[derive(Debug, Clone)]
pub struct OutboundMessage {
    pub content: String,
    pub reply_to: Option<String>,
}

/// Chat channel trait (I-CHAN-001)
#[cfg(feature = "brain")]
#[async_trait]
pub trait ChatChannel: Send + Sync {
    async fn start(&mut self) -> BrainResult<()>;
    async fn stop(&mut self) -> BrainResult<()>;
    async fn send(&self, user_id: &str, message: &OutboundMessage) -> BrainResult<()>;
    fn channel_type(&self) -> ChannelType;
}

/// Rate limiter for chat messages (I-CHAN-002: 20 msg/min per user)
#[cfg(feature = "brain")]
pub struct MessageRateLimiter {
    /// user_id -> list of message timestamps
    windows: Mutex<HashMap<String, Vec<Instant>>>,
    max_per_minute: usize,
}

#[cfg(feature = "brain")]
impl MessageRateLimiter {
    pub fn new(max_per_minute: usize) -> Self {
        Self {
            windows: Mutex::new(HashMap::new()),
            max_per_minute,
        }
    }

    /// Check if a message from this user is allowed
    pub fn check(&self, user_id: &str) -> bool {
        let mut windows = self.windows.lock().unwrap();
        let now = Instant::now();
        let window = std::time::Duration::from_secs(60);

        let entries = windows.entry(user_id.to_string()).or_default();

        // Remove expired entries
        entries.retain(|t| now.duration_since(*t) < window);

        if entries.len() >= self.max_per_minute {
            false
        } else {
            entries.push(now);
            true
        }
    }
}

#[cfg(feature = "brain")]
impl Default for MessageRateLimiter {
    fn default() -> Self {
        Self::new(20) // I-CHAN-002: default 20 msg/min
    }
}

/// Routes messages between channels and the brain
#[cfg(feature = "brain")]
pub struct MessageRouter {
    channels: Vec<Box<dyn ChatChannel>>,
    rate_limiter: MessageRateLimiter,
}

#[cfg(feature = "brain")]
impl MessageRouter {
    pub fn new() -> Self {
        Self {
            channels: Vec::new(),
            rate_limiter: MessageRateLimiter::default(),
        }
    }

    pub fn add_channel(&mut self, channel: Box<dyn ChatChannel>) {
        self.channels.push(channel);
    }

    /// Send a message to a specific channel type
    pub async fn send_to_channel(
        &self,
        channel_type: &ChannelType,
        user_id: &str,
        message: &OutboundMessage,
    ) -> BrainResult<()> {
        for channel in &self.channels {
            if &channel.channel_type() == channel_type {
                return channel.send(user_id, message).await;
            }
        }
        Ok(())
    }

    /// Check rate limit for an inbound message
    pub fn check_rate_limit(&self, user_id: &str) -> bool {
        self.rate_limiter.check(user_id)
    }

    /// Start all channels
    pub async fn start_all(&mut self) -> BrainResult<()> {
        for channel in &mut self.channels {
            channel.start().await?;
        }
        Ok(())
    }

    /// Stop all channels
    pub async fn stop_all(&mut self) -> BrainResult<()> {
        for channel in &mut self.channels {
            channel.stop().await?;
        }
        Ok(())
    }
}

#[cfg(feature = "brain")]
impl Default for MessageRouter {
    fn default() -> Self {
        Self::new()
    }
}
