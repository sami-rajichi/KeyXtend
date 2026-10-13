//! The last steps every sound takes: trim or cut, fade-out, peak normalising and 16-bit encoding.

use std::f64::consts::PI;

use kx_theme::sounds::Format;

use super::units::{db_gain, index, ms_at, samples};
use super::{Cut, Sound, SoundError, wav};

/// The part of `clip` from the first to the last sample louder than `trim_db` below its peak.
pub(crate) fn trim(clip: &[f64], trim_db: f64) -> &[f64] {
    let floor = peak(clip) * db_gain(trim_db);
    let loud = |x: &f64| x.abs() > floor;
    match (clip.iter().position(loud), clip.iter().rposition(loud)) {
        (Some(first), Some(last)) => &clip[first..=last],
        _ => &[],
    }
}

/// The part of `clip`, at `rate`, that `cut` names, ending early if the clip does; an error if it holds no sample.
pub(crate) fn window(clip: &[f64], rate: f64, cut: Cut) -> Result<&[f64], SoundError> {
    let at = |ms: u32| samples(f64::from(ms), rate);
    let start = at(cut.start_ms.unwrap_or_default());
    let end = cut
        .len_ms
        .map_or(clip.len(), |len| start.saturating_add(at(len)));
    clip.get(start..end.min(clip.len()))
        .filter(|part| !part.is_empty())
        .ok_or(SoundError::Window)
}

/// Fades, normalises to `gain_db` under the ceiling and encodes `clip` in `format`.
pub(crate) fn master(clip: &mut [f64], format: &Format, gain_db: f64) -> Result<Sound, SoundError> {
    let rate = f64::from(format.rate);
    fade_out(clip, samples(format.fade_ms, rate));
    // Rounded down to a whole 16-bit step, so rounding each sample never passes the ceiling.
    let full = f64::from(i16::MAX);
    let ceiling = (db_gain(format.ceiling_db) * full).floor() / full;
    normalise(clip, ceiling * db_gain(gain_db))?;
    Ok(Sound {
        bytes: wav::write(format.rate, &pcm16(clip))?,
        ms: ms_at(clip.len(), rate),
    })
}

/// The largest magnitude in `clip`.
fn peak(clip: &[f64]) -> f64 {
    clip.iter().fold(0.0, |top, x| top.max(x.abs()))
}

/// Fades the last `len` samples of `clip` to silence along a raised cosine.
fn fade_out(clip: &mut [f64], len: usize) {
    let tail = clip.len().saturating_sub(len);
    let len = index(clip.len() - tail);
    for (k, x) in clip[tail..].iter_mut().enumerate() {
        *x *= f64::midpoint(1.0, (PI * index(k + 1) / len).cos());
    }
}

/// Scales `clip` so its peak is `target`; an error if it is silent.
fn normalise(clip: &mut [f64], target: f64) -> Result<(), SoundError> {
    let top = peak(clip);
    if !(top.is_finite() && top > 0.0) {
        return Err(SoundError::Silent);
    }
    for x in clip.iter_mut() {
        *x *= target / top;
    }
    Ok(())
}

/// `clip` as 16-bit samples, each rounded to the nearest step.
#[allow(
    clippy::cast_possible_truncation,
    reason = "Each value is clamped to the i16 range first."
)]
fn pcm16(clip: &[f64]) -> Vec<i16> {
    let full = f64::from(i16::MAX);
    clip.iter()
        .map(|x| (x * full).round().clamp(-full, full) as i16)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trim_keeps_the_first_to_the_last_loud_sample() {
        let clip = [0.0, 1e-4, 0.5, -1.0, 0.2, 1e-5, 0.0];
        assert_eq!(trim(&clip, -50.0), [0.5, -1.0, 0.2]);
        assert!(trim(&[0.0; 4], -50.0).is_empty());
    }

    #[test]
    fn window_takes_start_and_length_and_clamps_the_end() {
        let clip: Vec<f64> = (0..10).map(f64::from).collect();
        let cut = |start_ms, len_ms| Cut { start_ms, len_ms };
        assert_eq!(
            window(&clip, 1_000.0, cut(Some(2), Some(3))).unwrap(),
            [2.0, 3.0, 4.0]
        );
        assert_eq!(
            window(&clip, 1_000.0, cut(None, Some(2))).unwrap(),
            [0.0, 1.0]
        );
        assert_eq!(
            window(&clip, 1_000.0, cut(Some(8), Some(9))).unwrap(),
            [8.0, 9.0]
        );
        assert!(window(&clip, 1_000.0, cut(Some(10), None)).is_err());
        assert!(window(&clip, 1_000.0, cut(Some(1), Some(0))).is_err());
    }
}
