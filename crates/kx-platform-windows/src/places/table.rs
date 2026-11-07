//! The places table: the cloud folders, the drive types listed, the attributes that hide an entry, the error codes
//! and the limits, read and checked from `places.toml`.

use std::io;
use std::path::Path;

use kx_module_api::ports::{PlaceError, PlaceKind, box_file, is_box_name};
use serde::Deserialize;
use thiserror::Error;

use crate::disk::shown;
use crate::redirect::one_name;

/// Why the places table is unusable.
#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub(super) enum TableError {
    /// The table is not TOML of the expected shape, or names a key it does not know.
    #[error("the places table does not parse: {0}")]
    Parse(String),
    /// A list or value is empty, a name is no plain name, or a cloud folder has no single base.
    #[error("the places table has an empty value, a bad name or a cloud folder without one base")]
    Bad,
}

/// The places table.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Table {
    /// The most entries a folder list gives.
    pub(super) max_listed: usize,
    /// The most subfolders a find looks in per place.
    pub(super) max_subfolders: usize,
    /// Starts a temp file's name, before the box's file name.
    temp_start: String,
    /// Comes before the process id and before the count in a temp file's name.
    temp_gap: String,
    /// Ends a temp file's name.
    pub(super) temp_end: String,
    /// How old another process's temp file is, in seconds, before a write deletes it.
    pub(super) temp_stale_s: u64,
    /// A labelled drive's name, with `LABEL` and `DRIVE` in it.
    labelled: String,
    /// File attributes that hide an entry.
    hidden: Vec<u32>,
    drives: Types,
    errors: Codes,
    /// The cloud folders, in the order offered.
    pub(super) cloud: Vec<Cloud>,
}

/// `GetDriveTypeW`'s types by the kind of place they become.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Types {
    /// Removable drives, which are suggested.
    removable: Vec<u32>,
    /// Fixed drives, which only the folder list shows.
    fixed: Vec<u32>,
    /// Network drives, which only the folder list shows and nothing else touches.
    remote: Vec<u32>,
}

/// Windows error codes by the port error they become.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Codes {
    /// The place is gone.
    missing: Vec<i32>,
    /// The drive has no room.
    full: Vec<i32>,
    /// The system refused the write.
    read_only: Vec<i32>,
}

/// One cloud folder: an environment variable's folder or a labelled drive, the folders below it, and its shown name.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Cloud {
    /// Its shown name; `None` shows the folder's own name.
    pub(super) name: Option<String>,
    /// The environment variable that holds the base folder.
    pub(super) env: Option<String>,
    /// The label of the drive that is the base folder.
    pub(super) label: Option<String>,
    /// The folders below the base, outermost first.
    #[serde(default)]
    pub(super) parts: Vec<String>,
}

/// Stands for a drive's label in `labelled`.
const LABEL: &str = "{label}";
/// Stands for a drive's letter in `labelled`.
const DRIVE: &str = "{drive}";

impl Table {
    /// Reads and checks the table in `text`.
    pub(super) fn parse(text: &str) -> Result<Self, TableError> {
        let t: Self = toml::from_str(text).map_err(|e| TableError::Parse(e.to_string()))?;
        let lists = [
            &t.drives.removable,
            &t.drives.fixed,
            &t.drives.remote,
            &t.hidden,
        ];
        let codes = [&t.errors.missing, &t.errors.full, &t.errors.read_only];
        let empty = lists.iter().any(|l| l.is_empty()) || codes.iter().any(|l| l.is_empty());
        let limits = t.max_listed > 0 && t.max_subfolders > 0 && t.temp_stale_s > 0;
        let labelled = t.labelled.contains(LABEL) && t.labelled.contains(DRIVE);
        let clouds = !t.cloud.is_empty() && t.cloud.iter().all(Cloud::valid);
        if empty || !limits || !labelled || !t.temp_ok() || !clouds {
            return Err(TableError::Bad);
        }
        Ok(t)
    }

    /// Whether temp names are plain names and never box names, with a gap no digit can be taken for.
    fn temp_ok(&self) -> bool {
        let temp = self.temp_name(0, 0);
        let gap = !self.temp_gap.is_empty() && !self.temp_gap.bytes().any(|b| b.is_ascii_digit());
        gap && one_name(&temp) && !is_box_name(&temp)
    }

    /// The temp file name of write `n` of process `pid`.
    pub(super) fn temp_name(&self, pid: u32, n: u64) -> String {
        let (gap, end) = (&self.temp_gap, &self.temp_end);
        format!("{}{gap}{pid}{gap}{n}{end}", self.temp_head())
    }

    /// The process whose temp file `name` is; `None` for a name of another shape.
    pub(super) fn temp_owner(&self, name: &str) -> Option<u32> {
        let (head, gap) = (self.temp_head(), &self.temp_gap);
        let middle = name.strip_prefix(&head)?.strip_prefix(gap.as_str())?;
        let (pid, n) = middle
            .strip_suffix(&self.temp_end)?
            .split_once(gap.as_str())?;
        let digits = |s: &str| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit());
        (digits(pid) && digits(n)).then(|| pid.parse().ok())?
    }

    /// What every temp file's name starts with: the table's start and the box's file name.
    fn temp_head(&self) -> String {
        format!("{}{}", self.temp_start, box_file())
    }

    /// The kind of place a drive of `GetDriveTypeW` type `code` is; `None` for a type not listed.
    pub(super) fn kind(&self, code: u32) -> Option<PlaceKind> {
        if self.drives.removable.contains(&code) {
            Some(PlaceKind::Removable)
        } else if self.drives.fixed.contains(&code) || self.drives.remote.contains(&code) {
            Some(PlaceKind::Drive)
        } else {
            None
        }
    }

    /// Whether `GetDriveTypeW` type `code` is a network drive.
    pub(super) fn remote(&self, code: u32) -> bool {
        self.drives.remote.contains(&code)
    }

    /// Whether an entry with `attributes` stays out of the lists.
    pub(super) fn hides(&self, attributes: u32) -> bool {
        self.hidden.iter().any(|bit| attributes & bit != 0)
    }

    /// The name of the drive whose root is `root`, with its `label` when it has one.
    pub(super) fn drive_name(&self, root: &Path, label: &str) -> String {
        let letter = shown(&root.to_string_lossy());
        if label.is_empty() {
            return letter;
        }
        // The label goes in last, so text in it is never taken for a stand-in.
        self.labelled.replace(DRIVE, &letter).replace(LABEL, label)
    }

    /// The port error `e` becomes.
    pub(super) fn error(&self, e: &io::Error) -> PlaceError {
        let has = |list: &[i32]| e.raw_os_error().is_some_and(|c| list.contains(&c));
        if has(&self.errors.missing) {
            PlaceError::Missing
        } else if has(&self.errors.full) {
            PlaceError::Full
        } else if has(&self.errors.read_only) {
            PlaceError::ReadOnly
        } else {
            PlaceError::Io
        }
    }
}

impl Cloud {
    /// Whether it has one base, a plain name for each part, and a shown name that is not empty.
    fn valid(&self) -> bool {
        let filled = |v: &Option<String>| v.as_deref().is_some_and(|s| !s.is_empty());
        let one_base = filled(&self.env) != filled(&self.label);
        let none_blank = [&self.name, &self.env, &self.label]
            .iter()
            .all(|v| v.as_deref() != Some(""));
        one_base && none_blank && self.parts.iter().all(|p| one_name(p))
    }
}

/// `error` when a call may return it, else `Io`.
pub(super) fn only(error: PlaceError, kept: &[PlaceError]) -> PlaceError {
    if kept.contains(&error) {
        error
    } else {
        PlaceError::Io
    }
}

#[cfg(test)]
mod tests;
