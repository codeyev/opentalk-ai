//! Windows system audio through WASAPI loopback.
//!
//! WASAPI can open an output device for capture, delivering whatever is
//! being played through it. `cpal` enables this loopback mode when an input
//! stream is built on an output device.
//!
//! Not yet verified on a physical Windows machine; see docs/TESTING.md.

use cpal::Stream;
use cpal::traits::{DeviceTrait, HostTrait};
use opentalk_audio_core::{AudioSource, CaptureError};

use crate::input_stream;
use crate::sink::SourceSink;

const SOURCE: AudioSource = AudioSource::System;

/// Capture of the default output device. Stops when dropped.
pub(crate) struct SystemAudioStream {
    _stream: Stream,
}

pub(crate) fn start(sink: SourceSink) -> Result<SystemAudioStream, CaptureError> {
    let device =
        cpal::default_host()
            .default_output_device()
            .ok_or(CaptureError::DeviceUnavailable {
                audio_source: SOURCE,
            })?;
    let config = device
        .default_output_config()
        .map_err(|error| input_stream::to_capture_error(SOURCE, &error))?;
    let stream = input_stream::start(&device, &config, sink)?;
    Ok(SystemAudioStream { _stream: stream })
}
