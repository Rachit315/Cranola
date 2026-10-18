//! Small signal-processing helpers used for levels and self-tests.

/// Floor used when converting silence to decibels.
pub const SILENCE_DBFS: f32 = -120.0;

pub fn to_dbfs(amplitude: f32) -> f32 {
    if amplitude <= 0.0 {
        SILENCE_DBFS
    } else {
        (20.0 * amplitude.log10()).max(SILENCE_DBFS)
    }
}

pub fn peak(samples: &[f32]) -> f32 {
    samples.iter().fold(0.0_f32, |m, s| m.max(s.abs()))
}

pub fn rms(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    let sum: f64 = samples.iter().map(|&s| f64::from(s) * f64::from(s)).sum();
    #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
    let mean = (sum / samples.len() as f64) as f32;
    mean.sqrt()
}

/// Averages interleaved frames into mono.
pub fn downmix(interleaved: &[f32], channels: usize) -> Vec<f32> {
    if channels <= 1 {
        return interleaved.to_vec();
    }
    #[allow(clippy::cast_precision_loss)]
    let scale = 1.0 / channels as f32;
    interleaved
        .chunks_exact(channels)
        .map(|frame| frame.iter().sum::<f32>() * scale)
        .collect()
}

/// Amplitude of a single frequency component via the Goertzel algorithm.
/// For a pure sine of amplitude `a` at `freq_hz` this returns roughly `a`.
pub fn goertzel_amplitude(samples: &[f32], sample_rate: u32, freq_hz: f32) -> f32 {
    if samples.is_empty() || sample_rate == 0 {
        return 0.0;
    }
    #[allow(clippy::cast_precision_loss)]
    let n = samples.len() as f32;
    #[allow(clippy::cast_precision_loss)]
    let omega = 2.0 * std::f32::consts::PI * freq_hz / sample_rate as f32;
    let coeff = 2.0 * omega.cos();
    let (mut s1, mut s2) = (0.0_f32, 0.0_f32);
    for &x in samples {
        let s0 = x + coeff * s1 - s2;
        s2 = s1;
        s1 = s0;
    }
    let power = s1 * s1 + s2 * s2 - coeff * s1 * s2;
    2.0 * power.max(0.0).sqrt() / n
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sine(freq: f32, amp: f32, rate: u32, secs: f32) -> Vec<f32> {
        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            clippy::cast_precision_loss
        )]
        let n = (rate as f32 * secs) as usize;
        #[allow(clippy::cast_precision_loss)]
        (0..n)
            .map(|i| amp * (2.0 * std::f32::consts::PI * freq * i as f32 / rate as f32).sin())
            .collect()
    }

    #[test]
    fn decibels() {
        assert!((to_dbfs(1.0) - 0.0).abs() < 1e-4);
        assert!((to_dbfs(0.5) + 6.0206).abs() < 1e-3);
        assert_eq!(to_dbfs(0.0), SILENCE_DBFS);
    }

    #[test]
    fn rms_of_sine_is_amp_over_sqrt2() {
        let s = sine(440.0, 0.5, 48_000, 1.0);
        assert!((rms(&s) - 0.5 / 2f32.sqrt()).abs() < 1e-3);
        assert!((peak(&s) - 0.5).abs() < 1e-3);
    }

    #[test]
    fn goertzel_finds_target_tone_and_ignores_others() {
        let rate = 48_000;
        let tone = sine(1000.0, 0.25, rate, 0.5);
        let other = sine(440.0, 0.25, rate, 0.5);
        assert!((goertzel_amplitude(&tone, rate, 1000.0) - 0.25).abs() < 0.01);
        assert!(goertzel_amplitude(&other, rate, 1000.0) < 0.01);
    }

    #[test]
    fn downmix_averages_frames() {
        assert_eq!(downmix(&[1.0, 0.0, 0.5, 0.5], 2), vec![0.5, 0.5]);
        assert_eq!(downmix(&[0.1, 0.2], 1), vec![0.1, 0.2]);
    }
}
