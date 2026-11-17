//! The Win32 side of the reveals: Explorer's desktop shell object opens the folder there, through `crate::explorer`.

use std::fs::{File, Metadata, OpenOptions};
use std::os::windows::ffi::OsStrExt;
use std::os::windows::fs::{MetadataExt, OpenOptionsExt};
use std::path::Path;

use kx_module_api::ports::{FolderError, FolderPath, FolderReveal};
use kx_platform::ui::{Reveal, RevealError};
use windows::Win32::Storage::FileSystem::{
    FILE_ATTRIBUTE_REPARSE_POINT, FILE_FLAG_BACKUP_SEMANTICS, FILE_FLAG_OPEN_REPARSE_POINT,
    FILE_LIST_DIRECTORY, FILE_READ_ATTRIBUTES, FILE_SHARE_READ, FILE_SHARE_WRITE,
};
use windows::core::BSTR;

use super::plain::{Found, Plain, found, show_plain};
use super::{Os, show};
use crate::explorer::Explorer;
use crate::places::fixed_drives;

/// Shows a folder in Explorer: the running Explorer opens it itself (Raymond Chen's "launch from the shell" route), so
/// the window belongs to Explorer and nothing starts with our token.
#[derive(Clone, Copy, Debug, Default)]
pub struct WinReveal;

impl Reveal for WinReveal {
    fn show(&self, folder: &Path) -> Result<(), RevealError> {
        show(&Win32, folder)
    }
}

/// Shows the voice module's models folder in Explorer the same way, only when it is a plain folder on a fixed local
/// drive, so it never runs a file, follows a link or reaches the network (ADR-0046).
#[derive(Clone, Copy, Debug, Default)]
pub struct WinFolderReveal;

impl FolderReveal for WinFolderReveal {
    fn show(&self, folder: &FolderPath) -> Result<(), FolderError> {
        show_plain(&Win32, folder)
    }
}

/// The Win32 calls behind `WinReveal` and `WinFolderReveal`.
struct Win32;

impl Os for Win32 {
    type Shell = Explorer;

    fn is_folder(&self, folder: &Path) -> bool {
        folder.is_absolute() && folder.is_dir()
    }

    fn shell(&self) -> Result<Explorer, i32> {
        Explorer::find()
    }

    fn open(&self, explorer: &Explorer, folder: &Path) -> Result<(), i32> {
        explorer.open(&wide(folder))
    }
}

impl Plain for Win32 {
    type Shell = Explorer;
    type Hold = File;

    fn local(&self, folder: &Path) -> bool {
        let letter = |p: &Path| {
            p.as_os_str()
                .as_encoded_bytes()
                .first()
                .map(u8::to_ascii_uppercase)
        };
        let want = letter(folder);
        want.is_some() && fixed_drives().iter().any(|(root, _)| letter(root) == want)
    }

    fn found(&self, folder: &Path) -> Found {
        std::fs::symlink_metadata(folder).map_or(Found::Other, |m| kind(&m))
    }

    fn hold(&self, folder: &Path) -> Result<File, Found> {
        // The folder itself, never a link's target. Listing takes part in sharing, unlike attributes alone, so with no
        // delete shared no one renames or removes it while held.
        let held = OpenOptions::new()
            .access_mode(FILE_LIST_DIRECTORY.0 | FILE_READ_ATTRIBUTES.0)
            .share_mode(FILE_SHARE_READ.0 | FILE_SHARE_WRITE.0)
            .custom_flags(FILE_FLAG_BACKUP_SEMANTICS.0 | FILE_FLAG_OPEN_REPARSE_POINT.0)
            .open(folder)
            .map_err(|_| Found::Other)?;
        match held.metadata().map_or(Found::Other, |m| kind(&m)) {
            Found::Folder => Ok(held),
            odd => Err(odd),
        }
    }

    fn shell(&self) -> Result<Explorer, i32> {
        Explorer::find()
    }

    fn explore(&self, explorer: &Explorer, folder: &Path) -> Result<(), i32> {
        explorer.explore(&wide(folder))
    }
}

/// What a look that follows no link tells of a path.
fn kind(meta: &Metadata) -> Found {
    let reparse = meta.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT.0 != 0;
    found(meta.is_dir(), reparse)
}

/// `folder` as the shell takes it.
fn wide(folder: &Path) -> BSTR {
    let wide: Vec<u16> = folder.as_os_str().encode_wide().collect();
    BSTR::from_wide(&wide)
}

#[cfg(test)]
mod tests;
