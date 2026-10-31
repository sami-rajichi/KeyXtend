//! The insert plan (spec §7 "Insert", ADR-0034 decision 2): the ticked entries in tick order, each written and then
//! pasted by the keyboard; between two texts the join's breaks and rule, pasted as text and never typed as Enter. A
//! file entry pastes only the files still there; one whose files are all gone leaves, and one out of reach is skipped.

use std::collections::{BTreeMap, VecDeque};
use std::path::PathBuf;

use kx_module_api::Redacted;
use kx_module_api::ports::{ClipKind, ClipPart, FileFact};

use crate::order::History;

/// The neutral line break; the platform writes its own, such as CR LF on Windows.
const BREAK: char = '\n';

/// What an insert pastes between two texts; the module fills it from the settings.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JoinConfig {
    /// Line breaks between two texts (`join_breaks`).
    pub breaks: u8,
    /// A separator line after them, then the same breaks again; empty for none (`join_rule`).
    pub rule: String,
}

impl JoinConfig {
    /// The text pasted between two texts.
    #[must_use]
    pub fn text(&self) -> String {
        let breaks: String = std::iter::repeat_n(BREAK, usize::from(self.breaks)).collect();
        if self.rule.is_empty() {
            breaks
        } else {
            format!("{breaks}{}{breaks}", self.rule)
        }
    }
}

/// One pasted step of an insert.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Step {
    /// The join between two texts.
    Join,
    /// Entry `id`; a file entry pastes only `files`, those still there.
    Entry {
        /// The entry.
        id: u64,
        /// The files a file entry pastes, or `None` for its parts as kept.
        files: Option<Redacted<Vec<PathBuf>>>,
    },
}

/// What the file checks before an insert found.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Sorted {
    /// The files still there of each file entry that has one.
    pub keep: BTreeMap<u64, Redacted<Vec<PathBuf>>>,
    /// The file entries whose every file is gone, which leave the history.
    pub gone: Vec<u64>,
    /// The file entries with no file in reach, which the insert skips and keeps.
    pub away: Vec<u64>,
}

/// An insert under way: the entries left to paste, in tick order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Insert {
    /// The entries left, the next first.
    queue: VecDeque<u64>,
    /// The files each file entry pastes.
    keep: BTreeMap<u64, Redacted<Vec<PathBuf>>>,
    /// The last pasted entry was text, so the next text takes a join first.
    after_text: bool,
    /// The join before the next entry is pasted.
    joined: bool,
}

impl Insert {
    /// An insert of `ids` in this order, less the file entries `sorted` found gone or out of reach.
    #[must_use]
    pub fn new(ids: Vec<u64>, sorted: Sorted) -> Self {
        let out = |id: &u64| sorted.gone.contains(id) || sorted.away.contains(id);
        Self {
            queue: ids.into_iter().filter(|id| !out(id)).collect(),
            keep: sorted.keep,
            after_text: false,
            joined: false,
        }
    }

    /// The next step over `history`: an entry gone meanwhile is skipped, and two texts in a row get the join between.
    pub fn next(&mut self, history: &History) -> Option<Step> {
        loop {
            let id = *self.queue.front()?;
            let Some(entry) = history.get(id) else {
                self.queue.pop_front();
                continue;
            };
            let text = is_text(entry.kind);
            if text && self.after_text && !self.joined {
                self.joined = true;
                return Some(Step::Join);
            }
            self.queue.pop_front();
            (self.after_text, self.joined) = (text, false);
            let files = self.keep.remove(&id);
            return Some(Step::Entry { id, files });
        }
    }
}

/// The file and media entries among `ids`, with their paths, which an insert checks first.
#[must_use]
pub fn files_of(ids: &[u64], history: &History) -> Vec<(u64, Redacted<Vec<PathBuf>>)> {
    let found = ids.iter().filter_map(|id| history.get(*id));
    let filed = found.filter(|e| has_files(e.kind));
    filed.map(|e| (e.id, e.files.clone())).collect()
}

/// Sorts `facts`, what the platform found at each file entry's paths in order, over `history`.
#[must_use]
pub fn sort(facts: &[(u64, Vec<FileFact>)], history: &History) -> Sorted {
    let mut sorted = Sorted::default();
    for (id, found) in facts {
        let Some(entry) = history.get(*id) else {
            continue;
        };
        let paths = entry.files.expose().iter().zip(found);
        let there: Vec<PathBuf> = paths
            .filter(|(_, f)| matches!(f, FileFact::Size(_)))
            .map(|(p, _)| p.clone())
            .collect();
        if !there.is_empty() {
            sorted.keep.insert(*id, Redacted::new(there));
        } else if found.iter().all(|f| *f == FileFact::Gone) {
            sorted.gone.push(*id);
        } else {
            sorted.away.push(*id);
        }
    }
    sorted
}

/// `parts` as an insert writes them: the file list cut to `files` when given.
#[must_use]
pub fn paste_parts(parts: Vec<ClipPart>, files: Option<&Redacted<Vec<PathBuf>>>) -> Vec<ClipPart> {
    let Some(files) = files else {
        return parts;
    };
    let cut = |part| match part {
        ClipPart::Files(_) => ClipPart::Files(files.clone()),
        other => other,
    };
    parts.into_iter().map(cut).collect()
}

/// True when entries of `kind` take a join between two of them.
fn is_text(kind: ClipKind) -> bool {
    matches!(kind, ClipKind::Text | ClipKind::Link)
}

/// True when entries of `kind` point to files, which an insert checks first.
fn has_files(kind: ClipKind) -> bool {
    matches!(kind, ClipKind::File | ClipKind::Media)
}

#[cfg(test)]
mod tests;
