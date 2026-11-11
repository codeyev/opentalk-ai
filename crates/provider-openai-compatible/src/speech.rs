//! Speech output through an `/audio/speech` endpoint in OpenAI's format.
//!
//! Written against the speech API as documented by OpenAI in November
//! 2026. Speech servers that run on the user's own computer commonly
//! offer the same endpoint.

use std::time::Duration;

use async_trait::async_trait;
use opentalk_speech::{SpeechError, SpeechSynthesizer, checked_pcm};
use serde_json::json;

use crate::endpoint::Endpoint;

/// Longest a single request may take.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

/// One voice of one speech model on an OpenAI-compatible service.
pub struct CompatibleSpeech {
    endpoint: Endpoint,
    /// `None` for services that need no key, such as a local server.
    api_key: Option<String>,
    model: String,
    voice: String,
    http: reqwest::Client,
}

impl CompatibleSpeech {
    #[must_use]
    pub fn new(endpoint: Endpoint, api_key: Option<String>, model: String, voice: String) -> Self {
        let http = reqwest::Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .build()
            .unwrap_or_default();
        Self {
            endpoint,
            api_key,
            model,
            voice,
            http,
        }
    }
}

#[async_trait]
impl SpeechSynthesizer for CompatibleSpeech {
    /// The API has no field for the language; the model reads it from the text.
    async fn synthesize(&self, text: &str, _language: &str) -> Result<Vec<u8>, SpeechError> {
        let body = json!({
            "model": self.model,
            "voice": self.voice,
            "input": text,
            // Raw 16-bit samples at 24 kHz, which is the app's speech format.
            "response_format": "pcm",
        });
        let mut request = self
            .http
            .post(self.endpoint.speech_url())
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .body(body.to_string());
        if let Some(api_key) = &self.api_key {
            request = request.bearer_auth(api_key);
        }
        let response = request.send().await.map_err(request_error)?;

        let status = response.status();
        let audio = response.bytes().await.map_err(request_error)?;
        match status.as_u16() {
            200..=299 => checked_pcm(audio.to_vec()),
            401 => Err(SpeechError::AuthenticationFailed),
            // 402 is "out of credit" on services that bill per request.
            402 | 429 => Err(SpeechError::RateLimited),
            404 => Err(SpeechError::ModelUnavailable),
            500..=599 => Err(SpeechError::ConnectionFailed),
            _ => Err(SpeechError::Provider),
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

#[cfg(test)]
mod tests {
    use opentalk_test_support::{MockServer, Route};

    use super::*;

    const SPEECH_PATH: &str = "/v1/audio/speech";

    async fn speak(server: &MockServer, api_key: Option<&str>) -> Result<Vec<u8>, SpeechError> {
        let endpoint = Endpoint::parse(&format!("{}/v1", server.base_url())).unwrap();
        CompatibleSpeech::new(
            endpoint,
            api_key.map(str::to_owned),
            "speech-model".to_owned(),
            "alloy".to_owned(),
        )
        .synthesize("Hello there.", "en")
        .await
    }

    #[tokio::test]
    async fn asks_for_raw_audio_and_returns_it() {
        let server = MockServer::start(vec![Route::new(SPEECH_PATH, 200, "abcdef")]).await;

        assert_eq!(
            speak(&server, Some("test-key")).await,
            Ok(b"abcdef".to_vec())
        );
        assert_eq!(
            server.request_to(SPEECH_PATH),
            json!({
                "model": "speech-model",
                "voice": "alloy",
                "input": "Hello there.",
                "response_format": "pcm",
            })
        );
        assert_eq!(
            server.header_of(SPEECH_PATH, "authorization").as_deref(),
            Some("Bearer test-key")
        );
    }

    #[tokio::test]
    async fn a_local_server_is_used_without_a_key() {
        let server = MockServer::start(vec![Route::new(SPEECH_PATH, 200, "abcd")]).await;

        assert!(speak(&server, None).await.is_ok());
        assert_eq!(server.header_of(SPEECH_PATH, "authorization"), None);
    }

    #[tokio::test]
    async fn failures_are_told_apart() {
        for (status, expected) in [
            (401, SpeechError::AuthenticationFailed),
            (402, SpeechError::RateLimited),
            (429, SpeechError::RateLimited),
            (404, SpeechError::ModelUnavailable),
            (400, SpeechError::Provider),
            (500, SpeechError::ConnectionFailed),
        ] {
            let server = MockServer::start(vec![Route::new(SPEECH_PATH, status, "{}")]).await;
            assert_eq!(speak(&server, None).await, Err(expected), "status {status}");
        }
        let server = MockServer::start(vec![Route::new(SPEECH_PATH, 200, "")]).await;
        assert_eq!(
            speak(&server, None).await,
            Err(SpeechError::MalformedResponse)
        );
    }
}
