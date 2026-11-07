//! A temp tree standing for the user's folders, drives and data folder, with the places port over it; nothing here
//! reaches a real cloud folder or drive.

use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering::SeqCst};
use std::sync::{Arc, Mutex};
use std::time::{Duration, UNIX_EPOCH};

use kx_crypto::DiskRoom;
use kx_module_api::ports::{BOX_MARK, BOX_VERSION, PlaceKind};
use kx_platform::APP_DIR_NAME;
use kx_test_support::tempdir::TempDir;

use super::super::quiet::mode;
use super::super::{Drive, WinPlaces};

/// A temp tree: environment variables, drives and the data folder, all inside one temp folder.
pub(super) struct Tree {
    _dir: TempDir,
    pub(super) top: PathBuf,
    pub(super) room: Room,
    /// The thread error mode the drive list and the disk room last ran under.
    pub(super) seen: Seen,
    env: HashMap<String, PathBuf>,
    drives: Vec<Drive>,
}

impl Tree {
    /// A fresh tree named after `label`, with its data folder made.
    pub(super) fn new(label: &str) -> Self {
        let dir = TempDir::new(label).unwrap();
        let top = dir.path().to_path_buf();
        let seen = Seen::default();
        let tree = Self {
            _dir: dir,
            top,
            room: Room::new(seen.clone()),
            seen,
            env: HashMap::new(),
            drives: Vec::new(),
        };
        fs::create_dir_all(tree.data()).unwrap();
        tree
    }

    /// The data folder, as the installed app's.
    pub(super) fn data(&self) -> PathBuf {
        self.top.join("Local").join(APP_DIR_NAME)
    }

    /// Makes the folder `rel` in the tree.
    pub(super) fn make(&self, rel: &str) -> PathBuf {
        let folder = self.top.join(rel);
        fs::create_dir_all(&folder).unwrap();
        folder
    }

    /// Sets the environment variable `name` to `folder`.
    pub(super) fn set_env(&mut self, name: &str, folder: &Path) {
        self.env.insert(name.to_owned(), folder.to_path_buf());
    }

    /// Adds a local drive of `kind` labelled `label`, whose root is the folder `rel`, made; drives list in the order
    /// added.
    pub(super) fn add_drive(&mut self, kind: PlaceKind, rel: &str, label: &str) -> PathBuf {
        self.push(kind, rel, label, false)
    }

    /// Adds a network drive labelled `label`, whose root is the folder `rel`, made.
    pub(super) fn add_share(&mut self, rel: &str, label: &str) -> PathBuf {
        self.push(PlaceKind::Drive, rel, label, true)
    }

    /// Adds a drive of `kind`, network or not.
    fn push(&mut self, kind: PlaceKind, rel: &str, label: &str, remote: bool) -> PathBuf {
        let root = self.make(rel);
        self.drives.push(Drive {
            kind,
            remote,
            root: root.clone(),
            label: label.to_owned(),
        });
        root
    }

    /// The port over the tree.
    pub(super) fn places(&self) -> WinPlaces {
        let (drives, seen) = (self.drives.clone(), self.seen.clone());
        let list = move || {
            seen.note();
            drives.clone()
        };
        let room = Box::new(self.room.clone());
        WinPlaces::at(self.env.clone(), Box::new(list), room, self.data())
    }
}

/// The thread error mode a call last ran under; `None` before any.
#[derive(Clone, Default)]
pub(super) struct Seen(Arc<Mutex<Option<u32>>>);

impl Seen {
    /// Notes this thread's error mode now.
    fn note(&self) {
        *self.0.lock().unwrap() = Some(mode());
    }

    /// The mode noted last, forgetting it.
    pub(super) fn take(&self) -> Option<u32> {
        self.0.lock().unwrap().take()
    }
}

/// A disk room whose free bytes the test sets, all of them free until then; it notes the mode it runs under.
#[derive(Clone)]
pub(super) struct Room {
    free: Arc<AtomicU64>,
    seen: Seen,
}

impl Room {
    /// A roomy disk that notes its calls' mode in `seen`.
    fn new(seen: Seen) -> Self {
        let free = Arc::new(AtomicU64::new(u64::MAX));
        Self { free, seen }
    }

    /// Leaves `bytes` free on every drive.
    pub(super) fn set_free(&self, bytes: u64) {
        self.free.store(bytes, SeqCst);
    }
}

impl DiskRoom for Room {
    fn free(&self, path: &Path) -> io::Result<u64> {
        self.seen.note();
        // The real room fails on a folder that is gone; so does this one.
        if path.is_dir() {
            Ok(self.free.load(SeqCst))
        } else {
            Err(io::ErrorKind::NotFound.into())
        }
    }

    fn drive(&self, _path: &Path) -> io::Result<String> {
        Ok(String::new())
    }
}

/// A box's bytes: its mark and version, then `tail`.
pub(super) fn boxed(tail: &[u8]) -> Vec<u8> {
    [&BOX_MARK[..], &[BOX_VERSION], tail].concat()
}

/// Writes `bytes` as `file`, last written `secs` after the Unix epoch.
pub(super) fn plant(file: &Path, bytes: &[u8], secs: u64) {
    fs::create_dir_all(file.parent().unwrap()).unwrap();
    fs::write(file, bytes).unwrap();
    let open = OpenOptions::new().write(true).open(file).unwrap();
    open.set_modified(UNIX_EPOCH + Duration::from_secs(secs))
        .unwrap();
}

/// The names in `dir`, sorted.
pub(super) fn names(dir: &Path) -> Vec<String> {
    let entries = fs::read_dir(dir).unwrap().map(|e| e.unwrap().file_name());
    let mut names: Vec<_> = entries.map(|n| n.to_string_lossy().into_owned()).collect();
    names.sort();
    names
}
