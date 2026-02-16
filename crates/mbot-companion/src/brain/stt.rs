//! Speech-to-Text Providers
//!
//! Trait-based STT with provider chain fallback. Supports raw audio bytes
//! (from phone uploads via voice API) in WAV, WebM, MP3, etc.
//!
//! Gated on `brain` feature so both `voice` and `voice-api` can use STT.
//!
//! Invariants:
//! - I-VCONV-005: API keys from env vars only
//! - I-VPIPE-001: End-to-end voice latency <3s target (10s timeout for STT API)
//! - I-VPIPE-006: No audio stored beyond session unless opt-in

use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use crate::brain::error::{BrainError, BrainResult};

/// Timeout for STT API calls in seconds (I-VPIPE-001)
const STT_TIMEOUT_SECS: u64 = 10;

/// OpenAI Whisper API endpoint
const WHISPER_API_URL: &str = "https://api.openai.com/v1/audio/transcriptions";

/// Groq Whisper API endpoint
const GROQ_API_URL: &str = "https://api.groq.com/openai/v1/audio/transcriptions";

/// Speech-to-text provider trait
///
/// Implementations accept raw audio bytes (WAV, MP3, WebM, etc.) and a format
/// hint string. The provider sends the audio to a cloud STT API and returns
/// the transcribed text.
#[async_trait]
pub trait SttProvider: Send + Sync {
    /// Transcribe raw audio bytes to text.
    async fn transcribe(&self, audio: &[u8], format: &str) -> Result<String, BrainError>;

    /// Human-readable provider name for logging
    fn provider_name(&self) -> &str;

    /// Whether this provider is currently available
    fn is_available(&self) -> bool;
}

/// Whisper API response format (shared between OpenAI and Groq)
#[derive(Deserialize)]
struct WhisperResponse {
    text: String,
}

// ---------------------------------------------------------------------------
// OpenAI Whisper Provider
// ---------------------------------------------------------------------------

/// OpenAI Whisper API provider (whisper-1 model)
///
/// Requires `OPENAI_API_KEY` environment variable (I-VCONV-005).
pub struct WhisperProvider {
    client: Client,
    api_key: String,
    available: AtomicBool,
}

impl WhisperProvider {
    pub fn new(api_key: String) -> BrainResult<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(STT_TIMEOUT_SECS))
            .build()
            .map_err(|e| BrainError::VoiceError(format!("HTTP client error: {}", e)))?;

        Ok(Self {
            client,
            api_key,
            available: AtomicBool::new(true),
        })
    }

    pub fn from_env() -> BrainResult<Self> {
        let api_key = std::env::var("OPENAI_API_KEY")
            .map_err(|_| BrainError::ConfigError("OPENAI_API_KEY not set".into()))?;
        Self::new(api_key)
    }

    pub fn mime_for_format(format: &str) -> &'static str {
        match format {
            "wav" => "audio/wav",
            "mp3" => "audio/mpeg",
            "webm" => "audio/webm",
            "ogg" => "audio/ogg",
            "flac" => "audio/flac",
            "m4a" => "audio/mp4",
            _ => "application/octet-stream",
        }
    }

    pub fn extension_for_format(format: &str) -> &'static str {
        match format {
            "wav" => "wav",
            "mp3" => "mp3",
            "ogg" => "ogg",
            "flac" => "flac",
            "m4a" => "m4a",
            "webm" => "webm",
            _ => "wav",
        }
    }
}

#[async_trait]
impl SttProvider for WhisperProvider {
    async fn transcribe(&self, audio: &[u8], format: &str) -> Result<String, BrainError> {
        let mime = Self::mime_for_format(format);
        let ext = Self::extension_for_format(format);

        let part = reqwest::multipart::Part::bytes(audio.to_vec())
            .file_name(format!("audio.{}", ext))
            .mime_str(mime)
            .map_err(|e| BrainError::VoiceError(format!("MIME error: {}", e)))?;

        let form = reqwest::multipart::Form::new()
            .text("model", "whisper-1")
            .text("response_format", "json")
            .part("file", part);

        let response = self
            .client
            .post(WHISPER_API_URL)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .multipart(form)
            .send()
            .await
            .map_err(|e| {
                self.available.store(false, Ordering::Relaxed);
                if e.is_timeout() {
                    BrainError::VoiceError("Whisper API timed out".into())
                } else {
                    BrainError::VoiceError(format!("Whisper API request failed: {}", e))
                }
            })?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            if status.as_u16() == 401 || status.as_u16() == 403 {
                self.available.store(false, Ordering::Relaxed);
            }
            return Err(BrainError::VoiceError(format!(
                "Whisper API error ({}): {}",
                status, body
            )));
        }

        self.available.store(true, Ordering::Relaxed);

        let result: WhisperResponse = response
            .json()
            .await
            .map_err(|e| BrainError::VoiceError(format!("Parse Whisper response: {}", e)))?;

        Ok(result.text)
    }

    fn provider_name(&self) -> &str {
        "whisper-api"
    }

    fn is_available(&self) -> bool {
        self.available.load(Ordering::Relaxed)
    }
}

// ---------------------------------------------------------------------------
// Groq Whisper Provider
// ---------------------------------------------------------------------------

/// Groq Whisper API provider (whisper-large-v3 model, faster and free tier)
///
/// Requires `GROQ_API_KEY` environment variable (I-VCONV-005).
pub struct GroqWhisperProvider {
    client: Client,
    api_key: String,
    available: AtomicBool,
}

impl GroqWhisperProvider {
    pub fn new(api_key: String) -> BrainResult<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(STT_TIMEOUT_SECS))
            .build()
            .map_err(|e| BrainError::VoiceError(format!("HTTP client error: {}", e)))?;

        Ok(Self {
            client,
            api_key,
            available: AtomicBool::new(true),
        })
    }

    pub fn from_env() -> BrainResult<Self> {
        let api_key = std::env::var("GROQ_API_KEY")
            .map_err(|_| BrainError::ConfigError("GROQ_API_KEY not set".into()))?;
        Self::new(api_key)
    }
}

#[async_trait]
impl SttProvider for GroqWhisperProvider {
    async fn transcribe(&self, audio: &[u8], format: &str) -> Result<String, BrainError> {
        let mime = WhisperProvider::mime_for_format(format);
        let ext = WhisperProvider::extension_for_format(format);

        let part = reqwest::multipart::Part::bytes(audio.to_vec())
            .file_name(format!("audio.{}", ext))
            .mime_str(mime)
            .map_err(|e| BrainError::VoiceError(format!("MIME error: {}", e)))?;

        let form = reqwest::multipart::Form::new()
            .text("model", "whisper-large-v3")
            .text("response_format", "json")
            .part("file", part);

        let response = self
            .client
            .post(GROQ_API_URL)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .multipart(form)
            .send()
            .await
            .map_err(|e| {
                self.available.store(false, Ordering::Relaxed);
                if e.is_timeout() {
                    BrainError::VoiceError("Groq API timed out".into())
                } else {
                    BrainError::VoiceError(format!("Groq API request failed: {}", e))
                }
            })?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            if status.as_u16() == 401 || status.as_u16() == 403 {
                self.available.store(false, Ordering::Relaxed);
            }
            return Err(BrainError::VoiceError(format!(
                "Groq API error ({}): {}",
                status, body
            )));
        }

        self.available.store(true, Ordering::Relaxed);

        let result: WhisperResponse = response
            .json()
            .await
            .map_err(|e| BrainError::VoiceError(format!("Parse Groq response: {}", e)))?;

        Ok(result.text)
    }

    fn provider_name(&self) -> &str {
        "groq-whisper"
    }

    fn is_available(&self) -> bool {
        self.available.load(Ordering::Relaxed)
    }
}

// ---------------------------------------------------------------------------
// STT Provider Chain (fallback)
// ---------------------------------------------------------------------------

/// Chain of STT providers with automatic fallback.
///
/// Tries each provider in order until one succeeds. Groq is added first by
/// `from_env()` since it is faster and has a free tier; OpenAI Whisper is
/// the fallback.
pub struct SttChain {
    providers: Vec<Box<dyn SttProvider>>,
}

impl SttChain {
    pub fn new() -> Self {
        Self {
            providers: Vec::new(),
        }
    }

    /// Auto-detect available providers from environment variables.
    pub fn from_env() -> Self {
        let mut chain = Self::new();

        // Groq first: faster and has a free tier
        if std::env::var("GROQ_API_KEY").is_ok() {
            if let Ok(provider) = GroqWhisperProvider::from_env() {
                tracing::info!("STT chain: added Groq Whisper provider");
                chain.add(Box::new(provider));
            }
        }

        // OpenAI Whisper as fallback
        if std::env::var("OPENAI_API_KEY").is_ok() {
            if let Ok(provider) = WhisperProvider::from_env() {
                tracing::info!("STT chain: added OpenAI Whisper provider");
                chain.add(Box::new(provider));
            }
        }

        if chain.providers.is_empty() {
            tracing::warn!(
                "STT chain: no providers available. \
                 Set GROQ_API_KEY or OPENAI_API_KEY to enable speech-to-text."
            );
        }

        chain
    }

    pub fn add(&mut self, provider: Box<dyn SttProvider>) {
        self.providers.push(provider);
    }

    pub fn provider_count(&self) -> usize {
        self.providers.len()
    }

    pub fn has_available_provider(&self) -> bool {
        self.providers.iter().any(|p| p.is_available())
    }

    pub fn provider_names(&self) -> Vec<&str> {
        self.providers.iter().map(|p| p.provider_name()).collect()
    }

    /// Transcribe audio bytes by trying each provider in order.
    pub async fn transcribe(&self, audio: &[u8], format: &str) -> Result<String, BrainError> {
        if self.providers.is_empty() {
            return Err(BrainError::VoiceError(
                "No STT providers configured. Set GROQ_API_KEY or OPENAI_API_KEY.".into(),
            ));
        }

        let mut last_error = None;

        for provider in &self.providers {
            if !provider.is_available() {
                tracing::debug!(
                    "STT chain: skipping unavailable provider '{}'",
                    provider.provider_name()
                );
                continue;
            }

            tracing::debug!(
                "STT chain: trying provider '{}'",
                provider.provider_name()
            );

            match provider.transcribe(audio, format).await {
                Ok(text) => {
                    tracing::debug!(
                        "STT chain: '{}' succeeded ({} chars)",
                        provider.provider_name(),
                        text.len()
                    );
                    return Ok(text);
                }
                Err(e) => {
                    tracing::warn!(
                        "STT chain: '{}' failed: {}",
                        provider.provider_name(),
                        e
                    );
                    last_error = Some(e);
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            BrainError::VoiceError("All STT providers unavailable".into())
        }))
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mime_for_format() {
        assert_eq!(WhisperProvider::mime_for_format("wav"), "audio/wav");
        assert_eq!(WhisperProvider::mime_for_format("mp3"), "audio/mpeg");
        assert_eq!(WhisperProvider::mime_for_format("webm"), "audio/webm");
        assert_eq!(WhisperProvider::mime_for_format("ogg"), "audio/ogg");
        assert_eq!(WhisperProvider::mime_for_format("flac"), "audio/flac");
        assert_eq!(WhisperProvider::mime_for_format("m4a"), "audio/mp4");
        assert_eq!(
            WhisperProvider::mime_for_format("unknown"),
            "application/octet-stream"
        );
    }

    #[test]
    fn test_extension_for_format() {
        assert_eq!(WhisperProvider::extension_for_format("wav"), "wav");
        assert_eq!(WhisperProvider::extension_for_format("mp3"), "mp3");
        assert_eq!(WhisperProvider::extension_for_format("webm"), "webm");
        assert_eq!(WhisperProvider::extension_for_format("unknown"), "wav");
    }

    #[test]
    fn test_stt_chain_empty() {
        let chain = SttChain::new();
        assert_eq!(chain.provider_count(), 0);
        assert!(!chain.has_available_provider());
        assert!(chain.provider_names().is_empty());
    }

    #[tokio::test]
    async fn test_stt_chain_no_providers_returns_error() {
        let chain = SttChain::new();
        let result = chain.transcribe(b"fake audio", "wav").await;
        assert!(result.is_err());
        match result.unwrap_err() {
            BrainError::VoiceError(msg) => {
                assert!(msg.contains("No STT providers configured"));
            }
            other => panic!("Expected VoiceError, got: {:?}", other),
        }
    }

    #[test]
    fn test_whisper_provider_missing_key() {
        std::env::remove_var("OPENAI_API_KEY");
        let result = WhisperProvider::from_env();
        assert!(result.is_err());
    }

    #[test]
    fn test_groq_provider_missing_key() {
        std::env::remove_var("GROQ_API_KEY");
        let result = GroqWhisperProvider::from_env();
        assert!(result.is_err());
    }
}
