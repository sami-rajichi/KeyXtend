//! A windowed-sinc resampler (Blackman window) for imported recordings.

use std::f64::consts::{PI, TAU};

use super::recipe::Resample;
use super::units::{floor_index, index};

/// The Blackman window's three terms.
const BLACKMAN: [f64; 3] = [0.42, 0.5, 0.08];

/// `clip` resampled from `from` to `to` samples per second.
pub(crate) fn resample(clip: &[f64], from: u32, to: u32, setup: &Resample) -> Vec<f64> {
    if from == to {
        return clip.to_vec();
    }
    // `step` is input samples per output sample; the cutoff is a share of the input's Nyquist.
    let step = f64::from(from) / f64::from(to);
    let cutoff = step.recip().min(1.0) * setup.rolloff;
    let reach = f64::from(setup.half_taps) / cutoff;
    let len = floor_index((index(clip.len()) / step).round());
    (0..len)
        .map(|n| point(clip, index(n) * step, cutoff, reach))
        .collect()
}

/// The band-limited value of `clip` at the fractional input position `t`.
fn point(clip: &[f64], t: f64, cutoff: f64, reach: f64) -> f64 {
    let first = floor_index((t - reach).ceil());
    let last = (floor_index(t + reach) + 1).min(clip.len());
    let taps = clip.get(first..last).unwrap_or_default();
    taps.iter()
        .zip(first..)
        .map(|(x, k)| x * kernel(t - index(k), cutoff, reach))
        .sum()
}

/// The windowed-sinc weight of an input sample `gap` samples away.
fn kernel(gap: f64, cutoff: f64, reach: f64) -> f64 {
    let pos = gap / reach;
    if pos.abs() >= 1.0 {
        return 0.0;
    }
    let [base, first, second] = BLACKMAN;
    let window = base + first * (PI * pos).cos() + second * (TAU * pos).cos();
    let arg = PI * cutoff * gap;
    let sinc = if arg.abs() < f64::EPSILON {
        1.0
    } else {
        arg.sin() / arg
    };
    cutoff * sinc * window
}

#[cfg(test)]
mod tests {
    use super::*;

    const SETUP: Resample = Resample {
        half_taps: 32,
        rolloff: 0.94,
    };

    #[test]
    fn the_same_rate_is_a_copy() {
        let clip = [0.1, -0.2, 0.3];
        assert_eq!(resample(&clip, 48_000, 48_000, &SETUP), clip);
    }

    #[test]
    fn the_length_follows_the_rate_and_a_level_stays_level() {
        let clip = vec![0.5; 4_410];
        let up = resample(&clip, 44_100, 48_000, &SETUP);
        assert_eq!(up.len(), 4_800);
        assert!(up[200..4_600].iter().all(|x| (x - 0.5).abs() < 1e-3));
        let down = resample(&clip, 44_100, 22_050, &SETUP);
        assert_eq!(down.len(), 2_205);
        assert!(down[100..2_100].iter().all(|x| (x - 0.5).abs() < 1e-3));
    }

    #[test]
    fn a_tone_above_the_new_nyquist_is_removed() {
        let tone: Vec<f64> = (0..9_600)
            .map(|i| (std::f64::consts::TAU * 20_000.0 * f64::from(i) / 96_000.0).sin())
            .collect();
        let down = resample(&tone, 96_000, 32_000, &SETUP);
        let loud = down[200..3_000].iter().fold(0.0_f64, |m, x| m.max(x.abs()));
        assert!(loud < 0.01, "an alias at {loud}");
    }
}
