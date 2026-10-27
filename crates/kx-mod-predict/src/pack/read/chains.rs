//! One chain section in place: a binary search over its sorted contexts, then that context's followers. Every count,
//! start and id is read where it is used, with checked arithmetic and slicing.

use std::cmp::Ordering;
use std::ops::Range;

use super::super::layout::{
    CHAIN_AT_TOTAL, CHAIN_HEAD, FOLLOWER_LEN, ID_BYTES, ID_MASK, SCORE_SHIFT, id_at, record_len,
    u32_at, usize_at,
};
use super::super::{PackFormatError, WordId};

/// Chain section `n`: contexts of `n − 1` ids, each with its followers, highest score first.
#[derive(Clone, Copy)]
pub(super) struct Chain<'a> {
    /// The section's bytes.
    pub sec: &'a [u8],
    /// The chain length.
    pub n: u8,
    /// The most followers a context yields.
    pub cap: usize,
    /// The pack's word count; a follower at or past it is skipped.
    pub words: u32,
}

impl Chain<'_> {
    /// `BadSection` unless the two counts and the record size add up to the section's length.
    pub fn check(&self) -> Result<(), PackFormatError> {
        let out = PackFormatError::OutOfBounds;
        let (contexts, total) = (self.contexts().ok_or(out)?, self.total().ok_or(out)?);
        let records = contexts.checked_mul(record_len(self.n));
        let entries = total.checked_mul(FOLLOWER_LEN);
        let len = records
            .zip(entries)
            .and_then(|(r, e)| r.checked_add(e)?.checked_add(CHAIN_HEAD));
        match len {
            Some(len) if len == self.sec.len() => Ok(()),
            _ => Err(PackFormatError::BadSection),
        }
    }

    /// The number of contexts.
    fn contexts(&self) -> Option<usize> {
        usize_at(self.sec, 0)
    }

    /// The number of followers of all contexts.
    fn total(&self) -> Option<usize> {
        usize_at(self.sec, CHAIN_AT_TOTAL)
    }

    /// Where context record `j` starts.
    fn record(&self, j: usize) -> Option<usize> {
        j.checked_mul(record_len(self.n))?.checked_add(CHAIN_HEAD)
    }

    /// The index of record `j`'s first follower, after its ids.
    fn start(&self, j: usize) -> Option<usize> {
        let ids = usize::from(self.n).checked_sub(1)?.checked_mul(ID_BYTES)?;
        usize_at(self.sec, self.record(j)?.checked_add(ids)?)
    }

    /// How record `j`'s context compares with `want`, id by id.
    fn compare(&self, j: usize, want: &[WordId]) -> Option<Ordering> {
        let at = self.record(j)?;
        for (i, w) in want.iter().enumerate() {
            let id = id_at(self.sec, i.checked_mul(ID_BYTES)?.checked_add(at)?)?;
            if id != *w {
                return Some(id.cmp(w));
            }
        }
        Some(Ordering::Equal)
    }

    /// The follower indexes of `want`, at most `cap` of them, by binary search over the sorted contexts; `None` when
    /// it is unknown or the section is damaged.
    pub fn find(&self, want: &[WordId]) -> Option<Range<usize>> {
        let count = self.contexts()?;
        let (mut lo, mut hi) = (0, count);
        while lo < hi {
            let mid = lo.midpoint(hi);
            match self.compare(mid, want)? {
                Ordering::Less => lo = mid + 1,
                Ordering::Greater => hi = mid,
                Ordering::Equal => {
                    let next = mid + 1;
                    let end = if next < count {
                        self.start(next)?
                    } else {
                        self.total()?
                    };
                    let start = self.start(mid)?;
                    return (start <= end).then(|| start..end.min(start.saturating_add(self.cap)));
                }
            }
        }
        None
    }

    /// Follower `f`: its id and score; `None` past the section or for an id past the word table.
    pub fn follower(&self, f: usize) -> Option<(WordId, u8)> {
        let first = self.record(self.contexts()?)?;
        let entry = u32_at(self.sec, f.checked_mul(FOLLOWER_LEN)?.checked_add(first)?)?;
        let id = entry & ID_MASK;
        let score = u8::try_from(entry >> SCORE_SHIFT).ok()?;
        (id < self.words).then_some((id, score))
    }
}
