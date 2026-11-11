//! OpenAI adapters.
//!
//! - [`OpenAiTranscriber`] implements [`StreamingTranscriber`] on top of
//!   OpenAI's Realtime API in transcription mode, over a WebSocket.
//! - [`OpenAiTranslator`] implements `Translator` on top of the Responses API.
//! - [`OpenAiReplyGenerator`] implements `ReplyGenerator` on the same API.
//!
//! Everything specific to OpenAI (URLs, event names, model names, error
//! codes) stays inside this crate.
//!
//! Data flow: the transcriber sends only the stretches of audio that the
//! pipeline identified as speech, as 24 kHz mono 16-bit PCM. The translator
//! sends the text of finished transcript segments and the glossary. The
//! reply generator sends the recent conversation and the user's business
//! context. All of it goes to `api.openai.com` over TLS, and each only once
//! the user has turned that feature on. This crate neither logs nor stores audio or text.

mod http;
mod protocol;
mod replies;
mod responses;
mod session;
mod translator;
mod transport;

pub use replies::{OpenAiReplyGenerator, reply_info};
pub use translator::{OpenAiTranslator, translation_info};

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use opentalk_transcription::{
    ProviderCapabilities, ProviderLocation, StreamingTranscriber, TranscriptionConfig,
    TranscriptionError, TranscriptionSession,
};
use tokio::time;

use crate::protocol::ServerEvent;
use crate::transport::{Connection, Connector, WebSocketConnector};

/// Identifier of this provider in settings and in the credential store.
pub const PROVIDER_ID: &str = "openai";

/// How long to wait for the connection, and then for the session to be accepted.
const SETUP_TIMEOUT: Duration = Duration::from_secs(10);

/// What this adapter can do. Available without an API key, for display in settings.
#[must_use]
pub fn capabilities() -> ProviderCapabilities {
    ProviderCapabilities {
        id: PROVIDER_ID,
        display_name: "OpenAI",
        location: ProviderLocation::Cloud,
        streams_partial_results: true,
        // The model used here does not report the language it heard.
        detects_language: false,
        // OpenAI does not publish a fixed list; unsupported codes are rejected when a session opens.
        languages: None,
        input_sample_rate: protocol::SAMPLE_RATE,
    }
}

/// Streaming transcription through OpenAI's Realtime API.
pub struct OpenAiTranscriber {
    api_key: String,
    connector: Arc<dyn Connector>,
}

impl OpenAiTranscriber {
    /// Creates a transcriber that authenticates with `api_key`.
    #[must_use]
    pub fn new(api_key: String) -> Self {
        Self::with_connector(api_key, Arc::new(WebSocketConnector))
    }

    fn with_connector(api_key: String, connector: Arc<dyn Connector>) -> Self {
        Self { api_key, connector }
    }
}

#[async_trait]
impl StreamingTranscriber for OpenAiTranscriber {
    fn capabilities(&self) -> ProviderCapabilities {
        capabilities()
    }

    async fn open(
        &self,
        config: &TranscriptionConfig,
    ) -> Result<TranscriptionSession, TranscriptionError> {
        let setup = async {
            let mut connection = self.connector.connect(&self.api_key).await?;
            configure(&mut connection, config).await?;
            Ok(connection)
        };
        let connection = time::timeout(SETUP_TIMEOUT, setup)
            .await
            .map_err(|_| TranscriptionError::Timeout)??;

        let (session, audio, events) = TranscriptionSession::channel();
        tokio::spawn(session::run(connection, audio, events));
        Ok(session)
    }
}

/// Sends the session settings and waits until OpenAI accepts or rejects them.
async fn configure(
    connection: &mut Connection,
    config: &TranscriptionConfig,
) -> Result<(), TranscriptionError> {
    let settings = protocol::session_update(config.language.as_deref());
    connection.send(settings).await?;
    loop {
        match protocol::parse_server_event(&connection.receive().await?)? {
            ServerEvent::SessionUpdated => return Ok(()),
            ServerEvent::Error(error) => {
                return Err(error.into_setup_error(config.language.as_deref()));
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests;
