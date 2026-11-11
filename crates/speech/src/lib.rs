//! Speech output: turning text into audio through a provider.
//!
//! The app reads translations and reply suggestions aloud. The voices
//! installed on the computer are used directly by the UI; this crate is
//! the interface for providers that synthesize speech elsewhere, such as
//! a cloud service or a speech server on this computer.
//!
//! Every provider delivers the same audio format, so the rest of the app
//! never deals with a vendor's encoding.

use async_trait::async_trait;
use thiserror::Error;

/// Sample rate of the audio every provider delivers.
pub const SAMPLE_RATE_HZ: u32 = 24_000;

/// Longest text accepted for one request, in characters. One thing said
/// in a conversation is far shorter; this bounds cost and delay.
pub const MAX_TEXT_CHARS: usize = 2_000;

/// Why a text could not be turned into speech.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum SpeechError {
    #[error("the provider rejected the API key")]
    AuthenticationFailed,
    #[error("the provider is limiting requests, or the account is out of quota")]
    RateLimited,
    #[error("the provider does not know the selected voice or model")]
    ModelUnavailable,
    #[error("the provider could not be reached")]
    ConnectionFailed,
    #[error("the provider did not respond in time")]
    Timeout,
    #[error("the provider's reply was not audio")]
    MalformedResponse,
    #[error("the provider reported an error")]
    Provider,
}

/// A provider that turns text into speech.
#[async_trait]
pub trait SpeechSynthesizer: Send + Sync {
    /// Speaks `text`, which is written in the language of the BCP 47 tag
    /// `language`.
    ///
    /// Returns the audio as mono, 16-bit little-endian PCM at
    /// [`SAMPLE_RATE_HZ`].
    async fn synthesize(&self, text: &str, language: &str) -> Result<Vec<u8>, SpeechError>;
}

/// Checks that a provider's reply is usable PCM audio.
///
/// # Errors
///
/// Returns [`SpeechError::MalformedResponse`] if there is no audio at all.
pub fn checked_pcm(mut audio: Vec<u8>) -> Result<Vec<u8>, SpeechError> {
    // A sample is two bytes; a trailing odd byte cannot be played.
    audio.truncate(audio.len() - audio.len() % 2);
    if audio.is_empty() {
        Err(SpeechError::MalformedResponse)
    } else {
        Ok(audio)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn audio_must_hold_at_least_one_whole_sample() {
        assert_eq!(checked_pcm(vec![1, 2, 3, 4]), Ok(vec![1, 2, 3, 4]));
        assert_eq!(checked_pcm(vec![1, 2, 3]), Ok(vec![1, 2]));
        assert_eq!(checked_pcm(vec![1]), Err(SpeechError::MalformedResponse));
        assert_eq!(checked_pcm(Vec::new()), Err(SpeechError::MalformedResponse));
    }
}
