//! The contract between the pipeline and a transcription provider.

use async_trait::async_trait;
use serde::Serialize;
use thiserror::Error;
use tokio::sync::mpsc;

/// Whether a provider processes audio on this machine or on a remote service.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ProviderLocation {
    Local,
    Cloud,
}

/// What a provider can do, as far as the provider documents it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderCapabilities {
    pub id: &'static str,
    pub display_name: &'static str,
    pub location: ProviderLocation,
    /// Whether partial text arrives while someone is still speaking.
    pub streams_partial_results: bool,
    /// Whether the provider reports which language it heard.
    pub detects_language: bool,
    /// Languages known to be supported, as BCP 47 tags. `None` means the
    /// provider does not publish a list, so support is only known by trying.
    pub languages: Option<&'static [&'static str]>,
    /// The mono 16-bit PCM sample rate the provider expects.
    pub input_sample_rate: u32,
}

/// Settings for one transcription session.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TranscriptionConfig {
    /// BCP 47 tag of the spoken language, or `None` to let the provider decide.
    pub language: Option<String>,
}

/// Audio sent to a provider. Turns are numbered from zero in the order
/// their [`AudioInput::EndOfTurn`] markers are sent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AudioInput {
    /// Mono 16-bit PCM at the provider's `input_sample_rate`.
    Samples(Vec<i16>),
    /// The speaker paused: everything since the previous marker is one turn.
    EndOfTurn,
}

/// What a provider reports back, already normalized.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProviderEvent {
    /// The text recognized so far for a turn that is still being processed.
    Partial { turn: u64, text: String },
    /// The finished text for a turn.
    Final {
        turn: u64,
        text: String,
        language: Option<String>,
    },
    /// The provider could not transcribe this one turn; later turns still work.
    TurnFailed { turn: u64 },
    /// The session is over. No further events follow.
    Failed(TranscriptionError),
}

/// Why transcription could not start or stopped. Messages describe the
/// connection and the provider's status only, never transcript text.
#[derive(Debug, Clone, PartialEq, Eq, Error, Serialize)]
#[serde(tag = "code", rename_all = "snake_case")]
pub enum TranscriptionError {
    #[error("no API key is saved for the transcription provider")]
    MissingApiKey,
    #[error("the transcription provider rejected the API key")]
    AuthenticationFailed,
    #[error("the transcription provider is rate limiting requests")]
    RateLimited,
    #[error("the transcription provider does not support the language {language}")]
    UnsupportedLanguage { language: String },
    #[error("could not connect to the transcription provider: {detail}")]
    ConnectionFailed { detail: String },
    #[error("the connection to the transcription provider was lost: {detail}")]
    ConnectionLost { detail: String },
    #[error("the transcription provider did not respond in time")]
    Timeout,
    #[error("the transcription provider sent a response that could not be understood")]
    MalformedResponse,
    #[error("the selected model is not available")]
    ModelUnavailable,
    #[error("the transcription provider reported an error: {detail}")]
    Provider { detail: String },
}

impl TranscriptionError {
    /// Whether opening a fresh session is likely to help.
    #[must_use]
    pub fn is_transient(&self) -> bool {
        matches!(self, Self::ConnectionLost { .. } | Self::Timeout)
    }
}

/// How many audio chunks may wait to be sent to a provider.
pub const AUDIO_QUEUE_CAPACITY: usize = 64;
/// How many provider events may wait to be read by the pipeline.
pub const EVENT_QUEUE_CAPACITY: usize = 64;

/// An open connection to a provider.
///
/// Send audio through `audio`; read results from `events`. Both queues are
/// bounded. Dropping the session, or just its `audio` sender, ends it.
#[derive(Debug)]
pub struct TranscriptionSession {
    pub audio: mpsc::Sender<AudioInput>,
    pub events: mpsc::Receiver<ProviderEvent>,
}

impl TranscriptionSession {
    /// Creates a session together with the ends the provider side works with.
    #[must_use]
    pub fn channel() -> (
        Self,
        mpsc::Receiver<AudioInput>,
        mpsc::Sender<ProviderEvent>,
    ) {
        let (audio, provider_audio) = mpsc::channel(AUDIO_QUEUE_CAPACITY);
        let (provider_events, events) = mpsc::channel(EVENT_QUEUE_CAPACITY);
        (Self { audio, events }, provider_audio, provider_events)
    }
}

/// A speech-to-text provider that accepts audio as it is captured.
#[async_trait]
pub trait StreamingTranscriber: Send + Sync {
    fn capabilities(&self) -> ProviderCapabilities;

    /// Opens a session. Resolves once the provider is ready for audio.
    ///
    /// # Errors
    ///
    /// Returns an error if the provider cannot be reached, rejects the
    /// credentials, or does not accept the configuration.
    async fn open(
        &self,
        config: &TranscriptionConfig,
    ) -> Result<TranscriptionSession, TranscriptionError>;
}
