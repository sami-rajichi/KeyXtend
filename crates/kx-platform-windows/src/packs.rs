//! Word packs read straight from disk (ADR-0032): each language's `<iso>.kxw`, mapped read-only from the first folder
//! that has it. The name check, the size limit and the folder order are pure; the map is in `win`.
#![cfg_attr(
    not(windows),
    allow(
        dead_code,
        reason = "Only the Windows packs run this logic; tests cover it everywhere."
    )
)]

use std::path::{Path, PathBuf};

use kx_crypto::RESERVED_NAMES;
use kx_module_api::ports::{PACK_EXT, PackError};

#[cfg(windows)]
mod win;

#[cfg(windows)]
pub use win::WinWordPacks;

/// The shortest language code a pack is named after (ISO 639-1).
pub const MIN_ISO_LEN: usize = 2;
/// The longest language code a pack is named after (ISO 639-2 and 639-3).
pub const MAX_ISO_LEN: usize = 3;
/// The largest pack that is mapped, 64 MiB.
pub const MAX_PACK_BYTES: u64 = 64 * 1024 * 1024;

/// The file name of `iso`'s pack; `None` unless `iso` is 2–3 lowercase ASCII letters and no Windows device name
/// (`RESERVED_NAMES`), so no path can leave a folder.
pub(crate) fn file_name(iso: &str) -> Option<String> {
    let fits = (MIN_ISO_LEN..=MAX_ISO_LEN).contains(&iso.len());
    let letters = iso.bytes().all(|b| b.is_ascii_lowercase());
    (fits && letters && !RESERVED_NAMES.contains(&iso)).then(|| format!("{iso}.{PACK_EXT}"))
}

/// A pack file's `len` as its view's length; `Unreadable` when the file is empty or over `MAX_PACK_BYTES`.
pub(crate) fn view_len(len: u64) -> Result<usize, PackError> {
    if len == 0 || len > MAX_PACK_BYTES {
        return Err(PackError::Unreadable);
    }
    usize::try_from(len).map_err(|_| PackError::Unreadable)
}

/// The OS side of a pack: Windows' file map, a fake disk in tests.
pub(crate) trait Os {
    /// One mapped pack.
    type View;

    /// The pack file at `path`, mapped; `Ok(None)` when no file is there.
    fn map(&self, path: &Path) -> Result<Option<Self::View>, PackError>;
}

/// `iso`'s pack from the first of `dirs` that has it, whose answer stands even when it is an error.
/// A bad code or a relative folder never reaches the disk, so the working folder never supplies a pack.
pub(crate) fn find<O: Os>(
    os: &O,
    dirs: &[PathBuf],
    iso: &str,
) -> Result<Option<O::View>, PackError> {
    let Some(name) = file_name(iso) else {
        return Ok(None);
    };
    for dir in dirs.iter().filter(|d| d.is_absolute()) {
        if let Some(view) = os.map(&dir.join(&name))? {
            return Ok(Some(view));
        }
    }
    Ok(None)
}

#[cfg(test)]
mod tests;
