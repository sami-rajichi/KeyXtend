//! The fake places' tree: folders and files in memory, the folders the test made vanish, read-only or tight on room,
//! the data folder, and the failures and counts of the calls.

use super::{PlaceCall, SUGGESTED};
use kx_module_api::Redacted;
use kx_module_api::ports::{Found, Place, PlaceError, PlaceKind, is_box_head, is_box_name};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::{Path, PathBuf};

/// Free bytes in a place until the test sets them: 1 TiB, more than any test fills.
const ROOMY: u64 = 1 << 40;

/// A file's bytes and when it was last written, in ms since the Unix epoch.
pub(super) struct File {
    pub(super) bytes: Vec<u8>,
    pub(super) modified_ms: i64,
}

/// The tree, what the test set for it, and what the calls did.
#[derive(Default)]
pub(super) struct Tree {
    /// The places added, in order.
    pub(super) tops: Vec<Place>,
    pub(super) dirs: BTreeSet<PathBuf>,
    pub(super) files: BTreeMap<PathBuf, File>,
    /// Folders that vanished, with all below them.
    pub(super) gone: BTreeSet<PathBuf>,
    /// Folders that refuse writes, with all below them.
    pub(super) read_only: BTreeSet<PathBuf>,
    /// Free bytes by folder, for all below it.
    pub(super) free: BTreeMap<PathBuf, u64>,
    pub(super) data: Option<PathBuf>,
    /// The time a write stamps, in ms since the Unix epoch.
    pub(super) now_ms: i64,
    pub(super) fails: HashMap<PlaceCall, PlaceError>,
    pub(super) calls: HashMap<PlaceCall, usize>,
}

impl Tree {
    /// Counts a `call` that cannot fail.
    pub(super) fn count(&mut self, call: PlaceCall) {
        *self.calls.entry(call).or_default() += 1;
    }

    /// Counts a `call`, then takes the failure scripted for it.
    pub(super) fn enter(&mut self, call: PlaceCall) -> Result<(), PlaceError> {
        self.count(call);
        self.fails.remove(&call).map_or(Ok(()), Err)
    }

    /// Makes `folder` and every folder above it.
    pub(super) fn make(&mut self, folder: &Path) {
        let above = folder.ancestors().filter(|a| !a.as_os_str().is_empty());
        self.dirs.extend(above.map(Path::to_path_buf));
    }

    /// Whether `path` or a folder above it is in `set`.
    fn under(set: &BTreeSet<PathBuf>, path: &Path) -> bool {
        path.ancestors().any(|a| set.contains(a))
    }

    /// Whether `path` vanished, or a folder above it did.
    pub(super) fn gone(&self, path: &Path) -> bool {
        Self::under(&self.gone, path)
    }

    /// `Missing` unless `folder` is there.
    pub(super) fn here(&self, folder: &Path) -> Result<(), PlaceError> {
        let here = self.dirs.contains(folder) && !self.gone(folder);
        here.then_some(()).ok_or(PlaceError::Missing)
    }

    /// The free bytes set nearest above `folder`.
    pub(super) fn free(&self, folder: &Path) -> u64 {
        let set = folder.ancestors().find_map(|a| self.free.get(a));
        set.copied().unwrap_or(ROOMY)
    }

    /// Whether `folder` is the data folder, holds it or lies in it.
    pub(super) fn refuses(&self, folder: &Path) -> bool {
        let data = self.data.as_deref();
        data.is_some_and(|d| folder.starts_with(d) || d.starts_with(folder))
    }

    /// Why `folder` cannot take `need` bytes now, if it cannot.
    pub(super) fn writable(&self, folder: &Path, need: u64) -> Result<(), PlaceError> {
        if self.refuses(folder) {
            return Err(PlaceError::Refused);
        }
        self.here(folder)?;
        if Self::under(&self.read_only, folder) {
            return Err(PlaceError::ReadOnly);
        }
        if self.free(folder) < need {
            return Err(PlaceError::Full);
        }
        Ok(())
    }

    /// The places there now whose kind is in `kinds`, in the order added.
    pub(super) fn tops(&self, kinds: &[PlaceKind]) -> Vec<Place> {
        let there = |p: &&Place| kinds.contains(&p.kind) && !self.gone(p.path.expose());
        self.tops.iter().filter(there).cloned().collect()
    }

    /// The suggested places there now, kind by kind in `SUGGESTED`'s order, each kind in the order added.
    pub(super) fn suggested(&self) -> Vec<Place> {
        SUGGESTED
            .iter()
            .flat_map(|&kind| self.tops(&[kind]))
            .collect()
    }

    /// The folders right in `folder` that are there, A to Z ignoring case.
    pub(super) fn subfolders(&self, folder: &Path) -> Vec<Place> {
        let inside = self.dirs.iter().filter(|d| self.within(d, folder));
        sorted(inside.map(|d| entry(PlaceKind::Folder, d)).collect())
    }

    /// The files right in `folder` that are there.
    pub(super) fn files_in<'a>(
        &'a self,
        folder: &'a Path,
    ) -> impl Iterator<Item = (&'a PathBuf, &'a File)> {
        self.files.iter().filter(|(f, _)| self.within(f, folder))
    }

    /// The boxes right in `folder`, found in `place`.
    pub(super) fn boxes(&self, place: &Place, folder: &Path) -> Vec<Found> {
        let named = |f: &Path| {
            f.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(is_box_name)
        };
        let boxes = self.files_in(folder);
        let boxes = boxes.filter(|(f, held)| named(f) && is_box_head(&held.bytes));
        let found = boxes.map(|(f, held)| Found {
            place: place.clone(),
            path: Redacted::new(f.clone()),
            modified_ms: held.modified_ms,
        });
        found.collect()
    }

    /// Whether `path` lies right in `folder` and is there.
    fn within(&self, path: &Path, folder: &Path) -> bool {
        path.parent() == Some(folder) && !self.gone(path)
    }
}

/// A folder or file in the list, named by its last part.
pub(super) fn entry(kind: PlaceKind, path: &Path) -> Place {
    let name = path.file_name().map(|n| n.to_string_lossy().into_owned());
    Place {
        kind,
        name: Redacted::new(name.unwrap_or_default()),
        path: Redacted::new(path.to_path_buf()),
    }
}

/// `list` A to Z by name, ignoring case.
pub(super) fn sorted(mut list: Vec<Place>) -> Vec<Place> {
    list.sort_by_cached_key(|p| p.name.expose().to_lowercase());
    list
}
