//! Whisper model files for local transcription: listing them, and
//! downloading one when the user asks.
//!
//! Models live in one folder inside the app's data directory. The UI only
//! ever names a model by its file name, which is checked and resolved
//! inside that folder, so the app cannot be pointed at any other file.

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use opentalk_provider_whisper::download::{self, DownloadError};
use opentalk_provider_whisper::models::{self, CATALOGUE, InstalledModel, ModelSpec};
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State};

use crate::providers::{self, PolicyError};

/// Event carrying a [`DownloadProgress`] while a model downloads.
pub const DOWNLOAD_EVENT: &str = "model-download";

/// Progress is reported each time this many more bytes have arrived.
const PROGRESS_STEP_BYTES: u64 = 2_000_000;

/// The folder that holds the Whisper model files.
pub fn models_directory(app: &AppHandle) -> Option<PathBuf> {
    app.path()
        .app_data_dir()
        .ok()
        .map(|data| data.join("models"))
}

/// The models the user has, and the ones the app can download.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WhisperModels {
    /// The models folder, so the user can add files of their own.
    pub directory: String,
    pub installed: Vec<InstalledModel>,
    pub catalogue: Vec<ModelSpec>,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadProgress {
    pub id: &'static str,
    pub received_bytes: u64,
    pub total_bytes: u64,
}

/// Why a download was not started.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(tag = "code", rename_all = "snake_case")]
pub enum RequestError {
    UnknownModel,
    /// Another download is already running.
    Busy,
    /// The app's data folder could not be found.
    StorageFailed,
}

/// Serializes as the inner error, so the UI sees one flat set of codes.
#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum ModelDownloadError {
    Policy(PolicyError),
    Request(RequestError),
    Download(DownloadError),
}

/// The one model download that may run at a time.
#[derive(Default)]
pub struct DownloadState {
    is_running: AtomicBool,
    cancelled: Arc<AtomicBool>,
}

impl DownloadState {
    /// Asks the running download, if any, to stop.
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }
}

/// Lists the installed models and the downloadable ones.
#[tauri::command(async)]
pub fn whisper_models(app: AppHandle) -> WhisperModels {
    let directory = models_directory(&app).unwrap_or_default();
    WhisperModels {
        installed: models::installed(&directory),
        directory: directory.to_string_lossy().into_owned(),
        catalogue: CATALOGUE.to_vec(),
    }
}

/// Downloads a catalogue model because the user asked for it. Refused in
/// local-only mode, since it contacts a download server.
#[tauri::command]
pub async fn download_whisper_model(
    app: AppHandle,
    state: State<'_, DownloadState>,
    id: String,
    local_only: bool,
) -> Result<(), ModelDownloadError> {
    providers::check_local_only(local_only, true).map_err(ModelDownloadError::Policy)?;
    let spec =
        ModelSpec::find(&id).ok_or(ModelDownloadError::Request(RequestError::UnknownModel))?;
    let directory =
        models_directory(&app).ok_or(ModelDownloadError::Request(RequestError::StorageFailed))?;
    if state.is_running.swap(true, Ordering::SeqCst) {
        return Err(ModelDownloadError::Request(RequestError::Busy));
    }
    state.cancelled.store(false, Ordering::SeqCst);

    let mut last_reported = 0;
    let report = |received_bytes: u64| {
        let is_finished = received_bytes == spec.size_bytes;
        if is_finished || received_bytes - last_reported >= PROGRESS_STEP_BYTES {
            last_reported = received_bytes;
            let progress = DownloadProgress {
                id: spec.id,
                received_bytes,
                total_bytes: spec.size_bytes,
            };
            let _ = app.emit(DOWNLOAD_EVENT, progress);
        }
    };
    let result = download::download(
        &spec,
        &spec.download_url(),
        &directory,
        &state.cancelled,
        report,
    )
    .await;
    state.is_running.store(false, Ordering::SeqCst);
    result.map(|_| ()).map_err(ModelDownloadError::Download)
}

/// Stops the running model download, if there is one.
#[tauri::command(async)]
pub fn cancel_model_download(state: State<'_, DownloadState>) {
    state.cancel();
}
