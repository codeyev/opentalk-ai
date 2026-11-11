//! Manual capture check: captures for a few seconds and prints input levels.
//!
//! ```text
//! cargo run -p opentalk-audio-capture --example capture_probe -- microphone system
//! ```
//!
//! Pass `microphone`, `system`, or both. Nothing is recorded or saved; only
//! loudness numbers are printed. A level that stays at zero while sound is
//! playing or someone is speaking means capture is not working.

use std::thread;
use std::time::Duration;

use opentalk_audio_capture::{CaptureObserver, CaptureSession, LevelsUpdate, list_microphones};
use opentalk_audio_core::{CaptureConfig, CaptureError, Level};

const CAPTURE_TIME: Duration = Duration::from_secs(4);

struct PrintLevels;

impl CaptureObserver for PrintLevels {
    fn levels(&self, update: LevelsUpdate) {
        println!(
            "microphone {}   system {}   dropped frames {}",
            describe(update.microphone),
            describe(update.system),
            update.dropped_frames
        );
    }

    fn interrupted(&self, error: CaptureError) {
        println!("capture interrupted: {error}");
    }
}

fn describe(level: Option<Level>) -> String {
    level.map_or_else(
        || "   (not captured)   ".to_owned(),
        |level| format!("peak {:.4} rms {:.4}", level.peak, level.rms),
    )
}

fn main() -> Result<(), CaptureError> {
    let sources: Vec<String> = std::env::args().skip(1).collect();
    let config = CaptureConfig {
        microphone: sources.iter().any(|source| source == "microphone"),
        microphone_id: None,
        system_audio: sources.iter().any(|source| source == "system"),
    };

    println!("Microphones:");
    for microphone in list_microphones()? {
        let marker = if microphone.is_default {
            " (default)"
        } else {
            ""
        };
        println!("  {}{marker}", microphone.name);
    }

    let session = CaptureSession::start(&config, PrintLevels)?;
    println!("Capturing for {} seconds...", CAPTURE_TIME.as_secs());
    thread::sleep(CAPTURE_TIME);
    session.stop();
    println!("Stopped.");
    Ok(())
}
