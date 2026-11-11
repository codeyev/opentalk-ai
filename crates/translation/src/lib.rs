//! Provider-neutral translation.
//!
//! - [`provider`] defines what a translation provider must implement.
//! - [`preservation`] checks a finished translation: numbers, links and
//!   glossary terms from the source must still be there. A translation
//!   that fails the check is still shown, but marked for the user to verify.
//!
//! - [`service`] combines the two: translate, then check.
//!
//! Nothing here knows about a specific vendor, and no text is logged or stored.

pub mod preservation;
pub mod prompt;
pub mod provider;
pub mod service;

pub use provider::{
    GlossaryTerm, Translation, TranslationError, TranslationEvent, TranslationProviderInfo,
    TranslationRequest, Translator,
};
pub use service::TranslationSettings;
