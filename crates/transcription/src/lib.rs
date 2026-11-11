//! Provider-neutral streaming transcription.
//!
//! - [`provider`] defines what a transcription provider must implement.
//! - [`pipeline`] turns one source's audio frames into transcript events:
//!   it resamples ([`resample`]), keeps only the stretches that contain
//!   speech ([`turns`]) and streams those to the provider.
//! - [`transcript`] holds the normalized results the UI receives.
//!
//! Nothing here knows about a specific vendor, and neither audio nor
//! transcript text is ever logged or written to disk.

pub mod pipeline;
pub mod provider;
pub mod resample;
pub mod transcript;
pub mod turns;

pub use provider::{
    AudioInput, ProviderCapabilities, ProviderEvent, ProviderLocation, StreamingTranscriber,
    TranscriptionConfig, TranscriptionError, TranscriptionSession,
};
pub use transcript::{TranscriptEvent, TranscriptSegment};
