//! Anthropic Claude API Provider
//!
//! Invariants:
//! - I-BRAIN-002: Implements LlmProvider trait
//! - I-BRAIN-003: Request timeout <=30s
//! - I-BRAIN-008: API key from env var only

#[cfg(feature = "brain")]
use async_trait::async_trait;
#[cfg(feature = "brain")]
use reqwest::Client;
#[cfg(feature = "brain")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "brain")]
use std::sync::atomic::{AtomicBool, Ordering};
#[cfg(feature = "brain")]
use std::time::Duration;

#[cfg(feature = "brain")]
use super::{LlmMessage, LlmProvider, LlmResponse, LlmRole};
#[cfg(feature = "brain")]
use super::config::LlmConfig;
#[cfg(feature = "brain")]
use crate::brain::error::{BrainError, BrainResult};

const ANTHROPIC_API_URL: &str = "https://api.anthropic.com/v1/messages";
const ANTHROPIC_VERSION: &str = "2023-06-01";

#[cfg(feature = "brain")]
#[derive(Serialize)]
struct ClaudeRequest {
    model: String,
    max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    messages: Vec<ClaudeMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
}

#[cfg(feature = "brain")]
#[derive(Serialize)]
struct ClaudeMessage {
    role: String,
    content: String,
}

#[cfg(feature = "brain")]
#[derive(Deserialize)]
struct ClaudeResponse {
    content: Vec<ClaudeContent>,
    model: String,
    stop_reason: Option<String>,
    usage: Option<ClaudeUsage>,
}

#[cfg(feature = "brain")]
#[derive(Deserialize)]
struct ClaudeContent {
    text: String,
}

#[cfg(feature = "brain")]
#[derive(Deserialize)]
struct ClaudeUsage {
    input_tokens: u32,
    output_tokens: u32,
}

#[cfg(feature = "brain")]
#[derive(Deserialize)]
struct ClaudeErrorResponse {
    error: ClaudeErrorDetail,
}

#[cfg(feature = "brain")]
#[derive(Deserialize)]
struct ClaudeErrorDetail {
    message: String,
}

/// Claude API LLM provider
#[cfg(feature = "brain")]
pub struct ClaudeProvider {
    client: Client,
    api_key: String,
    model: String,
    max_tokens: u32,
    temperature: f32,
    available: AtomicBool,
}

#[cfg(feature = "brain")]
impl ClaudeProvider {
    pub fn new(config: &LlmConfig) -> BrainResult<Self> {
        let api_key = config.anthropic_api_key.clone().ok_or_else(|| {
            BrainError::ConfigError("ANTHROPIC_API_KEY not set".into())
        })?;

        // I-BRAIN-003: timeout <=30s
        let timeout = Duration::from_secs(config.timeout_secs.min(30));

        let client = Client::builder()
            .timeout(timeout)
            .build()
            .map_err(|e| BrainError::LlmError(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self {
            client,
            api_key,
            model: config.claude_model.clone(),
            max_tokens: config.max_tokens,
            temperature: config.temperature,
            available: AtomicBool::new(true),
        })
    }

    fn build_request(&self, messages: &[LlmMessage]) -> ClaudeRequest {
        // Extract system message if present
        let system = messages
            .iter()
            .find(|m| m.role == LlmRole::System)
            .map(|m| m.content.clone());

        // Convert non-system messages
        let claude_messages: Vec<ClaudeMessage> = messages
            .iter()
            .filter(|m| m.role != LlmRole::System)
            .map(|m| ClaudeMessage {
                role: match m.role {
                    LlmRole::User => "user".to_string(),
                    LlmRole::Assistant => "assistant".to_string(),
                    LlmRole::System => unreachable!(),
                },
                content: m.content.clone(),
            })
            .collect();

        ClaudeRequest {
            model: self.model.clone(),
            max_tokens: self.max_tokens,
            system,
            messages: claude_messages,
            temperature: Some(self.temperature),
        }
    }
}

#[cfg(feature = "brain")]
#[async_trait]
impl LlmProvider for ClaudeProvider {
    async fn complete(&self, messages: &[LlmMessage]) -> BrainResult<LlmResponse> {
        let request = self.build_request(messages);

        let response = self
            .client
            .post(ANTHROPIC_API_URL)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", ANTHROPIC_VERSION)
            .header("content-type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    self.available.store(false, Ordering::Relaxed);
                    BrainError::LlmTimeout(30)
                } else {
                    self.available.store(false, Ordering::Relaxed);
                    BrainError::LlmError(format!("HTTP request failed: {}", e))
                }
            })?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            let error_msg = serde_json::from_str::<ClaudeErrorResponse>(&body)
                .map(|e| e.error.message)
                .unwrap_or(body);

            if status.as_u16() == 401 || status.as_u16() == 403 {
                self.available.store(false, Ordering::Relaxed);
            }

            return Err(BrainError::LlmError(format!(
                "Claude API error ({}): {}",
                status, error_msg
            )));
        }

        self.available.store(true, Ordering::Relaxed);

        let claude_response: ClaudeResponse = response
            .json()
            .await
            .map_err(|e| BrainError::LlmError(format!("Failed to parse response: {}", e)))?;

        let content = claude_response
            .content
            .into_iter()
            .map(|c| c.text)
            .collect::<Vec<_>>()
            .join("");

        Ok(LlmResponse {
            content,
            model: claude_response.model,
            input_tokens: claude_response.usage.as_ref().map(|u| u.input_tokens),
            output_tokens: claude_response.usage.as_ref().map(|u| u.output_tokens),
            finish_reason: claude_response.stop_reason,
        })
    }

    async fn complete_streaming(
        &self,
        messages: &[LlmMessage],
        _on_token: Box<dyn Fn(&str) + Send>,
    ) -> BrainResult<LlmResponse> {
        // For now, fall back to non-streaming
        // TODO: Implement SSE streaming with anthropic-beta header
        self.complete(messages).await
    }

    fn model_name(&self) -> &str {
        &self.model
    }

    fn is_available(&self) -> bool {
        self.available.load(Ordering::Relaxed)
    }
}
