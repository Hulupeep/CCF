//! Discord Bot Channel
//!
//! Invariants:
//! - I-CHAN-001: Implements ChatChannel trait
//! - I-CHAN-003: Bot token from MBOT_DISCORD_TOKEN env var

#[cfg(feature = "discord")]
use async_trait::async_trait;
#[cfg(feature = "discord")]
use tokio::sync::mpsc;

#[cfg(feature = "discord")]
use super::{ChannelType, ChatChannel, InboundMessage, OutboundMessage};
#[cfg(feature = "discord")]
use crate::brain::error::{BrainError, BrainResult};

/// Discord bot channel
#[cfg(feature = "discord")]
pub struct DiscordChannel {
    token: String,
    inbound_tx: mpsc::Sender<InboundMessage>,
    running: bool,
}

#[cfg(feature = "discord")]
impl DiscordChannel {
    /// Create new Discord channel (I-CHAN-003: token from env)
    pub fn new(inbound_tx: mpsc::Sender<InboundMessage>) -> BrainResult<Self> {
        let token = std::env::var("MBOT_DISCORD_TOKEN")
            .map_err(|_| BrainError::ConfigError("MBOT_DISCORD_TOKEN not set".into()))?;

        Ok(Self {
            token,
            inbound_tx,
            running: false,
        })
    }
}

#[cfg(feature = "discord")]
#[async_trait]
impl ChatChannel for DiscordChannel {
    async fn start(&mut self) -> BrainResult<()> {
        if self.running {
            return Ok(());
        }

        tracing::info!("Starting Discord bot...");
        self.running = true;

        // TODO: Start serenity client in background task

        Ok(())
    }

    async fn stop(&mut self) -> BrainResult<()> {
        self.running = false;
        tracing::info!("Discord bot stopped");
        Ok(())
    }

    async fn send(&self, user_id: &str, message: &OutboundMessage) -> BrainResult<()> {
        if !self.running {
            return Err(BrainError::ChannelError("Discord not running".into()));
        }

        // TODO: Use serenity to send message to channel/user
        tracing::debug!("Discord -> {}: {}", user_id, message.content);

        Ok(())
    }

    fn channel_type(&self) -> ChannelType {
        ChannelType::Discord
    }
}
