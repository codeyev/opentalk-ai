//! API keys, kept in the operating system's credential store (Keychain on
//! macOS, Credential Manager on Windows).
//!
//! Keys are never written to a file, logged, or returned to the UI. The UI
//! can save a key, delete it, and ask whether one exists; only the Rust core
//! reads it back, at the moment a provider session is opened.

use keyring::Entry;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Name under which this app's entries appear in the credential store.
const SERVICE: &str = "OpenTalk AI";

/// Longest API key accepted. Real keys are far shorter; this only bounds IPC input.
const MAX_KEY_LENGTH: usize = 512;

/// The services an API key can be saved for. Keys are kept separately, so
/// a key is only ever sent to the service it was entered for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum KeyOwner {
    Openai,
    /// The user's own OpenAI-compatible service, such as OpenRouter.
    Custom,
    /// ElevenLabs, for speech output.
    Elevenlabs,
}

/// The name a service's key is stored under in the credential store.
///
/// The custom service's key is stored per host, so a key entered for one
/// service is never sent to another when the user changes the address.
/// Returns `None` for a custom service whose address is not valid.
pub fn key_id(owner: KeyOwner, base_url: Option<&str>) -> Option<String> {
    match owner {
        KeyOwner::Openai => Some(opentalk_provider_openai::PROVIDER_ID.to_owned()),
        KeyOwner::Elevenlabs => Some(opentalk_provider_elevenlabs::PROVIDER_ID.to_owned()),
        KeyOwner::Custom => {
            let endpoint = opentalk_provider_openai_compatible::Endpoint::parse(base_url?).ok()?;
            Some(format!("service:{}", endpoint.host()))
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error, Serialize)]
#[serde(tag = "code", rename_all = "snake_case")]
pub enum SecretError {
    #[error("the API key is empty, too long, or contains spaces")]
    InvalidKey,
    #[error("the service address is not valid, so there is nowhere to keep its key")]
    InvalidService,
    #[error("the system credential store could not be used")]
    StoreUnavailable,
}

fn entry(provider_id: &str) -> Result<Entry, SecretError> {
    Entry::new(SERVICE, provider_id).map_err(|_| SecretError::StoreUnavailable)
}

/// Saves `api_key` for a provider, replacing any earlier key.
pub fn save_api_key(provider_id: &str, api_key: &str) -> Result<(), SecretError> {
    let is_valid = !api_key.is_empty()
        && api_key.len() <= MAX_KEY_LENGTH
        && api_key
            .chars()
            .all(|character| character.is_ascii_graphic());
    if !is_valid {
        return Err(SecretError::InvalidKey);
    }
    entry(provider_id)?
        .set_password(api_key)
        .map_err(|_| SecretError::StoreUnavailable)
}

/// Returns the saved key for a provider, or `None` if there is none.
pub fn api_key(provider_id: &str) -> Result<Option<String>, SecretError> {
    match entry(provider_id)?.get_password() {
        Ok(key) => Ok(Some(key)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(_) => Err(SecretError::StoreUnavailable),
    }
}

/// Deletes the saved key for a provider. Succeeds if there was none.
pub fn delete_api_key(provider_id: &str) -> Result<(), SecretError> {
    match entry(provider_id)?.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(_) => Err(SecretError::StoreUnavailable),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_service_keeps_its_own_key() {
        assert_eq!(key_id(KeyOwner::Openai, None).as_deref(), Some("openai"));
        assert_eq!(
            key_id(KeyOwner::Elevenlabs, None).as_deref(),
            Some("elevenlabs")
        );
        assert_eq!(
            key_id(KeyOwner::Custom, Some("https://openrouter.ai/api/v1")).as_deref(),
            Some("service:openrouter.ai")
        );
        assert_eq!(
            key_id(KeyOwner::Custom, Some("https://api.deepseek.com")).as_deref(),
            Some("service:api.deepseek.com")
        );
        // Without a valid address there is no service to keep a key for.
        assert_eq!(
            key_id(KeyOwner::Custom, Some("http://insecure.example")),
            None
        );
        assert_eq!(key_id(KeyOwner::Custom, None), None);
    }

    #[test]
    fn keys_with_spaces_or_no_content_are_rejected() {
        for key in [
            "",
            " ",
            "has space",
            "line\nbreak",
            &"x".repeat(MAX_KEY_LENGTH + 1),
        ] {
            assert_eq!(
                save_api_key("test-invalid", key),
                Err(SecretError::InvalidKey)
            );
        }
    }

    /// Writes to the real credential store, so it only runs when asked:
    /// `cargo test -p opentalk-desktop -- --ignored credential_store`.
    #[test]
    #[ignore = "uses the operating system's credential store"]
    fn credential_store_round_trip() {
        let provider = "test-round-trip";
        assert_eq!(delete_api_key(provider), Ok(()));
        assert_eq!(api_key(provider), Ok(None));

        assert_eq!(save_api_key(provider, "first-key"), Ok(()));
        assert_eq!(save_api_key(provider, "second-key"), Ok(()));
        assert_eq!(api_key(provider), Ok(Some("second-key".to_owned())));

        assert_eq!(delete_api_key(provider), Ok(()));
        assert_eq!(api_key(provider), Ok(None));
    }
}
