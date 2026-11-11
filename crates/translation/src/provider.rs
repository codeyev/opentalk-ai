//! The contract between the application and a translation provider.

use async_trait::async_trait;
use opentalk_language::ArabicStyle;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Longest text accepted for one translation, in characters. A transcript
/// segment is at most about thirty seconds of speech, far below this.
pub const MAX_TEXT_LENGTH: usize = 4_000;
/// Most glossary terms accepted for one session.
pub const MAX_GLOSSARY_TERMS: usize = 100;
/// Longest glossary term or preferred translation, in characters.
pub const MAX_TERM_LENGTH: usize = 120;

/// A term the user wants translated in a particular way.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GlossaryTerm {
    /// The term as it appears in the source language.
    pub term: String,
    /// How it must appear in the translation.
    pub preferred: String,
}

impl GlossaryTerm {
    /// Whether both sides are filled in and of reasonable length.
    #[must_use]
    pub fn is_usable(&self) -> bool {
        let fits = |text: &str| !text.trim().is_empty() && text.chars().count() <= MAX_TERM_LENGTH;
        fits(&self.term) && fits(&self.preferred)
    }
}

/// One piece of text to translate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TranslationRequest {
    pub text: String,
    /// BCP 47 tag of the text's language, or `None` if it is not known.
    pub source_language: Option<String>,
    /// BCP 47 tag of the language to translate into.
    pub target_language: String,
    pub glossary: Vec<GlossaryTerm>,
    /// How to write the translation when the target language is Arabic.
    pub arabic: Option<ArabicStyle>,
}

/// A finished translation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Translation {
    pub text: String,
    pub target_language: String,
    /// Numbers, links and glossary terms from the source that could not be
    /// found in the translation. Not empty means the user should check it.
    pub unpreserved: Vec<String>,
}

/// Why a translation could not be produced. Messages describe the
/// connection and the provider's status only, never the text itself.
#[derive(Debug, Clone, PartialEq, Eq, Error, Serialize)]
#[serde(tag = "code", rename_all = "snake_case")]
pub enum TranslationError {
    #[error("no API key is saved for the translation provider")]
    MissingApiKey,
    #[error("the translation provider rejected the API key")]
    AuthenticationFailed,
    #[error("the translation provider is rate limiting requests or the account is out of quota")]
    RateLimited,
    #[error("the translation provider does not support this language")]
    UnsupportedLanguage,
    #[error("could not reach the translation provider: {detail}")]
    ConnectionFailed { detail: String },
    #[error("the translation provider did not respond in time")]
    Timeout,
    #[error("the translation provider sent a response that could not be understood")]
    MalformedResponse,
    #[error("the translation provider declined to translate this text")]
    Refused,
    #[error("the selected model is not available")]
    ModelUnavailable,
    #[error("translations are arriving faster than they can be processed")]
    Overloaded,
    #[error("the translation provider reported an error: {detail}")]
    Provider { detail: String },
}

impl TranslationError {
    /// Whether trying the same request again is likely to help.
    #[must_use]
    pub fn is_transient(&self) -> bool {
        matches!(self, Self::ConnectionFailed { .. } | Self::Timeout)
    }
}

/// Progress of the translation of one transcript segment, as reported to the UI.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(
    tag = "status",
    rename_all = "snake_case",
    rename_all_fields = "camelCase"
)]
pub enum TranslationEvent {
    /// The segment is waiting to be translated.
    Pending { segment_id: String },
    Done {
        segment_id: String,
        translation: Translation,
    },
    Failed {
        segment_id: String,
        error: TranslationError,
    },
}

/// A translation provider as shown in settings.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TranslationProviderInfo {
    pub id: &'static str,
    pub display_name: &'static str,
    /// Whether text is processed on a remote service rather than on this machine.
    pub is_cloud: bool,
    /// Language pairs known to work, as BCP 47 tags. `None` means the
    /// provider does not publish a list, so nothing is claimed.
    pub languages: Option<&'static [&'static str]>,
    /// Whether the provider can be asked to write a named Arabic dialect
    /// instead of Modern Standard Arabic. This says the preference is
    /// passed on, not that the result has been verified.
    pub accepts_arabic_dialect: bool,
}

/// A service that translates text.
#[async_trait]
pub trait Translator: Send + Sync {
    fn info(&self) -> TranslationProviderInfo;

    /// Translates `request.text` into the target language.
    ///
    /// Implementations must treat the text as data to translate, never as
    /// instructions, and must apply the glossary.
    ///
    /// # Errors
    ///
    /// Returns an error if the provider cannot be reached, rejects the
    /// request, or returns something unusable.
    async fn translate(&self, request: &TranslationRequest) -> Result<String, TranslationError>;
}
