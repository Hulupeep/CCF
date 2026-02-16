//! Audio Capture - Microphone input using cpal
//!
//! Invariant I-VPIPE-003: Audio capture 16kHz mono PCM

#[cfg(feature = "voice")]
use crate::brain::error::{BrainError, BrainResult};

#[cfg(feature = "voice")]
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
#[cfg(feature = "voice")]
use std::sync::{Arc, Mutex};
#[cfg(feature = "voice")]
use tokio::sync::mpsc;

/// Audio capture from microphone (I-VPIPE-003: 16kHz mono PCM)
#[cfg(feature = "voice")]
pub struct AudioCapture {
    sample_rate: u32,
    buffer: Arc<Mutex<Vec<i16>>>,
    running: Arc<std::sync::atomic::AtomicBool>,
}

#[cfg(feature = "voice")]
impl AudioCapture {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            sample_rate,
            buffer: Arc::new(Mutex::new(Vec::new())),
            running: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }

    /// Start capturing audio from the default input device
    pub fn start(&self) -> BrainResult<()> {
        let host = cpal::default_host();
        let device = host.default_input_device().ok_or_else(|| {
            BrainError::VoiceError("No audio input device found".into())
        })?;

        let config = cpal::StreamConfig {
            channels: 1, // mono
            sample_rate: cpal::SampleRate(self.sample_rate),
            buffer_size: cpal::BufferSize::Default,
        };

        let buffer = Arc::clone(&self.buffer);
        let running = Arc::clone(&self.running);
        running.store(true, std::sync::atomic::Ordering::Relaxed);

        let stream = device.build_input_stream(
            &config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                if !running.load(std::sync::atomic::Ordering::Relaxed) {
                    return;
                }
                // Convert f32 samples to i16 PCM
                let samples: Vec<i16> = data
                    .iter()
                    .map(|&s| (s * 32767.0).clamp(-32768.0, 32767.0) as i16)
                    .collect();

                if let Ok(mut buf) = buffer.lock() {
                    buf.extend_from_slice(&samples);
                }
            },
            |err| {
                tracing::error!("Audio capture error: {}", err);
            },
            None,
        ).map_err(|e| BrainError::VoiceError(format!("Failed to build input stream: {}", e)))?;

        stream.play().map_err(|e| {
            BrainError::VoiceError(format!("Failed to start audio stream: {}", e))
        })?;

        // Keep stream alive by leaking it (will be stopped when running is set to false)
        std::mem::forget(stream);

        Ok(())
    }

    /// Stop capturing
    pub fn stop(&self) {
        self.running.store(false, std::sync::atomic::Ordering::Relaxed);
    }

    /// Take and clear the current audio buffer
    pub fn take_buffer(&self) -> Vec<i16> {
        let mut buf = self.buffer.lock().unwrap();
        std::mem::take(&mut *buf)
    }

    /// Get the current buffer size in samples
    pub fn buffer_len(&self) -> usize {
        self.buffer.lock().unwrap().len()
    }

    /// Whether capture is running
    pub fn is_running(&self) -> bool {
        self.running.load(std::sync::atomic::Ordering::Relaxed)
    }
}
