//! Tauri bootstrap and IPC surface for the OpenTalk AI desktop shell.
//!
//! This crate only wires the UI to the core: `app_info` lets the UI confirm
//! the core is reachable, the [`capture`] commands start and stop audio
//! capture, and the [`transcription`] commands manage the provider and its
//! API key. The logic itself lives in the crates under `crates/`.

mod capture;
mod providers;
mod replies;
mod secrets;
mod transcription;
mod translation;
mod whisper;

use serde::Serialize;

/// Static information about the running application core.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AppInfo {
    pub name: &'static str,
    pub version: &'static str,
}

impl AppInfo {
    /// Values baked in at compile time from this crate's manifest.
    #[must_use]
    pub const fn current() -> Self {
        Self {
            name: env!("CARGO_PKG_NAME"),
            version: env!("CARGO_PKG_VERSION"),
        }
    }
}

/// Returns the core's name and version. Takes no input and has no side effects.
#[tauri::command]
fn app_info() -> AppInfo {
    AppInfo::current()
}

/// Builds and runs the desktop application.
///
/// # Panics
///
/// Panics if the Tauri runtime cannot be started, since the app cannot
/// continue without a window.
pub fn run() {
    tauri::Builder::default()
        .manage(capture::CaptureState::default())
        .manage(replies::ReplyState::default())
        .manage(whisper::DownloadState::default())
        .invoke_handler(tauri::generate_handler![
            app_info,
            capture::list_microphones,
            capture::start_capture,
            capture::stop_capture,
            capture::is_capturing,
            transcription::transcription_provider,
            transcription::save_api_key,
            transcription::delete_api_key,
            translation::translation_provider,
            replies::reply_provider,
            replies::refresh_replies,
            providers::ollama_status,
            providers::custom_provider,
            whisper::whisper_models,
            whisper::download_whisper_model,
            whisper::cancel_model_download,
        ])
        .build(tauri::generate_context!())
        .expect("error while starting the OpenTalk AI application")
        .run(|app, event| {
            // Quitting must end capture the same way Stop does, so devices
            // are released and nothing is left running.
            if matches!(event, tauri::RunEvent::Exit) {
                capture::shut_down(app);
            }
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_info_matches_the_crate_manifest() {
        let info = AppInfo::current();
        assert_eq!(info.name, "opentalk-desktop");
        assert_eq!(info.version, env!("CARGO_PKG_VERSION"));
        assert!(!info.version.is_empty());
    }

    #[test]
    fn command_returns_the_current_app_info() {
        assert_eq!(app_info(), AppInfo::current());
    }
}
