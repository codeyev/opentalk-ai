//! The Whisper model as a transcription provider.

use std::path::Path;

use async_trait::async_trait;
use opentalk_transcription::{
    AudioInput, ProviderCapabilities, ProviderEvent, ProviderLocation, StreamingTranscriber,
    TranscriptionConfig, TranscriptionError, TranscriptionSession,
};
use tokio::sync::mpsc;
use tokio::task;
use whisper_rs::WhisperState;

use crate::PROVIDER_ID;
use crate::recognizer::{self, Recognizer, SAMPLE_RATE};

/// Longest turn kept, in samples: a little more than the pipeline's own
/// limit, so memory stays bounded even if a turn is never ended.
const MAX_TURN_SAMPLES: usize = SAMPLE_RATE as usize * 40;

/// What this provider can do. Available without a model, for display in settings.
#[must_use]
pub fn capabilities() -> ProviderCapabilities {
    ProviderCapabilities {
        id: PROVIDER_ID,
        display_name: "Whisper",
        location: ProviderLocation::Local,
        // Whisper transcribes a finished utterance, not a live stream.
        streams_partial_results: false,
        detects_language: true,
        // The models are multilingual, with quality that varies by language.
        languages: None,
        input_sample_rate: SAMPLE_RATE,
    }
}

/// Speech recognition on this computer with a Whisper model.
pub struct WhisperTranscriber {
    recognizer: Recognizer,
}

impl WhisperTranscriber {
    /// Loads the model file at `path`. This reads the whole model into
    /// memory and can take a second or two.
    ///
    /// # Errors
    ///
    /// Returns an error if the file is not a model Whisper can load.
    pub fn load(path: &Path) -> Result<Self, TranscriptionError> {
        Recognizer::load(path)
            .map(|recognizer| Self { recognizer })
            .ok_or_else(|| model_error("the model file could not be loaded"))
    }
}

fn model_error(detail: &str) -> TranscriptionError {
    TranscriptionError::Provider {
        detail: detail.to_owned(),
    }
}

/// The two-letter code Whisper uses for a BCP 47 tag: `ar-EG` gives `ar`.
fn whisper_language(tag: &str) -> String {
    tag.split('-')
        .next()
        .unwrap_or_default()
        .to_ascii_lowercase()
}

#[async_trait]
impl StreamingTranscriber for WhisperTranscriber {
    fn capabilities(&self) -> ProviderCapabilities {
        capabilities()
    }

    async fn open(
        &self,
        config: &TranscriptionConfig,
    ) -> Result<TranscriptionSession, TranscriptionError> {
        let language = config.language.as_deref().map(whisper_language);
        if let Some(code) = &language
            && !recognizer::knows_language(code)
        {
            return Err(TranscriptionError::UnsupportedLanguage {
                language: code.clone(),
            });
        }
        let state = self
            .recognizer
            .new_state()
            .ok_or_else(|| model_error("the model could not be started"))?;

        let (session, audio, events) = TranscriptionSession::channel();
        tokio::spawn(run(state, language, audio, events));
        Ok(session)
    }
}

/// Collects the audio of each turn and transcribes it when the turn ends.
/// Runs until the pipeline drops its audio sender.
async fn run(
    mut state: WhisperState,
    language: Option<String>,
    mut audio: mpsc::Receiver<AudioInput>,
    events: mpsc::Sender<ProviderEvent>,
) {
    let mut turn_samples: Vec<i16> = Vec::new();
    let mut turn = 0;
    while let Some(input) = audio.recv().await {
        match input {
            AudioInput::Samples(samples) => {
                let room = MAX_TURN_SAMPLES.saturating_sub(turn_samples.len());
                turn_samples.extend(samples.into_iter().take(room));
            }
            AudioInput::EndOfTurn => {
                let samples = std::mem::take(&mut turn_samples);
                let turn_language = language.clone();
                // The model is heavy work, so it runs off the async threads.
                // Its state goes along and comes back for the next turn.
                let finished = task::spawn_blocking(move || {
                    let result =
                        recognizer::recognize(&mut state, &samples, turn_language.as_deref());
                    (state, result)
                })
                .await;
                let Ok((returned_state, result)) = finished else {
                    let _ = events
                        .send(ProviderEvent::Failed(model_error("the model stopped")))
                        .await;
                    return;
                };
                state = returned_state;

                let event = match result {
                    Some(recognition) => ProviderEvent::Final {
                        turn,
                        text: recognition.text,
                        language: recognition.detected_language.or_else(|| language.clone()),
                    },
                    None => ProviderEvent::TurnFailed { turn },
                };
                turn += 1;
                if events.send(event).await.is_err() {
                    return;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_is_declared_as_local_with_final_results_only() {
        let capabilities = capabilities();
        assert_eq!(capabilities.location, ProviderLocation::Local);
        assert!(!capabilities.streams_partial_results);
        assert_eq!(capabilities.input_sample_rate, 16_000);
    }

    #[test]
    fn regional_tags_are_reduced_to_whispers_language_code() {
        assert_eq!(whisper_language("ar-EG"), "ar");
        assert_eq!(whisper_language("EN"), "en");
    }

    #[test]
    fn a_file_that_is_not_a_model_cannot_be_loaded() {
        let path =
            std::env::temp_dir().join(format!("opentalk-not-a-model-{}.bin", std::process::id()));
        std::fs::write(&path, b"this is not a model").unwrap();
        let result = WhisperTranscriber::load(&path);
        let _ = std::fs::remove_file(&path);
        assert!(result.is_err());
    }

    /// The samples of the 16 kHz mono 16-bit WAV file named by `WHISPER_WAV`.
    fn recorded_samples() -> Vec<i16> {
        let path = std::env::var("WHISPER_WAV").expect("set WHISPER_WAV to a WAV file");
        let wav = std::fs::read(path).expect("the WAV file should be readable");
        // Skip the header; what follows the "data" marker is little-endian samples.
        let marker = wav
            .windows(4)
            .position(|window| window == b"data")
            .expect("a data chunk");
        let (samples, _incomplete) = wav[marker + 8..].as_chunks::<2>();
        samples
            .iter()
            .map(|&sample| i16::from_le_bytes(sample))
            .collect()
    }

    /// Transcribes real speech with a real model on this computer:
    ///
    /// ```text
    /// WHISPER_MODEL=/path/to/ggml-base.bin WHISPER_WAV=/path/to/16khz-mono.wav \
    ///   cargo test -p opentalk-provider-whisper -- --ignored live --nocapture
    /// ```
    #[tokio::test]
    #[ignore = "needs a Whisper model file and a 16 kHz mono 16-bit WAV file"]
    async fn live_model_transcribes_a_recording() {
        let model = std::env::var("WHISPER_MODEL").expect("set WHISPER_MODEL to a model file");
        let samples = recorded_samples();

        let started = std::time::Instant::now();
        let transcriber =
            WhisperTranscriber::load(Path::new(&model)).expect("the model should load");
        println!("model loaded in {:.1?}", started.elapsed());

        let mut session = transcriber
            .open(&TranscriptionConfig { language: None })
            .await
            .unwrap();
        let started = std::time::Instant::now();
        session
            .audio
            .send(AudioInput::Samples(samples))
            .await
            .unwrap();
        session.audio.send(AudioInput::EndOfTurn).await.unwrap();
        let event = session.events.recv().await.expect("a result");
        println!("transcribed in {:.1?}: {event:?}", started.elapsed());
        assert!(
            matches!(event, ProviderEvent::Final { turn: 0, ref text, .. } if !text.is_empty())
        );
    }

    /// Runs a recording through the app's real pipeline (resampling,
    /// speech detection, then Whisper), as captured audio would be. Uses
    /// the same variables as the test above.
    #[tokio::test]
    #[ignore = "needs a Whisper model file and a 16 kHz mono 16-bit WAV file"]
    async fn live_pipeline_turns_a_recording_into_a_transcript_segment() {
        use std::sync::Arc;
        use std::time::Duration;

        use opentalk_audio_core::{AudioFrame, AudioSource};
        use opentalk_transcription::{TranscriptEvent, pipeline};

        const FRAME_SAMPLES: usize = 320; // 20 ms at 16 kHz

        let model = std::env::var("WHISPER_MODEL").expect("set WHISPER_MODEL to a model file");
        let mut samples: Vec<f32> = recorded_samples()
            .into_iter()
            .map(|sample| f32::from(sample) / 32_768.0)
            .collect();
        // A second of silence, so the speech detector sees the turn end.
        samples.extend(std::iter::repeat_n(0.0, SAMPLE_RATE as usize));

        let transcriber = Arc::new(WhisperTranscriber::load(Path::new(&model)).unwrap());
        let (frames, frame_receiver) = mpsc::channel(samples.len() / FRAME_SAMPLES + 1);
        let (events, mut event_receiver) = mpsc::channel(64);
        for (index, chunk) in samples.chunks(FRAME_SAMPLES).enumerate() {
            let frame = AudioFrame {
                source: AudioSource::System,
                captured_at: Duration::from_millis(index as u64 * 20),
                sample_rate: SAMPLE_RATE,
                samples: chunk.to_vec(),
            };
            frames.send(frame).await.unwrap();
        }
        drop(frames);

        let config = TranscriptionConfig { language: None };
        pipeline::run(
            AudioSource::System,
            transcriber,
            config,
            frame_receiver,
            events,
        )
        .await;

        let mut segments = Vec::new();
        while let Some(event) = event_receiver.recv().await {
            if let TranscriptEvent::Segment { segment } = event {
                println!(
                    "{} ms [{:?}] {}",
                    segment.start_ms, segment.language, segment.text
                );
                segments.push(segment);
            }
        }
        assert!(!segments.is_empty(), "the pipeline produced no transcript");
        assert!(segments.iter().all(|segment| segment.is_final));
    }
}
