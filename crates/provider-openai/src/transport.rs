//! The connection to OpenAI, separated from the protocol so tests can
//! replace the network with an in-memory fake.

use async_trait::async_trait;
use futures_util::{SinkExt, StreamExt};
use opentalk_transcription::TranscriptionError;
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::http::StatusCode;
use tokio_tungstenite::tungstenite::http::header::AUTHORIZATION;
use tokio_tungstenite::tungstenite::{self, Message};

const ENDPOINT: &str = "wss://api.openai.com/v1/realtime?intent=transcription";

/// Messages that may wait to be written to the socket.
const OUTGOING_CAPACITY: usize = 64;
/// Messages that may wait to be read by the session. Results are small and
/// infrequent; if this ever fills up the connection is treated as lost.
const INCOMING_CAPACITY: usize = 256;

/// A message from the server, or the reason the connection ended.
pub(crate) type Incoming = Result<String, TranscriptionError>;

/// An open, bidirectional stream of text messages. Dropping it closes the connection.
pub(crate) struct Connection {
    outgoing: mpsc::Sender<String>,
    incoming: mpsc::Receiver<Incoming>,
}

impl Connection {
    /// Creates a connection together with the ends its transport works with.
    pub(crate) fn channel() -> (Self, mpsc::Receiver<String>, mpsc::Sender<Incoming>) {
        let (outgoing, transport_outgoing) = mpsc::channel(OUTGOING_CAPACITY);
        let (transport_incoming, incoming) = mpsc::channel(INCOMING_CAPACITY);
        (
            Self { outgoing, incoming },
            transport_outgoing,
            transport_incoming,
        )
    }

    pub(crate) async fn send(&self, message: String) -> Result<(), TranscriptionError> {
        self.outgoing
            .send(message)
            .await
            .map_err(|_| lost("the connection closed"))
    }

    /// Waits for the next message. Fails once the connection has ended.
    pub(crate) async fn receive(&mut self) -> Incoming {
        self.incoming
            .recv()
            .await
            .unwrap_or_else(|| Err(lost("the connection closed")))
    }
}

fn lost(detail: impl ToString) -> TranscriptionError {
    TranscriptionError::ConnectionLost {
        detail: detail.to_string(),
    }
}

/// Opens connections to the transcription service.
#[async_trait]
pub(crate) trait Connector: Send + Sync {
    async fn connect(&self, api_key: &str) -> Result<Connection, TranscriptionError>;
}

/// The real transport: a TLS WebSocket to OpenAI.
pub(crate) struct WebSocketConnector;

#[async_trait]
impl Connector for WebSocketConnector {
    async fn connect(&self, api_key: &str) -> Result<Connection, TranscriptionError> {
        let failed = |detail: &str| TranscriptionError::ConnectionFailed {
            detail: detail.to_owned(),
        };
        let mut request = ENDPOINT
            .into_client_request()
            .map_err(|_| failed("invalid endpoint"))?;
        let authorization = format!("Bearer {api_key}")
            .parse()
            .map_err(|_| TranscriptionError::AuthenticationFailed)?;
        request.headers_mut().insert(AUTHORIZATION, authorization);

        let (socket, _) = tokio_tungstenite::connect_async(request)
            .await
            .map_err(connect_error)?;
        let (mut writer, mut reader) = socket.split();
        let (connection, mut outgoing, incoming) = Connection::channel();

        // Moves messages between the socket and the connection's queues
        // until either side closes.
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    message = outgoing.recv() => match message {
                        Some(text) => {
                            if let Err(error) = writer.send(Message::text(text)).await {
                                let _ = incoming.try_send(Err(lost(error)));
                                return;
                            }
                        }
                        // The session was dropped: say goodbye and stop.
                        None => {
                            let _ = writer.close().await;
                            return;
                        }
                    },
                    frame = reader.next() => {
                        let delivery = match frame {
                            Some(Ok(Message::Text(text))) => Ok(text.to_string()),
                            // Pings are answered by the library; other frame types carry nothing we use.
                            Some(Ok(Message::Ping(_) | Message::Pong(_) | Message::Binary(_) | Message::Frame(_))) => continue,
                            Some(Ok(Message::Close(_))) | None => Err(lost("the server closed the connection")),
                            Some(Err(error)) => Err(lost(error)),
                        };
                        let ended = delivery.is_err();
                        // Never wait here: a full queue must not stall the socket.
                        if incoming.try_send(delivery).is_err() || ended {
                            return;
                        }
                    }
                }
            }
        });
        Ok(connection)
    }
}

/// Maps a failed WebSocket handshake onto the app's error type.
fn connect_error(error: tungstenite::Error) -> TranscriptionError {
    match &error {
        tungstenite::Error::Http(response) => match response.status() {
            StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => {
                TranscriptionError::AuthenticationFailed
            }
            StatusCode::TOO_MANY_REQUESTS => TranscriptionError::RateLimited,
            status => TranscriptionError::ConnectionFailed {
                detail: format!("the server answered {status}"),
            },
        },
        _ => TranscriptionError::ConnectionFailed {
            detail: error.to_string(),
        },
    }
}
