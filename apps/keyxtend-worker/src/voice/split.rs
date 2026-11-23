//! A piece longer than a model or a cloud piece takes, such as 30 s for Whisper, is cut at its quietest stretch near
//! the middle until each part fits, as the trial did; the parts' texts join with a space, cut to fit a note.

#[cfg(any(test, feature = "engine"))]
use super::library::{Failed, Loaded, Stop};
#[cfg(any(test, feature = "engine"))]
use kx_ipc::MAX_TEXT_BYTES;
use kx_ipc::SOUND_HZ;
use std::ops::Range;
#[cfg(any(test, feature = "engine"))]
use zeroize::Zeroizing;

/// The stretch whose loudness picks a cut, and the step between the stretches weighed, in ms.
const QUIET_MS: usize = 300;
const STEP_MS: usize = 50;
/// Steps in the stretch weighed, which holds whole steps.
const STEPS: usize = QUIET_MS / STEP_MS;
const _: () = assert!(
    QUIET_MS.is_multiple_of(STEP_MS),
    "the stretch weighed is whole steps"
);
/// The cut falls between these shares of the part, in tenths, so neither side is tiny.
const FROM_TENTHS: usize = 3;
const TO_TENTHS: usize = 7;
/// Tenths in a whole.
const TENTHS: usize = 10;
/// Milliseconds in a second.
const MS_PER_S: usize = kx_ipc::MS_PER_S as usize;
/// What joins the parts' texts.
#[cfg(any(test, feature = "engine"))]
const JOIN: &str = " ";

/// A sample whose loudness picks a cut: the engine's floats or the recording's 16-bit samples.
pub trait Sample: Copy {
    /// Its power, in a unit every sample of one piece shares.
    fn power(self) -> f32;
}

impl Sample for f32 {
    fn power(self) -> f32 {
        self * self
    }
}

impl Sample for i16 {
    fn power(self) -> f32 {
        f32::from(self).powi(2)
    }
}

/// The text of `pcm` on `model` in `lang`, in parts of at most its longest run. It has room for a whole note from the
/// start, so it never moves and leaves no unwiped copy behind.
///
/// # Errors
/// The first part's failure; `Stopped` when `stop` was set; `Memory` when the room cannot be had.
#[cfg(any(test, feature = "engine"))]
pub fn text<M: Loaded + ?Sized>(
    model: &mut M,
    pcm: &[f32],
    lang: &str,
    stop: &Stop,
) -> Result<Zeroizing<String>, Failed> {
    let mut out = Zeroizing::new(String::new());
    out.try_reserve_exact(MAX_TEXT_BYTES)
        .map_err(|_| Failed::Memory)?;
    for span in spans(pcm, model.most()) {
        let part = model.run(&pcm[span], lang, stop)?;
        let part = part.trim();
        if !part.is_empty() {
            if !out.is_empty() {
                fit(&mut out, JOIN);
            }
            fit(&mut out, part);
        }
    }
    Ok(out)
}

/// Spans of `pcm` of at most `most` samples each, covering it in order; one span when it fits or has no limit.
pub fn spans<T: Sample>(pcm: &[T], most: Option<usize>) -> Vec<Range<usize>> {
    let most = most.unwrap_or(usize::MAX).max(1);
    let mut out = Vec::new();
    if pcm.len() <= most {
        out.push(0..pcm.len());
    } else {
        split(&Steps::of(pcm), 0..pcm.len(), most, &mut out);
    }
    out
}

/// Each whole step's power from the piece's start, weighed once, so no cut weighs a sample again.
struct Steps {
    /// Samples in a step.
    len: usize,
    /// Each step's power, in order.
    power: Vec<f64>,
}

impl Steps {
    /// The steps of `pcm`; a part step at its end is never weighed, as no stretch reaches it.
    fn of<T: Sample>(pcm: &[T]) -> Self {
        let len = SOUND_HZ * STEP_MS / MS_PER_S;
        let weigh = |step: &[T]| step.iter().map(|v| f64::from(v.power())).sum();
        let power = pcm.chunks_exact(len).map(weigh).collect();
        Self { len, power }
    }
}

/// Adds `span`'s parts to `out`, halving it at its quietest stretch until each fits.
fn split(steps: &Steps, span: Range<usize>, most: usize, out: &mut Vec<Range<usize>>) {
    if span.len() <= most {
        out.push(span);
        return;
    }
    let at = quiet(steps, span.clone());
    split(steps, span.start..at, most, out);
    split(steps, at..span.end, most, out);
}

/// Where to cut `span`: the middle of its quietest stretch between three and seven tenths of it, never at either end.
fn quiet(steps: &Steps, span: Range<usize>) -> usize {
    let len = span.len();
    let lo = span.start + len * FROM_TENTHS / TENTHS;
    let hi = span.start + len * TO_TENTHS / TENTHS;
    let win = steps.len * STEPS;
    let mut best = (f64::MAX, span.start + len / 2);
    let mut step = lo.div_ceil(steps.len);
    // A stretch that ends before `hi` holds whole steps of the piece, so it is always in `power`.
    while (step + STEPS) * steps.len < hi {
        let energy: f64 = steps.power[step..step + STEPS].iter().sum();
        if energy < best.0 {
            best = (energy, step * steps.len + win / 2);
        }
        step += 1;
    }
    best.1.clamp(
        span.start + 1,
        span.end.saturating_sub(1).max(span.start + 1),
    )
}

/// Adds as much of `more` as fits one note, cut at a character boundary.
#[cfg(any(test, feature = "engine"))]
fn fit(out: &mut String, more: &str) {
    let mut end = MAX_TEXT_BYTES.saturating_sub(out.len()).min(more.len());
    while !more.is_char_boundary(end) {
        end -= 1;
    }
    out.push_str(&more[..end]);
}

#[cfg(test)]
mod tests;
