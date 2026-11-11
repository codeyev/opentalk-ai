//! Turns one source's audio into transcript events.
//!
//! ```text
//! frames -> resample -> detect turns -> provider -> transcript events
//! ```
//!
//! One pipeline runs per audio source, so the microphone and system audio
//! stay on separate tracks. Every queue involved is bounded: if the provider
//! falls behind, the pipeline stops reading frames, the frame queue fills
//! up, and the capture side drops new frames instead of using more memory.

use std::sync::Arc;
use std::time::Duration;

use opentalk_audio_core::{AudioFrame, AudioSource};
use tokio::sync::mpsc;
use tokio::time;

use crate::provider::{
    AudioInput, ProviderEvent, StreamingTranscriber, TranscriptionConfig, TranscriptionError,
    TranscriptionSession,
};
use crate::resample::{self, Resampler};
use crate::transcript::{TranscriptEvent, TranscriptSegment};
use crate::turns::{TurnAction, TurnDetector};

/// How many times in a row a lost connection is reopened before giving up.
const MAX_RECONNECTS: u32 = 3;
/// Wait before the first reconnect; each further attempt waits this much longer.
const RECONNECT_DELAY: Duration = Duration::from_secs(1);
/// After capture stops, how long to wait for the provider to finish pending turns.
const FINAL_RESULTS_TIMEOUT: Duration = Duration::from_secs(5);

/// Transcribes `frames` from one source until the frame queue closes, which
/// happens when capture stops. Results are sent to `events`.
pub async fn run(
    source: AudioSource,
    transcriber: Arc<dyn StreamingTranscriber>,
    config: TranscriptionConfig,
    mut frames: mpsc::Receiver<AudioFrame>,
    events: mpsc::Sender<TranscriptEvent>,
) {
    let mut failed_attempts = 0;
    let mut session_number = 0;
    loop {
        session_number += 1;
        let session = match transcriber.open(&config).await {
            Ok(session) => session,
            Err(error) => {
                let _ = events.send(TranscriptEvent::Failed { source, error }).await;
                return;
            }
        };
        let mut active = ActiveSession {
            source,
            id_prefix: format!("{}-{session_number}", source_name(source)),
            language: config.language.clone(),
            input_rate: transcriber.capabilities().input_sample_rate,
            session,
            events: events.clone(),
            resampler: None,
            detector: TurnDetector::new(transcriber.capabilities().input_sample_rate),
            turn_starts_ms: Vec::new(),
            turns_ended: 0,
            turns_resolved: 0,
        };

        match active.run(&mut frames).await {
            SessionEnd::Stopped => return,
            SessionEnd::Failed(error) => {
                // A session that got results was healthy, so start counting afresh.
                if active.turns_resolved > 0 {
                    failed_attempts = 0;
                }
                failed_attempts += 1;
                if !error.is_transient() || failed_attempts > MAX_RECONNECTS {
                    let _ = events.send(TranscriptEvent::Failed { source, error }).await;
                    return;
                }
                time::sleep(RECONNECT_DELAY * failed_attempts).await;
            }
        }
    }
}

fn source_name(source: AudioSource) -> &'static str {
    match source {
        AudioSource::Microphone => "microphone",
        AudioSource::System => "system",
    }
}

/// Why a provider session ended.
enum SessionEnd {
    /// Capture stopped, or nobody is listening for results any more.
    Stopped,
    Failed(TranscriptionError),
}

fn connection_lost() -> SessionEnd {
    SessionEnd::Failed(TranscriptionError::ConnectionLost {
        detail: "the provider session ended".to_owned(),
    })
}

/// One provider session and the state needed to feed it.
struct ActiveSession {
    source: AudioSource,
    /// Start of every segment id from this session, unique per source and session.
    id_prefix: String,
    language: Option<String>,
    input_rate: u32,
    session: TranscriptionSession,
    events: mpsc::Sender<TranscriptEvent>,
    /// Created for the first frame, and again if the device's rate changes.
    resampler: Option<(u32, Resampler)>,
    detector: TurnDetector,
    /// When each turn started, indexed by turn number.
    turn_starts_ms: Vec<u64>,
    turns_ended: u64,
    turns_resolved: u64,
}

impl ActiveSession {
    async fn run(&mut self, frames: &mut mpsc::Receiver<AudioFrame>) -> SessionEnd {
        loop {
            let step = tokio::select! {
                frame = frames.recv() => match frame {
                    Some(frame) => self.feed(&frame).await,
                    None => return self.finish().await,
                },
                event = self.session.events.recv() => self.handle(event).await,
            };
            if let Err(end) = step {
                return end;
            }
        }
    }

    /// Resamples a frame, finds the speech in it and sends that to the provider.
    async fn feed(&mut self, frame: &AudioFrame) -> Result<(), SessionEnd> {
        let samples = resample::to_pcm16(&self.resample(frame));
        let started_ms = u64::try_from(frame.captured_at.as_millis()).unwrap_or(u64::MAX);
        for action in self.detector.push(&samples) {
            self.apply(action, started_ms).await?;
        }
        Ok(())
    }

    fn resample(&mut self, frame: &AudioFrame) -> Vec<f32> {
        let input_rate = self.input_rate;
        let (_, resampler) = self
            .resampler
            .take_if(|(rate, _)| *rate == frame.sample_rate)
            .unwrap_or_else(|| {
                (
                    frame.sample_rate,
                    Resampler::new(frame.sample_rate, input_rate),
                )
            });
        let (_, resampler) = self.resampler.insert((frame.sample_rate, resampler));
        resampler.process(&frame.samples)
    }

    async fn apply(&mut self, action: TurnAction, started_ms: u64) -> Result<(), SessionEnd> {
        match action {
            TurnAction::Start => {
                self.turn_starts_ms.push(started_ms);
                Ok(())
            }
            TurnAction::Audio(samples) => self.send(AudioInput::Samples(samples)).await,
            TurnAction::End => {
                self.turns_ended += 1;
                self.send(AudioInput::EndOfTurn).await
            }
        }
    }

    /// Sends audio to the provider. While waiting for room in the provider's
    /// queue it keeps handling results, so neither side can block the other.
    async fn send(&mut self, input: AudioInput) -> Result<(), SessionEnd> {
        let audio = self.session.audio.clone();
        loop {
            let event = tokio::select! {
                permit = audio.reserve() => match permit {
                    Ok(permit) => {
                        permit.send(input);
                        return Ok(());
                    }
                    Err(_) => return Err(self.reason_session_ended().await),
                },
                event = self.session.events.recv() => event,
            };
            self.handle(event).await?;
        }
    }

    /// The provider stopped accepting audio. Reads the results it sent before
    /// closing, so that the error it reported, if any, is the one passed on
    /// rather than a generic "connection lost".
    async fn reason_session_ended(&mut self) -> SessionEnd {
        loop {
            let event = self.session.events.recv().await;
            if let Err(end) = self.handle(event).await {
                return end;
            }
        }
    }

    /// Capture stopped: close the current turn and collect the results still on their way.
    async fn finish(&mut self) -> SessionEnd {
        for action in self.detector.finish() {
            if self.apply(action, 0).await.is_err() {
                return SessionEnd::Stopped;
            }
        }
        let wait_for_results = async {
            while self.turns_resolved < self.turns_ended {
                let event = self.session.events.recv().await;
                if self.handle(event).await.is_err() {
                    break;
                }
            }
        };
        // If the provider is slow the remaining turns are simply not shown.
        let _ = time::timeout(FINAL_RESULTS_TIMEOUT, wait_for_results).await;
        SessionEnd::Stopped
    }

    /// Converts a provider event into a transcript event for the application.
    async fn handle(&mut self, event: Option<ProviderEvent>) -> Result<(), SessionEnd> {
        let transcript_event = match event.ok_or_else(connection_lost)? {
            ProviderEvent::Failed(error) => return Err(SessionEnd::Failed(error)),
            ProviderEvent::Partial { turn, text } => self.segment(turn, text, None, false),
            ProviderEvent::Final {
                turn,
                text,
                language,
            } => {
                self.turns_resolved += 1;
                self.segment(turn, text, language, true)
            }
            ProviderEvent::TurnFailed { turn } => {
                self.turns_resolved += 1;
                Some(TranscriptEvent::Discarded {
                    id: self.segment_id(turn),
                })
            }
        };
        match transcript_event {
            Some(event) => self
                .events
                .send(event)
                .await
                .map_err(|_| SessionEnd::Stopped),
            None => Ok(()),
        }
    }

    /// Builds the event for a turn's text. Returns `None` for a turn this
    /// session never started, and for partial text that is still empty.
    fn segment(
        &self,
        turn: u64,
        text: String,
        language: Option<String>,
        is_final: bool,
    ) -> Option<TranscriptEvent> {
        let start_ms = *self.turn_starts_ms.get(usize::try_from(turn).ok()?)?;
        let id = self.segment_id(turn);
        if text.trim().is_empty() {
            // A finished turn without words was noise, not speech.
            return is_final.then_some(TranscriptEvent::Discarded { id });
        }
        Some(TranscriptEvent::Segment {
            segment: TranscriptSegment {
                id,
                source: self.source,
                start_ms,
                text,
                language: language.or_else(|| self.language.clone()),
                is_final,
            },
        })
    }

    fn segment_id(&self, turn: u64) -> String {
        format!("{}-{turn}", self.id_prefix)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use async_trait::async_trait;

    use super::*;
    use crate::provider::{ProviderCapabilities, ProviderLocation};

    const RATE: u32 = 24_000;
    const FRAME_MS: u64 = 20;
    const FRAME_SAMPLES: usize = (RATE as u64 * FRAME_MS / 1000) as usize;

    /// What a fake provider session does when it is opened.
    #[derive(Clone)]
    enum Script {
        /// Answers every turn with a partial and then a final result.
        Transcribe,
        /// Refuses to open.
        RefuseToOpen(TranscriptionError),
        /// Opens, then fails as soon as the first audio arrives.
        FailOnAudio(TranscriptionError),
    }

    /// A provider that follows one script per session, in order, and counts the audio it receives.
    struct FakeTranscriber {
        scripts: Mutex<Vec<Script>>,
        samples_received: Arc<Mutex<usize>>,
    }

    impl FakeTranscriber {
        fn new(scripts: Vec<Script>) -> Arc<Self> {
            Arc::new(Self {
                scripts: Mutex::new(scripts),
                samples_received: Arc::default(),
            })
        }

        fn samples_received(&self) -> usize {
            *self.samples_received.lock().unwrap()
        }
    }

    #[async_trait]
    impl StreamingTranscriber for FakeTranscriber {
        fn capabilities(&self) -> ProviderCapabilities {
            ProviderCapabilities {
                id: "fake",
                display_name: "Fake",
                location: ProviderLocation::Local,
                streams_partial_results: true,
                detects_language: false,
                languages: None,
                input_sample_rate: RATE,
            }
        }

        async fn open(
            &self,
            _config: &TranscriptionConfig,
        ) -> Result<TranscriptionSession, TranscriptionError> {
            let script = self.scripts.lock().unwrap().remove(0);
            if let Script::RefuseToOpen(error) = script {
                return Err(error);
            }
            let (session, mut audio, events) = TranscriptionSession::channel();
            let samples_received = Arc::clone(&self.samples_received);
            tokio::spawn(async move {
                let mut turn = 0;
                while let Some(input) = audio.recv().await {
                    if let Script::FailOnAudio(error) = &script {
                        let _ = events.send(ProviderEvent::Failed(error.clone())).await;
                        return;
                    }
                    match input {
                        AudioInput::Samples(samples) => {
                            *samples_received.lock().unwrap() += samples.len();
                        }
                        AudioInput::EndOfTurn => {
                            let partial = ProviderEvent::Partial {
                                turn,
                                text: format!("turn {turn}"),
                            };
                            let done = ProviderEvent::Final {
                                turn,
                                text: format!("turn {turn} done"),
                                language: None,
                            };
                            if events.send(partial).await.is_err()
                                || events.send(done).await.is_err()
                            {
                                return;
                            }
                            turn += 1;
                        }
                    }
                }
            });
            Ok(session)
        }
    }

    fn frame(index: u64, loud: bool) -> AudioFrame {
        let samples = (0..FRAME_SAMPLES)
            .map(|n| {
                if !loud {
                    0.0
                } else if (n / 24) % 2 == 0 {
                    0.4
                } else {
                    -0.4
                }
            })
            .collect();
        AudioFrame {
            source: AudioSource::Microphone,
            captured_at: Duration::from_millis(index * FRAME_MS),
            sample_rate: RATE,
            samples,
        }
    }

    /// Frames for `speech_ms` of speech starting at `start_ms`, followed by a second of silence.
    fn utterance(start_ms: u64, speech_ms: u64) -> Vec<AudioFrame> {
        let first = start_ms / FRAME_MS;
        let speech_frames = speech_ms / FRAME_MS;
        (0..speech_frames + 1_000 / FRAME_MS)
            .map(|offset| frame(first + offset, offset < speech_frames))
            .collect()
    }

    /// Runs the pipeline over `frames`, then stops capture and returns every event.
    async fn transcribe(
        transcriber: Arc<FakeTranscriber>,
        frames: Vec<AudioFrame>,
    ) -> Vec<TranscriptEvent> {
        let (frame_sender, frame_receiver) = mpsc::channel(frames.len().max(1));
        let (event_sender, mut event_receiver) = mpsc::channel(64);
        for frame in frames {
            frame_sender.send(frame).await.unwrap();
        }
        drop(frame_sender);

        let config = TranscriptionConfig {
            language: Some("en".to_owned()),
        };
        run(
            AudioSource::Microphone,
            transcriber,
            config,
            frame_receiver,
            event_sender,
        )
        .await;

        let mut events = Vec::new();
        while let Some(event) = event_receiver.recv().await {
            events.push(event);
        }
        events
    }

    fn segments(events: &[TranscriptEvent]) -> Vec<&TranscriptSegment> {
        events
            .iter()
            .filter_map(|event| match event {
                TranscriptEvent::Segment { segment } => Some(segment),
                TranscriptEvent::Discarded { .. } | TranscriptEvent::Failed { .. } => None,
            })
            .collect()
    }

    #[tokio::test(start_paused = true)]
    async fn speech_becomes_a_partial_and_then_a_final_segment() {
        let transcriber = FakeTranscriber::new(vec![Script::Transcribe]);
        let events = transcribe(transcriber, utterance(2_000, 600)).await;

        let segments = segments(&events);
        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0].text, "turn 0");
        assert!(!segments[0].is_final);
        assert_eq!(segments[1].text, "turn 0 done");
        assert!(segments[1].is_final);
        // The partial and the final are the same segment.
        assert_eq!(segments[0].id, segments[1].id);
        assert_eq!(segments[1].source, AudioSource::Microphone);
        assert_eq!(segments[1].language.as_deref(), Some("en"));
    }

    #[tokio::test(start_paused = true)]
    async fn a_segment_starts_when_the_speech_started() {
        let transcriber = FakeTranscriber::new(vec![Script::Transcribe]);
        let events = transcribe(transcriber, utterance(2_000, 600)).await;
        let start_ms = segments(&events)[0].start_ms;
        // Allow for the filter delay and window size of the detector.
        assert!(
            (2_000..2_100).contains(&start_ms),
            "started at {start_ms} ms"
        );
    }

    #[tokio::test(start_paused = true)]
    async fn silence_is_never_sent_to_the_provider() {
        let transcriber = FakeTranscriber::new(vec![Script::Transcribe]);
        let silence = (0..200).map(|index| frame(index, false)).collect();
        let events = transcribe(Arc::clone(&transcriber), silence).await;
        assert!(events.is_empty());
        assert_eq!(transcriber.samples_received(), 0);
    }

    #[tokio::test(start_paused = true)]
    async fn separate_utterances_get_separate_segment_ids() {
        let transcriber = FakeTranscriber::new(vec![Script::Transcribe]);
        let mut frames = utterance(0, 500);
        frames.extend(utterance(3_000, 500));
        let events = transcribe(transcriber, frames).await;

        let finals: Vec<_> = segments(&events)
            .into_iter()
            .filter(|segment| segment.is_final)
            .collect();
        assert_eq!(finals.len(), 2);
        assert_ne!(finals[0].id, finals[1].id);
        assert!(finals[0].start_ms < finals[1].start_ms);
    }

    #[tokio::test(start_paused = true)]
    async fn speech_cut_off_by_stop_is_still_transcribed() {
        let transcriber = FakeTranscriber::new(vec![Script::Transcribe]);
        // Speech with no closing silence: capture stops mid-sentence.
        let frames = (0..40).map(|index| frame(index, true)).collect();
        let events = transcribe(transcriber, frames).await;
        assert!(segments(&events).iter().any(|segment| segment.is_final));
    }

    #[tokio::test(start_paused = true)]
    async fn a_lost_connection_is_reopened() {
        let lost = TranscriptionError::ConnectionLost {
            detail: "reset".to_owned(),
        };
        let transcriber = FakeTranscriber::new(vec![Script::FailOnAudio(lost), Script::Transcribe]);
        let mut frames = utterance(0, 500);
        frames.extend(utterance(3_000, 500));
        let events = transcribe(transcriber, frames).await;

        assert!(segments(&events).iter().any(|segment| segment.is_final));
        assert!(
            !events
                .iter()
                .any(|event| matches!(event, TranscriptEvent::Failed { .. }))
        );
    }

    #[tokio::test(start_paused = true)]
    async fn repeated_connection_losses_end_with_a_failure() {
        let lost = TranscriptionError::ConnectionLost {
            detail: "reset".to_owned(),
        };
        let scripts = vec![Script::FailOnAudio(lost.clone()); (MAX_RECONNECTS + 1) as usize];
        let frames = (0..5).flat_map(|n| utterance(n * 3_000, 500)).collect();
        let events = transcribe(FakeTranscriber::new(scripts), frames).await;

        assert_eq!(
            events.last(),
            Some(&TranscriptEvent::Failed {
                source: AudioSource::Microphone,
                error: lost,
            })
        );
    }

    #[tokio::test(start_paused = true)]
    async fn a_rejected_key_fails_without_retrying() {
        let scripts = vec![Script::RefuseToOpen(
            TranscriptionError::AuthenticationFailed,
        )];
        let events = transcribe(FakeTranscriber::new(scripts), utterance(0, 500)).await;
        assert_eq!(
            events,
            vec![TranscriptEvent::Failed {
                source: AudioSource::Microphone,
                error: TranscriptionError::AuthenticationFailed,
            }]
        );
    }
}
