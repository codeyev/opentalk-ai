//! Finding the stretches of audio that contain speech.
//!
//! A simple energy-based detector: audio is examined in short windows, and
//! a window counts as speech when it is clearly louder than the background
//! noise heard so far. A turn starts with the first speech window and ends
//! after a pause. Only turns are sent to a provider, so silence and
//! background noise never leave the machine.
//!
//! This is deliberately basic. It can be replaced by a better detector
//! without touching the rest of the pipeline.

use std::collections::VecDeque;

/// Length of one analysis window, in milliseconds.
const WINDOW_MS: u32 = 20;
/// A pause this long ends the turn.
const END_OF_TURN_SILENCE_MS: u32 = 700;
/// Audio kept from just before speech was detected, so the first sound is not cut off.
const LEAD_IN_MS: u32 = 300;
/// A turn is cut after this long even without a pause, to bound latency and memory.
const MAX_TURN_MS: u32 = 30_000;
/// Quietest level (RMS, 0 to 1) that can count as speech.
const MIN_SPEECH_LEVEL: f32 = 0.006;
/// Speech must be this many times louder than the background noise.
const NOISE_MARGIN: f32 = 3.0;
/// How quickly the background noise estimate follows the input (0 to 1).
const NOISE_TRACKING: f32 = 0.05;

/// What the caller should do with the audio it has pushed so far.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TurnAction {
    /// Speech began. The audio of the turn follows.
    Start,
    /// Audio that belongs to the current turn.
    Audio(Vec<i16>),
    /// The speaker paused; the current turn is complete.
    End,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    Silence,
    Speech { windows: u32, silent_windows: u32 },
}

/// Splits a stream of mono 16-bit samples into turns of speech.
#[derive(Debug)]
pub struct TurnDetector {
    window_samples: usize,
    state: State,
    /// Samples that do not yet fill a whole window.
    pending: Vec<i16>,
    /// The most recent silent windows, sent ahead of the next turn.
    lead_in: VecDeque<Vec<i16>>,
    noise_level: f32,
}

impl TurnDetector {
    /// Creates a detector for audio at `sample_rate` hertz.
    #[must_use]
    pub fn new(sample_rate: u32) -> Self {
        Self {
            window_samples: (sample_rate * WINDOW_MS / 1000).max(1) as usize,
            state: State::Silence,
            pending: Vec::new(),
            lead_in: VecDeque::new(),
            noise_level: 0.0,
        }
    }

    /// Feeds more audio and returns what to do with it, in order.
    pub fn push(&mut self, samples: &[i16]) -> Vec<TurnAction> {
        self.pending.extend_from_slice(samples);
        let mut actions = Vec::new();
        while self.pending.len() >= self.window_samples {
            let window: Vec<i16> = self.pending.drain(..self.window_samples).collect();
            self.process_window(window, &mut actions);
        }
        actions
    }

    /// Ends the current turn, if there is one, because no more audio is coming.
    pub fn finish(&mut self) -> Vec<TurnAction> {
        self.pending.clear();
        self.lead_in.clear();
        match self.state {
            State::Silence => Vec::new(),
            State::Speech { .. } => {
                self.state = State::Silence;
                vec![TurnAction::End]
            }
        }
    }

    fn process_window(&mut self, window: Vec<i16>, actions: &mut Vec<TurnAction>) {
        let level = level_of(&window);
        let is_speech = level > MIN_SPEECH_LEVEL.max(self.noise_level * NOISE_MARGIN);

        match self.state {
            State::Silence if is_speech => {
                let mut audio: Vec<i16> = self.lead_in.drain(..).flatten().collect();
                audio.extend(window);
                actions.push(TurnAction::Start);
                actions.push(TurnAction::Audio(audio));
                self.state = State::Speech {
                    windows: 1,
                    silent_windows: 0,
                };
            }
            State::Silence => {
                self.noise_level += (level - self.noise_level) * NOISE_TRACKING;
                self.lead_in.push_back(window);
                if self.lead_in.len() > (LEAD_IN_MS / WINDOW_MS) as usize {
                    self.lead_in.pop_front();
                }
            }
            State::Speech {
                windows,
                silent_windows,
            } => {
                actions.push(TurnAction::Audio(window));
                let windows = windows + 1;
                let silent_windows = if is_speech { 0 } else { silent_windows + 1 };
                let paused = silent_windows >= END_OF_TURN_SILENCE_MS / WINDOW_MS;
                let too_long = windows >= MAX_TURN_MS / WINDOW_MS;
                if paused || too_long {
                    actions.push(TurnAction::End);
                    self.state = State::Silence;
                } else {
                    self.state = State::Speech {
                        windows,
                        silent_windows,
                    };
                }
            }
        }
    }
}

/// Root-mean-square level of 16-bit samples, from 0 to 1.
#[expect(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    reason = "an approximate loudness; window lengths are small"
)]
fn level_of(samples: &[i16]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    let sum_of_squares: f64 = samples
        .iter()
        .map(|&sample| {
            let value = f64::from(sample) / f64::from(i16::MAX);
            value * value
        })
        .sum();
    (sum_of_squares / samples.len() as f64).sqrt() as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    const RATE: u32 = 24_000;
    const SAMPLES_PER_MS: usize = (RATE / 1000) as usize;

    fn silence(milliseconds: usize) -> Vec<i16> {
        vec![0; milliseconds * SAMPLES_PER_MS]
    }

    /// A loud square wave: crude, but unmistakably not silence.
    fn speech(milliseconds: usize) -> Vec<i16> {
        (0..milliseconds * SAMPLES_PER_MS)
            .map(|n| if (n / 24) % 2 == 0 { 8_000 } else { -8_000 })
            .collect()
    }

    fn count(actions: &[TurnAction], wanted: &TurnAction) -> usize {
        actions.iter().filter(|action| *action == wanted).count()
    }

    fn audio_samples(actions: &[TurnAction]) -> usize {
        actions
            .iter()
            .map(|action| match action {
                TurnAction::Audio(audio) => audio.len(),
                TurnAction::Start | TurnAction::End => 0,
            })
            .sum()
    }

    #[test]
    fn silence_produces_nothing() {
        let mut detector = TurnDetector::new(RATE);
        assert!(detector.push(&silence(2_000)).is_empty());
        assert!(detector.finish().is_empty());
    }

    #[test]
    fn speech_followed_by_a_pause_is_one_turn() {
        let mut detector = TurnDetector::new(RATE);
        let mut actions = detector.push(&silence(500));
        actions.extend(detector.push(&speech(1_000)));
        actions.extend(detector.push(&silence(1_000)));

        assert_eq!(actions.first(), Some(&TurnAction::Start));
        assert_eq!(actions.last(), Some(&TurnAction::End));
        assert_eq!(count(&actions, &TurnAction::Start), 1);
        assert_eq!(count(&actions, &TurnAction::End), 1);
    }

    #[test]
    fn a_turn_includes_the_lead_in_and_the_closing_pause() {
        let mut detector = TurnDetector::new(RATE);
        let mut actions = detector.push(&silence(500));
        actions.extend(detector.push(&speech(1_000)));
        actions.extend(detector.push(&silence(1_000)));

        let expected_ms = (LEAD_IN_MS + 1_000 + END_OF_TURN_SILENCE_MS) as usize;
        assert_eq!(audio_samples(&actions), expected_ms * SAMPLES_PER_MS);
    }

    #[test]
    fn a_short_pause_does_not_end_the_turn() {
        let mut detector = TurnDetector::new(RATE);
        let mut actions = detector.push(&speech(500));
        actions.extend(detector.push(&silence(300)));
        actions.extend(detector.push(&speech(500)));
        assert_eq!(count(&actions, &TurnAction::Start), 1);
        assert_eq!(count(&actions, &TurnAction::End), 0);
    }

    #[test]
    fn two_utterances_are_two_turns() {
        let mut detector = TurnDetector::new(RATE);
        let mut actions = Vec::new();
        for _ in 0..2 {
            actions.extend(detector.push(&speech(500)));
            actions.extend(detector.push(&silence(1_000)));
        }
        assert_eq!(count(&actions, &TurnAction::Start), 2);
        assert_eq!(count(&actions, &TurnAction::End), 2);
    }

    #[test]
    fn very_long_speech_is_cut_into_bounded_turns() {
        let mut detector = TurnDetector::new(RATE);
        let actions = detector.push(&speech(MAX_TURN_MS as usize + 1_000));
        assert_eq!(count(&actions, &TurnAction::End), 1);
        assert_eq!(count(&actions, &TurnAction::Start), 2);
    }

    #[test]
    fn finishing_mid_speech_ends_the_turn() {
        let mut detector = TurnDetector::new(RATE);
        detector.push(&speech(500));
        assert_eq!(detector.finish(), vec![TurnAction::End]);
        assert!(detector.finish().is_empty());
    }

    #[test]
    fn steady_background_noise_is_not_speech() {
        let mut detector = TurnDetector::new(RATE);
        // A constant hum, louder than the minimum speech level.
        let hum: Vec<i16> = (0..3_000 * SAMPLES_PER_MS)
            .map(|n| if (n / 100) % 2 == 0 { 400 } else { -400 })
            .collect();
        let actions = detector.push(&hum);
        // The detector may react while it first learns the noise level, but
        // it must settle: the last second contains no new turn.
        let mut settled = TurnDetector::new(RATE);
        settled.push(&hum);
        assert!(
            count(
                &settled.push(&hum[..1_000 * SAMPLES_PER_MS]),
                &TurnAction::Start
            ) == 0
        );
        assert!(count(&actions, &TurnAction::Start) <= 1);
    }

    #[test]
    fn audio_arriving_in_small_pieces_gives_the_same_result() {
        let mut input = silence(400);
        input.extend(speech(600));
        input.extend(silence(900));

        let whole = TurnDetector::new(RATE).push(&input);
        let mut detector = TurnDetector::new(RATE);
        let pieces: Vec<TurnAction> = input
            .chunks(77)
            .flat_map(|chunk| detector.push(chunk))
            .collect();

        assert_eq!(audio_samples(&whole), audio_samples(&pieces));
        assert_eq!(
            count(&whole, &TurnAction::End),
            count(&pieces, &TurnAction::End)
        );
    }
}
