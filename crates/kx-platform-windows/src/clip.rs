//! The clipboard (spec §7, ADR-0034): a watch that tells each copy, marked copy and clear, a writer that sets parts
//! with our marker, and what is at copied paths. The table and the rules are pure; the Win32 calls are Windows-only.
#![cfg_attr(
    not(windows),
    allow(
        dead_code,
        reason = "Only the Windows clipboard runs this logic; tests cover it everywhere."
    )
)]

#[cfg(windows)]
mod app;
mod hdrop;
mod marks;
#[cfg(windows)]
pub mod open;
mod parts;
#[cfg(windows)]
mod read;
#[cfg(windows)]
mod watch;
#[cfg(windows)]
mod write;

use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use kx_module_api::Redacted;
use kx_module_api::ports::{ClipError, FileFact};
use serde::Deserialize;
use thiserror::Error;

pub(crate) use marks::Marks;
#[cfg(windows)]
pub use marks::{marked, markers};
#[cfg(windows)]
pub use read::read_text;
#[cfg(windows)]
pub use watch::WinClipboard;
#[cfg(windows)]
pub use write::{WinClipboardWriter, write_text};

/// The format table, compiled in so it is never read from disk.
const TABLE: &str = include_str!("clip.toml");

/// The shipped table, parsed once; tests prove it parses.
static SHIPPED: LazyLock<Result<Table, TableError>> = LazyLock::new(|| Table::parse(TABLE));

/// The shipped table. One that does not parse refuses every watch and write, so no marked copy is ever read.
pub(crate) fn table() -> Result<&'static Table, ClipError> {
    SHIPPED.as_ref().map_err(|e| {
        tracing::warn!(error = %e, "the clipboard table does not parse");
        ClipError::Refused
    })
}

/// Why the format table is unusable.
#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub(crate) enum TableError {
    /// The table is not TOML of the expected shape, or names a key it does not know.
    #[error("the clipboard table does not parse: {0}")]
    Parse(String),
    /// A name, a marker list or a limit is empty.
    #[error("the clipboard table has an empty name, marker list or limit")]
    Empty,
    /// Our own marker is none the watch skips, so our writes would come back as copies.
    #[error("the clipboard table's own marker is not one the watch skips")]
    Unmarked,
}

/// The format table: the markers, the registered formats we keep, and the app-name limits.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Table {
    /// Formats whose presence alone marks a copy.
    skip: Vec<String>,
    /// Formats holding a 32-bit number, where 0 marks a copy.
    skip_zero: Vec<String>,
    /// The marker each write of ours sets to 0.
    pub(crate) marker: String,
    /// The cloud switch each write of ours sets to 0.
    pub(crate) cloud_off: String,
    /// The HTML format's name.
    pub(crate) html: String,
    /// The RTF format's name.
    pub(crate) rtf: String,
    /// The PNG format's name.
    pub(crate) png: String,
    /// The format that tells a copy of files from a move.
    pub(crate) drop_effect: String,
    /// How an app's name is read.
    pub(crate) app: AppTable,
}

/// How the app behind a copy is named.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct AppTable {
    /// The most bytes of version information read.
    pub(crate) version_bytes: u32,
    /// The longest app name kept, in characters.
    pub(crate) name_chars: usize,
}

impl Table {
    /// Reads and checks the table in `text`.
    pub(crate) fn parse(text: &str) -> Result<Self, TableError> {
        let t: Self = toml::from_str(text).map_err(|e| TableError::Parse(e.to_string()))?;
        let names = [
            &t.marker,
            &t.cloud_off,
            &t.html,
            &t.rtf,
            &t.png,
            &t.drop_effect,
        ];
        let blank = t
            .skip
            .iter()
            .chain(&t.skip_zero)
            .chain(names)
            .any(|n| n.trim().is_empty());
        let no_marks = t.skip.is_empty() && t.skip_zero.is_empty();
        if blank || no_marks || t.app.version_bytes == 0 || t.app.name_chars == 0 {
            return Err(TableError::Empty);
        }
        if !t.marks().has(&t.marker) {
            return Err(TableError::Unmarked);
        }
        Ok(t)
    }

    /// The markers the product's watch skips.
    pub(crate) fn marks(&self) -> Marks {
        Marks::new(self.skip.clone(), self.skip_zero.clone())
    }
}

/// What is at each of `paths` now, in order: a file's size, 0 for a folder or a path Windows cannot read (an offline
/// drive keeps its entries), or gone when nothing is there.
pub(crate) fn facts(paths: &Redacted<Vec<PathBuf>>) -> Vec<FileFact> {
    paths.expose().iter().map(|p| fact(p)).collect()
}

/// What is at `path` now.
fn fact(path: &Path) -> FileFact {
    match std::fs::metadata(path) {
        Ok(m) if m.is_file() => FileFact::Size(m.len()),
        Err(e) if e.kind() == ErrorKind::NotFound => FileFact::Gone,
        Ok(_) | Err(_) => FileFact::Size(0),
    }
}

#[cfg(test)]
mod tests;
