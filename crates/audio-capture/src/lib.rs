//! Native audio capture for OpenTalk AI.
//!
//! - Microphone: the operating system's input API through `cpal`
//!   (Core Audio on macOS, WASAPI on Windows).
//! - System audio: ScreenCaptureKit on macOS, WASAPI loopback on Windows.
//!
//! Start a [`CaptureSession`] to capture; drop or stop it to release every
//! device. Captured audio is only held in memory and is never written to
//! disk.

mod input_stream;
mod microphone;
mod session;
mod sink;
mod system;

pub use microphone::list_microphones;
pub use session::{CaptureObserver, CaptureSession, LevelsUpdate};
