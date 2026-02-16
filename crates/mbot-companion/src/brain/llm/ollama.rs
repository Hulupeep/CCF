//! Ollama Local LLM Provider
//!
//! Invariants:
//! - I-BRAIN-002: Implements LlmProvider trait (second implementation)
//! - I-BRAIN-003: Request timeout <=30s
//! - I-BRAIN-007: Graceful degradation when unavailable

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

#[cfg(feature = "brain")]
#[derive(Serialize)]
struct OllamaRequest {
    model: String,
    messages: Vec<OllamaMessage>,
    stream: bool,
    options: OllamaOptions,
}

#[cfg(feature = "brain")]
#[derive(Serialize)]
struct OllamaMessage {
    role: String,
    content: String,
}

#[cfg(feature = "brain")]
#[derive(Serialize)]
struct OllamaOptions {
    temperature: f32,
    num_predict: u32,
}

#[cfg(feature = "brain")]
#[derive(Deserialize)]
struct OllamaResponse {
    message: OllamaResponseMessage,
    model: String,
    done: bool,
    #[serde(default)]
    prompt_eval_count: Option<u32>,
    #[serde(default)]
    eval_count: Option<u32>,
    #[serde(default)]
    done_reason: Option<String>,
}

#[cfg(feature = "brain")]
#[derive(Deserialize)]
struct OllamaResponseMessage {
    content: String,
}

/// Ollama local LLM provider
#[cfg(feature = "brain")]
pub struct OllamaProvider {
    client: Client,
    base_url: String,
    model: String,
    max_tokens: u32,
    temperature: f32,
    available: AtomicBool,
}

#[cfg(feature = "brain")]
impl OllamaProvider {
    pub fn new(config: &LlmConfig) -> BrainResult<Self> {
        // I-BRAIN-003: timeout <=30s
        let timeout = Duration::from_secs(config.timeout_secs.min(30));

        let client = Client::builder()
            .timeout(timeout)
            .build()
            .map_err(|e| BrainError::LlmError(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self {
            client,
            base_url: config.ollama_base_url.clone(),
            model: config.ollama_model.clone(),
            max_tokens: config.max_tokens,
            temperature: config.temperature,
            available: AtomicBool::new(true),
        })
    }

    /// Check if Ollama is running by hitting the version endpoint
    pub async fn health_check(&self) -> bool {
        let url = format!("{}/api/version", self.base_url);
        match self.client.get(&url).send().await {
            Ok(resp) => {
                let ok = resp.status().is_success();
                self.available.store(ok, Ordering::Relaxed);
                ok
            }
            Err(_) => {
                self.available.store(false, Ordering::Relaxed);
                false
            }
        }
    }

    fn build_messages(&self, messages: &[LlmMessage]) -> Vec<OllamaMessage> {
        messages
            .iter()
            .map(|m| OllamaMessage {
                role: match m.role {
                    LlmRole::System => "system".to_string(),
                    LlmRole::User => "user".to_string(),
                    LlmRole::Assistant => "assistant".to_string(),
                },
                content: m.content.clone(),
            })
            .collect()
    }
}

#[cfg(feature = "brain")]
#[async_trait]
impl LlmProvider for OllamaProvider {
    async fn complete(&self, messages: &[LlmMessage]) -> BrainResult<LlmResponse> {
        let url = format!("{}/api/chat", self.base_url);

        let request = OllamaRequest {
            model: self.model.clone(),
            messages: self.build_messages(messages),
            stream: false,
            options: OllamaOptions {
                temperature: self.temperature,
                num_predict: self.max_tokens,
            },
        };

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                self.available.store(false, Ordering::Relaxed);
                if e.is_timeout() {
                    BrainError::LlmTimeout(30)
                } else if e.is_connect() {
                    BrainError::LlmError("Ollama not running. Start with: ollama serve".into())
                } else {
                    BrainError::LlmError(format!("Ollama request failed: {}", e))
                }
            })?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            self.available.store(false, Ordering::Relaxed);
            return Err(BrainError::LlmError(format!(
                "Ollama error ({}): {}",
                status, body
            )));
        }

        self.available.store(true, Ordering::Relaxed);

        let ollama_response: OllamaResponse = response
            .json()
            .await
            .map_err(|e| BrainError::LlmError(format!("Failed to parse Ollama response: {}", e)))?;

        Ok(LlmResponse {
            content: ollama_response.message.content,
            model: ollama_response.model,
            input_tokens: ollama_response.prompt_eval_count,
            output_tokens: ollama_response.eval_count,
            finish_reason: ollama_response.done_reason,
        })
    }

    async fn complete_streaming(
        &self,
        messages: &[LlmMessage],
        _on_token: Box<dyn Fn(&str) + Send>,
    ) -> BrainResult<LlmResponse> {
        // For now, fall back to non-streaming
        // TODO: Implement Ollama streaming (stream: true + NDJSON)
        self.complete(messages).await
    }

    fn model_name(&self) -> &str {
        &self.model
    }

    fn is_available(&self) -> bool {
        self.available.load(Ordering::Relaxed)
    }
}
