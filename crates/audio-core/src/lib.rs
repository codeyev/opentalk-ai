//! Platform-neutral audio building blocks for OpenTalk AI.
//!
//! Capture adapters turn whatever the operating system delivers into
//! [`AudioFrame`]s (mono `f32`, see [`normalize`]) and push them into a
//! bounded [`queue`]. Nothing in this crate touches a device, the network or
//! the disk, and audio is only ever held in memory.

pub mod capture;
pub mod error;
pub mod frame;
pub mod level;
pub mod normalize;
pub mod queue;

pub use capture::{CaptureConfig, MicrophoneInfo};
pub use error::CaptureError;
pub use frame::{AudioFrame, AudioSource};
pub use level::{Level, LevelMeter};
pub use queue::{FrameReceiver, FrameSender, frame_queue};
