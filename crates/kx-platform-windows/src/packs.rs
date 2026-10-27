//! The Win32 side of word packs (ADR-0032): each pack file opened for reading with read sharing only, and mapped
//! read-only. The folder order, the name and the size rules are `kx_platform::packs`'.

use std::fs::{File, OpenOptions};
use std::io::ErrorKind;
use std::os::windows::fs::OpenOptionsExt;
use std::os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle};
use std::path::Path;

use kx_module_api::ports::{PackError, PackView};
use kx_platform::packs::{DiskPacks, PackDisk};
use windows::Win32::Foundation::HANDLE;
use windows::Win32::Storage::FileSystem::FILE_SHARE_READ;
use windows::Win32::System::Memory::{
    CreateFileMappingW, FILE_MAP_READ, MEMORY_MAPPED_VIEW_ADDRESS, MapViewOfFile, PAGE_READONLY,
    UnmapViewOfFile,
};
use windows::core::PCWSTR;

/// The word packs on Windows: `<iso>.kxw` in each folder, the first folder first, mapped read-only.
pub type WinWordPacks = DiskPacks<WinPackDisk>;

/// The Win32 calls behind `WinWordPacks`: others may read a mapped pack but never write, rename or delete it.
#[derive(Clone, Copy, Debug, Default)]
pub struct WinPackDisk;

/// One mapped pack: the view, the mapping and the file, which close in that order when it drops.
/// It has no `Debug`, as a file's debug output shows its path and so the user's name.
pub struct WinPackView {
    view: View,
    _mapping: OwnedHandle,
    _file: File,
}

impl PackView for WinPackView {
    fn bytes(&self) -> &[u8] {
        self.view.bytes()
    }
}

/// A read-only view of a whole pack file, unmapped once when dropped.
struct View {
    at: MEMORY_MAPPED_VIEW_ADDRESS,
    len: usize,
}

// SAFETY: the view is process-wide read-only memory; any thread may read or unmap it.
#[allow(unsafe_code, reason = "A mapped view may move between threads.")]
unsafe impl Send for View {}
// SAFETY: shared references only read the bytes; only the owner's drop unmaps them.
#[allow(unsafe_code, reason = "A read-only view is shared.")]
unsafe impl Sync for View {}

impl View {
    /// The whole file.
    #[allow(unsafe_code, reason = "Reads the mapped file.")]
    fn bytes(&self) -> &[u8] {
        // SAFETY: `at` maps `len` readable bytes until `self` drops. The file is shared for reading only, but a program
        // that mapped it writable first can still change the bytes, so readers check every offset; release packs sit
        // under Program Files.
        unsafe { std::slice::from_raw_parts(self.at.Value.cast::<u8>(), self.len) }
    }
}

impl Drop for View {
    #[allow(unsafe_code, reason = "Unmaps the view we own.")]
    fn drop(&mut self) {
        // SAFETY: unmaps, once, a view Windows gave us; no slice of it outlives `self`.
        let _ = unsafe { UnmapViewOfFile(self.at) };
    }
}

impl PackDisk for WinPackDisk {
    type File = File;
    type View = WinPackView;

    fn open(&self, path: &Path) -> Result<Option<(File, u64)>, PackError> {
        let file = match OpenOptions::new()
            .read(true)
            .share_mode(FILE_SHARE_READ.0)
            .open(path)
        {
            Ok(file) => file,
            Err(e) if e.kind() == ErrorKind::NotFound => return Ok(None),
            Err(_) => return Err(PackError::Unreadable),
        };
        let meta = file.metadata().map_err(|_| PackError::Unreadable)?;
        if !meta.is_file() {
            return Err(PackError::Unreadable);
        }
        Ok(Some((file, meta.len())))
    }

    fn map(&self, file: File, len: usize) -> Result<WinPackView, PackError> {
        let mapping = mapping(&file)?;
        let view = view(&mapping, len)?;
        Ok(WinPackView {
            view,
            _mapping: mapping,
            _file: file,
        })
    }
}

/// A read-only mapping of the whole of `file`, never copy-on-write.
#[allow(unsafe_code, reason = "Maps a file of ours.")]
fn mapping(file: &File) -> Result<OwnedHandle, PackError> {
    let handle = HANDLE(file.as_raw_handle());
    // SAFETY: `file` is open for reading and outlives the call; with no name, no other program can open the mapping.
    let made = unsafe { CreateFileMappingW(handle, None, PAGE_READONLY, 0, 0, PCWSTR::null()) }
        .map_err(|_| PackError::Unreadable)?;
    // SAFETY: Windows just made this handle, and nothing else owns or closes it.
    Ok(unsafe { OwnedHandle::from_raw_handle(made.0) })
}

/// A read-only view of the first `len` bytes of `mapping`.
#[allow(unsafe_code, reason = "Maps a view of a mapping of ours.")]
fn view(mapping: &OwnedHandle, len: usize) -> Result<View, PackError> {
    let handle = HANDLE(mapping.as_raw_handle());
    // SAFETY: a live read-only mapping, viewed for reading only; Windows checks `len` against the file.
    let at = unsafe { MapViewOfFile(handle, FILE_MAP_READ, 0, 0, len) };
    if at.Value.is_null() {
        return Err(PackError::Unreadable);
    }
    Ok(View { at, len })
}

#[cfg(test)]
mod tests;
