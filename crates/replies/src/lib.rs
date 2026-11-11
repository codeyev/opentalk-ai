//! Provider-neutral reply suggestions.
//!
//! - [`provider`] defines what a reply generator must implement.
//! - [`context`] keeps a bounded window of the conversation.
//! - [`validation`] checks what a generator returned before it is shown.
//! - [`coordinator`] decides when to generate, cancels requests that newer
//!   speech has made stale, and spaces requests out.
//!
//! Suggestions are only ever shown to the user. Nothing here can send a
//! message anywhere, and no conversation text is logged or stored.

pub mod context;
pub mod coordinator;
pub mod prompt;
pub mod provider;
pub mod validation;

pub use context::{ConversationContext, ConversationTurn, Speaker};
pub use coordinator::ReplyCoordinator;
pub use provider::{
    ReplyError, ReplyEvent, ReplyGenerator, ReplyProviderInfo, ReplyRequest, ReplySettings,
    ReplyStyle, ReplySuggestion,
};
