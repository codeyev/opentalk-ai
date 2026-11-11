//! Typed capture errors.
//!
//! Messages describe devices and operating-system failures only. They never
//! contain audio or conversation content, so they are safe to log and show.

use serde::Serialize;
use thiserror::Error;

use crate::frame::AudioSource;

#[derive(Debug, Clone, PartialEq, Eq, Error, Serialize)]
#[serde(
    tag = "code",
    rename_all = "snake_case",
    rename_all_fields = "camelCase"
)]
pub enum CaptureError {
    #[error("no audio source was selected")]
    NoSourceSelected,

    #[error("the microphone selection is not valid")]
    InvalidMicrophoneSelection,

    #[error("capture is already running")]
    AlreadyRunning,

    #[error("permission to capture {audio_source:?} audio was denied")]
    PermissionDenied { audio_source: AudioSource },

    #[error("no usable {audio_source:?} audio device was found")]
    DeviceUnavailable { audio_source: AudioSource },

    #[error("{audio_source:?} audio capture is not supported on this system")]
    Unsupported { audio_source: AudioSource },

    #[error("{audio_source:?} audio capture was interrupted: {detail}")]
    Interrupted {
        audio_source: AudioSource,
        detail: String,
    },

    #[error("{audio_source:?} audio capture failed: {detail}")]
    Backend {
        audio_source: AudioSource,
        detail: String,
    },
}
