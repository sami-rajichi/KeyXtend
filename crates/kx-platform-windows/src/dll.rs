//! Where the process loads libraries from: only its own folder and System32, never the working folder or `PATH`,
//! so a library planted there cannot run with the app's uiAccess rights.

use windows::Win32::System::LibraryLoader::{
    LOAD_LIBRARY_SEARCH_APPLICATION_DIR, LOAD_LIBRARY_SEARCH_SYSTEM32, SetDefaultDllDirectories,
};

use crate::PlatformError;

/// Limits libraries named without a folder to the app's folder and System32, for the rest of the process.
///
/// # Errors
/// `PlatformError::DllDirs` when Windows refuses.
#[allow(unsafe_code, reason = "Sets the process's library search folders.")]
pub fn pin() -> Result<(), PlatformError> {
    let folders = LOAD_LIBRARY_SEARCH_APPLICATION_DIR | LOAD_LIBRARY_SEARCH_SYSTEM32;
    // SAFETY: a plain process setting; libraries loaded with a full path are not affected.
    unsafe { SetDefaultDllDirectories(folders) }.map_err(|e| PlatformError::DllDirs(e.into()))
}
