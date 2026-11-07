//! Test helpers every store's tests share, and the port contract the file and memory stores all keep.

use crate::guard::{GuardError, KeyGuard};
use crate::shelf::DiskRoom;
use kx_module_api::ports::BlobName;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering::SeqCst};
use std::sync::{Arc, Mutex};
use zeroize::Zeroizing;

pub(crate) mod contract;

/// A day folder, as the history names one.
pub(crate) const DAY: &str = "2026-10-31";
/// The pinned folder.
pub(crate) const PINNED: &str = "pinned";
/// The free bytes `Scripted` answers.
pub(crate) const FREE: u64 = 7_000_000;
/// The drive name `Scripted` answers.
pub(crate) const DRIVE: &str = "D:";

/// A checked test name.
pub(crate) fn name(text: &str) -> BlobName {
    BlobName::new(text).unwrap()
}

/// A test guard that wraps by reversing the bytes, refuses on cue and counts its calls.
#[derive(Clone, Default)]
pub(crate) struct Mirror {
    pub(crate) refuse: Arc<AtomicBool>,
    calls: Arc<AtomicUsize>,
}

impl Mirror {
    pub(crate) fn refusing() -> Self {
        let guard = Self::default();
        guard.refuse.store(true, SeqCst);
        guard
    }

    pub(crate) fn calls(&self) -> usize {
        self.calls.load(SeqCst)
    }

    fn answer(&self, error: GuardError, bytes: &[u8]) -> Result<Vec<u8>, GuardError> {
        self.calls.fetch_add(1, SeqCst);
        if self.refuse.load(SeqCst) {
            return Err(error);
        }
        Ok(bytes.iter().rev().copied().collect())
    }
}

impl KeyGuard for Mirror {
    fn wrap(&self, key: &[u8]) -> Result<Vec<u8>, GuardError> {
        self.answer(GuardError::Unavailable, key)
    }

    fn unwrap(&self, wrapped: &[u8]) -> Result<Zeroizing<Vec<u8>>, GuardError> {
        self.answer(GuardError::Refused, wrapped)
            .map(Zeroizing::new)
    }
}

/// A test drive that answers `FREE` and `DRIVE`, or fails on cue, and records the folders it is asked about.
#[derive(Clone, Default)]
pub(crate) struct Scripted {
    fail: bool,
    asked: Arc<Mutex<Vec<PathBuf>>>,
}

impl Scripted {
    /// A drive that cannot be read.
    pub(crate) fn failing() -> Self {
        Self {
            fail: true,
            ..Self::default()
        }
    }

    fn answer<T>(&self, path: &Path, value: T) -> io::Result<T> {
        self.asked.lock().unwrap().push(path.to_owned());
        if self.fail {
            return Err(io::ErrorKind::PermissionDenied.into());
        }
        Ok(value)
    }

    /// The folders asked about, in order.
    pub(crate) fn asked(&self) -> Vec<PathBuf> {
        self.asked.lock().unwrap().clone()
    }
}

impl DiskRoom for Scripted {
    fn free(&self, path: &Path) -> io::Result<u64> {
        self.answer(path, FREE)
    }

    fn drive(&self, path: &Path) -> io::Result<String> {
        self.answer(path, DRIVE.to_owned())
    }
}

/// The file names in `dir`, sorted.
pub(crate) fn listing(dir: &Path) -> Vec<String> {
    let mut names: Vec<_> = std::fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}
