//! A provider for any service that speaks the OpenAI Chat Completions API.
//!
//! Many services accept the same request format: OpenRouter and other
//! gateways, OpenAI itself, and local servers such as LM Studio. The user
//! supplies the service's address, a model name and, for a remote service,
//! an API key; this crate runs the shared translation and reply tasks on
//! it.
//!
//! The same services often offer speech output at `/audio/speech`, in
//! OpenAI's format; [`CompatibleSpeech`] uses that.
//!
//! Only the widely implemented parts of the API are used. Extensions that
//! one service accepts and another rejects are left out.

mod adapters;
mod client;
mod endpoint;
mod speech;

pub use adapters::{CompatibleReplyGenerator, CompatibleTranslator, reply_info, translation_info};
pub use client::ChatClient;
pub use endpoint::{DEFAULT_BASE_URL, Endpoint, EndpointError};
pub use speech::CompatibleSpeech;

/// Identifier of this provider in settings and in the credential store.
pub const PROVIDER_ID: &str = "custom";
