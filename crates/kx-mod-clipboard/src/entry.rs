//! A kept copy (spec §7 "What is kept"): its kind, place, time, app, sizes, title, search text and content hash,
//! built from the copy's parts. The kind rules live in `kind`.

mod kind;

use std::borrow::Cow;
use std::path::{Path, PathBuf};

use kx_module_api::Redacted;
use kx_module_api::ports::{AppRef, ClipKind, ClipPart, Date, FileFact, LocalTime};
use kx_module_api::text::fold_any;

pub use kind::keep_parts;

use crate::hash;
use kind::kind_of;

/// What a run of blanks and line breaks reads as in a title and the search text; it also parts file names.
const SPACE: char = ' ';
/// What parts the search text's pieces; no query holds it, so none matches across two.
const PIECE_SEP: &str = "\n";

/// Where an entry is kept: the folder of its copy day, or the pinned one.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Place {
    /// The folder of this local calendar day.
    Day(Date),
    /// The pinned folder, which no sweep touches.
    Pinned,
}

/// When an entry was copied: the moment and the UTC offset there, so its day stays the local day of the copy.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Stamp {
    /// Milliseconds since the Unix epoch, in UTC.
    pub unix_ms: i64,
    /// The local offset from UTC then, in minutes east.
    pub offset_min: i16,
}

impl Stamp {
    /// The stamp of `now`.
    #[must_use]
    pub fn of(now: &LocalTime) -> Self {
        let (unix_ms, offset_min) = (now.unix_ms, now.offset_min);
        Self {
            unix_ms,
            offset_min,
        }
    }

    /// The wall time of the copy where it was made; `None` past the years a `Date` holds.
    #[must_use]
    pub fn local(self) -> Option<LocalTime> {
        LocalTime::at(self.unix_ms, self.offset_min)
    }
}

/// What building an entry needs from the settings; the module fills it.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct EntryConfig {
    /// The most characters a title keeps (`title_chars`).
    pub title_chars: usize,
    /// The most characters of the text, and of the file names, that search reads (`search_chars`).
    pub search_chars: usize,
}

/// One kept copy as the list holds it; its content lives in the store. Its `Debug` hides every text, path and app.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    /// Names it in the store and the panel; `History::add` numbers it.
    pub id: u64,
    /// Its place in the order, higher being newer, so the order never follows the wall clock; `History` sets it.
    pub seq: u64,
    /// What it holds.
    pub kind: ClipKind,
    /// Where it is kept.
    pub place: Place,
    /// When it was last copied or inserted, or unpinned.
    pub at: Stamp,
    /// The app that copied it, when known.
    pub app: Option<AppRef>,
    /// Bytes the history keeps for it: its parts' bytes.
    pub stored: u64,
    /// Bytes its row shows: what is kept, or the files' size for a file row.
    pub shown: u64,
    /// The start of its text, or its file names, on one line.
    pub title: Redacted<String>,
    /// The folded text, file names and app name that search matches; a line break parts them.
    pub search: Redacted<String>,
    /// The content hash, as `hash::of` gives it.
    pub hash: u64,
    /// How many blobs hold its content; the store sets it when it saves them.
    pub parts: u16,
    /// Its thumbnail is saved; the media queue sets it.
    pub thumb: bool,
    /// The paths of a file entry, else none.
    pub files: Redacted<Vec<PathBuf>>,
}

impl Entry {
    /// The entry for a copy by `app` at `now` of `parts`, as `keep_parts` left them, in the folder of that day;
    /// `History::add` numbers it.
    #[must_use]
    pub fn new(
        parts: &[ClipPart],
        app: Option<AppRef>,
        now: &LocalTime,
        cfg: &EntryConfig,
    ) -> Self {
        let kind = kind_of(parts);
        let files = files_of(parts).to_vec();
        let names = names(&files);
        let text = text_of(parts).unwrap_or_default();
        let head = if text.trim().is_empty() { &names } else { text };
        let search = search(text, &names, app.as_ref(), cfg.search_chars);
        let stored = parts.iter().map(bytes_of).fold(0, u64::saturating_add);
        Self {
            id: 0,
            seq: 0,
            kind,
            place: Place::Day(now.date),
            at: Stamp::of(now),
            app,
            stored,
            shown: stored,
            title: Redacted::new(title(head, cfg.title_chars)),
            search: Redacted::new(search),
            hash: hash::of(kind, parts),
            parts: 0,
            thumb: false,
            files: Redacted::new(files),
        }
    }

    /// True when it is pinned.
    #[must_use]
    pub fn pinned(&self) -> bool {
        self.place == Place::Pinned
    }

    /// Shows on a file row the size of the files still there, `facts` being what the platform found at its paths.
    pub fn size_files(&mut self, facts: &[FileFact]) {
        let sizes = facts.iter().map(|f| match f {
            FileFact::Size(n) => *n,
            FileFact::Gone => 0,
        });
        self.shown = sizes.fold(0, u64::saturating_add);
    }
}

/// The plain text part's text.
fn text_of(parts: &[ClipPart]) -> Option<&str> {
    parts.iter().find_map(|p| match p {
        ClipPart::Text(text) => Some(text.expose().as_str()),
        _ => None,
    })
}

/// The file part's paths, or none.
fn files_of(parts: &[ClipPart]) -> &[PathBuf] {
    let paths = parts.iter().find_map(|p| match p {
        ClipPart::Files(paths) => Some(paths.expose().as_slice()),
        _ => None,
    });
    paths.unwrap_or_default()
}

/// The names of `files`, parted by a space.
fn names(files: &[PathBuf]) -> String {
    let mut all = String::new();
    for path in files {
        if !all.is_empty() {
            all.push(SPACE);
        }
        all.push_str(&name(path));
    }
    all
}

/// The name of `path`; a path with no name, such as a drive, stands whole.
fn name(path: &Path) -> Cow<'_, str> {
    path.file_name()
        .unwrap_or(path.as_os_str())
        .to_string_lossy()
}

/// `text` on one line: each run of blanks, line breaks and control characters as one space.
fn flat(text: &str) -> impl Iterator<Item = char> + '_ {
    let mut gap = false;
    text.chars().filter_map(move |c| {
        let blank = c.is_whitespace() || c.is_control();
        let was = std::mem::replace(&mut gap, blank);
        match (blank, was) {
            (false, _) => Some(c),
            (true, false) => Some(SPACE),
            (true, true) => None,
        }
    })
}

/// The first `chars` characters of `text` on one line, without the blanks around them.
fn title(text: &str, chars: usize) -> String {
    let line: String = flat(text)
        .skip_while(|c| c.is_whitespace())
        .take(chars)
        .collect();
    line.trim_end().to_owned()
}

/// The search text: the first `chars` characters of `text` and of `names`, and the app's name, each on one line and
/// folded, the empty ones left out.
fn search(text: &str, names: &str, app: Option<&AppRef>, chars: usize) -> String {
    let cut = |s: &str, n: usize| fold_any(&flat(s).take(n).collect::<String>());
    let app = app.map(|a| cut(&a.name, usize::MAX)).unwrap_or_default();
    let pieces = [cut(text, chars), cut(names, chars), app];
    let held: Vec<&str> = pieces
        .iter()
        .map(String::as_str)
        .filter(|p| !p.is_empty())
        .collect();
    held.join(PIECE_SEP)
}

/// Bytes a part keeps: its text or bytes, or its paths.
fn bytes_of(part: &ClipPart) -> u64 {
    let n = match part {
        ClipPart::Text(text) => text.expose().len(),
        ClipPart::Html(bytes) | ClipPart::Rtf(bytes) | ClipPart::Image { bytes, .. } => {
            bytes.expose().len()
        }
        ClipPart::Files(paths) => paths
            .expose()
            .iter()
            .map(|p| p.as_os_str().as_encoded_bytes().len())
            .sum(),
    };
    u64::try_from(n).unwrap_or(u64::MAX)
}

#[cfg(test)]
mod tests;
