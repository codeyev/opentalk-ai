//! Translates finished transcript segments and reports progress to the UI.
//!
//! Translation only exists for a session when the user turned it on. Final
//! segments wait in a bounded queue and are translated one at a time, in
//! order; if speech arrives faster than translations come back, the newest
//! segments are reported as failed instead of piling up.

use std::sync::Arc;

use opentalk_language::{ArabicStyle, same_language};
use opentalk_provider_ollama::OllamaTranslator;
use opentalk_provider_openai::OpenAiTranslator;
use opentalk_provider_openai_compatible::CompatibleTranslator;
use opentalk_transcription::TranscriptSegment;
use opentalk_translation::provider::MAX_GLOSSARY_TERMS;
use opentalk_translation::{
    GlossaryTerm, TranslationError, TranslationEvent, TranslationProviderInfo, TranslationSettings,
    Translator, service,
};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};
use tokio::sync::mpsc;

use crate::providers::{self, CustomSetupError, ProviderChoice};
use crate::secrets::{self, SecretError};
use crate::transcription::is_valid_language_tag;

/// Event carrying a [`TranslationEvent`].
pub const TRANSLATION_EVENT: &str = "translation-event";

/// Segments that may wait to be translated.
const QUEUE_CAPACITY: usize = 16;

/// The UI's request to translate a session's transcript.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TranslationRequest {
    pub provider: ProviderChoice,
    /// BCP 47 tag of the language to translate into.
    pub target_language: String,
    pub glossary: Vec<GlossaryTerm>,
    /// How to write Arabic, used when the target language is Arabic.
    #[serde(default)]
    pub arabic: ArabicStyle,
}

/// The translation provider as shown in settings.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderStatus {
    #[serde(flatten)]
    pub info: TranslationProviderInfo,
    pub has_api_key: bool,
}

/// Accepts final transcript segments for translation.
pub struct SegmentTranslator {
    app: AppHandle,
    target_language: String,
    queue: mpsc::Sender<TranscriptSegment>,
}

impl SegmentTranslator {
    /// Queues a final segment for translation and tells the UI it is pending.
    /// Segments already in the target language are left alone.
    pub fn submit(&self, segment: &TranscriptSegment) {
        let already_translated = segment
            .language
            .as_deref()
            .is_some_and(|language| same_language(language, &self.target_language));
        if !segment.is_final || already_translated {
            return;
        }
        let segment_id = segment.id.clone();
        let event = match self.queue.try_send(segment.clone()) {
            Ok(()) => TranslationEvent::Pending { segment_id },
            Err(_) => TranslationEvent::Failed {
                segment_id,
                error: TranslationError::Overloaded,
            },
        };
        let _ = self.app.emit(TRANSLATION_EVENT, event);
    }
}

/// Starts the translation worker for one capture session.
///
/// The worker ends by itself when the returned translator is dropped.
pub fn start(
    app: &AppHandle,
    request: TranslationRequest,
) -> Result<SegmentTranslator, TranslationError> {
    if !is_valid_language_tag(&request.target_language) || !request.arabic.is_valid() {
        return Err(TranslationError::UnsupportedLanguage);
    }
    let translator = translator_for(&request.provider)?;
    let settings = TranslationSettings {
        target_language: request.target_language,
        glossary: request
            .glossary
            .into_iter()
            .filter(GlossaryTerm::is_usable)
            .take(MAX_GLOSSARY_TERMS)
            .collect(),
        arabic: request.arabic,
    };
    let target_language = settings.target_language.clone();

    let (queue, mut segments) = mpsc::channel::<TranscriptSegment>(QUEUE_CAPACITY);
    let worker_app = app.clone();
    tauri::async_runtime::spawn(async move {
        while let Some(segment) = segments.recv().await {
            let result = service::translate(
                translator.as_ref(),
                &segment.text,
                segment.language.as_deref(),
                &settings,
            )
            .await;
            let segment_id = segment.id;
            let event = match result {
                Ok(translation) => TranslationEvent::Done {
                    segment_id,
                    translation,
                },
                Err(error) => TranslationEvent::Failed { segment_id, error },
            };
            let _ = worker_app.emit(TRANSLATION_EVENT, event);
        }
    });

    Ok(SegmentTranslator {
        app: app.clone(),
        target_language,
        queue,
    })
}

/// Creates the translator the user chose.
fn translator_for(provider: &ProviderChoice) -> Result<Arc<dyn Translator>, TranslationError> {
    Ok(match provider {
        ProviderChoice::Openai => {
            let api_key = secrets::api_key(opentalk_provider_openai::PROVIDER_ID)
                .ok()
                .flatten()
                .ok_or(TranslationError::MissingApiKey)?;
            Arc::new(OpenAiTranslator::new(api_key))
        }
        ProviderChoice::Custom { base_url, model } => {
            let client =
                providers::custom_client(base_url, model).map_err(|error| match error {
                    CustomSetupError::InvalidAddress => TranslationError::ConnectionFailed {
                        detail: "the service address is not valid".to_owned(),
                    },
                    CustomSetupError::NoModel => TranslationError::ModelUnavailable,
                    CustomSetupError::MissingApiKey => TranslationError::MissingApiKey,
                })?;
            Arc::new(CompatibleTranslator::new(client))
        }
        ProviderChoice::Ollama { endpoint, model } => Arc::new(OllamaTranslator::new(
            providers::ollama_client(endpoint)?,
            providers::checked_model(model)?,
        )),
    })
}

/// Describes the cloud translation provider and whether a key is saved for it.
#[tauri::command(async)]
pub fn translation_provider() -> Result<ProviderStatus, SecretError> {
    let info = opentalk_provider_openai::translation_info();
    let has_api_key = secrets::api_key(info.id)?.is_some();
    Ok(ProviderStatus { info, has_api_key })
}
