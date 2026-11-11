//! Microphone discovery and capture.

use std::str::FromStr;

use cpal::traits::{DeviceTrait, HostTrait};
use cpal::{Device, DeviceId, Stream};
use opentalk_audio_core::{AudioSource, CaptureError, MicrophoneInfo};

use crate::input_stream;
use crate::sink::SourceSink;

const SOURCE: AudioSource = AudioSource::Microphone;

/// Lists the microphones currently connected, default first.
///
/// # Errors
///
/// Returns an error if the operating system's audio service cannot be queried.
pub fn list_microphones() -> Result<Vec<MicrophoneInfo>, CaptureError> {
    let host = cpal::default_host();
    let default_id = host
        .default_input_device()
        .and_then(|device| device.id().ok());
    let devices = host
        .input_devices()
        .map_err(|error| input_stream::to_capture_error(SOURCE, &error))?;

    let mut microphones: Vec<MicrophoneInfo> = devices
        .filter_map(|device| {
            let id = device.id().ok()?;
            let name = device.description().ok()?.name().to_owned();
            Some(MicrophoneInfo {
                is_default: Some(&id) == default_id.as_ref(),
                id: id.to_string(),
                name,
            })
        })
        .collect();
    microphones.sort_by_key(|microphone| !microphone.is_default);
    Ok(microphones)
}

/// Starts capturing the chosen microphone, or the default one for `None`.
/// Capture runs until the returned stream is dropped.
pub(crate) fn start(device_id: Option<&str>, sink: SourceSink) -> Result<Stream, CaptureError> {
    let device = find_device(device_id)?;
    let config = device
        .default_input_config()
        .map_err(|error| input_stream::to_capture_error(SOURCE, &error))?;
    input_stream::start(&device, &config, sink)
}

fn find_device(device_id: Option<&str>) -> Result<Device, CaptureError> {
    let host = cpal::default_host();
    let device = match device_id {
        None => host.default_input_device(),
        Some(id) => {
            let id =
                DeviceId::from_str(id).map_err(|_| CaptureError::InvalidMicrophoneSelection)?;
            host.device_by_id(&id)
        }
    };
    device.ok_or(CaptureError::DeviceUnavailable {
        audio_source: SOURCE,
    })
}
