//! The models host's disk on Windows: the fixed drives the places list shows, a folder's free bytes, a file's SHA-256
//! by CNG, the hidden and system attributes a scan skips, and a models folder made with rights of its own.

mod guard;
mod print;

use std::fs;
use std::io;
use std::os::windows::fs::MetadataExt;
use std::path::{Path, PathBuf};

use kx_crypto::DiskRoom;

use kx_module_api::Redacted;
use kx_module_api::ports::{Drive, Fingerprint};
use kx_platform::models::Disk;

use crate::WinDiskRoom;
use crate::places::{fixed_drives, hides};

pub(crate) use guard::make_guarded;

/// The Windows disk the models host reads and makes folders on.
#[derive(Clone, Copy, Debug, Default)]
pub struct WinDisk;

impl Disk for WinDisk {
    fn drives(&self) -> Vec<Drive> {
        let room = |(root, name): (PathBuf, String)| {
            let free = WinDiskRoom.free(&root).ok()?;
            let (root, name) = (Redacted::new(root), Redacted::new(name));
            Some(Drive { root, name, free })
        };
        fixed_drives().into_iter().filter_map(room).collect()
    }

    fn free(&self, folder: &Path) -> io::Result<u64> {
        WinDiskRoom.free(folder)
    }

    fn sha256(&self, file: &Path) -> io::Result<[u8; Fingerprint::BYTES]> {
        print::sha256(file)
    }

    fn hidden(&self, path: &Path) -> bool {
        fs::symlink_metadata(path).is_ok_and(|m| hides(m.file_attributes()))
    }

    fn make_guarded(&self, folder: &Path) -> io::Result<()> {
        make_guarded(folder)
    }
}

#[cfg(test)]
mod tests;
