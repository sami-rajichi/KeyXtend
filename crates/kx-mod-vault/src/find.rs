//! Search (design "Search line"): the query, folded as the clipboard's search folds it, matches anywhere in an entry's
//! name, a field's label or a shown field's value, each on its own; a hidden value is never searched. Every fold of a
//! value or of the query is made in a wiped buffer.

use std::collections::{BTreeMap, BTreeSet};

use kx_module_api::ports::Label;
use kx_module_api::text::{SPACE, fold_any_into, one_line};
use kx_module_api::{Redacted, Secret, Zeroizing, is_hidden};

use crate::entry::Entry;
use crate::kinds::tables;

/// The text each table label shows in the keyboard's language, by label id; the ui gives them (Task 20), and a label
/// with no text matches nothing.
pub type LabelNames = BTreeMap<String, String>;

/// A search: the folded query, and the table labels whose shown text holds it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Find {
    /// The query on one line, folded, without the blanks around it.
    query: Secret,
    /// The ids of the table labels it finds.
    labels: BTreeSet<String>,
}

/// The two wiped buffers one search reuses for each text: the fold, then the text on one line.
#[derive(Default)]
struct Scratch {
    folded: Zeroizing<String>,
    line: Zeroizing<String>,
}

impl Find {
    /// The search for the search line `query`, with `names` for the table labels' texts.
    #[must_use]
    pub fn new(query: &str, names: &LabelNames) -> Self {
        let query = folded(query);
        let mut scratch = Scratch::default();
        let ids = tables().labels().iter().map(|l| &l.id);
        let labels = ids
            .filter(|id| {
                names
                    .get(*id)
                    .is_some_and(|n| holds(n, &query, &mut scratch))
            })
            .cloned()
            .collect();
        Self {
            query: Redacted::new(query),
            labels,
        }
    }

    /// The entries of `all` it keeps, in their order; every text folds into the same wiped buffers.
    #[must_use]
    pub fn list<'a>(&self, all: impl IntoIterator<Item = &'a Entry>) -> Vec<&'a Entry> {
        let mut scratch = Scratch::default();
        let kept = all.into_iter().filter(|e| self.kept(e, &mut scratch));
        kept.collect()
    }

    /// True when `entry`'s name, one of its labels or one of its shown values holds the query, folding into `scratch`.
    fn kept(&self, entry: &Entry, scratch: &mut Scratch) -> bool {
        let query = self.query.expose().as_str();
        if query.is_empty() || holds(entry.name().expose(), query, scratch) {
            return true;
        }
        entry.fields().iter().any(|f| {
            let label = match &f.label {
                Label::Table(id) => self.labels.contains(id),
                Label::Own(own) => holds(own.expose(), query, scratch),
            };
            label || (!f.hidden && holds(f.value.expose(), query, scratch))
        })
    }
}

/// `text` as a search line reads it, in a wiped buffer: on one line, folded, without the blanks around it.
pub(crate) fn folded(text: &str) -> Zeroizing<String> {
    let mut scratch = Scratch::default();
    searched(text, &mut scratch);
    let mut line = scratch.line;
    trim(&mut line);
    line
}

/// True when `text`, as search reads it, holds `query`.
fn holds(text: &str, query: &str, scratch: &mut Scratch) -> bool {
    searched(text, scratch);
    scratch.line.contains(query)
}

/// `text` as search reads it, into `scratch.line`: folded, without the hidden marks no saved text holds, and on one
/// line, where line and paragraph separators read as spaces. The line never outgrows the fold, so a buffer too small
/// is swapped whole and the old one wiped.
fn searched(text: &str, scratch: &mut Scratch) {
    fold_any_into(text, &mut scratch.folded);
    let marked = |c: &char| is_hidden(*c) && !c.is_control() && !c.is_whitespace();
    let need = scratch.folded.len();
    let line = &mut scratch.line;
    if line.capacity() < need {
        *line = Zeroizing::new(String::with_capacity(need));
    }
    line.clear();
    line.extend(one_line(scratch.folded.chars().filter(|c| !marked(c))));
}

/// `text` without the one space a blank run around it reads as, in place.
fn trim(text: &mut String) {
    if text.ends_with(SPACE) {
        text.pop();
    }
    if text.starts_with(SPACE) {
        text.remove(0);
    }
}

#[cfg(test)]
mod tests;
