//! Local speech recognition with Whisper, built into the app.
//!
//! The Whisper engine (whisper.cpp) is compiled into the application, so
//! transcription needs no other program and no network: audio is turned
//! into text on the user's own computer. The only thing the user needs is
//! a model file, which [`models`] lists, and which [`download`] fetches
//! when, and only when, the user asks for it.
//!
//! Whisper works on whole utterances, not on a live stream. It therefore
//! reports one final result per turn of speech and no partial text.

pub mod download;
pub mod models;
mod recognizer;
mod transcriber;

pub use transcriber::{WhisperTranscriber, capabilities};

/// Identifier of this provider in settings.
pub const PROVIDER_ID: &str = "whisper";
