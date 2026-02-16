//! Audio Playback - Speaker output using rodio

#[cfg(feature = "voice")]
use crate::brain::error::{BrainError, BrainResult};
#[cfg(feature = "voice")]
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};
#[cfg(feature = "voice")]
use std::io::Cursor;

/// Audio playback via speakers
#[cfg(feature = "voice")]
pub struct AudioPlayback {
    _stream: OutputStream,
    stream_handle: OutputStreamHandle,
}

#[cfg(feature = "voice")]
impl AudioPlayback {
    pub fn new() -> BrainResult<Self> {
        let (stream, stream_handle) = OutputStream::try_default()
            .map_err(|e| BrainError::VoiceError(format!("No audio output device: {}", e)))?;

        Ok(Self {
            _stream: stream,
            stream_handle,
        })
    }

    /// Play audio from MP3 bytes (from ElevenLabs)
    pub fn play_mp3(&self, mp3_data: &[u8]) -> BrainResult<()> {
        let cursor = Cursor::new(mp3_data.to_vec());
        let source = Decoder::new(cursor)
            .map_err(|e| BrainError::VoiceError(format!("Failed to decode audio: {}", e)))?;

        let sink = Sink::try_new(&self.stream_handle)
            .map_err(|e| BrainError::VoiceError(format!("Failed to create sink: {}", e)))?;

        sink.append(source);
        sink.sleep_until_end();

        Ok(())
    }

    /// Play audio from raw PCM i16 samples
    pub fn play_pcm(&self, samples: &[i16], sample_rate: u32) -> BrainResult<()> {
        let source = rodio::buffer::SamplesBuffer::new(1, sample_rate, samples.to_vec());

        let sink = Sink::try_new(&self.stream_handle)
            .map_err(|e| BrainError::VoiceError(format!("Failed to create sink: {}", e)))?;

        sink.append(source);
        sink.sleep_until_end();

        Ok(())
    }

    /// Create a non-blocking sink for streaming playback (I-VPIPE-004)
    pub fn create_streaming_sink(&self) -> BrainResult<Sink> {
        Sink::try_new(&self.stream_handle)
            .map_err(|e| BrainError::VoiceError(format!("Failed to create sink: {}", e)))
    }
}
