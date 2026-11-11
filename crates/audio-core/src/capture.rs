//! What to capture, as requested by the user.

use serde::{Deserialize, Serialize};

use crate::error::CaptureError;

/// Longest accepted device identifier. Real identifiers are far shorter;
/// the limit only bounds what an IPC caller can send.
pub const MAX_DEVICE_ID_LENGTH: usize = 512;

/// The sources chosen for one capture session.
#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CaptureConfig {
    /// Whether to capture the microphone.
    pub microphone: bool,
    /// Which microphone to use; `None` means the system default.
    pub microphone_id: Option<String>,
    /// Whether to capture audio played by the computer.
    pub system_audio: bool,
}

impl CaptureConfig {
    /// Checks that the configuration asks for something capturable.
    ///
    /// # Errors
    ///
    /// Returns an error if no source is selected or the device id is empty
    /// or unreasonably long.
    pub fn validate(&self) -> Result<(), CaptureError> {
        if !self.microphone && !self.system_audio {
            return Err(CaptureError::NoSourceSelected);
        }
        match &self.microphone_id {
            Some(id) if id.is_empty() || id.len() > MAX_DEVICE_ID_LENGTH => {
                Err(CaptureError::InvalidMicrophoneSelection)
            }
            _ => Ok(()),
        }
    }
}

/// A microphone the user can choose.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MicrophoneInfo {
    /// Stable identifier to pass back in [`CaptureConfig::microphone_id`].
    pub id: String,
    /// Name shown to the user.
    pub name: String,
    pub is_default: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn at_least_one_source_is_required() {
        assert_eq!(
            CaptureConfig::default().validate(),
            Err(CaptureError::NoSourceSelected)
        );
    }

    #[test]
    fn either_source_alone_is_enough() {
        let microphone_only = CaptureConfig {
            microphone: true,
            ..CaptureConfig::default()
        };
        let system_only = CaptureConfig {
            system_audio: true,
            ..CaptureConfig::default()
        };
        assert_eq!(microphone_only.validate(), Ok(()));
        assert_eq!(system_only.validate(), Ok(()));
    }

    #[test]
    fn empty_or_oversized_device_ids_are_rejected() {
        for id in [String::new(), "x".repeat(MAX_DEVICE_ID_LENGTH + 1)] {
            let config = CaptureConfig {
                microphone: true,
                microphone_id: Some(id),
                system_audio: false,
            };
            assert_eq!(
                config.validate(),
                Err(CaptureError::InvalidMicrophoneSelection)
            );
        }
    }
}
