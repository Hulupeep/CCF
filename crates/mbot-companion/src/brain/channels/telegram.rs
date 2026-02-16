//! Telegram Bot Channel
//!
//! Invariants:
//! - I-CHAN-001: Implements ChatChannel trait
//! - I-CHAN-003: Bot token from MBOT_TELEGRAM_TOKEN env var

#[cfg(feature = "telegram")]
use async_trait::async_trait;
#[cfg(feature = "telegram")]
use tokio::sync::mpsc;

#[cfg(feature = "telegram")]
use super::{ChannelType, ChatChannel, InboundMessage, OutboundMessage};
#[cfg(feature = "telegram")]
use crate::brain::error::{BrainError, BrainResult};

/// Telegram bot channel
#[cfg(feature = "telegram")]
pub struct TelegramChannel {
    token: String,
    inbound_tx: mpsc::Sender<InboundMessage>,
    running: bool,
}

#[cfg(feature = "telegram")]
impl TelegramChannel {
    /// Create new Telegram channel (I-CHAN-003: token from env)
    pub fn new(inbound_tx: mpsc::Sender<InboundMessage>) -> BrainResult<Self> {
        let token = std::env::var("MBOT_TELEGRAM_TOKEN")
            .map_err(|_| BrainError::ConfigError("MBOT_TELEGRAM_TOKEN not set".into()))?;

        Ok(Self {
            token,
            inbound_tx,
            running: false,
        })
    }
}

#[cfg(feature = "telegram")]
#[async_trait]
impl ChatChannel for TelegramChannel {
    async fn start(&mut self) -> BrainResult<()> {
        if self.running {
            return Ok(());
        }

        tracing::info!("Starting Telegram bot...");
        self.running = true;

        // TODO: Start teloxide bot in background task
        // The bot will forward messages via inbound_tx

        Ok(())
    }

    async fn stop(&mut self) -> BrainResult<()> {
        self.running = false;
        tracing::info!("Telegram bot stopped");
        Ok(())
    }

    async fn send(&self, user_id: &str, message: &OutboundMessage) -> BrainResult<()> {
        if !self.running {
            return Err(BrainError::ChannelError("Telegram not running".into()));
        }

        // TODO: Use teloxide to send message to chat_id
        tracing::debug!("Telegram -> {}: {}", user_id, message.content);

        Ok(())
    }

    fn channel_type(&self) -> ChannelType {
        ChannelType::Telegram
    }
}
