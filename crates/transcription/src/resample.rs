//! Sample-rate conversion between a capture device and a provider.
//!
//! Uses band-limited interpolation: each output sample is a weighted sum of
//! nearby input samples, with weights from a windowed sinc function. When
//! lowering the rate the filter also removes frequencies the new rate
//! cannot represent, which would otherwise fold back as noise.

use std::f64::consts::PI;

/// Input samples on each side of an output sample that contribute to it,
/// before widening for downsampling. More is more accurate and slower.
const BASE_HALF_WIDTH: f64 = 16.0;
/// Keep slightly below the theoretical limit to leave room for the filter's roll-off.
const CUTOFF_MARGIN: f64 = 0.95;

/// Converts a stream of mono samples from one rate to another, chunk by
/// chunk. Splitting the input differently does not change the output.
#[derive(Debug)]
pub struct Resampler {
    /// Input samples advanced per output sample.
    step: f64,
    /// Filter cutoff as a fraction of the input rate's limit.
    cutoff: f64,
    half_width: usize,
    /// Input not yet fully consumed, preceded by `half_width` samples of history.
    buffer: Vec<f32>,
    /// How many input samples have been dropped from the front of `buffer`.
    dropped: u64,
    /// How many output samples have been produced.
    produced: u64,
}

impl Resampler {
    /// Creates a converter from `input_rate` to `output_rate`, both in hertz.
    ///
    /// # Panics
    ///
    /// Panics if either rate is zero.
    #[must_use]
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "the half width is a small positive number"
    )]
    pub fn new(input_rate: u32, output_rate: u32) -> Self {
        assert!(
            input_rate > 0 && output_rate > 0,
            "sample rates must be positive"
        );
        let step = f64::from(input_rate) / f64::from(output_rate);
        let cutoff = (1.0 / step).min(1.0) * CUTOFF_MARGIN;
        let half_width = (BASE_HALF_WIDTH / cutoff).ceil() as usize;
        Self {
            step,
            cutoff,
            half_width,
            // Start with silence as history so the first samples have neighbours.
            buffer: vec![0.0; half_width],
            dropped: 0,
            produced: 0,
        }
    }

    /// Feeds more input and returns the output that can be produced so far.
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_precision_loss,
        reason = "positions are non-negative buffer indices, far below the limits of these types"
    )]
    pub fn process(&mut self, input: &[f32]) -> Vec<f32> {
        self.buffer.extend_from_slice(input);

        let mut output = Vec::new();
        // An output sample needs `half_width` input samples after its position.
        while self.next_position() + self.half_width as f64 + 1.0 < self.buffer.len() as f64 {
            output.push(self.sample_at(self.next_position()));
            self.produced += 1;
        }

        // Drop input that no later output sample can reach.
        let consumed = (self.next_position() as usize).saturating_sub(self.half_width);
        self.buffer.drain(..consumed);
        self.dropped += consumed as u64;
        output
    }

    /// Where the next output sample falls, as an index into `buffer`.
    ///
    /// It is computed from the output count each time rather than by adding
    /// `step` repeatedly, so rounding errors cannot build up over a long
    /// session and the result does not depend on how the input was chunked.
    #[expect(
        clippy::cast_precision_loss,
        reason = "sample counts stay far below 2^52"
    )]
    fn next_position(&self) -> f64 {
        let from_start = self.half_width as f64 + self.produced as f64 * self.step;
        from_start - self.dropped as f64
    }

    /// The band-limited value of the input signal at a fractional position.
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_precision_loss,
        reason = "positions are non-negative buffer indices, and audio samples only need f32 precision"
    )]
    fn sample_at(&self, position: f64) -> f32 {
        let centre = position as usize;
        let first = centre.saturating_sub(self.half_width);
        let last = (centre + self.half_width).min(self.buffer.len() - 1);

        let mut sum = 0.0;
        let mut total_weight = 0.0;
        for index in first..=last {
            let weight = self.weight(index as f64 - position);
            sum += weight * f64::from(self.buffer[index]);
            total_weight += weight;
        }
        // Normalizing keeps a constant input at exactly the same level.
        if total_weight == 0.0 {
            0.0
        } else {
            (sum / total_weight) as f32
        }
    }

    /// Filter weight for an input sample `distance` samples from the output position.
    #[expect(
        clippy::cast_precision_loss,
        reason = "the half width is a small number"
    )]
    fn weight(&self, distance: f64) -> f64 {
        let reach = self.half_width as f64;
        if distance.abs() >= reach {
            return 0.0;
        }
        let window = 0.5 * (1.0 + (PI * distance / reach).cos());
        sinc(self.cutoff * distance) * window
    }
}

fn sinc(x: f64) -> f64 {
    if x == 0.0 {
        1.0
    } else {
        (PI * x).sin() / (PI * x)
    }
}

/// Converts samples in the range -1.0 to 1.0 into 16-bit PCM, clamping anything louder.
#[must_use]
#[expect(
    clippy::cast_possible_truncation,
    reason = "the value is clamped to the i16 range first"
)]
pub fn to_pcm16(samples: &[f32]) -> Vec<i16> {
    samples
        .iter()
        .map(|sample| (sample.clamp(-1.0, 1.0) * f32::from(i16::MAX)).round() as i16)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[expect(clippy::cast_precision_loss, reason = "test signal generation")]
    fn sine(frequency: f64, rate: u32, samples: usize) -> Vec<f32> {
        (0..samples)
            .map(|n| (2.0 * PI * frequency * n as f64 / f64::from(rate)).sin() as f32)
            .collect()
    }

    fn peak(samples: &[f32]) -> f32 {
        samples
            .iter()
            .fold(0.0, |peak, sample| peak.max(sample.abs()))
    }

    #[test]
    fn halving_the_rate_halves_the_length() {
        let output = Resampler::new(48_000, 24_000).process(&vec![0.0; 48_000]);
        assert!(
            (output.len() as i64 - 24_000).abs() < 50,
            "got {}",
            output.len()
        );
    }

    #[test]
    fn a_constant_signal_keeps_its_level() {
        let output = Resampler::new(44_100, 24_000).process(&vec![0.5; 44_100]);
        // Skip the start, where the filter still sees the initial silence.
        assert!(
            output[200..]
                .iter()
                .all(|sample| (sample - 0.5).abs() < 1e-4)
        );
    }

    #[test]
    fn a_speech_range_tone_survives_downsampling() {
        let output = Resampler::new(48_000, 24_000).process(&sine(1_000.0, 48_000, 48_000));
        let level = peak(&output[200..]);
        assert!((level - 1.0).abs() < 0.02, "peak was {level}");
    }

    #[test]
    fn a_tone_above_the_new_limit_is_removed() {
        // 20 kHz cannot be represented at 24 kHz (limit 12 kHz) and must not alias.
        let output = Resampler::new(48_000, 24_000).process(&sine(20_000.0, 48_000, 48_000));
        let level = peak(&output[200..]);
        assert!(level < 0.01, "peak was {level}");
    }

    #[test]
    fn chunking_the_input_does_not_change_the_output() {
        let input = sine(440.0, 44_100, 10_000);
        let whole = Resampler::new(44_100, 24_000).process(&input);

        let mut chunked_resampler = Resampler::new(44_100, 24_000);
        let chunked: Vec<f32> = input
            .chunks(441)
            .flat_map(|chunk| chunked_resampler.process(chunk))
            .collect();
        assert_eq!(whole, chunked);
    }

    #[test]
    fn equal_rates_reproduce_the_signal() {
        let input = sine(440.0, 24_000, 4_800);
        let output = Resampler::new(24_000, 24_000).process(&input);
        // The output is delayed by nothing: sample n of the output is sample n of the input.
        for (index, sample) in output.iter().enumerate().skip(100) {
            assert!((sample - input[index]).abs() < 1e-3);
        }
    }

    #[test]
    fn pcm16_conversion_scales_and_clamps() {
        assert_eq!(
            to_pcm16(&[0.0, 1.0, -1.0, 2.0, -2.0]),
            vec![0, 32_767, -32_767, 32_767, -32_767]
        );
    }
}
