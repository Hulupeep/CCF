//! Voice Session State Machine
//!
//! Manages the full voice interaction lifecycle:
//! Idle → Listening → Recording → Transcribing → Thinking → Speaking

#[cfg(feature = "voice")]
use crate::brain::error::{BrainError, BrainResult};
#[cfg(feature = "voice")]
use super::{VoiceState, capture::AudioCapture, vad::VoiceActivityDetector};

/// Voice session managing a single interaction cycle
#[cfg(feature = "voice")]
pub struct VoiceSession {
    state: VoiceState,
    capture: AudioCapture,
    vad: VoiceActivityDetector,
    recorded_audio: Vec<i16>,
    sample_rate: u32,
    max_record_samples: usize,
}

#[cfg(feature = "voice")]
impl VoiceSession {
    pub fn new(sample_rate: u32, vad_threshold: f32, max_record_secs: u32) -> Self {
        Self {
            state: VoiceState::Idle,
            capture: AudioCapture::new(sample_rate),
            vad: VoiceActivityDetector::new(vad_threshold),
            recorded_audio: Vec::new(),
            sample_rate,
            max_record_samples: (sample_rate * max_record_secs) as usize,
        }
    }

    pub fn state(&self) -> &VoiceState {
        &self.state
    }

    /// Start listening for speech
    pub fn start_listening(&mut self) -> BrainResult<()> {
        self.capture.start()?;
        self.state = VoiceState::Listening;
        self.vad.reset();
        self.recorded_audio.clear();
        Ok(())
    }

    /// Process audio frames - returns true when a complete utterance is captured
    pub fn process_frame(&mut self) -> bool {
        let buffer = self.capture.take_buffer();
        if buffer.is_empty() {
            return false;
        }

        let has_speech = self.vad.process_frame(&buffer);

        match self.state {
            VoiceState::Listening => {
                if has_speech {
                    self.state = VoiceState::Recording;
                    self.recorded_audio.extend_from_slice(&buffer);
                }
            }
            VoiceState::Recording => {
                self.recorded_audio.extend_from_slice(&buffer);

                // Check for end of speech or max duration
                if !has_speech || self.recorded_audio.len() >= self.max_record_samples {
                    self.state = VoiceState::Transcribing;
                    self.capture.stop();
                    return true; // Utterance complete
                }
            }
            _ => {}
        }

        false
    }

    /// Get the recorded audio for transcription
    pub fn recorded_audio(&self) -> &[i16] {
        &self.recorded_audio
    }

    /// Get the sample rate
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    /// Transition to thinking state (after transcription)
    pub fn start_thinking(&mut self) {
        self.state = VoiceState::Thinking;
    }

    /// Transition to speaking state (after LLM response)
    pub fn start_speaking(&mut self) {
        self.state = VoiceState::Speaking;
    }

    /// Return to idle state (after speaking completes)
    pub fn finish(&mut self) {
        self.state = VoiceState::Idle;
        self.recorded_audio.clear();
    }

    /// Cancel and return to idle
    pub fn cancel(&mut self) {
        self.capture.stop();
        self.state = VoiceState::Idle;
        self.recorded_audio.clear();
    }
}
