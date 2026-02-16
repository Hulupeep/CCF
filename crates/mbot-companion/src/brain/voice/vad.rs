//! Voice Activity Detection
//!
//! Invariant I-VPIPE-002: VAD prevents false triggers in ambient noise
//! Uses energy-based detection with adaptive noise floor.

#[cfg(feature = "voice")]
pub struct VoiceActivityDetector {
    /// Energy threshold above noise floor
    threshold: f32,
    /// Smoothed noise floor estimate
    noise_floor: f32,
    /// Adaptation rate for noise floor
    adaptation_rate: f32,
    /// Consecutive frames of speech needed to trigger
    min_speech_frames: usize,
    /// Current consecutive speech frame count
    speech_count: usize,
    /// Consecutive silence frames needed to end speech
    min_silence_frames: usize,
    /// Current consecutive silence count
    silence_count: usize,
    /// Whether we're currently in speech
    in_speech: bool,
}

#[cfg(feature = "voice")]
impl VoiceActivityDetector {
    pub fn new(threshold: f32) -> Self {
        Self {
            threshold,
            noise_floor: 0.0,
            adaptation_rate: 0.01,
            min_speech_frames: 3,
            speech_count: 0,
            min_silence_frames: 10,
            silence_count: 0,
            in_speech: false,
        }
    }

    /// Process a frame of audio samples, returns true if speech is detected
    pub fn process_frame(&mut self, samples: &[i16]) -> bool {
        let energy = Self::frame_energy(samples);

        // Update noise floor during silence
        if !self.in_speech {
            self.noise_floor = self.noise_floor * (1.0 - self.adaptation_rate)
                + energy * self.adaptation_rate;
        }

        let is_speech_frame = energy > self.noise_floor + self.threshold;

        if is_speech_frame {
            self.speech_count += 1;
            self.silence_count = 0;

            if self.speech_count >= self.min_speech_frames {
                self.in_speech = true;
            }
        } else {
            self.silence_count += 1;
            self.speech_count = 0;

            if self.silence_count >= self.min_silence_frames && self.in_speech {
                self.in_speech = false;
            }
        }

        self.in_speech
    }

    /// Calculate RMS energy of a frame
    fn frame_energy(samples: &[i16]) -> f32 {
        if samples.is_empty() {
            return 0.0;
        }

        let sum_sq: f64 = samples
            .iter()
            .map(|&s| (s as f64) * (s as f64))
            .sum();

        (sum_sq / samples.len() as f64).sqrt() as f32
    }

    /// Reset the detector state
    pub fn reset(&mut self) {
        self.noise_floor = 0.0;
        self.speech_count = 0;
        self.silence_count = 0;
        self.in_speech = false;
    }

    /// Whether currently detecting speech
    pub fn is_speech(&self) -> bool {
        self.in_speech
    }

    /// Current noise floor estimate
    pub fn noise_floor(&self) -> f32 {
        self.noise_floor
    }
}

#[cfg(feature = "voice")]
impl Default for VoiceActivityDetector {
    fn default() -> Self {
        Self::new(200.0) // Default threshold above noise floor
    }
}
