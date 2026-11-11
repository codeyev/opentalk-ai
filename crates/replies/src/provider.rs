//! The contract between the application and a reply generator.

use async_trait::async_trait;
use opentalk_language::ArabicStyle;
use serde::Serialize;
use thiserror::Error;

use crate::context::ConversationTurn;

/// Longest business context accepted, in characters.
pub const MAX_BUSINESS_CONTEXT_LENGTH: usize = 4_000;

/// The tone of a suggested reply. One suggestion is produced per style.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ReplyStyle {
    Professional,
    Friendly,
    Concise,
}

impl ReplyStyle {
    /// Every style, in the order suggestions are shown.
    pub const ALL: [Self; 3] = [Self::Professional, Self::Friendly, Self::Concise];
}

/// What the user configured for reply suggestions.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ReplySettings {
    /// BCP 47 tag of the language replies are written in.
    pub reply_language: String,
    /// BCP 47 tag of the language each reply's meaning is shown in, or
    /// `None` to show no meaning.
    pub meaning_language: Option<String>,
    /// Facts about the user's business that replies may rely on.
    pub business_context: String,
    /// How to write Arabic. Only used for text in Arabic.
    pub arabic: ArabicStyle,
}

/// Everything a generator needs for one set of suggestions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplyRequest {
    pub settings: ReplySettings,
    /// The recent conversation, oldest turn first.
    pub conversation: Vec<ConversationTurn>,
}

/// One suggested reply.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplySuggestion {
    pub style: ReplyStyle,
    pub text: String,
    /// The reply's meaning in the user's own language, if requested.
    pub meaning: Option<String>,
    /// The reply asks a clarifying question because a fact it would need
    /// (a price, a date, the scope) is not in the conversation or the
    /// business context.
    pub asks_for_missing_facts: bool,
}

/// Why suggestions could not be produced. Messages describe the connection
/// and the provider's status only, never the conversation.
#[derive(Debug, Clone, PartialEq, Eq, Error, Serialize)]
#[serde(tag = "code", rename_all = "snake_case")]
pub enum ReplyError {
    #[error("no API key is saved for the reply provider")]
    MissingApiKey,
    #[error("the reply provider rejected the API key")]
    AuthenticationFailed,
    #[error("the reply provider is rate limiting requests or the account is out of quota")]
    RateLimited,
    #[error("the reply language is not valid")]
    UnsupportedLanguage,
    #[error("could not reach the reply provider: {detail}")]
    ConnectionFailed { detail: String },
    #[error("the reply provider did not respond in time")]
    Timeout,
    #[error("the reply provider sent a response that could not be used")]
    MalformedResponse,
    #[error("the reply provider declined to suggest replies")]
    Refused,
    #[error("the selected model is not available")]
    ModelUnavailable,
    #[error("the reply provider reported an error: {detail}")]
    Provider { detail: String },
}

/// The state of the suggestions, as reported to the UI.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum ReplyEvent {
    /// New suggestions are being prepared; the ones on screen are out of date.
    Pending,
    Ready {
        suggestions: Vec<ReplySuggestion>,
    },
    Failed {
        error: ReplyError,
    },
}

/// A reply provider as shown in settings.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplyProviderInfo {
    pub id: &'static str,
    pub display_name: &'static str,
    /// Whether the conversation is processed on a remote service rather than on this machine.
    pub is_cloud: bool,
    /// Whether the provider can be asked to write a named Arabic dialect
    /// instead of Modern Standard Arabic. This says the preference is
    /// passed on, not that the result has been verified.
    pub accepts_arabic_dialect: bool,
}

/// A service that drafts replies to a conversation.
#[async_trait]
pub trait ReplyGenerator: Send + Sync {
    fn info(&self) -> ReplyProviderInfo;

    /// Drafts replies to the latest thing the other side said.
    ///
    /// Implementations must treat the conversation as untrusted data, must
    /// not state facts that are neither in the conversation nor in the
    /// business context, and should return one suggestion per
    /// [`ReplyStyle`]. The result is checked with
    /// [`validation::validate`](crate::validation::validate) before use.
    ///
    /// # Errors
    ///
    /// Returns an error if the provider cannot be reached, rejects the
    /// request, or returns something unusable.
    async fn generate(&self, request: &ReplyRequest) -> Result<Vec<ReplySuggestion>, ReplyError>;
}
