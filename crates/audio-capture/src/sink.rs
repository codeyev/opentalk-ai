//! Where a capture adapter hands over its audio.

use std::fmt::Display;
use std::sync::mpsc::SyncSender;
use std::time::Instant;

use opentalk_audio_core::{AudioFrame, AudioSource, CaptureError, FrameSender};

/// The output side of one audio source. Adapters call it from their audio
/// callbacks, so every method returns quickly and never blocks.
#[derive(Debug, Clone)]
pub(crate) struct SourceSink {
    source: AudioSource,
    session_start: Instant,
    frames: FrameSender,
    interruptions: SyncSender<CaptureError>,
}

impl SourceSink {
    pub(crate) fn new(
        source: AudioSource,
        session_start: Instant,
        frames: FrameSender,
        interruptions: SyncSender<CaptureError>,
    ) -> Self {
        Self {
            source,
            session_start,
            frames,
            interruptions,
        }
    }

    pub(crate) fn source(&self) -> AudioSource {
        self.source
    }

    /// Queues normalized mono audio, stamped with the time since the session started.
    pub(crate) fn push_mono(&self, sample_rate: u32, samples: &[f32]) {
        let captured_at = self.session_start.elapsed();
        for frame in AudioFrame::split(self.source, captured_at, sample_rate, samples) {
            self.frames.push(frame);
        }
    }

    /// Reports that the source stopped unexpectedly, for example because the
    /// device was unplugged. The session ends in response.
    pub(crate) fn report_interruption(&self, detail: impl Display) {
        let error = CaptureError::Interrupted {
            audio_source: self.source,
            detail: detail.to_string(),
        };
        // Never blocks an audio callback. The first report ends the session,
        // so later ones can be dropped, as can any sent while it shuts down.
        let _ = self.interruptions.try_send(error);
    }
}
