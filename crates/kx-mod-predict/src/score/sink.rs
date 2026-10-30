//! Where one reading's candidates go: prefixed by its lead, cased as its prefix asks, weighed by its lead's chance,
//! and kept only when a chip may show them.

use kx_module_api::text::Casing;

use super::best::Best;
use super::case::Case;
use crate::script;
use crate::track::Query;

/// One reading's way into the best guesses.
pub(super) struct Sink<'s> {
    /// The best texts so far, across readings.
    pub(super) best: &'s mut Best,
    /// The query, so a guess that changes nothing is dropped and takes the typed apostrophe.
    pub(super) query: &'s Query,
    /// The typed elisions each text starts with.
    pub(super) lead: &'s str,
    /// The case each candidate takes.
    pub(super) case: Case,
    /// How the language cases it.
    pub(super) casing: &'s Casing,
    /// The lead's chance, which every score of the reading is multiplied by.
    pub(super) scale: f64,
}

impl Sink<'_> {
    /// True when a candidate scoring `score` could enter.
    pub(super) fn wants(&self, score: f64) -> bool {
        self.best.wants(score * self.scale)
    }

    /// Keeps `word` at `score` when it can enter, its script spaces words, and picking it changes the text; words of
    /// unspaced scripts wait for the CJK input mode.
    pub(super) fn take(&mut self, word: &str, score: f64) {
        let score = score * self.scale;
        if !self.best.wants(score) {
            return;
        }
        let cased = format!("{}{}", self.lead, self.case.apply(word, self.casing));
        let text = self.query.styled(&cased);
        let spaced = script::of(&text).is_some_and(script::Script::spaced);
        if spaced && self.query.changes(&text) {
            self.best.offer(text, score, word);
        }
    }
}
