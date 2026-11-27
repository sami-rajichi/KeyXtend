//! The room left on a folder's drive, as the user who runs the worker may fill it.

use std::path::Path;

use windows::Win32::Storage::FileSystem::GetDiskFreeSpaceExW;
use windows::core::HSTRING;

/// The room left on `folder`'s drive, in MiB; `None` when it cannot be read.
#[allow(unsafe_code, reason = "Reads a drive's free room.")]
pub(super) fn room_mb(folder: &Path) -> Option<u64> {
    let path = HSTRING::from(folder.as_os_str());
    let mut free = 0_u64;
    // SAFETY: a live path and a local for Windows to fill.
    unsafe { GetDiskFreeSpaceExW(&path, Some(&raw mut free), None, None) }.ok()?;
    Some(free / u64::try_from(kx_ipc::MIB).ok()?)
}
