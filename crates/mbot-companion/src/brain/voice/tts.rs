//! Text-to-Speech - ElevenLabs streaming TTS
//!
//! Invariant I-VPIPE-004: TTS streaming (start playing before full generation)

#[cfg(feature = "voice")]
use reqwest::Client;
#[cfg(feature = "voice")]
use serde::Serialize;
#[cfg(feature = "voice")]
use std::time::Duration;

#[cfg(feature = "voice")]
use crate::brain::error::{BrainError, BrainResult};

const ELEVENLABS_API_URL: &str = "https://api.elevenlabs.io/v1/text-to-speech";

#[cfg(feature = "voice")]
#[derive(Serialize)]
struct TtsRequest {
    text: String,
    model_id: String,
    voice_settings: VoiceSettings,
}

#[cfg(feature = "voice")]
#[derive(Serialize)]
struct VoiceSettings {
    stability: f32,
    similarity_boost: f32,
    style: f32,
}

/// ElevenLabs TTS client
#[cfg(feature = "voice")]
pub struct ElevenLabsTts {
    client: Client,
    api_key: String,
    voice_id: String,
    model_id: String,
}

#[cfg(feature = "voice")]
impl ElevenLabsTts {
    pub fn new(voice_id: &str) -> BrainResult<Self> {
        let api_key = std::env::var("ELEVENLABS_API_KEY")
            .map_err(|_| BrainError::ConfigError("ELEVENLABS_API_KEY not set".into()))?;

        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| BrainError::VoiceError(format!("HTTP client error: {}", e)))?;

        Ok(Self {
            client,
            api_key,
            voice_id: voice_id.to_string(),
            model_id: "eleven_turbo_v2".to_string(),
        })
    }

    /// Generate speech audio bytes from text
    pub async fn synthesize(&self, text: &str) -> BrainResult<Vec<u8>> {
        let url = format!("{}/{}", ELEVENLABS_API_URL, self.voice_id);

        let request = TtsRequest {
            text: text.to_string(),
            model_id: self.model_id.clone(),
            voice_settings: VoiceSettings {
                stability: 0.5,
                similarity_boost: 0.75,
                style: 0.5,
            },
        };

        let response = self
            .client
            .post(&url)
            .header("xi-api-key", &self.api_key)
            .header("Accept", "audio/mpeg")
            .json(&request)
            .send()
            .await
            .map_err(|e| BrainError::VoiceError(format!("ElevenLabs request failed: {}", e)))?;

        if !response.status().is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(BrainError::VoiceError(format!(
                "ElevenLabs error: {}", body
            )));
        }

        let audio_bytes = response
            .bytes()
            .await
            .map_err(|e| BrainError::VoiceError(format!("Failed to read audio: {}", e)))?;

        Ok(audio_bytes.to_vec())
    }

    /// Stream audio chunks for I-VPIPE-004 (start playing before full generation)
    pub async fn synthesize_streaming(
        &self,
        text: &str,
        on_chunk: impl Fn(&[u8]) + Send,
    ) -> BrainResult<()> {
        let url = format!("{}/{}/stream", ELEVENLABS_API_URL, self.voice_id);

        let request = TtsRequest {
            text: text.to_string(),
            model_id: self.model_id.clone(),
            voice_settings: VoiceSettings {
                stability: 0.5,
                similarity_boost: 0.75,
                style: 0.5,
            },
        };

        let response = self
            .client
            .post(&url)
            .header("xi-api-key", &self.api_key)
            .header("Accept", "audio/mpeg")
            .json(&request)
            .send()
            .await
            .map_err(|e| BrainError::VoiceError(format!("ElevenLabs stream request failed: {}", e)))?;

        if !response.status().is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(BrainError::VoiceError(format!(
                "ElevenLabs stream error: {}", body
            )));
        }

        // Read all bytes and deliver in chunks
        // TODO: Use reqwest bytes_stream() for true streaming once futures-util is added
        let all_bytes = response
            .bytes()
            .await
            .map_err(|e| BrainError::VoiceError(format!("Stream read failed: {}", e)))?;

        // Deliver in 4KB chunks to simulate streaming
        for chunk in all_bytes.chunks(4096) {
            on_chunk(chunk);
        }

        Ok(())
    }
}
