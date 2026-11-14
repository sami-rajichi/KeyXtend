//! A piece longer than a model takes in one run, such as 30 s for Whisper, is cut at its quietest stretch near the
//! middle until each part fits, as the trial did; the parts' texts join with a space, cut to fit a note.

use super::library::{Failed, Loaded, Stop};
use kx_ipc::{MAX_TEXT_BYTES, SOUND_HZ};
use std::ops::Range;
use zeroize::Zeroizing;

/// The stretch whose loudness picks a cut, and the step between the stretches weighed, in ms.
const QUIET_MS: usize = 300;
const STEP_MS: usize = 50;
/// The cut falls between these shares of the part, in tenths, so neither side is tiny.
const FROM_TENTHS: usize = 3;
const TO_TENTHS: usize = 7;
/// Tenths in a whole, and milliseconds in a second.
const TENTHS: usize = 10;
const MS_PER_S: usize = 1000;
/// What joins the parts' texts.
const JOIN: &str = " ";

/// The text of `pcm` on `model` in `lang`, in parts of at most its longest run. It has room for a whole note from the
/// start, so it never moves and leaves no unwiped copy behind.
///
/// # Errors
/// The first part's failure; `Stopped` when `stop` was set; `Memory` when the room cannot be had.
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
pub fn spans(pcm: &[f32], most: Option<usize>) -> Vec<Range<usize>> {
    let mut out = Vec::new();
    split(
        pcm,
        0..pcm.len(),
        most.unwrap_or(usize::MAX).max(1),
        &mut out,
    );
    out
}

/// Adds `span`'s parts to `out`, halving it at its quietest stretch until each fits.
fn split(pcm: &[f32], span: Range<usize>, most: usize, out: &mut Vec<Range<usize>>) {
    if span.len() <= most {
        out.push(span);
        return;
    }
    let at = span.start + quiet(&pcm[span.clone()]);
    split(pcm, span.start..at, most, out);
    split(pcm, at..span.end, most, out);
}

/// Where to cut `x`: the middle of its quietest stretch between three and seven tenths of it, never at either end.
fn quiet(x: &[f32]) -> usize {
    let win = SOUND_HZ * QUIET_MS / MS_PER_S;
    let step = SOUND_HZ * STEP_MS / MS_PER_S;
    let (lo, hi) = (x.len() * FROM_TENTHS / TENTHS, x.len() * TO_TENTHS / TENTHS);
    let mut best = (f32::MAX, x.len() / 2);
    let mut at = lo;
    while at + win < hi {
        let energy: f32 = x[at..at + win].iter().map(|v| v * v).sum();
        if energy < best.0 {
            best = (energy, at + win / 2);
        }
        at += step;
    }
    best.1.clamp(1, x.len().saturating_sub(1).max(1))
}

/// Adds as much of `more` as fits one note, cut at a character boundary.
fn fit(out: &mut String, more: &str) {
    let mut end = MAX_TEXT_BYTES.saturating_sub(out.len()).min(more.len());
    while !more.is_char_boundary(end) {
        end -= 1;
    }
    out.push_str(&more[..end]);
}

#[cfg(test)]
mod tests;
