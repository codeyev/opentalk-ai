//! Opening a `cpal` input stream that feeds a [`SourceSink`].
//!
//! Used for microphones on every platform, and for WASAPI loopback on
//! Windows, where the system output is opened as if it were an input.

use cpal::traits::{DeviceTrait, StreamTrait};
use cpal::{
    Device, ErrorKind, FromSample, Sample, SampleFormat, SizedSample, Stream, SupportedStreamConfig,
};
use opentalk_audio_core::{AudioSource, CaptureError, normalize};

use crate::sink::SourceSink;

/// Starts capturing from `device` in the given configuration.
pub(crate) fn start(
    device: &Device,
    config: &SupportedStreamConfig,
    sink: SourceSink,
) -> Result<Stream, CaptureError> {
    let source = sink.source();
    let stream = match config.sample_format() {
        SampleFormat::I8 => build::<i8>(device, config, sink),
        SampleFormat::I16 => build::<i16>(device, config, sink),
        SampleFormat::I32 => build::<i32>(device, config, sink),
        SampleFormat::U8 => build::<u8>(device, config, sink),
        SampleFormat::U16 => build::<u16>(device, config, sink),
        SampleFormat::U32 => build::<u32>(device, config, sink),
        SampleFormat::F32 => build::<f32>(device, config, sink),
        SampleFormat::F64 => build::<f64>(device, config, sink),
        other => Err(CaptureError::Backend {
            audio_source: source,
            detail: format!("unsupported sample format {other}"),
        }),
    }?;
    stream
        .play()
        .map_err(|error| to_capture_error(source, &error))?;
    Ok(stream)
}

/// Builds a stream whose samples of type `T` are converted to mono `f32`.
fn build<T>(
    device: &Device,
    config: &SupportedStreamConfig,
    sink: SourceSink,
) -> Result<Stream, CaptureError>
where
    T: SizedSample,
    f32: FromSample<T>,
{
    let source = sink.source();
    let channels = usize::from(config.channels());
    let sample_rate = config.sample_rate();
    let error_sink = sink.clone();

    device
        .build_input_stream(
            config.config(),
            move |samples: &[T], _| {
                let as_f32: Vec<f32> = samples
                    .iter()
                    .map(|&sample| f32::from_sample(sample))
                    .collect();
                sink.push_mono(
                    sample_rate,
                    &normalize::downmix_interleaved(&as_f32, channels),
                );
            },
            move |error| error_sink.report_interruption(error),
            None,
        )
        .map_err(|error| to_capture_error(source, &error))
}

/// Maps a `cpal` error onto the app's typed capture errors.
pub(crate) fn to_capture_error(source: AudioSource, error: &cpal::Error) -> CaptureError {
    match error.kind() {
        ErrorKind::PermissionDenied => CaptureError::PermissionDenied {
            audio_source: source,
        },
        ErrorKind::DeviceNotAvailable => CaptureError::DeviceUnavailable {
            audio_source: source,
        },
        _ => CaptureError::Backend {
            audio_source: source,
            detail: error.to_string(),
        },
    }
}
