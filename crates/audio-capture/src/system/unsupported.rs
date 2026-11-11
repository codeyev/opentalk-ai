//! Placeholder for platforms without a system-audio adapter.

use opentalk_audio_core::{AudioSource, CaptureError};

use crate::sink::SourceSink;

/// Never constructed: system audio cannot be captured on this platform.
pub(crate) enum SystemAudioStream {}

pub(crate) fn start(_sink: SourceSink) -> Result<SystemAudioStream, CaptureError> {
    Err(CaptureError::Unsupported {
        audio_source: AudioSource::System,
    })
}
