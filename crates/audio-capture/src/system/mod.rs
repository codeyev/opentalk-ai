//! System audio capture: whatever the computer is playing.
//!
//! Each platform module exposes the same two items: a `SystemAudioStream`
//! type that captures until it is dropped, and a `start` function.

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub(crate) use macos::{SystemAudioStream, start};

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
pub(crate) use windows::{SystemAudioStream, start};

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
mod unsupported;
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub(crate) use unsupported::{SystemAudioStream, start};
