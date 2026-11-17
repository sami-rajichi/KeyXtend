//! A listen-only run's speech packed for a cloud service: each stretch of speech with its pad joins the piece before it
//! while the piece stays within `cap` samples, so two pieces in a row always hold more than `cap`; a longer stretch is
//! split at `cap`.

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

    /// Adds the stretch `span` after the last, split at `cap` when longer; an empty one adds nothing.
    pub fn add(&mut self, span: Range<usize>) {
        let mut at = span.start;
        while at < span.end {
            let part = at..span.end.min(at.saturating_add(self.cap));
            at = part.end;
            self.join(part);
        }
    }

    /// The pieces in order.
    pub fn pieces(self) -> Vec<Vec<Range<usize>>> {
        self.pieces
    }

    /// Joins `part`, no longer than `cap`, to the last piece, or starts a new one when it would pass `cap`.
    fn join(&mut self, part: Range<usize>) {
        let len = part.len();
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
mod tests {
    use super::*;

    #[test]
    fn stretches_join_in_order_while_their_piece_stays_within_the_cap() {
        let mut pack = Packer::new(9);
        for span in [0..3, 5..9, 12..15, 15..20, 30..30] {
            pack.add(span);
        }
        assert_eq!(pack.pieces(), [vec![0..3, 5..9], vec![12..15, 15..20]]);
    }

    #[test]
    fn a_stretch_past_the_cap_is_split_and_two_pieces_in_a_row_pass_it() {
        let mut pack = Packer::new(4);
        pack.add(0..1);
        pack.add(2..12);
        let pieces = pack.pieces();
        let ends: Vec<Vec<(usize, usize)>> = pieces
            .iter()
            .map(|p| p.iter().map(|r| (r.start, r.end)).collect())
            .collect();
        assert_eq!(ends, [[(0, 1)], [(2, 6)], [(6, 10)], [(10, 12)]]);
        let sizes: Vec<usize> = pieces
            .iter()
            .map(|p| p.iter().map(ExactSizeIterator::len).sum())
            .collect();
        assert!(sizes.iter().all(|&n| n <= 4));
        assert!(sizes.windows(2).all(|w| w[0] + w[1] > 4));
    }

    #[test]
    fn nothing_added_is_no_piece() {
        assert!(Packer::new(0).pieces().is_empty());
    }
}
