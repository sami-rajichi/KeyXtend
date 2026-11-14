//! The cutter: the voice detector marks each frame as speech or silence, and a piece is cut only at a pause once it is
//! `piece_s` long, keeping `pad_ms` of the silence after its speech. Pieces follow each other with no gap, so no sample
//! is lost; the engine hears each one trimmed to its speech plus `pad_ms` around it.

use super::wipe::Buf;
use kx_ipc::VoiceFault;
use std::ops::Range;

/// Tells speech from silence, one frame of 16 kHz samples at a time.
pub trait Detector {
    /// Samples in one frame, at least 1.
    fn frame(&self) -> usize;

    /// True when `frame`, exactly `frame()` samples, holds speech.
    fn speech(&mut self, frame: &[i16]) -> bool;
}

/// Opens the voice detector a setup names; P11 task 12 adds the shipped one.
pub trait Detectors {
    /// The detector in the file at `path`, or `None` when this build has none.
    ///
    /// # Errors
    /// `NoFile` or `BadFile` when the file cannot serve.
    fn open(&mut self, path: &str) -> Result<Option<Box<dyn Detector>>, VoiceFault>;
}

/// This build's detectors: none yet, so a run listens without cutting.
pub struct NoDetectors;

impl Detectors for NoDetectors {
    fn open(&mut self, _: &str) -> Result<Option<Box<dyn Detector>>, VoiceFault> {
        Ok(None)
    }
}

/// One piece of a recording, as sample numbers from its start.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Piece {
    /// Its first sample.
    pub start: usize,
    /// The sample after its last.
    pub end: usize,
    /// From its first speech frame to the end of its last, or `None` when it is silent.
    pub speech: Option<Range<usize>>,
}

impl Piece {
    /// What the engine hears: the speech with `pad` samples around it, within the piece; `None` when it is silent.
    pub fn trim(&self, pad: usize) -> Option<Range<usize>> {
        let speech = self.speech.as_ref()?;
        let start = speech.start.saturating_sub(pad).max(self.start);
        Some(start..speech.end.saturating_add(pad).min(self.end))
    }
}

/// What the cutter found.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Cut {
    /// Speech started, or stopped.
    Heard(bool),
    /// A piece ended.
    Piece(Piece),
}

/// Cuts a stream of 16 kHz samples into pieces at pauses.
pub struct Cutter {
    det: Box<dyn Detector>,
    frame: usize,
    /// The shortest piece and the silence kept after speech, in samples.
    piece: usize,
    pad: usize,
    /// The current piece's first sample, and the samples judged so far.
    start: usize,
    pos: usize,
    /// The current piece's first speech sample and the end of its last speech frame.
    first: Option<usize>,
    last: Option<usize>,
    /// The end of the last speech frame of the whole stream.
    spoke: Option<usize>,
    speaking: bool,
    /// Samples waiting for a whole frame.
    pending: Buf<i16>,
}

impl Cutter {
    /// A cutter over `det` making pieces of at least `piece` samples, keeping `pad` samples of silence after speech.
    pub fn new(det: Box<dyn Detector>, piece: usize, pad: usize) -> Self {
        Self {
            frame: det.frame().max(1),
            det,
            piece,
            pad,
            start: 0,
            pos: 0,
            first: None,
            last: None,
            spoke: None,
            speaking: false,
            pending: Buf::new(),
        }
    }

    /// Judges the whole frames `x` completes, adding to `cuts` what they show.
    ///
    /// # Errors
    /// `Memory` when the waiting samples cannot be kept.
    pub fn take(&mut self, mut x: &[i16], cuts: &mut Vec<Cut>) -> Result<(), VoiceFault> {
        if !self.pending.is_empty() {
            let (head, tail) = x.split_at((self.frame - self.pending.len()).min(x.len()));
            self.pending.extend(head)?;
            x = tail;
            if self.pending.len() < self.frame {
                return Ok(());
            }
            let mut frame = std::mem::take(&mut self.pending);
            self.judge(&frame, cuts);
            frame.clear();
            self.pending = frame;
        }
        let mut frames = x.chunks_exact(self.frame);
        for frame in &mut frames {
            self.judge(frame, cuts);
        }
        self.pending.extend(frames.remainder())
    }

    /// Ends the stream: the waiting samples join the last frame's kind, and the last piece is added unless empty.
    pub fn finish(&mut self, cuts: &mut Vec<Cut>) {
        let tail = self.pending.len();
        self.pending.clear();
        if tail > 0 && self.speaking {
            self.last = Some(self.pos + tail);
            self.spoke = self.last;
        }
        self.pos += tail;
        if self.pos > self.start {
            cuts.push(Cut::Piece(self.piece_to(self.pos)));
        }
        (self.start, self.first, self.last) = (self.pos, None, None);
    }

    /// Samples of silence since the last speech, or `None` before any speech.
    pub fn quiet(&self) -> Option<usize> {
        self.spoke.map(|end| self.pos - end)
    }

    /// Asks the detector about one frame, tells a change of kind, and cuts after a silent one when it may.
    fn judge(&mut self, frame: &[i16], cuts: &mut Vec<Cut>) {
        let speech = self.det.speech(frame);
        let end = self.pos + self.frame;
        if speech {
            self.first.get_or_insert(self.pos);
            (self.last, self.spoke) = (Some(end), Some(end));
        }
        if speech != self.speaking {
            self.speaking = speech;
            cuts.push(Cut::Heard(speech));
        }
        self.pos = end;
        if !speech {
            self.try_cut(cuts);
        }
    }

    /// Cuts once the piece holds speech, is `piece` long and its speech is followed by `pad` of silence.
    fn try_cut(&mut self, cuts: &mut Vec<Cut>) {
        let Some(last) = self.last else {
            return;
        };
        let at = (last + self.pad).max(self.start + self.piece);
        if at > self.pos {
            return;
        }
        cuts.push(Cut::Piece(self.piece_to(at)));
        (self.start, self.first, self.last) = (at, None, None);
    }

    /// The current piece, ending at `end`.
    fn piece_to(&self, end: usize) -> Piece {
        let speech = self.first.zip(self.last).map(|(a, b)| a..b);
        Piece {
            start: self.start,
            end,
            speech,
        }
    }
}

#[cfg(test)]
mod tests;
