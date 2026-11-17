//! A listen-only run's speech packed for a cloud service: each stretch of speech with its pad joins the piece before it
//! while the piece stays within `cap` samples, so two pieces in a row always hold more than `cap`. A longer stretch is
//! cut at its quietest stretches first, as a model's long piece is.

use super::split;
use std::ops::Range;

/// The pieces packed so far, each as the stretches of the recording it joins.
pub struct Packer {
    cap: usize,
    pieces: Vec<Vec<Range<usize>>>,
    /// Samples in the last piece.
    size: usize,
}

impl Packer {
    /// A packer of pieces of at most `cap` samples, at least one.
    pub fn new(cap: usize) -> Self {
        Self {
            cap: cap.max(1),
            pieces: Vec::new(),
            size: 0,
        }
    }

    /// Adds the stretch `span` of `sound` after the last, cut at its quietest stretches when longer than `cap`; an
    /// empty one adds nothing.
    pub fn add(&mut self, sound: &[i16], span: Range<usize>) {
        match sound.get(span.clone()) {
            Some(stretch) if span.len() > self.cap => {
                for part in split::spans(stretch, Some(self.cap)) {
                    self.join(span.start + part.start..span.start + part.end);
                }
            }
            _ => self.join(span),
        }
    }

    /// The pieces in order.
    pub fn pieces(self) -> Vec<Vec<Range<usize>>> {
        self.pieces
    }

    /// Joins `part`, no longer than `cap`, to the last piece, or starts a new one when it would pass `cap`; an empty
    /// part joins nothing.
    fn join(&mut self, part: Range<usize>) {
        let len = part.len();
        if len == 0 {
            return;
        }
        match self.pieces.last_mut() {
            Some(last) if self.size + len <= self.cap => {
                last.push(part);
                self.size += len;
            }
            _ => {
                self.pieces.push(vec![part]);
                self.size = len;
            }
        }
    }
}

#[cfg(test)]
mod tests;
