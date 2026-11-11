//! Which provider does a job, as chosen by the user, and the rule that
//! keeps a local-only session local.

use opentalk_provider_ollama::{LocalEndpoint, OllamaClient, OllamaError, OllamaStatus};
use opentalk_provider_openai_compatible::{ChatClient, Endpoint};
use serde::{Deserialize, Serialize};

use crate::secrets::{self, KeyOwner, SecretError};

/// Longest endpoint address or model name accepted from the UI.
const MAX_TEXT_LENGTH: usize = 200;

/// The provider the user chose for translation or for reply suggestions.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(
    tag = "id",
    rename_all = "lowercase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum ProviderChoice {
    /// OpenAI, a cloud service, using the saved API key.
    Openai,
    /// A model on any service that speaks the OpenAI Chat Completions
    /// API, such as OpenRouter, using the key saved for it.
    Custom { base_url: String, model: String },
    /// A model in the Ollama server on this computer.
    Ollama { endpoint: String, model: String },
}

impl ProviderChoice {
    /// Whether using this provider sends data to another machine.
    pub fn is_cloud(&self) -> bool {
        match self {
            Self::Openai => true,
            // A custom service is local only if its address is on this
            // computer; an address that cannot be read counts as cloud.
            Self::Custom { base_url, .. } => {
                !Endpoint::parse(base_url).is_ok_and(|endpoint| endpoint.is_local())
            }
            Self::Ollama { .. } => false,
        }
    }
}

/// A session setting was refused because of a rule the user set.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(tag = "code", rename_all = "snake_case")]
pub enum PolicyError {
    /// Local-only mode is on and the session would have used a cloud service.
    LocalOnly,
}

/// Enforces local-only mode: with it on, a session may not use any cloud
/// provider. The UI already hides those choices; this is the check that
/// cannot be bypassed from the page.
pub fn check_local_only(local_only: bool, uses_cloud: bool) -> Result<(), PolicyError> {
    if local_only && uses_cloud {
        Err(PolicyError::LocalOnly)
    } else {
        Ok(())
    }
}

/// A client for the Ollama server at `endpoint`, which must be on this computer.
pub fn ollama_client(endpoint: &str) -> Result<OllamaClient, OllamaError> {
    if endpoint.len() > MAX_TEXT_LENGTH {
        return Err(OllamaError::NotLocal);
    }
    LocalEndpoint::parse(endpoint).map(OllamaClient::new)
}

/// Checks that `model` is a plausible model name before it is used.
pub fn checked_model(model: &str) -> Result<String, OllamaError> {
    let model = model.trim();
    let is_plausible = model.len() <= MAX_TEXT_LENGTH
        && model
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "._-:/".contains(c));
    if model.is_empty() {
        Err(OllamaError::NoModelSelected)
    } else if is_plausible {
        Ok(model.to_owned())
    } else {
        Err(OllamaError::ModelMissing {
            model: model.chars().take(MAX_TEXT_LENGTH).collect(),
        })
    }
}

/// Why a client for the custom service could not be set up.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CustomSetupError {
    /// The address is not `https`, or `http` on this computer.
    InvalidAddress,
    NoModel,
    /// The service is remote and no key is saved for it.
    MissingApiKey,
}

/// A client for one model on the user's OpenAI-compatible service, using
/// the key saved for it. A service on this computer may have no key.
pub fn custom_client(base_url: &str, model: &str) -> Result<ChatClient, CustomSetupError> {
    if base_url.len() > MAX_TEXT_LENGTH {
        return Err(CustomSetupError::InvalidAddress);
    }
    let endpoint = Endpoint::parse(base_url).map_err(|_| CustomSetupError::InvalidAddress)?;
    let model = model.trim();
    let is_plausible = !model.is_empty()
        && model.len() <= MAX_TEXT_LENGTH
        && model.chars().all(|c| c.is_ascii_graphic());
    if !is_plausible {
        return Err(CustomSetupError::NoModel);
    }
    let api_key = secrets::key_id(KeyOwner::Custom, Some(base_url))
        .and_then(|id| secrets::api_key(&id).ok().flatten());
    if api_key.is_none() && !endpoint.is_local() {
        return Err(CustomSetupError::MissingApiKey);
    }
    Ok(ChatClient::new(endpoint, api_key, model.to_owned()))
}

/// Whether an API key is saved for the custom service.
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomProviderStatus {
    pub has_api_key: bool,
}

/// Reports whether a key is saved for the custom service at `base_url`.
/// An address that is not valid has no key.
#[tauri::command(async)]
pub fn custom_provider(base_url: String) -> Result<CustomProviderStatus, SecretError> {
    let has_api_key = match secrets::key_id(KeyOwner::Custom, Some(&base_url)) {
        Some(id) => secrets::api_key(&id)?.is_some(),
        None => false,
    };
    Ok(CustomProviderStatus { has_api_key })
}

/// Health check and model discovery for the local Ollama server. Reads
/// only; it never downloads or changes a model.
#[tauri::command]
pub async fn ollama_status(endpoint: String) -> Result<OllamaStatus, OllamaError> {
    ollama_client(&endpoint)?.status().await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_only_mode_refuses_cloud_providers() {
        assert_eq!(check_local_only(true, true), Err(PolicyError::LocalOnly));
        assert_eq!(check_local_only(true, false), Ok(()));
        assert_eq!(check_local_only(false, true), Ok(()));
    }

    #[test]
    fn a_custom_service_is_cloud_unless_it_runs_on_this_computer() {
        let custom = |base_url: &str| ProviderChoice::Custom {
            base_url: base_url.to_owned(),
            model: "vendor/model".to_owned(),
        };
        assert!(custom("https://openrouter.ai/api/v1").is_cloud());
        assert!(!custom("http://localhost:1234/v1").is_cloud());
        // An address that cannot be read is treated as cloud, the safe side for local-only mode.
        assert!(custom("not a url").is_cloud());
    }

    #[test]
    fn a_custom_client_needs_a_valid_address_and_model() {
        assert_eq!(
            custom_client("http://openrouter.ai/api/v1", "vendor/model").err(),
            Some(CustomSetupError::InvalidAddress)
        );
        assert_eq!(
            custom_client("http://localhost:1234/v1", "  ").err(),
            Some(CustomSetupError::NoModel)
        );
        assert_eq!(
            custom_client("http://localhost:1234/v1", "has space").err(),
            Some(CustomSetupError::NoModel)
        );
        // A server on this computer needs no key.
        assert!(custom_client("http://localhost:1234/v1", "local-model").is_ok());
    }

    #[test]
    fn the_provider_choice_is_read_from_the_ui_format() {
        let choice: ProviderChoice = serde_json::from_str(
            r#"{"id":"custom","baseUrl":"https://openrouter.ai/api/v1","model":"vendor/model"}"#,
        )
        .unwrap();
        assert!(matches!(choice, ProviderChoice::Custom { .. }));
        assert_eq!(
            serde_json::from_str::<ProviderChoice>(r#"{"id":"openai"}"#).unwrap(),
            ProviderChoice::Openai
        );
    }

    #[test]
    fn only_openai_counts_as_cloud() {
        assert!(ProviderChoice::Openai.is_cloud());
        let local = ProviderChoice::Ollama {
            endpoint: "http://localhost:11434".to_owned(),
            model: "qwen3.5:9b".to_owned(),
        };
        assert!(!local.is_cloud());
    }

    #[test]
    fn the_ollama_endpoint_must_be_on_this_computer() {
        assert!(ollama_client("http://localhost:11434").is_ok());
        assert_eq!(
            ollama_client("http://example.com:11434").err(),
            Some(OllamaError::NotLocal)
        );
        let overlong = format!("http://localhost:11434/{}", "x".repeat(MAX_TEXT_LENGTH));
        assert_eq!(ollama_client(&overlong).err(), Some(OllamaError::NotLocal));
    }

    #[test]
    fn model_names_are_checked() {
        assert_eq!(checked_model(" qwen3.5:9b "), Ok("qwen3.5:9b".to_owned()));
        assert_eq!(
            checked_model("library/model-name_v2:latest").as_deref(),
            Ok("library/model-name_v2:latest")
        );
        assert_eq!(checked_model(""), Err(OllamaError::NoModelSelected));
        assert!(matches!(
            checked_model("model; rm -rf"),
            Err(OllamaError::ModelMissing { .. })
        ));
    }
}
