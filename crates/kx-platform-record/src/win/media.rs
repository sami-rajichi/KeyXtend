//! Media Foundation, loaded at run time: the worker delay-loads its libraries, so each is first loaded here from the
//! system folder, and a system without them (a Windows N edition) answers `NoVideo` before any of its calls is made.

use windows::Win32::Media::MediaFoundation::{MF_VERSION, MFSTARTUP_LITE, MFShutdown, MFStartup};
use windows::Win32::System::LibraryLoader::{LOAD_LIBRARY_SEARCH_SYSTEM32, LoadLibraryExW};
use windows::core::HSTRING;

use crate::record::RecordFault;

/// Media Foundation, loaded and started until dropped.
pub(super) struct Mf;

impl Mf {
    /// Loads `dlls` from the system folder, then starts Media Foundation without its sockets, which only network
    /// sources use.
    ///
    /// # Errors
    /// `NoVideo` when a library is missing or Media Foundation will not start (edge 14).
    #[allow(unsafe_code, reason = "Starts Media Foundation.")]
    pub(super) fn start(dlls: &[String]) -> Result<Self, RecordFault> {
        load(dlls)?;
        // SAFETY: its libraries are loaded; a plain start, ended once when dropped.
        unsafe { MFStartup(MF_VERSION, MFSTARTUP_LITE) }.map_err(|_| RecordFault::NoVideo)?;
        Ok(Self)
    }
}

impl Drop for Mf {
    #[allow(unsafe_code, reason = "Ends Media Foundation.")]
    fn drop(&mut self) {
        // SAFETY: started above.
        let _ = unsafe { MFShutdown() };
    }
}

/// Loads each of `dlls` from the system folder alone, so no other folder can plant one; each stays loaded for the
/// process, which the delay-load helper then finds by name.
///
/// # Errors
/// `NoVideo` for the first that is missing.
#[allow(unsafe_code, reason = "Loads a system library by name.")]
pub(super) fn load(dlls: &[String]) -> Result<(), RecordFault> {
    for dll in dlls {
        let name = HSTRING::from(dll.as_str());
        // SAFETY: a plain load by name from the system folder; the handle is kept for the process, never freed.
        unsafe { LoadLibraryExW(&name, None, LOAD_LIBRARY_SEARCH_SYSTEM32) }
            .map_err(|_| RecordFault::NoVideo)?;
    }
    Ok(())
}
