//! One running transcription session: audio out, transcript events in.

use std::collections::HashMap;

use opentalk_transcription::{AudioInput, ProviderEvent, TranscriptionError};
use tokio::sync::mpsc;

use crate::protocol::{self, ServerEvent};
use crate::transport::Connection;

/// Runs until the pipeline drops its audio sender or the connection ends.
pub(crate) async fn run(
    mut connection: Connection,
    mut audio: mpsc::Receiver<AudioInput>,
    events: mpsc::Sender<ProviderEvent>,
) {
    let mut turns = Turns::default();
    loop {
        let outcome = tokio::select! {
            input = audio.recv() => match input {
                Some(AudioInput::Samples(samples)) => {
                    connection.send(protocol::append_audio(&samples)).await.map(|()| None)
                }
                Some(AudioInput::EndOfTurn) => {
                    connection.send(protocol::commit_audio()).await.map(|()| None)
                }
                // The pipeline is done; dropping the connection closes it.
                None => return,
            },
            message = connection.receive() => {
                message.and_then(|text| protocol::parse_server_event(&text)).and_then(|event| turns.translate(event))
            }
        };
        let event = match outcome {
            Ok(Some(event)) => event,
            Ok(None) => continue,
            Err(error) => ProviderEvent::Failed(error),
        };
        let ended = matches!(event, ProviderEvent::Failed(_));
        if events.send(event).await.is_err() || ended {
            return;
        }
    }
}

/// Keeps track of which OpenAI item belongs to which turn, and of the text
/// received so far for each.
///
/// OpenAI identifies each committed stretch of audio by an item id, while
/// the pipeline numbers turns from zero. Items are created in the same
/// order as turns, so the first time an item id is seen it is given the
/// next turn number.
#[derive(Default)]
struct Turns {
    next_turn: u64,
    by_item: HashMap<String, Turn>,
}

struct Turn {
    number: u64,
    text: String,
}

impl Turns {
    fn turn(&mut self, item_id: String) -> &mut Turn {
        self.by_item.entry(item_id).or_insert_with(|| {
            let number = self.next_turn;
            self.next_turn += 1;
            Turn {
                number,
                text: String::new(),
            }
        })
    }

    /// Converts a server event into a normalized event, if it carries one.
    fn translate(
        &mut self,
        event: ServerEvent,
    ) -> Result<Option<ProviderEvent>, TranscriptionError> {
        let translated = match event {
            ServerEvent::Committed { item_id } => {
                self.turn(item_id);
                None
            }
            ServerEvent::Delta { item_id, text } => {
                let turn = self.turn(item_id);
                turn.text.push_str(&text);
                Some(ProviderEvent::Partial {
                    turn: turn.number,
                    text: turn.text.clone(),
                })
            }
            ServerEvent::Completed {
                item_id,
                text,
                language,
            } => {
                let turn = self.turn(item_id.clone()).number;
                self.by_item.remove(&item_id);
                Some(ProviderEvent::Final {
                    turn,
                    text,
                    language,
                })
            }
            ServerEvent::ItemFailed { item_id } => {
                let turn = self.turn(item_id.clone()).number;
                self.by_item.remove(&item_id);
                Some(ProviderEvent::TurnFailed { turn })
            }
            // A commit with too little audio creates no item, so that turn
            // number is used up here to keep later turns aligned.
            ServerEvent::Error(error) if error.is_empty_commit() => {
                let turn = self.next_turn;
                self.next_turn += 1;
                Some(ProviderEvent::TurnFailed { turn })
            }
            ServerEvent::Error(error) => return Err(error.into_error()),
            ServerEvent::SessionUpdated | ServerEvent::Other => None,
        };
        Ok(translated)
    }
}
