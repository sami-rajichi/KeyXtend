//! Reads over the lexicon's keys: the ids a prefix stream finds, and the first id at or past a key.

use std::ops::Range;

use fst::automaton::{StartsWith, Str};
use fst::map::{Map, Stream};
use fst::{IntoStreamer, Streamer};

use super::super::WordId;
use super::run;

/// The first id of the first key at or past `bound` in `lexicon`; `words`, the pack's word count, when none is.
pub(super) fn first_at(lexicon: &Map<&[u8]>, words: u32, bound: &[u8]) -> WordId {
    let mut keys = lexicon.range().ge(bound).into_stream();
    let first = keys.next().and_then(|(_, value)| run(value, words));
    first.map_or(words, |ids| ids.start)
}

/// The least byte string past every string that starts with `prefix`; `None` when no string is.
pub(super) fn successor(prefix: &[u8]) -> Option<Vec<u8>> {
    let mut next = prefix.to_vec();
    while let Some(last) = next.pop() {
        if let Some(up) = last.checked_add(1) {
            next.push(up);
            return Some(next);
        }
    }
    None
}

/// The ids of each key a prefix stream finds, reading at most as many keys as the pack has words.
pub(super) struct Completions<'s> {
    /// The lexicon's keys that start with the prefix.
    pub(super) stream: Stream<'s, StartsWith<Str<'s>>>,
    /// The ids of the current key not yet given.
    pub(super) run: Range<WordId>,
    /// Keys still allowed, so a damaged lexicon cannot stream forever.
    pub(super) keys_left: u32,
    /// The pack's word count.
    pub(super) words: u32,
}

impl Iterator for Completions<'_> {
    type Item = WordId;

    fn next(&mut self) -> Option<WordId> {
        loop {
            if let Some(id) = self.run.next() {
                return Some(id);
            }
            self.keys_left = self.keys_left.checked_sub(1)?;
            let (_, value) = self.stream.next()?;
            self.run = run(value, self.words).unwrap_or_default();
        }
    }
}
