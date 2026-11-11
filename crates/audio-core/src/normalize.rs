//! The format-normalization boundary.
//!
//! Devices deliver audio in many shapes: different sample types, one or
//! more channels, interleaved or one buffer per channel. Capture adapters
//! convert samples to `f32` and then call one of the functions here, so
//! everything after this point sees mono `f32` in the range -1.0 to 1.0.
//!
//! Sample rate is deliberately left unchanged. Resampling belongs to the
//! stage that knows which rate a transcription provider needs.

/// Averages interleaved channels (`L R L R ...`) into mono.
///
/// A trailing partial group of samples is ignored. Zero channels yields no
/// audio.
#[must_use]
pub fn downmix_interleaved(samples: &[f32], channels: usize) -> Vec<f32> {
    match channels {
        0 => Vec::new(),
        1 => samples.to_vec(),
        _ => samples.chunks_exact(channels).map(average).collect(),
    }
}

/// Averages planar channels (one slice per channel) into mono.
///
/// The result is as long as the shortest channel.
#[must_use]
pub fn downmix_planar(channels: &[&[f32]]) -> Vec<f32> {
    let length = channels
        .iter()
        .map(|channel| channel.len())
        .min()
        .unwrap_or(0);
    (0..length)
        .map(|index| {
            average_of(
                channels.iter().map(|channel| channel[index]),
                channels.len(),
            )
        })
        .collect()
}

fn average(samples: &[f32]) -> f32 {
    average_of(samples.iter().copied(), samples.len())
}

#[expect(
    clippy::cast_precision_loss,
    reason = "channel counts are tiny, far below the point where f32 loses precision"
)]
fn average_of(samples: impl Iterator<Item = f32>, count: usize) -> f32 {
    samples.sum::<f32>() / count as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mono_passes_through_unchanged() {
        assert_eq!(
            downmix_interleaved(&[0.1, -0.2, 0.3], 1),
            vec![0.1, -0.2, 0.3]
        );
    }

    #[test]
    fn stereo_is_averaged() {
        assert_eq!(
            downmix_interleaved(&[1.0, 0.0, -0.5, 0.5], 2),
            vec![0.5, 0.0]
        );
    }

    #[test]
    fn trailing_partial_group_is_ignored() {
        assert_eq!(downmix_interleaved(&[1.0, 1.0, 0.25], 2), vec![1.0]);
    }

    #[test]
    fn zero_channels_yields_no_audio() {
        assert!(downmix_interleaved(&[1.0, 1.0], 0).is_empty());
        assert!(downmix_planar(&[]).is_empty());
    }

    #[test]
    fn planar_channels_are_averaged_up_to_the_shortest() {
        let left = [1.0, 0.5, 0.25];
        let right = [0.0, 0.5];
        assert_eq!(downmix_planar(&[&left, &right]), vec![0.5, 0.5]);
    }
}
