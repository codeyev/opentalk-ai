//! A capture session: the selected sources, running until stopped.

use std::panic::{self, AssertUnwindSafe};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use opentalk_audio_core::{
    AudioFrame, AudioSource, CaptureConfig, CaptureError, FrameReceiver, Level, LevelMeter,
    frame_queue,
};
use serde::Serialize;

use crate::sink::SourceSink;
use crate::{microphone, system};

/// Frames the queue can hold. At typical callback sizes (about 10 ms each,
/// two sources) this is a few seconds of audio; beyond it frames are dropped.
const QUEUE_CAPACITY: usize = 512;
/// How often input levels are reported to the observer.
const LEVEL_INTERVAL: Duration = Duration::from_millis(100);
/// Interruption reports that may wait to be read. One is enough to end the session.
const INTERRUPTION_QUEUE_CAPACITY: usize = 4;
/// Longest the worker waits for audio before checking whether to stop.
const POLL_INTERVAL: Duration = Duration::from_millis(20);

/// Input levels for the sources being captured.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelsUpdate {
    /// `None` when the microphone is not being captured.
    pub microphone: Option<Level>,
    /// `None` when system audio is not being captured.
    pub system: Option<Level>,
    /// Frames discarded so far because the queue was full.
    pub dropped_frames: u64,
}

/// Receives updates from a running session, on the session's worker thread.
pub trait CaptureObserver: Send + 'static {
    /// Called about ten times a second with the latest input levels.
    fn levels(&self, update: LevelsUpdate);

    /// Called with every captured frame, in order. The default discards
    /// it. Implementations must return quickly and must not block.
    fn audio(&self, frame: AudioFrame) {
        drop(frame);
    }

    /// Called once if capture stops by itself, for example because a device
    /// was unplugged. The session has released its devices by then.
    fn interrupted(&self, error: CaptureError);
}

/// A running capture. Stopping or dropping it releases every device and
/// discards any audio still in memory.
#[derive(Debug)]
pub struct CaptureSession {
    stop_requested: Arc<AtomicBool>,
    worker: Option<JoinHandle<()>>,
}

impl CaptureSession {
    /// Starts capturing the sources in `config`.
    ///
    /// Returns once every selected source is running, so a permission or
    /// device problem is reported here rather than later.
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration is invalid or any selected
    /// source cannot be started. Nothing is left running in that case.
    pub fn start(
        config: &CaptureConfig,
        observer: impl CaptureObserver,
    ) -> Result<Self, CaptureError> {
        config.validate()?;

        let stop_requested = Arc::new(AtomicBool::new(false));
        let (startup_sender, startup_result) = mpsc::channel();
        let worker_config = config.clone();
        let worker_stop = Arc::clone(&stop_requested);

        // Audio streams must stay on the thread that created them, so the
        // worker both opens the streams and consumes their audio.
        let worker = thread::Builder::new()
            .name("opentalk-capture".to_owned())
            .spawn(move || match Worker::start(&worker_config) {
                Ok(worker) => {
                    let _ = startup_sender.send(Ok(()));
                    // A panic while consuming audio must not leave the UI
                    // showing a live session. Unwinding drops the worker,
                    // which releases the devices; then the observer is told.
                    let run = AssertUnwindSafe(|| worker.run(&worker_stop, &observer));
                    if panic::catch_unwind(run).is_err() {
                        observer.interrupted(CaptureError::Backend {
                            audio_source: first_source(&worker_config),
                            detail: "audio processing stopped unexpectedly".to_owned(),
                        });
                    }
                }
                Err(error) => {
                    let _ = startup_sender.send(Err(error));
                }
            })
            .map_err(|error| CaptureError::Backend {
                audio_source: first_source(config),
                detail: error.to_string(),
            })?;

        let session = Self {
            stop_requested,
            worker: Some(worker),
        };
        match startup_result.recv() {
            Ok(Ok(())) => Ok(session),
            Ok(Err(error)) => Err(error),
            Err(_) => Err(CaptureError::Backend {
                audio_source: first_source(config),
                detail: "the capture thread ended unexpectedly".to_owned(),
            }),
        }
    }

    /// Whether capture is still going. It becomes `false` once the session
    /// has been interrupted and has released its devices.
    #[must_use]
    pub fn is_running(&self) -> bool {
        self.worker
            .as_ref()
            .is_some_and(|worker| !worker.is_finished())
    }

    /// Stops capture and waits until every device has been released.
    pub fn stop(mut self) {
        self.shut_down();
    }

    fn shut_down(&mut self) {
        self.stop_requested.store(true, Ordering::Relaxed);
        if let Some(worker) = self.worker.take() {
            // A panicked worker has already dropped its streams; nothing more to do.
            let _ = worker.join();
        }
    }
}

impl Drop for CaptureSession {
    fn drop(&mut self) {
        self.shut_down();
    }
}

fn first_source(config: &CaptureConfig) -> AudioSource {
    if config.microphone {
        AudioSource::Microphone
    } else {
        AudioSource::System
    }
}

/// Owns the native streams and consumes their audio on the worker thread.
/// Dropping it stops the streams and discards any queued audio.
struct Worker {
    frames: FrameReceiver,
    interruptions: Receiver<CaptureError>,
    /// One meter per captured source; `None` for a source that is not captured.
    microphone_meter: Option<LevelMeter>,
    system_meter: Option<LevelMeter>,
    /// Held only to keep capture running; dropping a stream stops it.
    _microphone_stream: Option<cpal::Stream>,
    _system_stream: Option<system::SystemAudioStream>,
}

impl Worker {
    fn start(config: &CaptureConfig) -> Result<Self, CaptureError> {
        let session_start = Instant::now();
        let (frame_sender, frames) = frame_queue(QUEUE_CAPACITY);
        let (interruption_sender, interruptions) = mpsc::sync_channel(INTERRUPTION_QUEUE_CAPACITY);
        let sink = |source| {
            SourceSink::new(
                source,
                session_start,
                frame_sender.clone(),
                interruption_sender.clone(),
            )
        };

        let microphone_stream = config
            .microphone
            .then(|| {
                microphone::start(
                    config.microphone_id.as_deref(),
                    sink(AudioSource::Microphone),
                )
            })
            .transpose()?;
        let system_stream = config
            .system_audio
            .then(|| system::start(sink(AudioSource::System)))
            .transpose()?;

        Ok(Self {
            frames,
            interruptions,
            microphone_meter: microphone_stream.as_ref().map(|_| LevelMeter::default()),
            system_meter: system_stream.as_ref().map(|_| LevelMeter::default()),
            _microphone_stream: microphone_stream,
            _system_stream: system_stream,
        })
    }

    /// Consumes audio until asked to stop or a source is interrupted.
    ///
    /// Each frame is measured for its level and then handed to the observer,
    /// which decides whether anything else happens to it.
    fn run(mut self, stop_requested: &AtomicBool, observer: &impl CaptureObserver) {
        let mut last_report = Instant::now();
        while !stop_requested.load(Ordering::Relaxed) {
            if let Ok(error) = self.interruptions.try_recv() {
                // Release the devices before telling the observer capture has ended.
                drop(self);
                observer.interrupted(error);
                return;
            }
            if let Some(frame) = self.frames.next(POLL_INTERVAL) {
                let meter = match frame.source {
                    AudioSource::Microphone => &mut self.microphone_meter,
                    AudioSource::System => &mut self.system_meter,
                };
                if let Some(meter) = meter {
                    meter.add(&frame.samples);
                }
                observer.audio(frame);
            }
            if last_report.elapsed() >= LEVEL_INTERVAL {
                last_report = Instant::now();
                observer.levels(self.take_levels());
            }
        }
    }

    fn take_levels(&mut self) -> LevelsUpdate {
        LevelsUpdate {
            microphone: self.microphone_meter.as_mut().map(LevelMeter::take),
            system: self.system_meter.as_mut().map(LevelMeter::take),
            dropped_frames: self.frames.dropped_frames(),
        }
    }
}
