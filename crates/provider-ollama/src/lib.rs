//! Local AI through Ollama.
//!
//! [Ollama](https://ollama.com) runs language models on the user's own
//! computer and serves them over HTTP. This crate uses such a server for
//! translation and reply suggestions, so that text never leaves the machine.
//!
//! Two rules are enforced here rather than left to configuration:
//!
//! - The server address must be on this computer ([`LocalEndpoint`]), and
//!   requests bypass any system proxy. Nothing in this crate can make a
//!   request to another machine.
//! - Models are never downloaded. If the chosen model is not installed the
//!   request fails with [`OllamaError::ModelMissing`], and the user installs
//!   it themselves with Ollama's own tools.

mod adapters;
mod client;
mod endpoint;
#[cfg(test)]
mod testing;

pub use adapters::{OllamaReplyGenerator, OllamaTranslator, reply_info, translation_info};
pub use client::{LocalModel, OllamaClient, OllamaError, OllamaStatus};
pub use endpoint::{DEFAULT_ENDPOINT, LocalEndpoint};

/// Identifier of this provider in settings.
pub const PROVIDER_ID: &str = "ollama";
