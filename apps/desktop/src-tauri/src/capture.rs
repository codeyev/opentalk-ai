//! IPC commands for audio capture.
//!
//! The UI starts and stops capture explicitly; nothing here starts on its
//! own. While a session runs, the UI receives [`LEVELS_EVENT`] updates, and
//! an [`INTERRUPTED_EVENT`] if capture ends by itself.

use std::sync::{Mutex, MutexGuard, PoisonError};

use opentalk_audio_capture::{CaptureObserver, CaptureSession, LevelsUpdate};
use opentalk_audio_core::{AudioFrame, CaptureConfig, CaptureError, MicrophoneInfo};
use opentalk_replies::ReplyError;
use opentalk_transcription::TranscriptionError;
use opentalk_translation::TranslationError;
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State};

use crate::providers::{self, PolicyError};
use crate::replies::{self, ReplyRequest, ReplyState};
use crate::transcription::{self, FrameRouter, SegmentConsumers, TranscriptionRequest};
use crate::translation::{self, TranslationRequest};

/// Event carrying a [`LevelsUpdate`] about ten times a second.
pub const LEVELS_EVENT: &str = "capture-levels";
/// Event carrying the [`CaptureError`] that ended a session unexpectedly.
pub const INTERRUPTED_EVENT: &str = "capture-interrupted";

/// The app's single capture session, if one has been started.
#[derive(Default)]
pub struct CaptureState {
    session: Mutex<Option<CaptureSession>>,
}

impl CaptureState {
    fn session(&self) -> MutexGuard<'_, Option<CaptureSession>> {
        // A poisoned lock only means another command panicked; the session
        // slot itself is still valid to use.
        self.session.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

/// Why a session could not be started. Serializes as the inner error, so
/// the UI sees one flat set of error codes.
#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum StartError {
    Capture(CaptureError),
    Transcription(TranscriptionError),
    Translation(TranslationError),
    Replies(ReplyError),
    Policy(PolicyError),
}

/// Forwards session updates to the UI as events, and audio to transcription.
struct UiObserver {
    app: AppHandle,
    transcription: FrameRouter,
}

impl CaptureObserver for UiObserver {
    fn levels(&self, update: LevelsUpdate) {
        // Emitting only fails while the app is shutting down.
        let _ = self.app.emit(LEVELS_EVENT, update);
    }

    fn audio(&self, frame: AudioFrame) {
        self.transcription.route(frame);
    }

    fn interrupted(&self, error: CaptureError) {
        let _ = self.app.emit(INTERRUPTED_EVENT, error);
    }
}

/// Lists the microphones the user can choose from.
#[tauri::command(async)]
pub fn list_microphones() -> Result<Vec<MicrophoneInfo>, CaptureError> {
    opentalk_audio_capture::list_microphones()
}

/// What the UI wants done with a session's audio beyond capturing it.
/// Translation and replies work on the transcript, so they only take effect
/// together with transcription.
#[derive(Debug, Default, serde::Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct Features {
    pub transcription: Option<TranscriptionRequest>,
    pub translation: Option<TranslationRequest>,
    pub replies: Option<ReplyRequest>,
    /// Local-only mode: refuse to start if any feature would use a cloud service.
    pub local_only: bool,
}

impl Features {
    /// Whether any requested feature sends data to a cloud service.
    fn uses_cloud(&self) -> bool {
        self.transcription
            .as_ref()
            .is_some_and(|request| request.provider.is_cloud())
            || self
                .translation
                .as_ref()
                .is_some_and(|request| request.provider.is_cloud())
            || self
                .replies
                .as_ref()
                .is_some_and(|request| request.provider.is_cloud())
    }
}

/// Starts capturing the selected sources, with the requested `features`.
/// Fails if capture is already running.
#[tauri::command(async)]
pub fn start_capture(
    app: AppHandle,
    state: State<'_, CaptureState>,
    reply_state: State<'_, ReplyState>,
    config: CaptureConfig,
    features: Features,
) -> Result<(), StartError> {
    let mut session = state.session();
    if session.as_ref().is_some_and(CaptureSession::is_running) {
        return Err(StartError::Capture(CaptureError::AlreadyRunning));
    }
    // Clear out a session that was interrupted earlier before starting a new one.
    *session = None;
    reply_state.clear();

    config.validate().map_err(StartError::Capture)?;
    providers::check_local_only(features.local_only, features.uses_cloud())
        .map_err(StartError::Policy)?;
    let transcription = match features.transcription {
        Some(request) => {
            let consumers = SegmentConsumers {
                translator: features
                    .translation
                    .map(|request| translation::start(&app, request))
                    .transpose()
                    .map_err(StartError::Translation)?,
                replies: features
                    .replies
                    .map(|request| replies::start(&app, &reply_state, request))
                    .transpose()
                    .map_err(StartError::Replies)?,
            };
            transcription::start(&app, &config, request, consumers)
                .map_err(StartError::Transcription)?
        }
        // Without a transcript there is nothing to translate or reply to.
        None => FrameRouter::disabled(),
    };
    let observer = UiObserver { app, transcription };
    let started = CaptureSession::start(&config, observer);
    if started.is_err() {
        reply_state.clear();
    }
    *session = Some(started.map_err(StartError::Capture)?);
    Ok(())
}

/// Whether a capture session is running right now. The UI asks this when
/// it loads, so that what it shows always matches what the core is doing.
#[tauri::command(async)]
pub fn is_capturing(state: State<'_, CaptureState>) -> bool {
    state
        .session()
        .as_ref()
        .is_some_and(CaptureSession::is_running)
}

/// Ends capture when the app quits: stops the session, cancels reply
/// generation and stops any model download.
pub fn shut_down(app: &AppHandle) {
    if let Some(session) = app.state::<CaptureState>().session().take() {
        session.stop();
    }
    app.state::<ReplyState>().clear();
    app.state::<crate::whisper::DownloadState>().cancel();
}

/// Stops capture and releases every device. Does nothing if capture is not running.
#[tauri::command(async)]
pub fn stop_capture(state: State<'_, CaptureState>, reply_state: State<'_, ReplyState>) {
    if let Some(session) = state.session().take() {
        session.stop();
    }
    reply_state.clear();
}
