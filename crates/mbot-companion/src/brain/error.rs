//! Brain layer error types
//!
//! Invariant I-BRAIN-007: Graceful degradation when LLM unavailable

#[cfg(feature = "brain")]
use thiserror::Error;

#[cfg(feature = "brain")]
#[derive(Error, Debug)]
pub enum BrainError {
    #[error("LLM provider error: {0}")]
    LlmError(String),

    #[error("LLM request timed out after {0}s")]
    LlmTimeout(u64),

    #[error("No LLM providers available")]
    NoProvidersAvailable,

    #[error("Safety filter rejected action: {0}")]
    SafetyViolation(String),

    #[error("Memory store error: {0}")]
    MemoryError(String),

    #[error("Channel error: {0}")]
    ChannelError(String),

    #[error("Voice pipeline error: {0}")]
    VoiceError(String),

    #[error("Autonomy engine error: {0}")]
    AutonomyError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("SQLite error: {0}")]
    SqliteError(#[from] rusqlite::Error),

    #[error("JSON serialization error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

#[cfg(feature = "brain")]
pub type BrainResult<T> = Result<T, BrainError>;
