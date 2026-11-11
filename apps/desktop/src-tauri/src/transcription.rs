//! Connects captured audio to the transcription provider and the UI.
//!
//! When the user has turned transcription on, each captured source gets its
//! own pipeline. Frames reach the pipelines through bounded queues, and
//! results reach the UI as [`TRANSCRIPT_EVENT`] events. With transcription
//! off, none of this exists and no audio leaves the capture session.

use std::sync::Arc;

use opentalk_audio_core::{AudioFrame, AudioSource, CaptureConfig};
use opentalk_provider_openai::OpenAiTranscriber;
use opentalk_provider_whisper::WhisperTranscriber;
use opentalk_provider_whisper::models as whisper_models;
use opentalk_replies::ReplyCoordinator;
use opentalk_transcription::{
    ProviderCapabilities, StreamingTranscriber, TranscriptEvent, TranscriptSegment,
    TranscriptionConfig, TranscriptionError, pipeline,
};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};
use tokio::sync::mpsc;

use crate::replies;
use crate::secrets::{self, KeyOwner, SecretError};
use crate::translation::SegmentTranslator;
use crate::whisper;

/// Event carrying a [`TranscriptEvent`].
pub const TRANSCRIPT_EVENT: &str = "transcript-event";

/// Frames that may wait for a pipeline: a few seconds of audio per source.
/// When a pipeline falls further behind than this, new frames are dropped.
const FRAME_QUEUE_CAPACITY: usize = 256;
/// Transcript events that may wait to be sent to the UI.
const EVENT_QUEUE_CAPACITY: usize = 64;
/// Longest language tag accepted from the UI.
const MAX_LANGUAGE_TAG_LENGTH: usize = 35;

/// The UI's request to transcribe a capture session.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TranscriptionRequest {
    pub provider: TranscriptionProvider,
    /// BCP 47 tag of the spoken language, or `None` to let the provider decide.
    pub language: Option<String>,
}

/// The transcription provider the user chose.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(tag = "id", rename_all = "lowercase", deny_unknown_fields)]
pub enum TranscriptionProvider {
    /// OpenAI's streaming transcription, a cloud service.
    Openai,
    /// The Whisper model in this file of the models folder, on this computer.
    Whisper { model: String },
}

impl TranscriptionProvider {
    /// Whether using this provider sends audio to another machine.
    pub fn is_cloud(&self) -> bool {
        matches!(self, Self::Openai)
    }
}

/// Creates the transcriber the user chose.
fn transcriber_for(
    app: &AppHandle,
    provider: &TranscriptionProvider,
) -> Result<Arc<dyn StreamingTranscriber>, TranscriptionError> {
    Ok(match provider {
        TranscriptionProvider::Openai => {
            let api_key = secrets::api_key(opentalk_provider_openai::PROVIDER_ID)
                .ok()
                .flatten()
                .ok_or(TranscriptionError::MissingApiKey)?;
            Arc::new(OpenAiTranscriber::new(api_key))
        }
        TranscriptionProvider::Whisper { model } => {
            let path = whisper::models_directory(app)
                .and_then(|directory| whisper_models::installed_path(&directory, model))
                .ok_or(TranscriptionError::ModelUnavailable)?;
            Arc::new(WhisperTranscriber::load(&path)?)
        }
    })
}

/// The cloud transcription provider as shown in settings.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderStatus {
    #[serde(flatten)]
    pub capabilities: ProviderCapabilities,
    pub has_api_key: bool,
}

/// Whether `tag` has the shape of a BCP 47 language tag. Applied to tags received from the UI.
pub fn is_valid_language_tag(tag: &str) -> bool {
    !tag.is_empty()
        && tag.len() <= MAX_LANGUAGE_TAG_LENGTH
        && tag
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || character == '-')
}

/// What else happens to finished transcript segments in a session.
#[derive(Default)]
pub struct SegmentConsumers {
    pub translator: Option<SegmentTranslator>,
    pub replies: Option<Arc<ReplyCoordinator>>,
}

impl SegmentConsumers {
    fn consume(&self, segment: &TranscriptSegment) {
        if let Some(translator) = &self.translator {
            translator.submit(segment);
        }
        if let Some(coordinator) = &self.replies {
            replies::heard(coordinator, segment);
        }
    }
}

/// Sends each captured frame to the pipeline for its source.
pub struct FrameRouter {
    microphone: Option<mpsc::Sender<AudioFrame>>,
    system: Option<mpsc::Sender<AudioFrame>>,
}

impl FrameRouter {
    /// A router that discards everything, for sessions without transcription.
    pub fn disabled() -> Self {
        Self {
            microphone: None,
            system: None,
        }
    }

    /// Hands a frame to its pipeline without waiting. If the pipeline's
    /// queue is full the frame is dropped, which keeps memory bounded.
    pub fn route(&self, frame: AudioFrame) {
        let queue = match frame.source {
            AudioSource::Microphone => &self.microphone,
            AudioSource::System => &self.system,
        };
        if let Some(queue) = queue {
            let _ = queue.try_send(frame);
        }
    }
}

/// Starts one transcription pipeline per captured source. Segments are
/// also handed to `consumers` for translation and reply suggestions.
///
/// The pipelines end by themselves when the returned router is dropped,
/// which happens when the capture session ends.
pub fn start(
    app: &AppHandle,
    capture: &CaptureConfig,
    request: TranscriptionRequest,
    consumers: SegmentConsumers,
) -> Result<FrameRouter, TranscriptionError> {
    if let Some(language) = &request.language
        && !is_valid_language_tag(language)
    {
        return Err(TranscriptionError::UnsupportedLanguage {
            language: language.chars().take(MAX_LANGUAGE_TAG_LENGTH).collect(),
        });
    }
    let transcriber = transcriber_for(app, &request.provider)?;
    let config = TranscriptionConfig {
        language: request.language,
    };

    let (events, mut event_receiver) = mpsc::channel(EVENT_QUEUE_CAPACITY);
    let pipeline_for = |source, enabled: bool| {
        enabled.then(|| {
            let (frames, frame_receiver) = mpsc::channel(FRAME_QUEUE_CAPACITY);
            tauri::async_runtime::spawn(pipeline::run(
                source,
                Arc::clone(&transcriber),
                config.clone(),
                frame_receiver,
                events.clone(),
            ));
            frames
        })
    };
    let router = FrameRouter {
        microphone: pipeline_for(AudioSource::Microphone, capture.microphone),
        system: pipeline_for(AudioSource::System, capture.system_audio),
    };

    // Forwards results to the UI until every pipeline has finished.
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        while let Some(event) = event_receiver.recv().await {
            if let TranscriptEvent::Segment { segment } = &event {
                consumers.consume(segment);
            }
            let _ = app.emit(TRANSCRIPT_EVENT, event);
        }
    });
    Ok(router)
}

/// Describes the cloud transcription provider and whether a key is saved for it.
#[tauri::command(async)]
pub fn transcription_provider() -> Result<ProviderStatus, SecretError> {
    let capabilities = opentalk_provider_openai::capabilities();
    let has_api_key = secrets::api_key(capabilities.id)?.is_some();
    Ok(ProviderStatus {
        capabilities,
        has_api_key,
    })
}

/// Saves the API key for one service in the system credential store.
/// `base_url` says which custom service the key is for.
#[tauri::command(async)]
pub fn save_api_key(
    provider: KeyOwner,
    api_key: String,
    base_url: Option<String>,
) -> Result<(), SecretError> {
    let id = secrets::key_id(provider, base_url.as_deref()).ok_or(SecretError::InvalidService)?;
    secrets::save_api_key(&id, &api_key)
}

/// Removes the API key for one service from the system credential store.
#[tauri::command(async)]
pub fn delete_api_key(provider: KeyOwner, base_url: Option<String>) -> Result<(), SecretError> {
    let id = secrets::key_id(provider, base_url.as_deref()).ok_or(SecretError::InvalidService)?;
    secrets::delete_api_key(&id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn language_tags_are_checked_before_use() {
        for tag in ["en", "ar-EG", "zh-Hant-TW"] {
            assert!(is_valid_language_tag(tag), "{tag} should be accepted");
        }
        for tag in [
            "",
            "en US",
            "en_US",
            "<script>",
            &"a".repeat(MAX_LANGUAGE_TAG_LENGTH + 1),
        ] {
            assert!(!is_valid_language_tag(tag), "{tag:?} should be rejected");
        }
    }

    #[test]
    fn only_the_cloud_transcriber_counts_as_cloud() {
        assert!(TranscriptionProvider::Openai.is_cloud());
        let local = TranscriptionProvider::Whisper {
            model: "ggml-base.bin".to_owned(),
        };
        assert!(!local.is_cloud());
    }

    #[test]
    fn a_disabled_router_discards_frames() {
        let frame = AudioFrame {
            source: AudioSource::Microphone,
            captured_at: std::time::Duration::ZERO,
            sample_rate: 48_000,
            samples: vec![0.0; 4],
        };
        FrameRouter::disabled().route(frame);
    }

    #[test]
    fn a_full_pipeline_queue_drops_frames_instead_of_waiting() {
        let (frames, mut receiver) = mpsc::channel(1);
        let router = FrameRouter {
            microphone: Some(frames),
            system: None,
        };
        let frame = |marker| AudioFrame {
            source: AudioSource::Microphone,
            captured_at: std::time::Duration::ZERO,
            sample_rate: 48_000,
            samples: vec![marker],
        };
        router.route(frame(1.0));
        router.route(frame(2.0));
        assert_eq!(
            receiver.try_recv().map(|frame| frame.samples),
            Ok(vec![1.0])
        );
        assert!(receiver.try_recv().is_err());
    }
}
