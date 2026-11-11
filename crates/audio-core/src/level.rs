//! Input level metering, so the UI can show that audio is really arriving.

use serde::Serialize;

/// Loudness of the audio seen since the last reading, both from 0.0 to 1.0.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize)]
pub struct Level {
    /// Largest absolute sample value.
    pub peak: f32,
    /// Root mean square: the average loudness.
    pub rms: f32,
}

/// Accumulates samples and reports their [`Level`]. It keeps running totals
/// only, never the audio itself.
#[derive(Debug, Default)]
pub struct LevelMeter {
    peak: f32,
    sum_of_squares: f64,
    count: u64,
}

impl LevelMeter {
    pub fn add(&mut self, samples: &[f32]) {
        for &sample in samples {
            self.peak = self.peak.max(sample.abs());
            self.sum_of_squares += f64::from(sample) * f64::from(sample);
        }
        self.count += samples.len() as u64;
    }

    /// Returns the level since the previous call and starts a new reading.
    /// With no samples the level is silence.
    #[expect(
        clippy::cast_precision_loss,
        clippy::cast_possible_truncation,
        reason = "an approximate loudness for display; exactness is not needed"
    )]
    pub fn take(&mut self) -> Level {
        let rms = if self.count == 0 {
            0.0
        } else {
            (self.sum_of_squares / self.count as f64).sqrt() as f32
        };
        let level = Level {
            peak: self.peak.min(1.0),
            rms: rms.min(1.0),
        };
        *self = Self::default();
        level
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_samples_reads_as_silence() {
        assert_eq!(LevelMeter::default().take(), Level::default());
    }

    #[test]
    fn reports_peak_and_rms() {
        let mut meter = LevelMeter::default();
        meter.add(&[0.5, -0.5]);
        meter.add(&[0.5, -0.5]);
        let level = meter.take();
        assert!((level.peak - 0.5).abs() < 1e-6);
        assert!((level.rms - 0.5).abs() < 1e-6);
    }

    #[test]
    fn taking_a_reading_resets_the_meter() {
        let mut meter = LevelMeter::default();
        meter.add(&[1.0]);
        let _ = meter.take();
        assert_eq!(meter.take(), Level::default());
    }

    #[test]
    fn out_of_range_samples_are_clamped() {
        let mut meter = LevelMeter::default();
        meter.add(&[3.0]);
        let level = meter.take();
        assert!((level.peak - 1.0).abs() < 1e-6);
        assert!((level.rms - 1.0).abs() < 1e-6);
    }
}
