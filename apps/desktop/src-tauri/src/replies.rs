//! Reply suggestions for the running capture session.
//!
//! Suggestions exist for a session only when the user turned them on. The
//! coordinator hears every finished transcript segment, generates drafts
//! after the other side speaks, and reports them to the UI. The drafts are
//! only displayed: nothing in this app can send a message.

use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use opentalk_audio_core::AudioSource;
use opentalk_language::ArabicStyle;
use opentalk_provider_ollama::OllamaReplyGenerator;
use opentalk_provider_openai::OpenAiReplyGenerator;
use opentalk_provider_openai_compatible::CompatibleReplyGenerator;
use opentalk_replies::provider::MAX_BUSINESS_CONTEXT_LENGTH;
use opentalk_replies::{
    ReplyCoordinator, ReplyError, ReplyGenerator, ReplyProviderInfo, ReplySettings, Speaker,
};
use opentalk_transcription::TranscriptSegment;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, State};

use crate::providers::{self, CustomSetupError, ProviderChoice};
use crate::secrets::{self, SecretError};
use crate::transcription::is_valid_language_tag;

/// Event carrying an `opentalk_replies::ReplyEvent`.
pub const REPLY_EVENT: &str = "reply-event";

/// The UI's request for reply suggestions during a session.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReplyRequest {
    pub provider: ProviderChoice,
    /// BCP 47 tag of the language replies are written in.
    pub reply_language: String,
    /// BCP 47 tag of the language each reply's meaning is shown in, if any.
    pub meaning_language: Option<String>,
    pub business_context: String,
    /// How to write Arabic, used for replies or meanings in Arabic.
    #[serde(default)]
    pub arabic: ArabicStyle,
}

/// The reply provider as shown in settings.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderStatus {
    #[serde(flatten)]
    pub info: ReplyProviderInfo,
    pub has_api_key: bool,
}

/// The coordinator of the running session, if it has reply suggestions.
#[derive(Default)]
pub struct ReplyState {
    coordinator: Mutex<Option<Arc<ReplyCoordinator>>>,
}

impl ReplyState {
    fn coordinator(&self) -> MutexGuard<'_, Option<Arc<ReplyCoordinator>>> {
        self.coordinator
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    }

    /// Ends reply suggestions, cancelling any request in progress.
    pub fn clear(&self) {
        *self.coordinator() = None;
    }
}

/// Tells the coordinator what a finished transcript segment said. Speech
/// from the microphone is the user; speech from system audio is the other side.
pub fn heard(coordinator: &ReplyCoordinator, segment: &TranscriptSegment) {
    if segment.is_final {
        let speaker = match segment.source {
            AudioSource::Microphone => Speaker::Me,
            AudioSource::System => Speaker::Other,
        };
        coordinator.heard(speaker, &segment.text);
    }
}

/// Creates the coordinator for a new capture session and remembers it, so
/// the UI can ask for a refresh.
pub fn start(
    app: &AppHandle,
    state: &ReplyState,
    request: ReplyRequest,
) -> Result<Arc<ReplyCoordinator>, ReplyError> {
    let languages = [
        Some(&request.reply_language),
        request.meaning_language.as_ref(),
    ];
    if languages
        .into_iter()
        .flatten()
        .any(|tag| !is_valid_language_tag(tag))
    {
        return Err(ReplyError::UnsupportedLanguage);
    }
    if !request.arabic.is_valid() {
        return Err(ReplyError::UnsupportedLanguage);
    }
    let generator = generator_for(&request.provider)?;
    let settings = ReplySettings {
        reply_language: request.reply_language,
        meaning_language: request.meaning_language,
        business_context: request
            .business_context
            .chars()
            .take(MAX_BUSINESS_CONTEXT_LENGTH)
            .collect(),
        arabic: request.arabic,
    };

    let app = app.clone();
    let coordinator = Arc::new(ReplyCoordinator::new(generator, settings, move |event| {
        let _ = app.emit(REPLY_EVENT, event);
    }));
    *state.coordinator() = Some(Arc::clone(&coordinator));
    Ok(coordinator)
}

/// Creates the reply generator the user chose.
fn generator_for(provider: &ProviderChoice) -> Result<Arc<dyn ReplyGenerator>, ReplyError> {
    Ok(match provider {
        ProviderChoice::Openai => {
            let api_key = secrets::api_key(opentalk_provider_openai::PROVIDER_ID)
                .ok()
                .flatten()
                .ok_or(ReplyError::MissingApiKey)?;
            Arc::new(OpenAiReplyGenerator::new(api_key))
        }
        ProviderChoice::Custom { base_url, model } => {
            let client =
                providers::custom_client(base_url, model).map_err(|error| match error {
                    CustomSetupError::InvalidAddress => ReplyError::ConnectionFailed {
                        detail: "the service address is not valid".to_owned(),
                    },
                    CustomSetupError::NoModel => ReplyError::ModelUnavailable,
                    CustomSetupError::MissingApiKey => ReplyError::MissingApiKey,
                })?;
            Arc::new(CompatibleReplyGenerator::new(client))
        }
        ProviderChoice::Ollama { endpoint, model } => Arc::new(OllamaReplyGenerator::new(
            providers::ollama_client(endpoint)?,
            providers::checked_model(model)?,
        )),
    })
}

/// Generates fresh suggestions now. Does nothing if the running session has
/// no reply suggestions or the other side has not spoken yet.
#[tauri::command]
pub async fn refresh_replies(state: State<'_, ReplyState>) -> Result<(), ReplyError> {
    if let Some(coordinator) = state.coordinator().as_ref() {
        coordinator.refresh();
    }
    Ok(())
}

/// Describes the reply provider and whether a key is saved for it.
#[tauri::command(async)]
pub fn reply_provider() -> Result<ProviderStatus, SecretError> {
    let info = opentalk_provider_openai::reply_info();
    let has_api_key = secrets::api_key(info.id)?.is_some();
    Ok(ProviderStatus { info, has_api_key })
}
