//! LLM Configuration
//!
//! Invariant I-BRAIN-008: API keys from env vars only, never source code

#[cfg(feature = "brain")]
use crate::brain::error::{BrainError, BrainResult};

/// LLM provider configuration loaded from environment variables
#[cfg(feature = "brain")]
#[derive(Debug, Clone)]
pub struct LlmConfig {
    /// Anthropic API key (from ANTHROPIC_API_KEY env var)
    pub anthropic_api_key: Option<String>,
    /// Claude model to use (default: claude-sonnet-4-5-20250929)
    pub claude_model: String,
    /// Ollama base URL (default: http://localhost:11434)
    pub ollama_base_url: String,
    /// Ollama model to use (default: llama3.2)
    pub ollama_model: String,
    /// Request timeout in seconds (I-BRAIN-003: must be <=30)
    pub timeout_secs: u64,
    /// Max tokens to generate
    pub max_tokens: u32,
    /// Temperature for generation
    pub temperature: f32,
}

#[cfg(feature = "brain")]
impl LlmConfig {
    /// Load config from environment variables (I-BRAIN-008)
    pub fn from_env() -> BrainResult<Self> {
        let timeout_secs = std::env::var("MBOT_LLM_TIMEOUT_SECS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(30)
            .min(30); // I-BRAIN-003: enforce <=30s

        Ok(Self {
            anthropic_api_key: std::env::var("ANTHROPIC_API_KEY").ok(),
            claude_model: std::env::var("MBOT_CLAUDE_MODEL")
                .unwrap_or_else(|_| "claude-sonnet-4-5-20250929".to_string()),
            ollama_base_url: std::env::var("MBOT_OLLAMA_URL")
                .unwrap_or_else(|_| "http://localhost:11434".to_string()),
            ollama_model: std::env::var("MBOT_OLLAMA_MODEL")
                .unwrap_or_else(|_| "llama3.2".to_string()),
            timeout_secs,
            max_tokens: std::env::var("MBOT_LLM_MAX_TOKENS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(1024),
            temperature: std::env::var("MBOT_LLM_TEMPERATURE")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(0.7),
        })
    }

    /// Check if Claude API is configured
    pub fn has_claude(&self) -> bool {
        self.anthropic_api_key.as_ref().map_or(false, |k| !k.is_empty())
    }
}

#[cfg(feature = "brain")]
impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            anthropic_api_key: None,
            claude_model: "claude-sonnet-4-5-20250929".to_string(),
            ollama_base_url: "http://localhost:11434".to_string(),
            ollama_model: "llama3.2".to_string(),
            timeout_secs: 30,
            max_tokens: 1024,
            temperature: 0.7,
        }
    }
}
