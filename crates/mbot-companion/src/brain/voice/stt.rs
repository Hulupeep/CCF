//! Speech-to-Text (voice feature)
//!
//! Re-exports core STT providers from brain::stt, plus adds PCM-to-WAV encoding
//! for local microphone capture (requires hound via voice feature).

#[cfg(feature = "voice")]
pub use crate::brain::stt::{
    GroqWhisperProvider, SttChain, SttProvider, WhisperProvider,
};

/// Encode PCM i16 samples into a WAV byte buffer.
///
/// Used when audio comes from local microphone capture (cpal) as raw PCM
/// samples rather than as a pre-encoded file from a phone upload.
#[cfg(feature = "voice")]
pub fn encode_pcm_to_wav(audio: &[i16], sample_rate: u32) -> Vec<u8> {
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut cursor = std::io::Cursor::new(Vec::new());
    {
        let mut writer = hound::WavWriter::new(&mut cursor, spec).expect("WAV writer creation");
        for &sample in audio {
            writer.write_sample(sample).expect("WAV sample write");
        }
        writer.finalize().expect("WAV finalize");
    }
    cursor.into_inner()
}

#[cfg(all(test, feature = "voice"))]
mod tests {
    use super::*;

    #[test]
    fn test_encode_pcm_to_wav() {
        let samples: Vec<i16> = vec![0; 16000]; // 1 second of silence at 16kHz
        let wav = encode_pcm_to_wav(&samples, 16000);
        assert!(wav.len() > 44);
        assert_eq!(&wav[0..4], b"RIFF");
        assert_eq!(&wav[8..12], b"WAVE");
    }
}
