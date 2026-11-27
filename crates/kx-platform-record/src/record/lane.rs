//! One source's sound on the track's timeline: each packet goes where it was heard, so the sound stays in step with the
//! picture. A packet within the drift of where the last ended follows it with no gap; a real gap is silence; an
//! overlap, or sound before what was written, is dropped; a cut silences the lane from its place on.

use std::collections::VecDeque;

use super::CHANNELS;

/// Samples in one stereo frame.
const PAIR: usize = CHANNELS as usize;

/// A source's sound waiting to be written, by its place on the timeline in sound frames.
pub(super) struct Lane {
    /// The place of the first frame waiting; nothing before it is taken again.
    head: i64,
    /// The frames waiting, left and right in turn, with no gap.
    buf: VecDeque<f32>,
    /// Where the last packet ended, which the next one follows when it strays no more than `drift`.
    end: Option<i64>,
    /// Nothing at or after this place is kept, while set.
    cut: Option<i64>,
    drift: i64,
    keep: usize,
}

impl Lane {
    /// An empty lane whose packets may stray `drift` frames, holding at most `keep` frames.
    pub(super) fn new(drift: i64, keep: usize) -> Self {
        Self {
            head: 0,
            buf: VecDeque::new(),
            end: None,
            cut: None,
            drift,
            keep,
        }
    }

    /// The frames waiting.
    fn len(&self) -> i64 {
        i64::try_from(self.buf.len() / PAIR).unwrap_or(i64::MAX)
    }

    /// Puts stereo `samples` heard from place `at` on.
    pub(super) fn push(&mut self, at: i64, samples: &[f32]) {
        let frames = i64::try_from(samples.len() / PAIR).unwrap_or(0);
        let start = match self.end {
            Some(end) if (at - end).abs() <= self.drift => end,
            _ => at,
        };
        self.end = Some(self.end.map_or(start + frames, |e| e.max(start + frames)));
        if self.buf.is_empty() && start > self.head {
            self.head = start;
        }
        let tail = self.head + self.len();
        let skip = (tail - start).clamp(0, frames);
        let stop = self.cut.map_or(start + frames, |c| c.min(start + frames));
        let kept = (stop - start - skip).max(0);
        if kept == 0 {
            return;
        }
        let pad = (start - tail).max(0);
        if usize::try_from(pad).unwrap_or(usize::MAX) >= self.keep {
            // Silence past the queue would be trimmed whole, with all before it, and a take gives silence anyway.
            self.buf.clear();
            self.head = start;
        } else {
            self.buf.extend(std::iter::repeat_n(0.0, to_samples(pad)));
        }
        let from = to_samples(skip);
        self.buf.extend(&samples[from..from + to_samples(kept)]);
        self.trim();
    }

    /// Silences the lane from place `at` on, dropping what waits there.
    pub(super) fn cut(&mut self, at: i64) {
        let keep = (at - self.head).clamp(0, self.len());
        self.buf.truncate(to_samples(keep));
        self.cut = Some(at);
    }

    /// Lifts the cut for a new stream, which follows nothing of the old one.
    pub(super) fn resume(&mut self) {
        (self.cut, self.end) = (None, None);
    }

    /// The stereo samples of `frames` frames from place `from`, silence where none waits; what lies before them is
    /// dropped.
    pub(super) fn take(&mut self, from: i64, frames: usize) -> Vec<f32> {
        let mut out = vec![0.0; frames * PAIR];
        let stale = (from - self.head).clamp(0, self.len());
        self.buf.drain(..to_samples(stale));
        self.head = self.head.max(from);
        let lead = usize::try_from(self.head - from)
            .unwrap_or(usize::MAX)
            .min(frames);
        let n = (self.buf.len() / PAIR).min(frames - lead);
        for (o, s) in out[lead * PAIR..]
            .iter_mut()
            .zip(self.buf.drain(..n * PAIR))
        {
            *o = s;
        }
        self.head += i64::try_from(n).unwrap_or(0);
        let past = from + i64::try_from(frames).unwrap_or(0);
        if self.buf.is_empty() {
            self.head = self.head.max(past);
        }
        out
    }

    /// Drops the oldest frames past `keep`.
    fn trim(&mut self) {
        let over = (self.buf.len() / PAIR).saturating_sub(self.keep);
        self.buf.drain(..over * PAIR);
        self.head += i64::try_from(over).unwrap_or(0);
    }
}

/// The samples in `frames` stereo frames, none when negative.
fn to_samples(frames: i64) -> usize {
    usize::try_from(frames).unwrap_or(0) * PAIR
}

#[cfg(test)]
mod tests;
