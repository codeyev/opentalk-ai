//! Running the Whisper model on one turn of speech.

use std::path::Path;
use std::sync::Arc;
use std::thread;

use whisper_rs::{
    FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters, WhisperState,
};

/// The sample rate Whisper models are trained on.
pub(crate) const SAMPLE_RATE: u32 = 16_000;
/// A segment Whisper itself rates this likely to be non-speech is dropped.
/// Whisper tends to invent short phrases ("Thank you.") for noise and silence.
const MAX_NO_SPEECH_PROBABILITY: f32 = 0.6;
/// Most threads to use; more does not help and starves the rest of the app.
const MAX_THREADS: usize = 8;

/// What Whisper heard in one turn.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Recognition {
    pub(crate) text: String,
    /// The language Whisper detected, as a two-letter code, when it was
    /// asked to detect it.
    pub(crate) detected_language: Option<String>,
}

/// A loaded Whisper model. Cheap to clone; the model is shared.
#[derive(Clone)]
pub(crate) struct Recognizer {
    context: Arc<WhisperContext>,
}

impl Recognizer {
    /// Loads the model file at `path`. Returns `None` if it cannot be loaded.
    pub(crate) fn load(path: &Path) -> Option<Self> {
        // Whisper otherwise writes its progress to the terminal.
        whisper_rs::install_logging_hooks();
        let context =
            WhisperContext::new_with_params(path, WhisperContextParameters::default()).ok()?;
        Some(Self {
            context: Arc::new(context),
        })
    }

    /// Creates the working memory for one session.
    pub(crate) fn new_state(&self) -> Option<WhisperState> {
        self.context.create_state().ok()
    }
}

/// Whether Whisper knows the language with this two-letter code.
pub(crate) fn knows_language(code: &str) -> bool {
    whisper_rs::get_lang_id(code).is_some()
}

/// Transcribes one turn of 16 kHz mono speech. `language` is a code
/// Whisper knows, or `None` to detect the language. Returns `None` if the
/// model fails to run.
pub(crate) fn recognize(
    state: &mut WhisperState,
    samples: &[i16],
    language: Option<&str>,
) -> Option<Recognition> {
    let audio: Vec<f32> = samples
        .iter()
        .map(|&sample| f32::from(sample) / 32_768.0)
        .collect();

    let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
    params.set_language(Some(language.unwrap_or("auto")));
    params.set_n_threads(thread_count());
    // Each turn stands alone; text from an earlier turn must not leak into this one.
    params.set_no_context(true);
    params.set_suppress_blank(true);
    params.set_suppress_nst(true);
    params.set_print_special(false);
    params.set_print_progress(false);
    params.set_print_realtime(false);
    params.set_print_timestamps(false);
    state.full(params, &audio).ok()?;

    let text = state
        .as_iter()
        .filter(|segment| segment.no_speech_probability() <= MAX_NO_SPEECH_PROBABILITY)
        .filter_map(|segment| {
            segment
                .to_str_lossy()
                .ok()
                .map(|text| text.trim().to_owned())
        })
        .filter(|segment| !is_annotation(segment))
        .collect::<Vec<_>>()
        .join(" ");
    let detected_language = language
        .is_none()
        .then(|| whisper_rs::get_lang_str(state.full_lang_id_from_state()))
        .flatten()
        .map(str::to_owned);
    Some(Recognition {
        text,
        detected_language,
    })
}

fn thread_count() -> i32 {
    let available = thread::available_parallelism().map_or(1, usize::from);
    i32::try_from(available.min(MAX_THREADS)).unwrap_or(1)
}

/// Whether a segment is a note about a sound rather than words, such as
/// `[BLANK_AUDIO]` or `(music)`. Whisper writes these for noise and silence.
pub(crate) fn is_annotation(segment: &str) -> bool {
    let wrapped_in =
        |open: char, close: char| segment.starts_with(open) && segment.ends_with(close);
    segment.is_empty() || wrapped_in('[', ']') || wrapped_in('(', ')') || wrapped_in('*', '*')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sound_notes_are_not_speech() {
        for note in ["[BLANK_AUDIO]", "(music)", "[Music]", "*sighs*", ""] {
            assert!(is_annotation(note), "{note:?}");
        }
        for speech in ["Hello there.", "It costs (about) ten.", "[1] First point"] {
            assert!(!is_annotation(speech), "{speech:?}");
        }
    }

    #[test]
    fn whisper_knows_common_languages_and_rejects_nonsense() {
        for code in ["en", "az", "ru", "ar", "tr"] {
            assert!(knows_language(code), "{code}");
        }
        assert!(!knows_language("zz"));
    }
}
