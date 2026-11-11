//! The JSON messages exchanged with OpenAI's Realtime API in transcription mode.
//!
//! Written against the Realtime transcription guide as published in
//! November 2026. Only the fields this adapter needs are modelled.

use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use opentalk_transcription::TranscriptionError;
use serde::Deserialize;
use serde_json::{Value, json};

/// The only PCM sample rate transcription sessions accept.
pub(crate) const SAMPLE_RATE: u32 = 24_000;

/// Streams partial text while audio arrives and finishes a turn when the
/// client commits the audio buffer, which is what the pipeline's turn
/// detection does.
const MODEL: &str = "gpt-live-transcribe";

/// Error code for committing a buffer that holds too little audio.
const EMPTY_COMMIT_CODE: &str = "input_audio_buffer_commit_empty";

/// Builds the message that configures a transcription session.
///
/// Server-side turn detection is switched off because the pipeline decides
/// where turns end. `language` is a BCP 47 tag; OpenAI takes the bare
/// language code, so a region such as the `EG` in `ar-EG` is dropped.
pub(crate) fn session_update(language: Option<&str>) -> String {
    let mut transcription = json!({ "model": MODEL });
    if let Some(code) = language.and_then(language_code) {
        transcription["languages"] = json!([code]);
    }
    json!({
        "type": "session.update",
        "session": {
            "type": "transcription",
            "audio": {
                "input": {
                    "format": { "type": "audio/pcm", "rate": SAMPLE_RATE },
                    "transcription": transcription,
                    "turn_detection": null,
                }
            }
        }
    })
    .to_string()
}

/// The language part of a BCP 47 tag, lowercased: `ar-EG` becomes `ar`.
fn language_code(tag: &str) -> Option<String> {
    let code = tag.split('-').next()?.trim().to_ascii_lowercase();
    (!code.is_empty()).then_some(code)
}

/// Builds the message that appends audio to the current turn.
pub(crate) fn append_audio(samples: &[i16]) -> String {
    let bytes: Vec<u8> = samples
        .iter()
        .flat_map(|sample| sample.to_le_bytes())
        .collect();
    json!({ "type": "input_audio_buffer.append", "audio": BASE64.encode(bytes) }).to_string()
}

/// Builds the message that ends the current turn.
pub(crate) fn commit_audio() -> String {
    json!({ "type": "input_audio_buffer.commit" }).to_string()
}

/// A message from OpenAI, reduced to what the adapter acts on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ServerEvent {
    SessionUpdated,
    /// The audio buffer was committed and became the item `item_id`.
    Committed {
        item_id: String,
    },
    /// More text for an item.
    Delta {
        item_id: String,
        text: String,
    },
    /// The finished text for an item.
    Completed {
        item_id: String,
        text: String,
        language: Option<String>,
    },
    /// One item could not be transcribed.
    ItemFailed {
        item_id: String,
    },
    Error(ApiError),
    /// Any event this adapter does not need.
    Other,
}

/// The `error` object of an error event.
#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize)]
pub(crate) struct ApiError {
    #[serde(default, rename = "type")]
    kind: Option<String>,
    #[serde(default)]
    code: Option<String>,
    #[serde(default)]
    message: Option<String>,
    #[serde(default)]
    param: Option<String>,
}

impl ApiError {
    /// Whether this error only means one commit held too little audio.
    pub(crate) fn is_empty_commit(&self) -> bool {
        self.code.as_deref() == Some(EMPTY_COMMIT_CODE)
    }

    /// Maps an error received during a session onto the app's error type.
    pub(crate) fn into_error(self) -> TranscriptionError {
        let kind = self.kind.as_deref().unwrap_or_default();
        let code = self.code.as_deref().unwrap_or_default();
        if kind == "authentication_error" || code == "invalid_api_key" {
            TranscriptionError::AuthenticationFailed
        } else if kind == "rate_limit_error" || code == "rate_limit_exceeded" {
            TranscriptionError::RateLimited
        } else {
            TranscriptionError::Provider {
                detail: self.message.unwrap_or_else(|| "unknown error".to_owned()),
            }
        }
    }

    /// Maps an error received while configuring the session. A complaint
    /// about the language setting means the language is not supported.
    pub(crate) fn into_setup_error(self, language: Option<&str>) -> TranscriptionError {
        let about_language = self
            .param
            .as_deref()
            .is_some_and(|param| param.contains("language"));
        match language {
            Some(language) if about_language => TranscriptionError::UnsupportedLanguage {
                language: language.to_owned(),
            },
            _ => self.into_error(),
        }
    }
}

#[derive(Deserialize)]
struct Language {
    code: String,
}

/// Reads one message from OpenAI.
///
/// # Errors
///
/// Returns [`TranscriptionError::MalformedResponse`] if the message is not
/// JSON or a known event lacks a field it must have.
pub(crate) fn parse_server_event(message: &str) -> Result<ServerEvent, TranscriptionError> {
    let malformed = || TranscriptionError::MalformedResponse;
    let event: Value = serde_json::from_str(message).map_err(|_| malformed())?;
    let text_field = |name: &str| event[name].as_str().map(str::to_owned);
    let item_id = || text_field("item_id").ok_or_else(malformed);

    let parsed = match event["type"].as_str().ok_or_else(malformed)? {
        "session.updated" => ServerEvent::SessionUpdated,
        "input_audio_buffer.committed" => ServerEvent::Committed {
            item_id: item_id()?,
        },
        "conversation.item.input_audio_transcription.delta" => ServerEvent::Delta {
            item_id: item_id()?,
            text: text_field("delta").unwrap_or_default(),
        },
        "conversation.item.input_audio_transcription.completed" => {
            let languages: Vec<Language> =
                serde_json::from_value(event["languages"].clone()).unwrap_or_default();
            ServerEvent::Completed {
                item_id: item_id()?,
                text: text_field("transcript").ok_or_else(malformed)?,
                language: languages.into_iter().next().map(|language| language.code),
            }
        }
        "conversation.item.input_audio_transcription.failed" => ServerEvent::ItemFailed {
            item_id: item_id()?,
        },
        "error" => ServerEvent::Error(
            serde_json::from_value(event["error"].clone()).map_err(|_| malformed())?,
        ),
        _ => ServerEvent::Other,
    };
    Ok(parsed)
}
