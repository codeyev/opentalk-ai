//! macOS system audio through ScreenCaptureKit.
//!
//! ScreenCaptureKit is Apple's supported way to capture what the computer
//! plays. It needs the user's "Screen & System Audio Recording" permission
//! and always attaches to a display, so a tiny, slow video stream is
//! configured alongside the audio and its frames are ignored.

use opentalk_audio_core::{AudioSource, CaptureError, normalize};
use screencapturekit::prelude::*;

use crate::sink::SourceSink;

const SOURCE: AudioSource = AudioSource::System;
const SAMPLE_RATE: u32 = 48_000;
const CHANNELS: u32 = 2;
/// Smallest video size ScreenCaptureKit accepts comfortably; the video is unused.
const UNUSED_VIDEO_SIZE: u32 = 2;
const BYTES_PER_SAMPLE: usize = size_of::<f32>();

/// A running ScreenCaptureKit stream. Stops when dropped.
pub(crate) struct SystemAudioStream {
    stream: SCStream,
}

impl Drop for SystemAudioStream {
    fn drop(&mut self) {
        // Stopping can only fail if the stream already stopped, which is the goal.
        let _ = self.stream.stop_capture();
    }
}

pub(crate) fn start(sink: SourceSink) -> Result<SystemAudioStream, CaptureError> {
    let content = SCShareableContent::get().map_err(content_error)?;
    let display = content
        .displays()
        .into_iter()
        .next()
        .ok_or(CaptureError::DeviceUnavailable {
            audio_source: SOURCE,
        })?;

    let filter = SCContentFilter::create()
        .with_display(&display)
        .with_excluding_windows(&[])
        .build()
        .map_err(backend_error)?;
    let configuration = SCStreamConfiguration::new()
        .with_width(UNUSED_VIDEO_SIZE)
        .with_height(UNUSED_VIDEO_SIZE)
        .with_minimum_frame_interval(&CMTime::new(1, 1))
        .with_captures_audio(true)
        .with_excludes_current_process_audio(true)
        .with_sample_rate(SAMPLE_RATE.cast_signed())
        .with_channel_count(CHANNELS.cast_signed());

    let delegate = InterruptionReporter { sink: sink.clone() };
    let mut stream =
        SCStream::new_with_delegate(&filter, &configuration, delegate).map_err(backend_error)?;
    stream
        .add_output_handler(AudioHandler { sink }, SCStreamOutputType::Audio)
        .map_err(backend_error)?;
    stream.start_capture().map_err(backend_error)?;
    Ok(SystemAudioStream { stream })
}

/// Listing shareable content is the step that fails when the user has not
/// granted the screen and system audio recording permission.
fn content_error(error: SCError) -> CaptureError {
    match error {
        SCError::PermissionDenied(_) | SCError::NoShareableContent(_) => {
            CaptureError::PermissionDenied {
                audio_source: SOURCE,
            }
        }
        other => backend_error(other),
    }
}

fn backend_error(detail: impl ToString) -> CaptureError {
    CaptureError::Backend {
        audio_source: SOURCE,
        detail: detail.to_string(),
    }
}

/// Tells the session when ScreenCaptureKit stops the stream on its own,
/// for example when the user revokes permission.
struct InterruptionReporter {
    sink: SourceSink,
}

impl SCStreamDelegateTrait for InterruptionReporter {
    fn did_stop_with_error(&self, error: SCError) {
        self.sink.report_interruption(error);
    }
}

/// Converts each audio buffer ScreenCaptureKit delivers into mono frames.
struct AudioHandler {
    sink: SourceSink,
}

impl SCStreamOutputTrait for AudioHandler {
    fn did_output_sample_buffer(&self, sample: CMSampleBuffer, output_type: SCStreamOutputType) {
        if output_type != SCStreamOutputType::Audio {
            return;
        }
        if let Some((sample_rate, mono)) = mono_samples(&sample) {
            self.sink.push_mono(sample_rate, &mono);
        }
    }
}

/// Reads a buffer as mono `f32`, or returns `None` if it is not the 32-bit
/// float audio that ScreenCaptureKit is documented to deliver.
fn mono_samples(sample: &CMSampleBuffer) -> Option<(u32, Vec<f32>)> {
    let format = sample.format_description()?;
    if !format.audio_is_float() || format.audio_bits_per_channel()? != 32 {
        return None;
    }
    let sample_rate = sample_rate_hz(format.audio_sample_rate()?)?;

    let buffers = sample.audio_buffer_list().ok()?;
    let channels: Vec<Vec<f32>> = buffers
        .iter()
        .map(|buffer| f32_samples(buffer.data()))
        .collect();
    let mono = match channels.as_slice() {
        // One buffer holds every channel interleaved.
        [interleaved] => {
            let channel_count = usize::try_from(format.audio_channel_count()?).ok()?;
            normalize::downmix_interleaved(interleaved, channel_count)
        }
        // Otherwise there is one buffer per channel.
        planar => {
            let slices: Vec<&[f32]> = planar.iter().map(Vec::as_slice).collect();
            normalize::downmix_planar(&slices)
        }
    };
    Some((sample_rate, mono))
}

fn f32_samples(bytes: &[u8]) -> Vec<f32> {
    bytes
        .chunks_exact(BYTES_PER_SAMPLE)
        .map(|chunk| f32::from_ne_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
        .collect()
}

/// Converts a sample rate reported as `f64` to whole hertz, rejecting nonsense values.
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "the value is checked to be a positive rate well inside u32 range"
)]
fn sample_rate_hz(rate: f64) -> Option<u32> {
    (rate.is_finite() && rate >= 1.0 && rate <= f64::from(u32::MAX)).then(|| rate.round() as u32)
}
