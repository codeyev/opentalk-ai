//! Normalized transcription results, independent of any provider.

use opentalk_audio_core::AudioSource;
use serde::Serialize;

use crate::provider::TranscriptionError;

/// One stretch of speech from one source.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TranscriptSegment {
    /// Stable for the life of the segment: a partial segment and the final
    /// one that replaces it share the same id.
    pub id: String,
    pub source: AudioSource,
    /// When the speech started, in milliseconds since capture began.
    pub start_ms: u64,
    pub text: String,
    /// BCP 47 tag of the spoken language, or `None` if it is not known.
    pub language: Option<String>,
    /// Partial segments may still change; final segments will not.
    pub is_final: bool,
}

/// What the transcription pipeline reports to the application.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TranscriptEvent {
    /// A new segment, or newer text for an existing one (matched by id).
    Segment { segment: TranscriptSegment },
    /// A segment that turned out to contain no speech and should be removed.
    Discarded { id: String },
    /// Transcription for this source has stopped. Capture is not affected.
    Failed {
        source: AudioSource,
        error: TranscriptionError,
    },
}
