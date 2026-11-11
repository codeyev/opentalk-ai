//! Contract tests against an in-memory fake of the OpenAI connection.
//! They never touch the network. A live test is at the bottom, ignored by default.

use std::sync::Mutex;

use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use opentalk_transcription::{AudioInput, ProviderEvent};
use serde_json::{Value, json};
use tokio::sync::mpsc;

use super::*;
use crate::transport::Incoming;

/// The server's side of a fake connection.
struct FakeServer {
    from_client: mpsc::Receiver<String>,
    to_client: mpsc::Sender<Incoming>,
}

impl FakeServer {
    async fn receive(&mut self) -> Value {
        let message = self
            .from_client
            .recv()
            .await
            .expect("the client closed the connection");
        serde_json::from_str(&message).expect("the client sent invalid JSON")
    }

    async fn send(&self, event: Value) {
        self.to_client.send(Ok(event.to_string())).await.unwrap();
    }

    async fn send_raw(&self, text: &str) {
        self.to_client.send(Ok(text.to_owned())).await.unwrap();
    }
}

/// Hands out one prepared connection, and remembers the key it was asked to use.
struct FakeConnector {
    connection: Mutex<Option<Connection>>,
    api_key_used: Mutex<Option<String>>,
}

#[async_trait]
impl Connector for FakeConnector {
    async fn connect(&self, api_key: &str) -> Result<Connection, TranscriptionError> {
        *self.api_key_used.lock().unwrap() = Some(api_key.to_owned());
        Ok(self
            .connection
            .lock()
            .unwrap()
            .take()
            .expect("only one connection was prepared"))
    }
}

fn fake() -> (OpenAiTranscriber, FakeServer, Arc<FakeConnector>) {
    let (connection, from_client, to_client) = Connection::channel();
    let connector = Arc::new(FakeConnector {
        connection: Mutex::new(Some(connection)),
        api_key_used: Mutex::new(None),
    });
    let transcriber = OpenAiTranscriber::with_connector("test-key".to_owned(), connector.clone());
    (
        transcriber,
        FakeServer {
            from_client,
            to_client,
        },
        connector,
    )
}

fn config(language: Option<&str>) -> TranscriptionConfig {
    TranscriptionConfig {
        language: language.map(str::to_owned),
    }
}

/// Opens a session against a server that accepts the configuration.
async fn open_session(language: Option<&str>) -> (TranscriptionSession, FakeServer, Value) {
    let (transcriber, mut server, _) = fake();
    let config = config(language);
    let (session, settings) = tokio::join!(transcriber.open(&config), async {
        let settings = server.receive().await;
        server.send(json!({ "type": "session.created" })).await;
        server.send(json!({ "type": "session.updated" })).await;
        settings
    });
    (session.expect("the session should open"), server, settings)
}

fn delta(item_id: &str, text: &str) -> Value {
    json!({ "type": "conversation.item.input_audio_transcription.delta", "item_id": item_id, "content_index": 0, "delta": text })
}

fn completed(item_id: &str, text: &str) -> Value {
    json!({ "type": "conversation.item.input_audio_transcription.completed", "item_id": item_id, "content_index": 0, "transcript": text })
}

#[tokio::test]
async fn opening_configures_a_transcription_session() {
    let (_session, _server, settings) = open_session(Some("ar-EG")).await;

    assert_eq!(settings["type"], "session.update");
    let session = &settings["session"];
    assert_eq!(session["type"], "transcription");
    let input = &session["audio"]["input"];
    assert_eq!(
        input["format"],
        json!({ "type": "audio/pcm", "rate": 24_000 })
    );
    assert_eq!(input["transcription"]["model"], "gpt-live-transcribe");
    // Only the language code is sent, not the region.
    assert_eq!(input["transcription"]["languages"], json!(["ar"]));
    assert!(input["turn_detection"].is_null());
}

#[tokio::test]
async fn no_language_is_sent_when_none_is_chosen() {
    let (_session, _server, settings) = open_session(None).await;
    assert!(
        settings["session"]["audio"]["input"]["transcription"]
            .get("languages")
            .is_none()
    );
}

#[tokio::test]
async fn the_api_key_is_passed_to_the_connection() {
    let (transcriber, mut server, connector) = fake();
    let config = config(None);
    let _ = tokio::join!(transcriber.open(&config), async {
        server.receive().await;
        server.send(json!({ "type": "session.updated" })).await;
    });
    assert_eq!(
        connector.api_key_used.lock().unwrap().as_deref(),
        Some("test-key")
    );
}

#[tokio::test]
async fn audio_is_sent_as_base64_little_endian_pcm() {
    let (session, mut server, _) = open_session(None).await;
    session
        .audio
        .send(AudioInput::Samples(vec![1, -2, 256]))
        .await
        .unwrap();

    let message = server.receive().await;
    assert_eq!(message["type"], "input_audio_buffer.append");
    let bytes = BASE64.decode(message["audio"].as_str().unwrap()).unwrap();
    assert_eq!(bytes, vec![1, 0, 0xFE, 0xFF, 0, 1]);
}

#[tokio::test]
async fn the_end_of_a_turn_commits_the_audio_buffer() {
    let (session, mut server, _) = open_session(None).await;
    session.audio.send(AudioInput::EndOfTurn).await.unwrap();
    assert_eq!(
        server.receive().await,
        json!({ "type": "input_audio_buffer.commit" })
    );
}

#[tokio::test]
async fn deltas_accumulate_into_partial_text_and_end_with_a_final() {
    let (mut session, server, _) = open_session(None).await;
    server.send(delta("item_a", "Hello")).await;
    server.send(delta("item_a", " there")).await;
    server
        .send(json!({ "type": "input_audio_buffer.committed", "item_id": "item_a" }))
        .await;
    server.send(completed("item_a", "Hello there.")).await;

    let partial = |text: &str| ProviderEvent::Partial {
        turn: 0,
        text: text.to_owned(),
    };
    assert_eq!(session.events.recv().await, Some(partial("Hello")));
    assert_eq!(session.events.recv().await, Some(partial("Hello there")));
    assert_eq!(
        session.events.recv().await,
        Some(ProviderEvent::Final {
            turn: 0,
            text: "Hello there.".to_owned(),
            language: None,
        })
    );
}

#[tokio::test]
async fn a_detected_language_is_reported_when_the_model_returns_one() {
    let (mut session, server, _) = open_session(None).await;
    let mut event = completed("item_a", "Bonjour.");
    event["languages"] = json!([{ "code": "fr" }]);
    server.send(event).await;

    assert_eq!(
        session.events.recv().await,
        Some(ProviderEvent::Final {
            turn: 0,
            text: "Bonjour.".to_owned(),
            language: Some("fr".to_owned()),
        })
    );
}

#[tokio::test]
async fn results_arriving_out_of_order_keep_their_turn_numbers() {
    let (mut session, server, _) = open_session(None).await;
    server
        .send(json!({ "type": "input_audio_buffer.committed", "item_id": "first" }))
        .await;
    server
        .send(json!({ "type": "input_audio_buffer.committed", "item_id": "second" }))
        .await;
    server.send(completed("second", "Two.")).await;
    server.send(completed("first", "One.")).await;

    let finished = |turn, text: &str| ProviderEvent::Final {
        turn,
        text: text.to_owned(),
        language: None,
    };
    assert_eq!(session.events.recv().await, Some(finished(1, "Two.")));
    assert_eq!(session.events.recv().await, Some(finished(0, "One.")));
}

#[tokio::test]
async fn a_failed_item_fails_only_its_turn() {
    let (mut session, server, _) = open_session(None).await;
    server
        .send(json!({ "type": "conversation.item.input_audio_transcription.failed", "item_id": "item_a", "error": { "message": "x" } }))
        .await;
    server.send(completed("item_b", "Still working.")).await;

    assert_eq!(
        session.events.recv().await,
        Some(ProviderEvent::TurnFailed { turn: 0 })
    );
    assert!(matches!(
        session.events.recv().await,
        Some(ProviderEvent::Final { turn: 1, .. })
    ));
}

#[tokio::test]
async fn an_empty_commit_uses_up_its_turn_number() {
    let (mut session, server, _) = open_session(None).await;
    server
        .send(json!({ "type": "error", "error": { "type": "invalid_request_error", "code": "input_audio_buffer_commit_empty", "message": "buffer too small" } }))
        .await;
    server.send(completed("item_a", "Next.")).await;

    assert_eq!(
        session.events.recv().await,
        Some(ProviderEvent::TurnFailed { turn: 0 })
    );
    assert!(matches!(
        session.events.recv().await,
        Some(ProviderEvent::Final { turn: 1, .. })
    ));
}

#[tokio::test]
async fn unknown_events_are_ignored() {
    let (mut session, server, _) = open_session(None).await;
    server
        .send(json!({ "type": "some.future.event", "anything": 1 }))
        .await;
    server.send(completed("item_a", "Hi.")).await;
    assert!(matches!(
        session.events.recv().await,
        Some(ProviderEvent::Final { turn: 0, .. })
    ));
}

#[tokio::test]
async fn a_rate_limit_error_ends_the_session() {
    let (mut session, server, _) = open_session(None).await;
    server
        .send(json!({ "type": "error", "error": { "type": "rate_limit_error", "message": "slow down" } }))
        .await;
    assert_eq!(
        session.events.recv().await,
        Some(ProviderEvent::Failed(TranscriptionError::RateLimited))
    );
    assert_eq!(session.events.recv().await, None);
}

#[tokio::test]
async fn a_message_that_is_not_json_ends_the_session() {
    let (mut session, server, _) = open_session(None).await;
    server.send_raw("not json").await;
    assert_eq!(
        session.events.recv().await,
        Some(ProviderEvent::Failed(TranscriptionError::MalformedResponse))
    );
}

#[tokio::test]
async fn a_dropped_connection_is_reported_as_lost() {
    let (mut session, server, _) = open_session(None).await;
    drop(server);
    assert!(matches!(
        session.events.recv().await,
        Some(ProviderEvent::Failed(
            TranscriptionError::ConnectionLost { .. }
        ))
    ));
}

#[tokio::test]
async fn dropping_the_session_closes_the_connection() {
    let (session, mut server, _) = open_session(None).await;
    drop(session);
    assert_eq!(server.from_client.recv().await, None);
}

#[tokio::test]
async fn a_rejected_key_fails_to_open() {
    let (transcriber, mut server, _) = fake();
    let config = config(None);
    let (result, ()) = tokio::join!(transcriber.open(&config), async {
        server.receive().await;
        server
            .send(json!({ "type": "error", "error": { "type": "authentication_error", "code": "invalid_api_key", "message": "bad key" } }))
            .await;
    });
    assert_eq!(result.err(), Some(TranscriptionError::AuthenticationFailed));
}

#[tokio::test]
async fn a_rejected_language_fails_to_open() {
    let (transcriber, mut server, _) = fake();
    let config = config(Some("tlh"));
    let (result, ()) = tokio::join!(transcriber.open(&config), async {
        server.receive().await;
        server
            .send(json!({ "type": "error", "error": { "type": "invalid_request_error", "param": "session.audio.input.transcription.languages", "message": "unsupported" } }))
            .await;
    });
    assert_eq!(
        result.err(),
        Some(TranscriptionError::UnsupportedLanguage {
            language: "tlh".to_owned()
        })
    );
}

#[tokio::test(start_paused = true)]
async fn opening_times_out_when_the_server_stays_silent() {
    let (transcriber, _server, _) = fake();
    let result = transcriber.open(&config(None)).await;
    assert_eq!(result.err(), Some(TranscriptionError::Timeout));
}

/// Talks to the real OpenAI service. Run it deliberately with:
///
/// ```text
/// OPENAI_API_KEY=... cargo test -p opentalk-provider-openai -- --ignored real_key
/// ```
#[tokio::test]
#[ignore = "uses the network and a real API key"]
async fn live_session_opens_with_a_real_key() {
    let api_key = std::env::var("OPENAI_API_KEY").expect("set OPENAI_API_KEY to run this test");
    let session = OpenAiTranscriber::new(api_key)
        .open(&config(Some("en")))
        .await;
    assert!(
        session.is_ok(),
        "could not open a session: {:?}",
        session.err()
    );
}

/// Checks the real endpoint, TLS and error mapping without needing an
/// account: a made-up key must be rejected as an authentication failure.
///
/// ```text
/// cargo test -p opentalk-provider-openai -- --ignored invalid_key
/// ```
#[tokio::test]
#[ignore = "uses the network"]
async fn live_transcription_rejects_an_invalid_key() {
    let session = OpenAiTranscriber::new("invalid-key-for-testing".to_owned())
        .open(&config(None))
        .await;
    assert_eq!(
        session.err(),
        Some(TranscriptionError::AuthenticationFailed)
    );
}
