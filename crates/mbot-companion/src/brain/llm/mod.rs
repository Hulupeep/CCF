//! LLM Provider Abstraction Layer
//!
//! # Invariants
//! - **I-BRAIN-001**: No LLM code in mbot-core
//! - **I-BRAIN-002**: LlmProvider must be trait-based with >=2 implementations
//! - **I-BRAIN-003**: All LLM requests timeout <=30s
//! - **I-BRAIN-007**: Graceful degradation when LLM unavailable
//! - **I-BRAIN-008**: API keys from env vars only

#[cfg(feature = "brain")]
pub mod config;
#[cfg(feature = "brain")]
pub mod claude;
#[cfg(feature = "brain")]
pub mod ollama;

#[cfg(feature = "brain")]
use crate::brain::error::{BrainError, BrainResult};

#[cfg(feature = "brain")]
use async_trait::async_trait;
#[cfg(feature = "brain")]
use serde::{Deserialize, Serialize};

/// Role in a conversation message
#[cfg(feature = "brain")]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum LlmRole {
    System,
    User,
    Assistant,
}

/// A single message in a conversation
#[cfg(feature = "brain")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmMessage {
    pub role: LlmRole,
    pub content: String,
}

#[cfg(feature = "brain")]
impl LlmMessage {
    pub fn system(content: impl Into<String>) -> Self {
        Self { role: LlmRole::System, content: content.into() }
    }

    pub fn user(content: impl Into<String>) -> Self {
        Self { role: LlmRole::User, content: content.into() }
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self { role: LlmRole::Assistant, content: content.into() }
    }
}

/// Response from an LLM provider
#[cfg(feature = "brain")]
#[derive(Debug, Clone)]
pub struct LlmResponse {
    pub content: String,
    pub model: String,
    pub input_tokens: Option<u32>,
    pub output_tokens: Option<u32>,
    pub finish_reason: Option<String>,
}

/// Trait for LLM providers (I-BRAIN-002: trait-based, >=2 implementations)
#[cfg(feature = "brain")]
#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// Send a completion request (I-BRAIN-003: must timeout <=30s)
    async fn complete(&self, messages: &[LlmMessage]) -> BrainResult<LlmResponse>;

    /// Send a streaming completion request
    async fn complete_streaming(
        &self,
        messages: &[LlmMessage],
        on_token: Box<dyn Fn(&str) + Send>,
    ) -> BrainResult<LlmResponse>;

    /// Get the model name this provider uses
    fn model_name(&self) -> &str;

    /// Check if this provider is currently available
    fn is_available(&self) -> bool;
}

/// Provider chain - tries providers in order until one succeeds (I-BRAIN-007)
#[cfg(feature = "brain")]
#[derive(Clone)]
pub struct ProviderChain {
    providers: Vec<std::sync::Arc<dyn LlmProvider>>,
}

#[cfg(feature = "brain")]
impl ProviderChain {
    pub fn new() -> Self {
        Self { providers: Vec::new() }
    }

    pub fn add_provider(mut self, provider: std::sync::Arc<dyn LlmProvider>) -> Self {
        self.providers.push(provider);
        self
    }

    /// Try each provider in order; return first success or NoProvidersAvailable
    pub async fn complete(&self, messages: &[LlmMessage]) -> BrainResult<LlmResponse> {
        for provider in &self.providers {
            if !provider.is_available() {
                tracing::debug!("Provider {} not available, skipping", provider.model_name());
                continue;
            }

            match provider.complete(messages).await {
                Ok(response) => return Ok(response),
                Err(e) => {
                    tracing::warn!(
                        "Provider {} failed: {}, trying next",
                        provider.model_name(),
                        e
                    );
                    continue;
                }
            }
        }

        Err(BrainError::NoProvidersAvailable)
    }

    /// Try streaming with each provider in order
    pub async fn complete_streaming(
        &self,
        messages: &[LlmMessage],
        on_token: Box<dyn Fn(&str) + Send>,
    ) -> BrainResult<LlmResponse> {
        for provider in &self.providers {
            if !provider.is_available() {
                continue;
            }

            match provider.complete_streaming(messages, on_token).await {
                Ok(response) => return Ok(response),
                Err(e) => {
                    tracing::warn!(
                        "Provider {} streaming failed: {}, trying next",
                        provider.model_name(),
                        e
                    );
                    // Need to create a new callback for next provider
                    return Err(e);
                }
            }
        }

        Err(BrainError::NoProvidersAvailable)
    }

    /// Check if any provider is available
    pub fn any_available(&self) -> bool {
        self.providers.iter().any(|p| p.is_available())
    }

    /// Get all provider names
    pub fn provider_names(&self) -> Vec<&str> {
        self.providers.iter().map(|p| p.model_name()).collect()
    }
}

#[cfg(feature = "brain")]
impl Default for ProviderChain {
    fn default() -> Self {
        Self::new()
    }
}
