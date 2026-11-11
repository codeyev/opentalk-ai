//! The unit of audio that flows through the pipeline.

use std::time::Duration;

use serde::{Deserialize, Serialize};

/// Where a piece of audio came from. Sources are kept on separate tracks so
/// the UI can label who is speaking.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AudioSource {
    /// The user's microphone.
    Microphone,
    /// Audio played by the computer, such as the other side of a call.
    System,
}

/// Largest number of samples one frame may hold: one second at 48 kHz.
/// Longer input is split, so a single frame can never grow without limit.
pub const MAX_FRAME_SAMPLES: usize = 48_000;

/// A short run of normalized audio from one source.
#[derive(Debug, Clone, PartialEq)]
pub struct AudioFrame {
    pub source: AudioSource,
    /// Time since the capture session started, from a monotonic clock.
    pub captured_at: Duration,
    /// Samples per second. This is the device's native rate; frames are not
    /// resampled at this stage.
    pub sample_rate: u32,
    /// Mono samples in the range -1.0 to 1.0.
    pub samples: Vec<f32>,
}

impl AudioFrame {
    /// Splits `samples` into frames of at most [`MAX_FRAME_SAMPLES`] samples.
    /// Returns no frames for empty input.
    #[must_use]
    pub fn split(
        source: AudioSource,
        captured_at: Duration,
        sample_rate: u32,
        samples: &[f32],
    ) -> Vec<Self> {
        samples
            .chunks(MAX_FRAME_SAMPLES)
            .map(|chunk| Self {
                source,
                captured_at,
                sample_rate,
                samples: chunk.to_vec(),
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_input_produces_no_frames() {
        let frames = AudioFrame::split(AudioSource::Microphone, Duration::ZERO, 48_000, &[]);
        assert!(frames.is_empty());
    }

    #[test]
    fn short_input_becomes_one_frame() {
        let frames = AudioFrame::split(
            AudioSource::System,
            Duration::from_millis(20),
            44_100,
            &[0.5; 3],
        );
        assert_eq!(frames.len(), 1);
        assert_eq!(frames[0].source, AudioSource::System);
        assert_eq!(frames[0].captured_at, Duration::from_millis(20));
        assert_eq!(frames[0].sample_rate, 44_100);
        assert_eq!(frames[0].samples, vec![0.5; 3]);
    }

    #[test]
    fn long_input_is_split_into_bounded_frames() {
        let samples = vec![0.0; MAX_FRAME_SAMPLES * 2 + 1];
        let frames = AudioFrame::split(AudioSource::Microphone, Duration::ZERO, 48_000, &samples);
        let lengths: Vec<usize> = frames.iter().map(|frame| frame.samples.len()).collect();
        assert_eq!(lengths, vec![MAX_FRAME_SAMPLES, MAX_FRAME_SAMPLES, 1]);
    }
}
