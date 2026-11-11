//! Speech output through ElevenLabs, a cloud service.
//!
//! Written against the text-to-speech API as documented by ElevenLabs in
//! November 2026. The text to speak is sent to ElevenLabs; nothing else is.

use std::time::Duration;

use async_trait::async_trait;
use opentalk_speech::{SAMPLE_RATE_HZ, SpeechError, SpeechSynthesizer, checked_pcm};
use serde_json::{Value, json};

/// Identifier of this provider in settings and in the credential store.
pub const PROVIDER_ID: &str = "elevenlabs";

/// A model that speaks the most languages, Azerbaijani among them.
pub const DEFAULT_MODEL: &str = "eleven_v3";

const API_BASE_URL: &str = "https://api.elevenlabs.io";

/// Longest a single request may take.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

/// Longest voice or model identifier accepted.
const MAX_ID_LENGTH: usize = 100;

/// The voice or model identifier is not one ElevenLabs could have issued.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidSettings;

/// A voice on ElevenLabs, spoken by one of its models.
pub struct ElevenLabsSpeech {
    base_url: String,
    api_key: String,
    voice: String,
    model: String,
    http: reqwest::Client,
}

impl ElevenLabsSpeech {
    /// Sets up a client for the voice `voice`, spoken by `model`.
    ///
    /// # Errors
    ///
    /// Returns an error if either identifier is empty or holds anything
    /// but letters, digits, `_` and `-`. The voice becomes part of the
    /// request address, so nothing else may get into it.
    pub fn new(api_key: String, voice: &str, model: &str) -> Result<Self, InvalidSettings> {
        Self::with_base_url(API_BASE_URL.to_owned(), api_key, voice, model)
    }

    fn with_base_url(
        base_url: String,
        api_key: String,
        voice: &str,
        model: &str,
    ) -> Result<Self, InvalidSettings> {
        let (voice, model) = (voice.trim(), model.trim());
        if !is_identifier(voice) || !is_identifier(model) {
            return Err(InvalidSettings);
        }
        // Building only fails if the TLS backend cannot start; the default
        // client reports the same problem on the first request instead.
        let http = reqwest::Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .build()
            .unwrap_or_default();
        Ok(Self {
            base_url,
            api_key,
            voice: voice.to_owned(),
            model: model.to_owned(),
            http,
        })
    }

    /// The request body for speaking `text` in `language`.
    fn request_body(&self, text: &str, language: &str) -> Value {
        let mut body = json!({ "text": text, "model_id": self.model });
        // Naming the language keeps a short sentence from being read as a
        // related language. The multilingual v2 models do not take it.
        if !self.model.contains("multilingual_v2")
            && let Some(code) = language_code(language)
        {
            body["language_code"] = Value::String(code);
        }
        body
    }
}

fn is_identifier(text: &str) -> bool {
    !text.is_empty()
        && text.len() <= MAX_ID_LENGTH
        && text
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

/// The two-letter language code of a BCP 47 tag, such as `az` for `az-AZ`.
fn language_code(tag: &str) -> Option<String> {
    let primary = tag.split('-').next()?;
    let is_two_letters = primary.len() == 2 && primary.chars().all(|c| c.is_ascii_alphabetic());
    is_two_letters.then(|| primary.to_ascii_lowercase())
}

#[async_trait]
impl SpeechSynthesizer for ElevenLabsSpeech {
    async fn synthesize(&self, text: &str, language: &str) -> Result<Vec<u8>, SpeechError> {
        let url = format!(
            "{}/v1/text-to-speech/{}?output_format=pcm_{SAMPLE_RATE_HZ}",
            self.base_url, self.voice
        );
        let response = self
            .http
            .post(url)
            .header("xi-api-key", &self.api_key)
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .body(self.request_body(text, language).to_string())
            .send()
            .await
            .map_err(request_error)?;

        let status = response.status();
        let body = response.bytes().await.map_err(request_error)?;
        if status.is_success() {
            checked_pcm(body.to_vec())
        } else {
            Err(error_for(status.as_u16(), &body))
        }
    }
}

fn request_error(error: reqwest::Error) -> SpeechError {
    if error.is_timeout() {
        SpeechError::Timeout
    } else {
        SpeechError::ConnectionFailed
    }
}

/// Reads why a request was rejected from its status and error body.
fn error_for(status: u16, body: &[u8]) -> SpeechError {
    let body: Value = serde_json::from_slice(body).unwrap_or(Value::Null);
    // The reason is named in `detail.status`, for example `quota_exceeded`.
    let reason = body["detail"]["status"].as_str().unwrap_or_default();
    if reason.contains("quota") {
        return SpeechError::RateLimited;
    }
    match status {
        401 => SpeechError::AuthenticationFailed,
        402 | 429 => SpeechError::RateLimited,
        // An unknown voice is 404; an unknown model is reported as a bad request.
        404 => SpeechError::ModelUnavailable,
        _ if reason.contains("model") || reason.contains("voice") => SpeechError::ModelUnavailable,
        500..=599 => SpeechError::ConnectionFailed,
        _ => SpeechError::Provider,
    }
}

#[cfg(test)]
mod tests {
    use opentalk_test_support::{MockServer, Route};

    use super::*;

    const SPEECH_PATH: &str = "/v1/text-to-speech/voice-1?output_format=pcm_24000";

    fn client(server: &MockServer, model: &str) -> ElevenLabsSpeech {
        ElevenLabsSpeech::with_base_url(
            server.base_url().to_owned(),
            "test-key".to_owned(),
            "voice-1",
            model,
        )
        .unwrap()
    }

    async fn speak(status: u16, body: &str) -> Result<Vec<u8>, SpeechError> {
        let server = MockServer::start(vec![Route::new(SPEECH_PATH, status, body)]).await;
        client(&server, DEFAULT_MODEL)
            .synthesize("Salam", "az")
            .await
    }

    #[tokio::test]
    async fn returns_the_audio_and_sends_the_text_once_with_the_key() {
        let server = MockServer::start(vec![Route::new(SPEECH_PATH, 200, "abcd")]).await;
        let audio = client(&server, DEFAULT_MODEL)
            .synthesize("Salam, necəsiniz?", "az-AZ")
            .await;

        assert_eq!(audio, Ok(b"abcd".to_vec()));
        assert_eq!(server.requested_paths(), [SPEECH_PATH]);
        assert_eq!(
            server.request_to(SPEECH_PATH),
            json!({ "text": "Salam, necəsiniz?", "model_id": "eleven_v3", "language_code": "az" })
        );
        assert_eq!(
            server.header_of(SPEECH_PATH, "xi-api-key").as_deref(),
            Some("test-key")
        );
    }

    #[tokio::test]
    async fn the_language_is_left_out_where_it_cannot_be_given() {
        let server = MockServer::start(vec![Route::new(SPEECH_PATH, 200, "abcd")]).await;
        let _ = client(&server, "eleven_multilingual_v2")
            .synthesize("Hello", "en")
            .await;
        // A three-letter tag has no two-letter code to send.
        let _ = client(&server, DEFAULT_MODEL)
            .synthesize("Hello", "fil")
            .await;

        let requests = server.requests_to(SPEECH_PATH);
        assert_eq!(
            requests,
            [
                json!({ "text": "Hello", "model_id": "eleven_multilingual_v2" }),
                json!({ "text": "Hello", "model_id": "eleven_v3" }),
            ]
        );
    }

    #[tokio::test]
    async fn failures_are_told_apart() {
        let quota = r#"{"detail":{"status":"quota_exceeded","message":"..."}}"#;
        let model = r#"{"detail":{"status":"model_not_found","message":"..."}}"#;
        for (status, body, expected) in [
            (401, "{}", SpeechError::AuthenticationFailed),
            (401, quota, SpeechError::RateLimited),
            (429, "{}", SpeechError::RateLimited),
            (404, "{}", SpeechError::ModelUnavailable),
            (400, model, SpeechError::ModelUnavailable),
            (422, "{}", SpeechError::Provider),
            (503, "not json", SpeechError::ConnectionFailed),
        ] {
            assert_eq!(speak(status, body).await, Err(expected), "status {status}");
        }
    }

    #[tokio::test]
    async fn an_empty_reply_is_not_audio() {
        assert_eq!(speak(200, "").await, Err(SpeechError::MalformedResponse));
    }

    #[test]
    fn identifiers_that_could_change_the_request_address_are_refused() {
        let new = |voice: &str, model: &str| {
            ElevenLabsSpeech::new("key".to_owned(), voice, model).map(|_| ())
        };
        assert_eq!(new(" JBFqnCBsd6RMkjVDRZzb ", DEFAULT_MODEL), Ok(()));
        for voice in ["", "a/b", "voice?x=1", "../v1", "has space"] {
            assert_eq!(new(voice, DEFAULT_MODEL), Err(InvalidSettings), "{voice}");
        }
        assert_eq!(new("voice", ""), Err(InvalidSettings));
    }
}
