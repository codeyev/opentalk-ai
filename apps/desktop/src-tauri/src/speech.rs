//! Reading text aloud through a speech provider.
//!
//! The UI asks for one text at a time and plays the audio it gets back.
//! The text is sent to the chosen provider and nowhere else; it is never
//! logged or stored.

use opentalk_provider_elevenlabs::ElevenLabsSpeech;
use opentalk_provider_openai_compatible::{CompatibleSpeech, Endpoint};
use opentalk_speech::{MAX_TEXT_CHARS, SpeechError, SpeechSynthesizer};
use serde::{Deserialize, Serialize};

use crate::providers::check_local_only;
use crate::secrets::{self, KeyOwner, SecretError};
use crate::transcription::is_valid_language_tag;

/// Longest address, model or voice name accepted from the UI.
const MAX_SETTING_LENGTH: usize = 200;

/// The speech provider the user chose. The voices installed on the
/// computer are used by the UI itself and never reach the core.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(
    tag = "id",
    rename_all = "lowercase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum SpeechChoice {
    /// ElevenLabs, a cloud service, using the key saved for it.
    Elevenlabs { voice: String, model: String },
    /// Any service with an OpenAI-style speech endpoint, using the key
    /// saved for its address.
    Compatible {
        base_url: String,
        model: String,
        voice: String,
    },
}

impl SpeechChoice {
    /// Whether using this provider sends text to another machine.
    fn is_cloud(&self) -> bool {
        match self {
            Self::Elevenlabs { .. } => true,
            // An address that cannot be read counts as cloud, the safe side.
            Self::Compatible { base_url, .. } => {
                !Endpoint::parse(base_url).is_ok_and(|endpoint| endpoint.is_local())
            }
        }
    }
}

/// Why a text was not spoken.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(tag = "code", rename_all = "snake_case")]
pub enum SynthesisError {
    /// Local-only mode is on and the provider is a cloud service.
    LocalOnly,
    /// The address, model, voice, text or language is missing or not usable.
    InvalidSettings,
    MissingApiKey,
    AuthenticationFailed,
    RateLimited,
    ModelUnavailable,
    ConnectionFailed,
    Timeout,
    MalformedResponse,
    Provider,
}

impl From<SpeechError> for SynthesisError {
    fn from(error: SpeechError) -> Self {
        match error {
            SpeechError::AuthenticationFailed => Self::AuthenticationFailed,
            SpeechError::RateLimited => Self::RateLimited,
            SpeechError::ModelUnavailable => Self::ModelUnavailable,
            SpeechError::ConnectionFailed => Self::ConnectionFailed,
            SpeechError::Timeout => Self::Timeout,
            SpeechError::MalformedResponse => Self::MalformedResponse,
            SpeechError::Provider => Self::Provider,
        }
    }
}

/// A model or voice name: short, and without spaces or control characters.
fn checked_name(name: &str) -> Result<String, SynthesisError> {
    let name = name.trim();
    let is_plausible = !name.is_empty()
        && name.len() <= MAX_SETTING_LENGTH
        && name.chars().all(|c| c.is_ascii_graphic());
    if is_plausible {
        Ok(name.to_owned())
    } else {
        Err(SynthesisError::InvalidSettings)
    }
}

/// Sets up the chosen provider with the key saved for it.
fn synthesizer_for(choice: &SpeechChoice) -> Result<Box<dyn SpeechSynthesizer>, SynthesisError> {
    let saved_key = |owner, base_url| {
        secrets::key_id(owner, base_url).and_then(|id| secrets::api_key(&id).ok().flatten())
    };
    match choice {
        SpeechChoice::Elevenlabs { voice, model } => {
            // Checked before the key is read, so a typo is reported as one.
            let (voice, model) = (checked_name(voice)?, checked_name(model)?);
            let api_key =
                saved_key(KeyOwner::Elevenlabs, None).ok_or(SynthesisError::MissingApiKey)?;
            let speech = ElevenLabsSpeech::new(api_key, &voice, &model)
                .map_err(|_| SynthesisError::InvalidSettings)?;
            Ok(Box::new(speech))
        }
        SpeechChoice::Compatible {
            base_url,
            model,
            voice,
        } => {
            if base_url.len() > MAX_SETTING_LENGTH {
                return Err(SynthesisError::InvalidSettings);
            }
            let endpoint =
                Endpoint::parse(base_url).map_err(|_| SynthesisError::InvalidSettings)?;
            let (model, voice) = (checked_name(model)?, checked_name(voice)?);
            let api_key = saved_key(KeyOwner::Custom, Some(base_url.as_str()));
            // A server on this computer may need no key.
            if api_key.is_none() && !endpoint.is_local() {
                return Err(SynthesisError::MissingApiKey);
            }
            Ok(Box::new(CompatibleSpeech::new(
                endpoint, api_key, model, voice,
            )))
        }
    }
}

/// Turns `text` into speech with the chosen provider and returns the
/// audio: mono, 16-bit little-endian PCM at 24 kHz.
///
/// Called only for text the user asked to hear: a Speak button, or
/// translations while "speak translations" is on.
#[tauri::command]
pub async fn synthesize_speech(
    provider: SpeechChoice,
    text: String,
    language: String,
    local_only: bool,
) -> Result<tauri::ipc::Response, SynthesisError> {
    check_local_only(local_only, provider.is_cloud()).map_err(|_| SynthesisError::LocalOnly)?;
    let text = text.trim().to_owned();
    if text.is_empty() || text.chars().count() > MAX_TEXT_CHARS || !is_valid_language_tag(&language)
    {
        return Err(SynthesisError::InvalidSettings);
    }
    // Reading the credential store can block, so it runs off the async threads.
    let synthesizer = tauri::async_runtime::spawn_blocking(move || synthesizer_for(&provider))
        .await
        .map_err(|_| SynthesisError::Provider)??;
    let audio = synthesizer.synthesize(&text, &language).await?;
    Ok(tauri::ipc::Response::new(audio))
}

/// Whether an API key is saved for a service. The key itself never
/// reaches the UI.
#[tauri::command(async)]
pub fn has_api_key(provider: KeyOwner, base_url: Option<String>) -> Result<bool, SecretError> {
    match secrets::key_id(provider, base_url.as_deref()) {
        Some(id) => Ok(secrets::api_key(&id)?.is_some()),
        // An address that is not valid has no key.
        None => Ok(false),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn compatible(base_url: &str, model: &str, voice: &str) -> SpeechChoice {
        SpeechChoice::Compatible {
            base_url: base_url.to_owned(),
            model: model.to_owned(),
            voice: voice.to_owned(),
        }
    }

    #[test]
    fn the_speech_choice_is_read_from_the_ui_format() {
        let choice: SpeechChoice = serde_json::from_str(
            r#"{"id":"compatible","baseUrl":"https://api.openai.com/v1","model":"m","voice":"v"}"#,
        )
        .unwrap();
        assert_eq!(choice, compatible("https://api.openai.com/v1", "m", "v"));
        assert!(matches!(
            serde_json::from_str(r#"{"id":"elevenlabs","voice":"abc","model":"eleven_v3"}"#),
            Ok(SpeechChoice::Elevenlabs { .. })
        ));
    }

    #[test]
    fn only_a_server_on_this_computer_counts_as_local() {
        let elevenlabs = SpeechChoice::Elevenlabs {
            voice: "abc".to_owned(),
            model: "eleven_v3".to_owned(),
        };
        assert!(elevenlabs.is_cloud());
        assert!(compatible("https://api.openai.com/v1", "m", "v").is_cloud());
        assert!(!compatible("http://localhost:8880/v1", "m", "v").is_cloud());
        assert!(compatible("not a url", "m", "v").is_cloud());
    }

    #[test]
    fn unusable_settings_are_refused_before_anything_is_sent() {
        let refused = |choice: SpeechChoice| synthesizer_for(&choice).err();
        assert_eq!(
            refused(compatible("http://example.com/v1", "m", "v")),
            Some(SynthesisError::InvalidSettings)
        );
        assert_eq!(
            refused(compatible("http://localhost:8880/v1", " ", "v")),
            Some(SynthesisError::InvalidSettings)
        );
        assert_eq!(
            refused(compatible("http://localhost:8880/v1", "m", "has space")),
            Some(SynthesisError::InvalidSettings)
        );
        assert_eq!(
            refused(SpeechChoice::Elevenlabs {
                voice: String::new(),
                model: "eleven_v3".to_owned(),
            }),
            Some(SynthesisError::InvalidSettings)
        );
        // A server on this computer needs no key.
        assert!(synthesizer_for(&compatible("http://localhost:8880/v1", "m", "v")).is_ok());
    }
}
