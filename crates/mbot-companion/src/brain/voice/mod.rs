//! Real-Time Voice Pipeline
//!
//! Invariants:
//! - I-VPIPE-001: End-to-end voice latency <3s target
//! - I-VPIPE-002: VAD prevents false triggers in ambient noise
//! - I-VPIPE-003: Audio capture 16kHz mono PCM
//! - I-VPIPE-004: TTS streaming (start playing before full generation)
//! - I-VPIPE-005: Voice pipeline disableable (enabled: bool)
//! - I-VPIPE-006: No audio stored beyond session unless opt-in

#[cfg(feature = "voice")]
pub mod capture;
#[cfg(feature = "voice")]
pub mod vad;
#[cfg(feature = "voice")]
pub mod stt;
#[cfg(feature = "voice")]
pub mod tts;
#[cfg(feature = "voice")]
pub mod playback;
#[cfg(feature = "voice")]
pub mod session;

#[cfg(feature = "voice")]
use crate::brain::error::{BrainError, BrainResult};

/// Voice pipeline configuration
#[cfg(feature = "voice")]
#[derive(Debug, Clone)]
pub struct VoiceConfig {
    /// Whether the voice pipeline is enabled (I-VPIPE-005)
    pub enabled: bool,
    /// Sample rate for audio capture (I-VPIPE-003: 16000)
    pub sample_rate: u32,
    /// VAD energy threshold (I-VPIPE-002)
    pub vad_threshold: f32,
    /// Minimum speech duration in ms before processing
    pub min_speech_ms: u32,
    /// Maximum recording duration in seconds
    pub max_record_secs: u32,
    /// Whether to store audio beyond the session (I-VPIPE-006: default false)
    pub persist_audio: bool,
    /// ElevenLabs API key (from env)
    pub elevenlabs_api_key: Option<String>,
    /// ElevenLabs voice ID
    pub elevenlabs_voice_id: String,
    /// STT provider to use
    pub stt_provider: SttProviderType,
}

#[cfg(feature = "voice")]
#[derive(Debug, Clone)]
pub enum SttProviderType {
    WhisperApi,
    GroqWhisper,
}

#[cfg(feature = "voice")]
impl Default for VoiceConfig {
    fn default() -> Self {
        Self {
            enabled: false, // I-VPIPE-005: disabled by default
            sample_rate: 16000, // I-VPIPE-003
            vad_threshold: 0.02,
            min_speech_ms: 500,
            max_record_secs: 30,
            persist_audio: false, // I-VPIPE-006
            elevenlabs_api_key: std::env::var("ELEVENLABS_API_KEY").ok(),
            elevenlabs_voice_id: std::env::var("MBOT_VOICE_ID")
                .unwrap_or_else(|_| "default".to_string()),
            stt_provider: SttProviderType::WhisperApi,
        }
    }
}

/// Top-level voice pipeline state machine
#[cfg(feature = "voice")]
pub struct VoicePipeline {
    config: VoiceConfig,
    state: VoiceState,
}

#[cfg(feature = "voice")]
#[derive(Debug, Clone, PartialEq)]
pub enum VoiceState {
    Disabled,
    Idle,
    Listening,
    Recording,
    Transcribing,
    Thinking,
    Speaking,
}

#[cfg(feature = "voice")]
impl VoicePipeline {
    pub fn new(config: VoiceConfig) -> Self {
        let state = if config.enabled {
            VoiceState::Idle
        } else {
            VoiceState::Disabled
        };

        Self { config, state }
    }

    pub fn state(&self) -> &VoiceState {
        &self.state
    }

    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Enable the voice pipeline (I-VPIPE-005)
    pub fn enable(&mut self) {
        self.config.enabled = true;
        if self.state == VoiceState::Disabled {
            self.state = VoiceState::Idle;
        }
    }

    /// Disable the voice pipeline (I-VPIPE-005)
    pub fn disable(&mut self) {
        self.config.enabled = false;
        self.state = VoiceState::Disabled;
    }

    /// Transition to a new state
    pub fn transition(&mut self, new_state: VoiceState) -> BrainResult<()> {
        if !self.config.enabled && new_state != VoiceState::Disabled {
            return Err(BrainError::VoiceError("Voice pipeline is disabled".into()));
        }

        tracing::debug!("Voice: {:?} -> {:?}", self.state, new_state);
        self.state = new_state;
        Ok(())
    }
}
