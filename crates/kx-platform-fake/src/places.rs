//! A fake places service: an in-memory tree of cloud folders, removable drives, drives and folders with their files,
//! where places vanish and come back, fill up, turn read-only or hold the data folder, one failure comes on cue per
//! call, and calls are counted by kind.

use crate::lock;
use kx_module_api::Redacted;
use kx_module_api::ports::{Found, Place, PlaceError, PlaceKind, PlaceState, Places, box_file};
use std::cmp::Reverse;
use std::fmt;
use std::path::Path;
use std::sync::{Arc, Mutex};
use tree::{File, Tree, entry, sorted};

mod tree;

/// One kind of places call, to fail or count.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum PlaceCall {
    /// `suggested`.
    Suggested,
    /// `folders`.
    Folders,
    /// `files`.
    Files,
    /// `find`.
    Find,
    /// `read`.
    Read,
    /// `write`.
    Write,
    /// `state`.
    State,
    /// `modified`; a failure set for it gives `None`.
    Modified,
}

impl PlaceCall {
    /// Whether the port lets this call return `error`.
    fn returns(self, error: PlaceError) -> bool {
        use PlaceError::{Io, Missing, TooLarge};
        match self {
            Self::Folders | Self::Files => matches!(error, Missing | Io),
            Self::Read => matches!(error, TooLarge | Missing | Io),
            Self::Write => error != TooLarge,
            Self::Modified => error == Io,
            Self::Suggested | Self::Find | Self::State => false,
        }
    }
}

/// The kinds `suggested` gives, in its order.
const SUGGESTED: [PlaceKind; 2] = [PlaceKind::Cloud, PlaceKind::Removable];
/// The kinds that lead the folder list, together in the order added.
const DRIVES: [PlaceKind; 2] = [PlaceKind::Removable, PlaceKind::Drive];

/// Places shared by their clones, held in memory: a write replaces the box at once or not at all, so a scripted write
/// failure, one that broke half-way, leaves the old box whole.
///
/// Vanishing, read-only and free bytes set on a folder hold for all below it, and a vanished place keeps its files for
/// when it comes back.
#[derive(Clone, Default)]
pub struct FakePlaces {
    tree: Arc<Mutex<Tree>>,
}

impl FakePlaces {
    /// Places with no folder, roomy, at time zero.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a place of `kind` named `name` at `path`, and its folder: cloud folders, then removable drives, are
    /// suggested, and removable drives and drives lead the folder list, each in the order added.
    pub fn add(&self, kind: PlaceKind, name: &str, path: &Path) {
        let top = matches!(
            kind,
            PlaceKind::Cloud | PlaceKind::Removable | PlaceKind::Drive
        );
        debug_assert!(top, "a {kind:?} is no place to add");
        let mut tree = lock(&self.tree);
        tree.make(path);
        let (name, path) = (Redacted::new(name.to_owned()), Redacted::new(path.into()));
        tree.tops.push(Place { kind, name, path });
    }

    /// Makes `folder` and the folders above it.
    pub fn add_folder(&self, folder: &Path) {
        lock(&self.tree).make(folder);
    }

    /// Keeps `bytes` as `file`, last written at `modified_ms`, as an earlier run or another computer left it.
    pub fn plant(&self, file: &Path, bytes: Vec<u8>, modified_ms: i64) {
        let mut tree = lock(&self.tree);
        if let Some(folder) = file.parent() {
            tree.make(folder);
        }
        tree.files.insert(file.into(), File { bytes, modified_ms });
    }

    /// The bytes kept as `file`, read without a call.
    #[must_use]
    pub fn file(&self, file: &Path) -> Option<Vec<u8>> {
        lock(&self.tree).files.get(file).map(|f| f.bytes.clone())
    }

    /// Deletes `file` behind the module's back, as a user does by hand; no call is counted.
    pub fn delete_by_hand(&self, file: &Path) {
        lock(&self.tree).files.remove(file);
    }

    /// Makes `folder` and all below it vanish, as a stick unplugged.
    pub fn vanish(&self, folder: &Path) {
        lock(&self.tree).gone.insert(folder.into());
    }

    /// Brings back `folder`, which vanished, with its files.
    pub fn come_back(&self, folder: &Path) {
        lock(&self.tree).gone.remove(folder);
    }

    /// Leaves `bytes` free in `folder` and all below it.
    pub fn set_free(&self, folder: &Path, bytes: u64) {
        lock(&self.tree).free.insert(folder.into(), bytes);
    }

    /// Makes `folder` and all below it refuse writes, or take them again.
    pub fn set_read_only(&self, folder: &Path, on: bool) {
        let mut tree = lock(&self.tree);
        if on {
            tree.read_only.insert(folder.into());
        } else {
            tree.read_only.remove(folder);
        }
    }

    /// Makes `folder` the data folder, which refuses writes in it, above it and below it.
    pub fn set_data_dir(&self, folder: &Path) {
        lock(&self.tree).data = Some(folder.into());
    }

    /// Stamps later writes with `ms` since the Unix epoch.
    pub fn set_now(&self, ms: i64) {
        lock(&self.tree).now_ms = ms;
    }

    /// Refuses the next `call` with `error`, once, replacing one set for that call and not yet used; a debug build
    /// stops on an error the port never returns there.
    pub fn fail(&self, call: PlaceCall, error: PlaceError) {
        debug_assert!(
            call.returns(error),
            "the port never returns {error:?} from {call:?}"
        );
        lock(&self.tree).fails.insert(call, error);
    }

    /// How many calls of kind `call` were asked, refused ones too.
    #[must_use]
    pub fn calls(&self, call: PlaceCall) -> usize {
        lock(&self.tree).calls.get(&call).copied().unwrap_or(0)
    }
}

/// Counts the places, folders, files and calls, never a path, a name or a file's bytes.
impl fmt::Debug for FakePlaces {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let tree = lock(&self.tree);
        f.debug_struct("FakePlaces")
            .field("places", &tree.tops.len())
            .field("folders", &tree.dirs.len())
            .field("files", &tree.files.len())
            .field("calls", &tree.calls)
            .finish_non_exhaustive()
    }
}

/// The length of `bytes` as the port counts it.
fn len(bytes: &[u8]) -> u64 {
    u64::try_from(bytes.len()).unwrap_or(u64::MAX)
}

impl Places for FakePlaces {
    fn suggested(&self) -> Vec<Place> {
        let mut tree = lock(&self.tree);
        tree.count(PlaceCall::Suggested);
        tree.suggested()
    }

    fn folders(&self, folder: Option<&Path>) -> Result<Vec<Place>, PlaceError> {
        let mut tree = lock(&self.tree);
        tree.enter(PlaceCall::Folders)?;
        let Some(folder) = folder else {
            return Ok(tree.tops(&DRIVES));
        };
        tree.here(folder)?;
        Ok(tree.subfolders(folder))
    }

    fn files(&self, folder: &Path, ext: &str) -> Result<Vec<Place>, PlaceError> {
        let mut tree = lock(&self.tree);
        tree.enter(PlaceCall::Files)?;
        tree.here(folder)?;
        let fits = |f: &Path| f.extension().is_some_and(|e| e.eq_ignore_ascii_case(ext));
        let picked = tree.files_in(folder).filter(|(f, _)| fits(f));
        Ok(sorted(
            picked.map(|(f, _)| entry(PlaceKind::File, f)).collect(),
        ))
    }

    fn find(&self) -> Vec<Found> {
        let mut tree = lock(&self.tree);
        tree.count(PlaceCall::Find);
        let mut found = Vec::new();
        for place in tree.suggested() {
            let top = place.path.expose();
            found.extend(tree.boxes(&place, top));
            for sub in tree.subfolders(top) {
                found.extend(tree.boxes(&place, sub.path.expose()));
            }
        }
        found.sort_by_key(|f| Reverse(f.modified_ms));
        found
    }

    fn modified(&self, file: &Path) -> Option<i64> {
        let mut tree = lock(&self.tree);
        tree.enter(PlaceCall::Modified).ok()?;
        let held = tree.files.get(file).filter(|_| !tree.gone(file))?;
        Some(held.modified_ms)
    }

    fn read(&self, file: &Path, cap: u64) -> Result<Redacted<Vec<u8>>, PlaceError> {
        let mut tree = lock(&self.tree);
        tree.enter(PlaceCall::Read)?;
        let held = tree.files.get(file).filter(|_| !tree.gone(file));
        let held = held.ok_or(PlaceError::Missing)?;
        if len(&held.bytes) > cap {
            return Err(PlaceError::TooLarge);
        }
        Ok(Redacted::new(held.bytes.clone()))
    }

    fn write(&self, folder: &Path, bytes: &[u8]) -> Result<(), PlaceError> {
        let mut tree = lock(&self.tree);
        tree.enter(PlaceCall::Write)?;
        tree.writable(folder, len(bytes))?;
        let modified_ms = tree.now_ms;
        let file = File {
            bytes: bytes.to_vec(),
            modified_ms,
        };
        tree.files.insert(folder.join(box_file()), file);
        Ok(())
    }

    fn state(&self, folder: &Path, need: u64) -> PlaceState {
        let mut tree = lock(&self.tree);
        tree.count(PlaceCall::State);
        if tree.refuses(folder) {
            PlaceState::Refused
        } else if tree.here(folder).is_err() {
            PlaceState::Missing
        } else if tree.free(folder) < need {
            PlaceState::Full
        } else {
            PlaceState::Ready
        }
    }
}

#[cfg(test)]
mod tests;
